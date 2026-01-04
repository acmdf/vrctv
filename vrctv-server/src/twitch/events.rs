use std::env;

use axum::extract::ws::Message;
use log::{error, info};
use vrctv_common::{
    CustomRewardResponse, Notify, ServerMessage, TwitchEvent, TwitchEventSource,
    TwitchTriggerRequest,
};
use reqwest::Error;
use tokio::sync::mpsc::Sender;
use twitch_api::{
    HelixClient,
    eventsub::{self, Event, Payload, channel::chat::Fragment},
    helix::{
        ClientRequestError,
        points::{
            CreateCustomRewardBody, CreateCustomRewardRequest, CustomRewardRedemptionStatus,
            DeleteCustomRewardRequest, GetCustomRewardRequest, UpdateCustomRewardBody,
            UpdateCustomRewardRequest, UpdateRedemptionStatusBody, UpdateRedemptionStatusRequest,
        },
    },
    twitch_oauth2::{ClientId, ClientSecret, UserToken},
};

use crate::server::{
    ClientConnection, send_all_message, send_error, send_message, send_task_response,
};

/// Handle Twitch token errors, such as refreshing the token if it has expired
/// Returns Ok(true) if the token was refreshed, Ok(false) if no action was taken
pub async fn handle_token_error(
    http_client: &reqwest::Client,
    error: &ClientRequestError<Error>,
    token: &mut UserToken,
) -> Result<bool, String> {
    if let ClientRequestError::RequestError(e) = &error {
        if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            let client_secret =
                env::var("TWITCH_SECRET").map_err(|_| "Missing TWITCH_SECRET env var")?;
            let client_id =
                env::var("TWITCH_CLIENT").map_err(|_| "Missing TWITCH_CLIENT env var")?;
            let new_token = UserToken::from_existing_or_refresh_token(
                http_client,
                token.access_token.clone(),
                token
                    .refresh_token
                    .clone()
                    .ok_or("No refresh token available")?,
                ClientId::new(client_id),
                ClientSecret::new(client_secret),
            )
            .await;
            *token = new_token.map_err(|e| {
                error!("Failed to refresh Twitch token: {}", e);
                "Failed to refresh Twitch token".to_string()
            })?;

            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// Handle Twitch trigger requests
/// Returns Ok(true) if the token was refreshed and the caller should retry, Ok(false) otherwise
pub async fn handle_twitch_trigger(
    http_client: &reqwest::Client,
    twitch: &mut UserToken,
    trigger_request: TwitchTriggerRequest,
    tx: &Sender<Message>,
) -> Result<bool, String> {
    info!("Handling Twitch trigger request: {:?}", trigger_request);

    let client = HelixClient::with_client(http_client.clone());
    match trigger_request {
        TwitchTriggerRequest::ChannelPointsFulfill {
            request_id,
            reward_id,
            redemption_id,
        } => {
            let request = UpdateRedemptionStatusRequest::new(
                twitch.user_id.clone(),
                reward_id,
                redemption_id,
            );
            let body = UpdateRedemptionStatusBody::status(CustomRewardRedemptionStatus::Fulfilled);

            match client.req_patch(request, body, twitch).await {
                Ok(d) => {
                    info!("Successfully fulfilled redemption: {:?}", d);
                    let _ = send_task_response(true, None, tx, request_id).await;
                }
                Err(e) => {
                    if handle_token_error(http_client, &e, twitch).await? {
                        return Ok(true);
                    } else {
                        error!("Failed to fulfill redemption: {}", e);
                        let _ = send_error(e, "twitch_fullfill_redemption", tx, request_id).await;
                    }
                }
            }
        }
        TwitchTriggerRequest::ChannelPointsCancel {
            request_id,
            reward_id,
            redemption_id,
        } => {
            let request = UpdateRedemptionStatusRequest::new(
                twitch.user_id.clone(),
                reward_id,
                redemption_id,
            );
            let body = UpdateRedemptionStatusBody::status(CustomRewardRedemptionStatus::Canceled);

            match client.req_patch(request, body, twitch).await {
                Ok(d) => {
                    info!("Successfully cancelled redemption: {:?}", d);
                    let _ = send_task_response(true, None, tx, request_id).await;
                }
                Err(e) => {
                    if handle_token_error(http_client, &e, twitch).await? {
                        return Ok(true);
                    } else {
                        error!("Failed to cancel redemption: {}", e);
                        let _ = send_error(e, "twitch_cancel_redemption", tx, request_id).await;
                    }
                }
            }
        }
        TwitchTriggerRequest::UpdateCustomRewards {
            request_id,
            rewards,
        } => {
            let request = GetCustomRewardRequest::broadcaster_id(twitch.user_id.clone())
                .only_manageable_rewards(true);

            match client.req_get(request, twitch).await {
                Ok(d) => {
                    info!("Successfully fetched previous custom rewards: {:?}", d);

                    let data = d.data;

                    for reward in rewards.clone() {
                        if let Some(existing) = data.iter().find(|r| r.title == reward.title) {
                            let update_request = UpdateCustomRewardRequest::new(
                                twitch.user_id.clone(),
                                existing.id.clone(),
                            );
                            let mut update_body = UpdateCustomRewardBody::default();

                            if existing.cost != reward.cost as usize {
                                update_body.cost = Some(reward.cost as usize);
                            }
                            if existing.prompt != reward.prompt {
                                update_body.prompt = Some(reward.prompt.into());
                            }
                            if existing.is_enabled != reward.is_enabled {
                                update_body.is_enabled = Some(reward.is_enabled);
                            }
                            if existing.global_cooldown_setting.is_enabled
                                != reward.is_global_cooldown_enabled
                            {
                                update_body.is_global_cooldown_enabled =
                                    Some(reward.is_global_cooldown_enabled);
                            }
                            if existing.global_cooldown_setting.global_cooldown_seconds
                                != reward.global_cooldown_seconds
                            {
                                update_body.global_cooldown_seconds =
                                    Some(reward.global_cooldown_seconds as usize);
                            }

                            if update_body == UpdateCustomRewardBody::default() {
                                info!("No changes for custom reward: {}", reward.title);
                                continue;
                            }

                            match client.req_patch(update_request, update_body, twitch).await {
                                Ok(updated) => {
                                    info!("Successfully updated custom reward: {:?}", updated);
                                }
                                Err(e) => {
                                    error!("Failed to update custom reward: {}", e);
                                    let _ = send_error(
                                        e,
                                        "twitch_update_custom_reward",
                                        tx,
                                        request_id,
                                    )
                                    .await;
                                }
                            }
                        } else {
                            // Create the reward
                            let create_request = CreateCustomRewardRequest::broadcaster_id(
                                twitch.user_id.to_string(),
                            );
                            let mut create_body =
                                CreateCustomRewardBody::new(reward.title, reward.cost as usize);
                            create_body.prompt = Some(reward.prompt.into());
                            create_body.is_enabled = Some(reward.is_enabled);
                            create_body.is_global_cooldown_enabled =
                                Some(reward.is_global_cooldown_enabled);
                            create_body.global_cooldown_seconds =
                                Some(reward.global_cooldown_seconds as usize);

                            match client.req_post(create_request, create_body, twitch).await {
                                Ok(created) => {
                                    info!("Successfully created custom reward: {:?}", created);
                                }
                                Err(e) => {
                                    error!("Failed to create custom reward: {}", e);
                                    let _ = send_error(
                                        e,
                                        "twitch_create_custom_reward",
                                        tx,
                                        request_id,
                                    )
                                    .await;
                                }
                            }
                        }
                    }

                    for reward in data {
                        if !rewards.iter().any(|r| r.title == reward.title) {
                            // Delete the reward
                            let delete_request = DeleteCustomRewardRequest::new(
                                twitch.user_id.clone(),
                                reward.id.clone(),
                            );

                            match client.req_delete(delete_request, twitch).await {
                                Ok(deleted) => {
                                    info!("Successfully disabled custom reward: {:?}", deleted);
                                }
                                Err(e) => {
                                    error!("Failed to disable custom reward: {}", e);
                                    let _ = send_error(
                                        e,
                                        "twitch_disable_custom_reward",
                                        tx,
                                        request_id,
                                    )
                                    .await;
                                }
                            }
                        }
                    }

                    let _ = send_task_response(true, None, tx, request_id).await;
                }
                Err(e) => {
                    if handle_token_error(http_client, &e, twitch).await? {
                        return Ok(true);
                    } else {
                        error!("Failed to fetch custom rewards: {}", e);
                        let _ = send_error(e, "twitch_get_custom_rewards", tx, request_id).await;
                    }
                }
            }
        }
        TwitchTriggerRequest::GetCustomRewards { request_id } => {
            let request = GetCustomRewardRequest::broadcaster_id(twitch.user_id.clone())
                .only_manageable_rewards(true);

            match client.req_get(request, twitch).await {
                Ok(d) => {
                    info!("Successfully fetched custom rewards: {:?}", d);

                    let data = d.data;

                    let msg = ServerMessage::CustomRewards {
                        rewards: data
                            .iter()
                            .map(|d| CustomRewardResponse {
                                id: d.id.to_string(),
                                title: d.title.clone(),
                                prompt: d.prompt.clone(),
                                cost: d.cost.try_into().unwrap_or(0),
                                is_enabled: d.is_enabled,
                                is_global_cooldown_enabled: d.global_cooldown_setting.is_enabled,
                                global_cooldown_seconds: d
                                    .global_cooldown_setting
                                    .global_cooldown_seconds,
                            })
                            .collect(),
                    };

                    let _ = send_message(msg, tx).await;
                    let _ = send_task_response(true, None, tx, request_id).await;
                }
                Err(e) => {
                    if handle_token_error(http_client, &e, twitch).await? {
                        return Ok(true);
                    } else {
                        error!("Failed to fetch custom rewards: {}", e);
                        let _ = send_error(e, "twitch_get_custom_rewards", tx, request_id).await;
                    }
                }
            }
        }
    }

    Ok(false)
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
                        sender: message.from_user_name.to_string(),
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

            send_all_message(
                ServerMessage::TwitchEvent(TwitchEvent {
                    user_id: message.chatter_user_id.to_string(),
                    user_name: message.chatter_user_name.to_string(),
                    event: TwitchEventSource::Message {
                        sender: message.chatter_user_name.to_string(),
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
