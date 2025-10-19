use std::fmt::Debug;

use anyhow::Result;
use futures_util::StreamExt;
use log::{info, warn};
use tokio_tungstenite::tungstenite::{self, protocol::WebSocketConfig};
use twitch_api::{
    HelixClient,
    eventsub::{
        self, Event,
        channel::ChannelChatMessageV1,
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
    },
    twitch_oauth2::{UserToken, url},
};

pub struct EventSubWebsocket {
    /// The session id of the websocket connection
    pub session_id: Option<String>,
    /// The url to use for websocket
    pub connect_url: url::Url,
    /// The current connection
    pub connection: Option<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    /// The token used to authenticate with the Twitch API
    pub token: UserToken,
    /// The client used to make requests to the Twitch API
    pub client: HelixClient<'static, reqwest::Client>,
}

impl EventSubWebsocket {
    /// Connect to the websocket and return the stream
    pub async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    > {
        info!("connecting to twitch");
        let (socket, _) = tokio_tungstenite::connect_async_with_config(
            self.connect_url.clone(),
            Some(WebSocketConfig::default()),
            false,
        )
        .await?;

        Ok(socket)
    }

    /// Handle a single iteration of the websocket connection
    /// Returns Ok(true) if you should continue running, Ok(false) if you should stop
    pub async fn run(&mut self) -> Result<(bool, Option<Event>)> {
        let mut connection = match self.connection.take() {
            Some(conn) => conn,
            None => self.connect().await?,
        };

        if let Some(msg) = connection.next().await {
            self.connection = Some(connection);

            let msg = match msg {
                Err(tungstenite::Error::Protocol(
                    tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                )) => {
                    warn!(
                        "connection was sent an unexpected frame or was reset, reestablishing it"
                    );

                    self.connection = Some(self.connect().await?);
                    return Ok((true, None));
                }
                _ => msg?,
            };

            self.process_message(msg).await
        } else {
            warn!("connection closed by server");
            return Ok((true, None));
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut connection) = self.connection.take() {
            connection.close(None).await?;
            info!("disconnected from twitch eventsub websocket");
        }
        self.connection = None;
        Ok(())
    }

    /// Process a message from the websocket
    /// Returns Ok(true) if you should stay connected, Ok(false) if you should disconnect
    /// and Err if there was an error processing the message
    pub async fn process_message(
        &mut self,
        msg: tungstenite::Message,
    ) -> Result<(bool, Option<Event>)> {
        match msg {
            tungstenite::Message::Text(s) => {
                // Parse the message into a [twitch_api::eventsub::EventsubWebsocketData]
                match Event::parse_websocket(&s)? {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session },
                        ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.process_welcome_message(session).await?;
                        Ok((true, None))
                    }
                    // Here is where you would handle the events you want to listen to
                    EventsubWebsocketData::Notification {
                        metadata: _,
                        payload,
                    } => {
                        info!("received event: {payload:?}");
                        Ok((true, Some(payload)))
                    }
                    EventsubWebsocketData::Revocation {
                        metadata,
                        payload: _,
                    } => {
                        warn!("subscription revoked: {metadata:?}");
                        Ok((false, None))
                    }
                    EventsubWebsocketData::Keepalive {
                        metadata: _,
                        payload: _,
                    } => Ok((true, None)),
                    _ => Ok((true, None)),
                }
            }
            tungstenite::Message::Close(_) => {
                warn!("connection closed by server");
                Ok((false, None))
            }
            _ => Ok((true, None)),
        }
    }

    pub async fn process_welcome_message(&mut self, data: SessionData<'_>) -> Result<()> {
        self.session_id = Some(data.id.to_string());
        if let Some(url) = data.reconnect_url {
            self.connect_url = url.parse()?;
        }

        let transport = eventsub::Transport::websocket(data.id.clone());

        macro_rules! subscribe {
            ( $( $event:ty ),+ ) => {
                $(
                    let subscription = <$event>::broadcaster_user_id(self.token.user_id.clone());
                    match self.client
                        .create_eventsub_subscription(
                            subscription.clone(),
                            transport.clone(),
                            &self.token,
                        )
                        .await {
                            Ok(_) => info!("subscribed to {subscription:?}"),
                            Err(e) => warn!("failed to subscribe to {subscription:?}: {}", e),
                        }
                )+
            };
        }
        // Takes the subscription already made, and just adds the error handling
        macro_rules! send_add_error_handling {
            ( $event:expr ) => {
                let event = $event;
                match self
                    .client
                    .create_eventsub_subscription(event.clone(), transport.clone(), &self.token)
                    .await
                {
                    Ok(_) => info!("subscribed to {event:?}"),
                    Err(e) => warn!("failed to subscribe to {event:?}: {}", e),
                }
            };
        }

        subscribe!(
            eventsub::channel::ChannelBitsUseV1,
            // eventsub::channel::ChannelPollBeginV1,
            // eventsub::channel::ChannelPollProgressV1,
            // eventsub::channel::ChannelPollEndV1,
            // eventsub::channel::ChannelPointsAutomaticRewardRedemptionAddV1,
            eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1,
            eventsub::channel::ChannelPointsCustomRewardRedemptionUpdateV1
        );

        send_add_error_handling!(ChannelChatMessageV1::new(
            self.token.user_id.clone(),
            self.token.user_id.clone()
        ));
        send_add_error_handling!(eventsub::channel::ChannelChatNotificationV1::new(
            self.token.user_id.clone(),
            self.token.user_id.clone()
        ));
        send_add_error_handling!(eventsub::user::UserWhisperMessageV1::new(
            self.token.user_id.clone()
        ));

        Ok(())
    }
}

impl Debug for EventSubWebsocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSubWebsocket")
            .field("session_id", &self.session_id)
            .field("token", &"UserToken { ... }")
            .field("client", &"HelixClient { ... }")
            .field("connect_url", &self.connect_url)
            .field("connected", &self.connection.is_some())
            .finish()
    }
}
