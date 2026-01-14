use std::{collections::HashMap};

use anyhow::Result;
use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use log::debug;
use rusqlite::params;

use crate::{
    AppState,
    config::config,
    db::Database,
    entities::{ActiveKey, StreamlabsUser},
};
/// IMPORTANT NOTE: Streamlabs uses OAuth2 tokens that do NOT expire
pub mod socket;

#[derive(Debug, Clone)]
pub struct UserToken {
    /// The access token used to authenticate requests with
    pub access_token: String,
    /// Username of user associated with this token
    pub login: String,
    /// User ID of the user associated with this token
    pub user_id: i64,
    /// The refresh token used to extend the life of this user token
    pub refresh_token: String,
    /// The socket token that can be used to open the socketio connection
    pub socket_token: String,
}

impl UserToken {
    pub async fn validate_token(
        http_client: &reqwest::Client,
        access_token: &str,
        refresh_token: &str,
    ) -> Result<UserToken> {
        let resp = http_client
            .get("https://streamlabs.com/api/v2.0/user")
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            debug!("Failed to validate token: HTTP {:?}", resp);

            if resp.status().is_redirection() {
                if let Some(location) = resp.headers().get(reqwest::header::LOCATION) {
                    debug!("Redirection location: {:?}", location);
                    debug!("Access token: {access_token}");
                    return Err(anyhow::anyhow!(
                        "Token validation redirected to {}",
                        location.to_str().unwrap_or("<invalid UTF-8>")
                    ));
                }
            }

            return Err(anyhow::anyhow!(
                "Failed to validate token: HTTP {}",
                resp.status()
            ));
        }
        let v: serde_json::Value = resp.json().await?;
        debug!("Streamlabs validation response: {:?}", v);
        let streamlabs = v
            .get("streamlabs")
            .ok_or_else(|| anyhow::anyhow!("No streamlabs field in validation response"))?;

        let id = streamlabs
            .get("id")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| anyhow::anyhow!("No id field in streamlabs validation response"))?;
        let display_name = streamlabs
            .get("display_name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("No display_name field in streamlabs validation response")
            })?;

        let socket_resp = http_client
            .get("https://streamlabs.com/api/v2.0/socket/token")
            .bearer_auth(access_token)
            .send()
            .await?;

        if !socket_resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get socket token: HTTP {}",
                socket_resp.status()
            ));
        }

        let socket_v: serde_json::Value = socket_resp.json().await?;
        debug!("Streamlabs socket token response: {:?}", socket_v);
        let socket_token = socket_v
            .get("socket_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("No socket_token field in socket token response"))?;

        Ok(UserToken {
            user_id: id,
            login: display_name.to_string(),
            socket_token: socket_token.to_string(),
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
        })
    }

    pub async fn from_refresh_token(
        http_client: &reqwest::Client,
        redirect_uri: String,
        refresh_token: String,
        client_id: String,
        client_secret: String,
    ) -> Result<Self> {
        let resp = http_client
            .post("https://streamlabs.com/api/v2.0/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("redirect_uri", &redirect_uri),
                ("refresh_token", &refresh_token),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to refresh token: HTTP {}",
                resp.status()
            ));
        }

        let v: serde_json::Value = resp.json().await?;
        debug!("Streamlabs refresh response: {:?}", v);
        let access_token = v
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("No access_token field in refresh response"))?;
        let refresh_token = v
            .get("refresh_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("No refresh_token field in refresh response"))?;

        Self::validate_token(http_client, access_token, refresh_token).await
    }

    pub async fn from_existing_or_refresh_token(
        http_client: &reqwest::Client,
        redirect_uri: String,
        access_token: String,
        refresh_token: String,
        client_id: String,
        client_secret: String,
    ) -> Result<Self> {
        match Self::validate_token(http_client, &access_token, &refresh_token).await {
            Ok(v) => Ok(v),
            Err(_) => {
                Self::from_refresh_token(
                    http_client,
                    redirect_uri,
                    refresh_token,
                    client_id,
                    client_secret,
                )
                .await
            }
        }
    }
}

pub async fn use_authorization_code(
    http_client: &reqwest::Client,
    code: &str,
) -> Result<(String, String), String> {
    let config = config().await;

    let client = config.streamlabs_oauth().client().to_string();
    let client_secret = config.streamlabs_oauth().secret().to_string();
    let callback_url = config.streamlabs_oauth().redirect().to_string();

    let resp = http_client
        .post("https://streamlabs.com/api/v2.0/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", &client),
            ("client_secret", &client_secret),
            ("redirect_uri", &callback_url),
            ("code", &code),
        ])
        .send()
        .await
        .map_err(|e| {
            format!("Failed to get Streamlabs token: {e}. Check your client ID and secret?")
        })?;

    if !resp.status().is_success() {
        return Err(format!("Failed to get token: HTTP {}", resp.status()));
    }

    let v: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Streamlabs token response: {e}"))?;
    debug!("Streamlabs authentication response: {:?}", v);
    let access_token = v
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| format!("No access_token field in authentication response"))?;
    let refresh_token = v
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| format!("No refresh_token field in authentication response"))?;

    Ok((access_token.to_string(), refresh_token.to_string()))
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
            format!("Streamlabs returned an error: {}: {}", error, description),
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

    let conn = database.connection().unwrap();

    let (auth_token, refresh_token) = match use_authorization_code(&http_client, code).await {
        Ok(token) => token,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Streamlabs Authorization Code Error: {}", e),
            );
        }
    };

    match UserToken::validate_token(&http_client, &auth_token, &refresh_token).await {
        Ok(streamlabs_user) => {
            debug!("Streamlabs validated user: {:?}", streamlabs_user);

            // Insert or update the user in the database
            let user = StreamlabsUser::new(streamlabs_user.user_id);
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
                "INSERT INTO active_stream_labs_keys (authentication, refresh, user, state, version)
                 VALUES (?, ?, ?, ?, 1)
                 ON CONFLICT(user) DO UPDATE SET
                   authentication = excluded.authentication,
                   refresh = excluded.refresh,
                   state = excluded.state,
                   version = active_stream_labs_keys.version + 1;",
                params![
                    streamlabs_user.access_token,
                    streamlabs_user.refresh_token,
                    user.id,
                    state,
                ],
            )
            .unwrap();

            // Notify any waiting client
            let mut table = app_state.connection_table.lock().await;
            if let Some(client) = table.get_mut(state) {
                client.context.lock().await.streamlabs = Some(streamlabs_user);

                if let Err(e) = client.send(client.get_connect_message().await).await {
                    debug!("Failed to send Streamlabs connect message: {}", e);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to send Streamlabs connect message: {}", e),
                    );
                }
            }

            (
                axum::http::StatusCode::OK,
                "Streamlabs authentication successful! You can close this tab.".to_string(),
            )
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Streamlabs Validation Error: {}", e),
            );
        }
    }
}
