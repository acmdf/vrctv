use std::{collections::HashMap, env};

use axum::{
    Extension,
    extract::{Query, State, ws::Message},
    response::IntoResponse,
};
use log::{debug, error, info};
use project_lily_common::{CustomRewardResponse, ServerMessage, TwitchTriggerRequest};
use reqwest::Url;
use rusqlite::params;
use tokio::sync::mpsc::Sender;
use twitch_api::{
    HelixClient,
    helix::points::{
        CreateCustomRewardBody, CreateCustomRewardRequest, CustomRewardRedemptionStatus,
        DeleteCustomRewardRequest, GetCustomRewardRequest, UpdateCustomRewardBody,
        UpdateCustomRewardRequest, UpdateRedemptionStatusBody, UpdateRedemptionStatusRequest,
    },
    twitch_oauth2::{
        ClientSecret, TwitchToken, UserToken, UserTokenBuilder, client::Client,
        id::TwitchTokenResponse,
    },
};

use crate::{
    AppState,
    db::Database,
    entities::{ActiveKey, TwitchUser},
    server::{send_error, send_message, send_task_response},
};

pub mod eventsub;

pub async fn use_authorization_code(
    http_client: &reqwest::Client,
    code: &str,
) -> Result<UserToken, String> {
    let client = env::var("TWITCH_CLIENT").map_err(|_| "Missing TWITCH_CLIENT env var")?;
    let client_secret = env::var("TWITCH_SECRET").map_err(|_| "Missing TWITCH_SECRET env var")?;
    let callback_url =
        env::var("TWITCH_REDIRECT").map_err(|_| "Missing TWITCH_REDIRECT env var")?;

    let callback_url =
        Url::parse(&callback_url).map_err(|e| format!("Invalid TWITCH_REDIRECT URL: {}", e))?;
    let builder = UserTokenBuilder::new(client, client_secret.clone(), callback_url);
    info!("Getting Twitch token with code: {}", code);
    let request = builder.get_user_token_request(code);
    let response = http_client.req(request).await.map_err(|e| {
        format!("Failed to get Twitch token: {e}. Check your client ID and secret?")
    })?;
    info!(
        "Got Twitch token response: {:?}",
        String::from_utf8(response.body().to_vec())
    );
    let twitch_response = TwitchTokenResponse::from_response(&response).map_err(|e| {
        format!("Failed to get Twitch token response: {e}. Check your client ID and secret?")
    })?;
    let validated_req = twitch_response
        .access_token
        .validate_token(http_client)
        .await
        .map_err(|e| {
            format!("Failed to validate Twitch token: {e}. Check your client ID and secret?")
        })?;

    UserToken::from_response(
        twitch_response,
        validated_req,
        Some(ClientSecret::new(client_secret)),
    )
    .map_err(|e| {
        format!(
            "Failed to create UserToken from Twitch response: {e}. Check your client ID and secret?"
        )
    })
}

pub async fn auth_callback(
    Query(params): Query<HashMap<String, String>>,
    Extension(database): Extension<Database>,
    Extension(http_client): Extension<reqwest::Client>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(|s| s.as_str())
            .unwrap_or("No description provided");
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Twitch returned an error: {}: {}", error, description),
        );
    }

    let code = if let Some(code) = params.get("code") {
        code
    } else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Missing code parameter".to_string(),
        );
    };
    let state = if let Some(state) = params.get("state") {
        state
    } else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Missing state parameter".to_string(),
        );
    };
    let scopes = if let Some(scopes) = params.get("scope") {
        scopes
    } else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Missing scope parameter".to_string(),
        );
    };

    // Check that the scopes match what we expect
    let expected_scopes =
        env::var("TWITCH_SCOPES").expect("You must provide a TWITCH_SCOPES env var");
    if scopes != &expected_scopes {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!(
                "Invalid scopes: expected '{}', got '{}'",
                expected_scopes, scopes
            ),
        );
    }

    let conn = database.connection().unwrap();

    let twitch_user = match use_authorization_code(&http_client, code).await {
        Ok(token) => token,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Twitch Token Error: {}", e),
            );
        }
    };

    match twitch_user.validate_token(&http_client).await {
        Ok(validated) => {
            debug!("Twitch validated user: {:?}", validated);

            // Insert or update the user in the database
            let user = TwitchUser::new(match twitch_user.user_id.as_str().parse() {
                Ok(id) => id,
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid user ID from Twitch".to_string(),
                    );
                }
            });
            match user.insert(&conn) {
                Ok(_) => {}
                Err(e) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    );
                }
            }

            // Insert the active key
            ActiveKey::new(state.clone())
                .insert(&conn)
                .unwrap_or_else(|e| {
                    debug!("Failed to insert active key: {}", e);
                });
            conn.execute(
                "INSERT INTO active_twitch_keys (authentication, refresh, user, state, version)
                 VALUES (?, ?, ?, ?, 1)
                 ON CONFLICT(user) DO UPDATE SET
                   authentication = excluded.authentication,
                   refresh = excluded.refresh,
                   state = excluded.state,
                   version = active_twitch_keys.version + 1;",
                params![
                    twitch_user.access_token.clone().secret(),
                    twitch_user
                        .refresh_token
                        .clone()
                        .map(|t| t.secret().to_string())
                        .unwrap_or("".into()),
                    user.id,
                    state,
                ],
            )
            .unwrap();

            // Notify any waiting client
            let mut table = app_state.connection_table.lock().await;
            if let Some(client) = table.get_mut(state) {
                client.context.lock().await.twitch = Some(twitch_user);

                if let Err(e) = client.send(client.get_connect_message().await).await {
                    debug!("Failed to send Twitch connect message: {}", e);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to send Twitch connect message: {}", e),
                    );
                }
            }

            (
                axum::http::StatusCode::OK,
                "Twitch authentication successful! You can close this tab.".to_string(),
            )
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Twitch Validation Error: {}", e),
            );
        }
    }
}

pub async fn handle_twitch_trigger(
    http_client: &reqwest::Client,
    twitch: &UserToken,
    trigger_request: TwitchTriggerRequest,
    tx: &Sender<Message>,
) -> Result<(), String> {
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
                    error!("Failed to fulfill redemption: {}", e);
                    let _ = send_error(e, "twitch_fullfill_redemption", tx, request_id).await;
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
                    error!("Failed to cancel redemption: {}", e);
                    let _ = send_error(e, "twitch_cancel_redemption", tx, request_id).await;
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
                    error!("Failed to fetch previous custom rewards: {}", e);
                    let _ = send_error(e, "twitch_set_custom_rewards", tx, request_id).await;
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
                    error!("Failed to fetch custom rewards: {}", e);
                    let _ = send_error(e, "twitch_get_custom_rewards", tx, request_id).await;
                }
            }
        }
    }

    Ok(())
}
