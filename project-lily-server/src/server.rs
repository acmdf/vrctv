use std::{env, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use project_lily_common::{
    ChangeAvatar, ClientMessage, ConnectRequest, ConnectResponse, ErrorMessage, Notify,
    ServerMessage, TaskResponse, TwitchEvent, TwitchEventSource,
};
use tokio::sync::{
    Mutex,
    mpsc::{self, Sender},
};
use twitch_api::{
    HelixClient, TWITCH_EVENTSUB_WEBSOCKET_URL,
    eventsub::{self, Event, Payload, channel::chat::Fragment},
    twitch_oauth2::{self, AccessToken, ClientId, ClientSecret, RefreshToken},
};

use crate::{
    AppState,
    db::Database,
    entities::{ActiveKey, ActiveStreamLabsKey, ActiveTwitchKey},
    streamlabs,
    twitch::{eventsub::EventSubWebsocket, handle_twitch_trigger},
};

#[derive(Debug)]
pub struct ClientContext {
    pub addr: SocketAddr,
    pub state_token: Option<String>,
    pub twitch: Option<twitch_oauth2::UserToken>,
    pub streamlabs: Option<streamlabs::UserToken>,
}

#[derive(Clone, Debug)]
pub struct ClientConnection {
    pub sender: Vec<Sender<Message>>,
    pub context: Arc<Mutex<ClientContext>>,

    /// The url to use for websocket
    pub twitch_connection: Option<Arc<Mutex<EventSubWebsocket>>>,
}

impl ClientConnection {
    pub async fn send(&self, msg: ServerMessage) -> Result<(), String> {
        let msg_text = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        let encoded_text = Message::Text(msg_text.into());

        for sender in &self.sender {
            sender
                .send(encoded_text.clone())
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn get_connect_message(&self) -> ServerMessage {
        let context = self.context.lock().await;
        ServerMessage::ConnectResponse(ConnectResponse {
            has_twitch: context.twitch.is_some(),
            twitch_id: context
                .twitch
                .as_ref()
                .map(|t| {
                    t.user_id
                        .clone()
                        .as_str()
                        .parse()
                        .map_err(|e| format!("Failed to parse twitch user id: {}", e))
                })
                .transpose()
                .ok()
                .flatten(),
            twitch_name: context.twitch.as_ref().map(|t| t.login.clone().take()),
            has_streamlabs: context.streamlabs.is_some(),
            streamlabs_id: context.streamlabs.as_ref().map(|s| s.user_id.to_string()),
            streamlabs_name: context.streamlabs.as_ref().map(|s| s.login.clone()),
        })
    }
}

/// Actual websocket statemachine (one will be spawned per connection)
pub async fn handle_client(
    db: Database,
    http_client: reqwest::Client,
    socket: WebSocket,
    who: SocketAddr,
    app_state: AppState,
) {
    let client_context = Arc::new(Mutex::new(ClientContext {
        addr: who,
        state_token: None,
        twitch: None,
        streamlabs: None,
    }));
    let mut connection = None;
    let (mut tx, mut rx) = socket.split();

    let (table_tx, mut table_rx) = mpsc::channel::<Message>(32);

    loop {
        if let Some(state_token) = { client_context.lock().await.state_token.clone() } {
            // if we have a state token, we can register the connection in the connection table
            // In a block so we drop the lock as soon as possible
            {
                let mut table = app_state.connection_table.lock().await;
                if !table.contains_key(&state_token) {
                    let twitch_connection = match &client_context.lock().await.twitch {
                        Some(token) => Some(Arc::new(Mutex::new(EventSubWebsocket {
                            session_id: None,
                            connect_url: TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
                            connection: None,
                            token: token.clone(),
                            client: HelixClient::with_client(http_client.clone()),
                        }))),
                        None => None,
                    };
                    connection = twitch_connection.clone();

                    // Add the sender to the connection table
                    table.insert(
                        state_token,
                        ClientConnection {
                            sender: vec![table_tx.clone()],
                            context: client_context.clone(),
                            twitch_connection,
                        },
                    );
                } else {
                    // Add the sender to the existing connection
                    if let Some(client) = table.get_mut(&state_token) {
                        // Only add if we don't already have this sender
                        if client.sender.iter().any(|s| s.same_channel(&table_tx)) {
                            // We already have this sender, do nothing
                        } else {
                            info!("Adding new sender for existing connection: {}, has {}", state_token, client.sender.len());
                            client.sender.push(table_tx.clone());
                        }
                    }
                }
            }
        }

        tokio::select! {
            biased;

            Some(msg) = rx.next() => {
                match msg {
                    Ok(msg) => {
                        let message_res = handle_message(&db, &http_client, &table_tx, msg, &app_state, client_context.clone()).await;
                        if let Err(e) = message_res {
                            error!("Error handling message from {}: {}", who, e);
                            let _ = send_error(e, "server", &table_tx, -1).await;
                            break;
                        } else if let Ok(false) = message_res {
                            info!("Closing connection from {}", who);
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message from {}: {}", who, e);
                        break;
                    }
                }
            }
            Some(msg) = table_rx.recv() => {
                info!("Sending message to {}: {:?}", who, msg);
                // send message to client
                if let Err(e) = tx.send(msg).await {
                    error!("Error sending message to {}: {}", who, e);
                    break;
                }
            }
            Some(twitch_message) = async {
                if let Some(conn) = connection.clone() {
                    let lock = conn.lock().await.run().await;
                    Some(lock)
                } else {
                    None
                }
            } => {
                match twitch_message {
                    Ok((true, Some(event))) => {
                        info!("Received Twitch event for {}: {:?}", who, event);
                        let senders = {
                            let table = app_state.connection_table.lock().await;
                            if let Some(client) = &client_context.lock().await.state_token {
                                table.get(client).cloned()
                            } else {
                                None
                            }
                        };

                        if let Some(senders) = senders {
                            if let Err(e) = handle_event(&event, &senders).await {
                                error!("Error handling Twitch event for {}: {}", who, e);
                                let _ = send_error(e, "twitch", &table_tx, -1).await;
                            }
                        } else {
                            error!("No connection found for Twitch event for {}", who);
                        }
                    }
                    Ok((true, None)) => {}
                    Ok((false, _)) => {
                        info!("Twitch connection closed for {}", who);
                        // Twitch connection closed, we can drop it, so get the client to reconnect
                        break;
                    }
                    Err(e) => {
                        error!("Error receiving Twitch message for {}: {}", who, e);
                        let _ = send_error(e, "twitch", &table_tx, -1).await;
                        break;
                    }
                }
            }
            else => {
                // both streams closed
                break;
            }
        }
    }

    // If we have a state token, remove the sender from the connection table
    if let Some(state_token) = &client_context.lock().await.state_token {
        let mut table = app_state.connection_table.lock().await;
        if let Some(client) = table.get_mut(state_token) {
            client.sender.retain(|s| !s.same_channel(&table_tx));
            if client.sender.is_empty() {
                if let Some(twitch_connection) = &client.twitch_connection {
                    let mut twitch_connection = twitch_connection.lock().await;
                    if let Err(e) = twitch_connection.disconnect().await {
                        error!("Error disconnecting Twitch connection for {}: {}", who, e);
                    }
                }
                table.remove(state_token);
            }
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

pub async fn send_message(msg: ServerMessage, tx: &Sender<Message>) -> Result<(), String> {
    let msg_text = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
    let encoded_text = Message::Text(msg_text.into());

    tx.send(encoded_text).await.map_err(|e| e.to_string())
}

pub async fn send_all_message(msg: ServerMessage, conn: &ClientConnection) -> Result<(), String> {
    conn.send(msg).await.map_err(|e| e.to_string())
}

/// Returns an error to the client
pub async fn send_error<E: std::fmt::Display>(
    error: E,
    source: &str,
    tx: &Sender<Message>,
    request_id: i32,
) -> Result<(), String> {
    let error_response = ServerMessage::Error(ErrorMessage {
        request_id,
        source: source.into(),
        message: format!("{}", error),
    });

    send_message(error_response, tx).await
}

pub async fn send_task_response(
    success: bool,
    message: Option<String>,
    tx: &Sender<Message>,
    request_id: i32,
) -> Result<(), String> {
    let response = ServerMessage::TaskResponse(TaskResponse {
        request_id,
        success,
        message,
    });

    send_message(response, tx).await
}

/// Handle a single message from the client
/// Returns Ok(true) to continue, Ok(false) to close the connection, Err to indicate an error
pub async fn handle_message(
    connection: &Database,
    http_client: &reqwest::Client,
    tx: &Sender<Message>,
    msg: Message,
    app_state: &AppState,
    context: Arc<Mutex<ClientContext>>,
) -> Result<bool, String> {
    match msg {
        Message::Ping(bytes) => {
            debug!("Received ping: {:?}", bytes);
            // Don't need to respond, axum does it automatically
        }
        Message::Pong(bytes) => {
            debug!("Received pong: {:?}", bytes);
        }
        Message::Close(_) => {
            info!("Received close message");
            return Ok(false);
        }
        Message::Binary(_) => {
            error!("Unexpected binary message");
            return Err("Unexpected binary message".into());
        }
        Message::Text(text) => {
            let mut context = context.lock().await;
            let client_msg: ClientMessage = serde_json::from_str(text.as_str())
                .map_err(|e| format!("Failed to parse message: {}", e))?;
            let conn = connection
                .connection()
                .map_err(|e| format!("Database connection error: {}", e))?;

            match client_msg {
                ClientMessage::CodeRequest(_) => {
                    // Wait for rate limiter
                    let _ = app_state.rate_limiter.new_user.lock().await;

                    // Generate a new state token
                    let state_token = uuid::Uuid::new_v4().to_string();
                    context.state_token = Some(state_token.clone());

                    // Send the state token back to the client
                    let response = ServerMessage::CodeResponse(project_lily_common::CodeResponse {
                        state_token: state_token.clone(),
                    });
                    let response_text =
                        serde_json::to_string(&response).map_err(|e| e.to_string())?;
                    tx.send(Message::Text(response_text.into()))
                        .await
                        .map_err(|e| e.to_string())?;
                    info!("Sent state token to client: {}", state_token);
                }
                ClientMessage::Connect(ConnectRequest { state_token }) => {
                    context.state_token = Some(state_token.clone());

                    // Register the connection if it doesn't already exist
                    let existing = {
                        ActiveKey::get(&conn, &state_token)
                            .map_err(|e| format!("Database error: {}", e))?
                            .is_some()
                    };

                    if !existing {
                        // Insert the new active key
                        let active_key = ActiveKey::new(state_token.clone());
                        active_key
                            .insert(&conn)
                            .map_err(|e| format!("Database error: {}", e))?;
                    }

                    let existing_connection = {
                        let table = app_state.connection_table.lock().await;
                        match table.get(&state_token) {
                            Some(c) => Some(c.context.clone()),
                            None => None,
                        }
                    };

                    if let Some(existing_connection) = existing_connection {
                        // If we already have a connection, copy over the tokens
                        info!("Existing connection found for state token: {}", state_token);
                        let existing_context = existing_connection.lock().await;
                        context.twitch = existing_context.twitch.clone();
                        context.streamlabs = existing_context.streamlabs.clone();
                    } else {
                        // Otherwise, we will check the database for existing connections

                        // Check twitch connection if it exists
                        let twitch_user = ActiveTwitchKey::get_by_active_key(&conn, &state_token)
                            .map_err(|e| format!("Database error: {}", e))?;
                        if let Some(twitch_user) = twitch_user {
                            // Check the connection table for an existing connection
                            // Wait for rate limiter
                            let _ = app_state.rate_limiter.twitch.lock().await;

                            let token = twitch_oauth2::UserToken::from_existing_or_refresh_token(
                                http_client,
                                AccessToken::new(twitch_user.authentication),
                                RefreshToken::new(twitch_user.refresh),
                                ClientId::new(
                                    env::var("TWITCH_CLIENT")
                                        .expect("You must provide a TWITCH_CLIENT env var"),
                                ),
                                Some(ClientSecret::new(
                                    env::var("TWITCH_SECRET")
                                        .expect("You must provide a TWITCH_SECRET env var"),
                                )),
                            )
                            .await
                            .map_err(|e| format!("Twitch Validation Error: {}", e))?;

                            info!("Twitch user connected: {}", token.login);
                            context.twitch = Some(token);
                        } else {
                            info!("No Twitch user connected");
                        }

                        // Check streamlabs connection if it exists
                        if let Some(streamlabs_user) =
                            ActiveStreamLabsKey::get_by_active_key(&conn, &state_token)
                                .map_err(|e| format!("Database error: {}", e))?
                        {
                            // Wait for rate limiter
                            let _ = app_state.rate_limiter.streamlabs.lock().await;

                            let token = streamlabs::UserToken::from_existing_or_refresh_token(
                                &http_client,
                                env::var("STREAMLABS_REDIRECT")
                                    .expect("You must provide a STREAMLABS_REDIRECT env var"),
                                streamlabs_user.authentication,
                                streamlabs_user.refresh,
                                env::var("STREAMLABS_CLIENT")
                                    .expect("You must provide a STREAMLABS_CLIENT env var"),
                                env::var("STREAMLABS_SECRET")
                                    .expect("You must provide a STREAMLABS_SECRET env var"),
                            )
                            .await;

                            if let Ok(token) = token {
                                info!("Streamlabs user connected: {}", token.login);
                                context.streamlabs = Some(token);
                            }
                        } else {
                            info!("No Streamlabs user connected");
                        }
                    }

                    // Send a connect response
                    let response = ServerMessage::ConnectResponse(ConnectResponse {
                        has_twitch: context.twitch.is_some(),
                        twitch_id: context
                            .twitch
                            .as_ref()
                            .map(|t| {
                                t.user_id
                                    .clone()
                                    .as_str()
                                    .parse()
                                    .map_err(|e| format!("Failed to parse twitch user id: {}", e))
                            })
                            .transpose()?,
                        twitch_name: context.twitch.as_ref().map(|t| t.login.clone().take()),
                        has_streamlabs: context.streamlabs.is_some(),
                        streamlabs_id: context.streamlabs.as_ref().map(|s| s.user_id.to_string()),
                        streamlabs_name: context.streamlabs.as_ref().map(|s| s.login.clone()),
                    });

                    let response_text =
                        serde_json::to_string(&response).map_err(|e| e.to_string())?;
                    tx.send(Message::Text(response_text.into()))
                        .await
                        .map_err(|e| e.to_string())?;
                }
                ClientMessage::TwitchTrigger(trigger_request) => {
                    if context.twitch.is_none() {
                        return Err("Twitch not connected".into());
                    }

                    let twitch = context.twitch.as_ref().unwrap();
                    handle_twitch_trigger(&http_client, twitch, trigger_request, tx).await?;
                }
            }
        }
    }

    Ok(true)
}

pub async fn handle_event(event: &Event, conn: &ClientConnection) -> Result<(), String> {
    match event {
        Event::ChannelPointsCustomRewardRedemptionAddV1(Payload {
            message: payload, ..
        }) => {
            let message = match payload {
                eventsub::Message::Notification(message) => message,
                _ => {
                    error!("Unexpected payload type for ChannelPointsCustomRewardRedemptionAddV1");
                    return Err("Unexpected payload type".into());
                }
            };

            send_all_message(
                ServerMessage::TwitchEvent(TwitchEvent {
                    user_id: message.user_login.to_string(),
                    user_name: message.user_name.to_string(),
                    event: TwitchEventSource::ChannelPoints {
                        reward_id: message.reward.id.to_string(),
                        reward_name: message.reward.title.to_string(),
                    },
                }),
                &conn,
            )
            .await
        }
        Event::ChannelBitsUseV1(Payload {
            message: payload, ..
        }) => {
            let message = match payload {
                eventsub::Message::Notification(message) => message,
                _ => {
                    error!("Unexpected payload type for ChannelBitsUseV1");
                    return Err("Unexpected payload type".into());
                }
            };

            send_all_message(
                ServerMessage::TwitchEvent(TwitchEvent {
                    user_id: message.user_login.to_string(),
                    user_name: message.user_name.to_string(),
                    event: TwitchEventSource::BitDonation {
                        amount: message.bits as u32,
                        message: message.message.as_ref().map(|m| m.text.clone()),
                        emojis: message.message.as_ref().map(|m| {
                            m.fragments
                                .iter()
                                .filter_map(|f| match f {
                                    Fragment::Emote { text, .. } => Some(text.clone()),
                                    Fragment::Cheermote { text, .. } => Some(text.clone()),
                                    _ => None,
                                })
                                .collect()
                        }),
                    },
                }),
                conn,
            )
            .await
        }
        Event::UserWhisperMessageV1(Payload {
            message: payload, ..
        }) => {
            let message = match payload {
                eventsub::Message::Notification(message) => message,
                _ => {
                    error!("Unexpected payload type for UserWhisperMessageV1");
                    return Err("Unexpected payload type".into());
                }
            };

            if message.from_user_login.as_str() == "acmdf" {
                if message.whisper.text == "furry_mode" {
                    let _ = send_all_message(
                        ServerMessage::ChangeAvatar(ChangeAvatar {
                            id: "avtr_66069c77-8ecb-439c-9643-cfb1fbfb1363".into(),
                        }),
                        conn,
                    )
                    .await;
                } else if message.whisper.text == "normal_mode" {
                    let _ = send_all_message(
                        ServerMessage::ChangeAvatar(ChangeAvatar {
                            id: "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c".into(),
                        }),
                        conn,
                    )
                    .await;
                }
            }

            let _ = send_all_message(
                ServerMessage::Notify(Notify {
                    title: message.from_user_name.to_string(),
                    message: message.whisper.text.to_string(),
                }),
                conn,
            )
            .await;

            send_all_message(
                ServerMessage::TwitchEvent(TwitchEvent {
                    user_id: message.from_user_login.to_string(),
                    user_name: message.from_user_name.to_string(),
                    event: TwitchEventSource::Whisper {
                        message: message.whisper.text.to_string(),
                    },
                }),
                conn,
            )
            .await
        }
        Event::ChannelChatMessageV1(Payload {
            message: payload, ..
        }) => {
            let message = match payload {
                eventsub::Message::Notification(message) => message,
                _ => {
                    error!("Unexpected payload type for UserWhisperMessageV1");
                    return Err("Unexpected payload type".into());
                }
            };

            if message.chatter_user_name.as_str() == "acmdf" {
                if message.message.text == "furry_mode" {
                    let _ = send_all_message(
                        ServerMessage::ChangeAvatar(ChangeAvatar {
                            id: "avtr_66069c77-8ecb-439c-9643-cfb1fbfb1363".into(),
                        }),
                        conn,
                    )
                    .await;
                } else if message.message.text == "normal_mode" {
                    let _ = send_all_message(
                        ServerMessage::ChangeAvatar(ChangeAvatar {
                            id: "avtr_da3a3a4d-4936-4652-aa2b-442650e99f5c".into(),
                        }),
                        conn,
                    )
                    .await;
                }
            }

            send_all_message(
                ServerMessage::TwitchEvent(TwitchEvent {
                    user_id: message.chatter_user_id.to_string(),
                    user_name: message.chatter_user_name.to_string(),
                    event: TwitchEventSource::Message {
                        message: message.message.text.to_string(),
                    },
                }),
                conn,
            )
            .await
        }
        _ => Ok(()),
    }
}
