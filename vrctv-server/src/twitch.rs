use std::{collections::HashMap, env};

use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use log::{debug, info};
use reqwest::Url;
use rusqlite::params;
use twitch_api::twitch_oauth2::{
    ClientSecret, TwitchToken, UserToken, UserTokenBuilder, client::Client, id::TwitchTokenResponse,
};

use crate::{
    AppState,
    db::Database,
    entities::{ActiveKey, TwitchUser},
};

pub mod events;
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
