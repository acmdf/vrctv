use std::{collections::HashMap, env, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Extension, Router,
    extract::{
        ConnectInfo, Path, State,
        ws::{WebSocketUpgrade},
    },
    http::HeaderValue,
    response::{Html, IntoResponse, Redirect},
    routing::{any, get},
};
use axum_extra::{TypedHeader, headers};
use listenfd::ListenFd;
use log::{debug, info};
use tokio::{
    net::TcpListener,
    signal,
    sync::Mutex,
    time::{Interval, interval},
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use twitch_api::client::ClientDefault;

use crate::{
    db::Database,
    server::{ClientConnection, handle_client},
};

mod db;
mod entities;
mod server;
mod streamlabs;
mod twitch;

#[derive(Debug, Clone)]
pub struct RateLimitHolder {
    pub twitch: Arc<Mutex<Interval>>,
    pub streamlabs: Arc<Mutex<Interval>>,
    pub new_user: Arc<Mutex<Interval>>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub rate_limiter: RateLimitHolder,
    pub connection_table: Arc<Mutex<HashMap<String, ClientConnection>>>,
}

#[tokio::main]
async fn main() {
    // Tracing debugging/logging setup
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    // build our application with a route
    let app = app();

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        // if we are given a tcp listener on listen fd 0, we use that one
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            debug!("using listener from listenfd: {:?}", listener);
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => {
            debug!("no listener from listenfd, defaulting to TcpListener::bind");
            TcpListener::bind("0.0.0.0:3000").await.unwrap()
        }
    };

    // run it
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn app() -> Router {
    let connection_table = Arc::new(Mutex::new(HashMap::new()));

    // Setup the database
    let db = Database::new("./db.sqlite").unwrap();
    {
        let conn = db.connection().unwrap();

        conn.execute_batch(
            "
            BEGIN;
            CREATE TABLE IF NOT EXISTS twitch_users (
                id INTEGER PRIMARY KEY,
                joined_at TIMESTAMP NOT NULL
            );
            CREATE TABLE IF NOT EXISTS streamlabs_users (
                id INTEGER PRIMARY KEY,
                joined_at TIMESTAMP NOT NULL
            );
            CREATE TABLE IF NOT EXISTS active_keys (
                state TEXT PRIMARY KEY,
                created_at TIMESTAMP NOT NULL
            );
            CREATE TABLE IF NOT EXISTS active_twitch_keys (
                id INTEGER PRIMARY KEY,
                authentication TEXT NOT NULL,
                refresh TEXT NOT NULL,
                user INTEGER NOT NULL UNIQUE,
                state TEXT NOT NULL,
                version INTEGER NOT NULL,
                FOREIGN KEY(user) REFERENCES twitch_users(id),
                FOREIGN KEY(state) REFERENCES active_keys(state)
            );
            CREATE TABLE IF NOT EXISTS active_stream_labs_keys (
                id INTEGER PRIMARY KEY,
                authentication TEXT NOT NULL,
                refresh TEXT NOT NULL,
                user INTEGER NOT NULL UNIQUE,
                state TEXT NOT NULL,
                version INTEGER NOT NULL,
                FOREIGN KEY(user) REFERENCES streamlabs_users(id),
                FOREIGN KEY(state) REFERENCES active_keys(state)
            );
            COMMIT;
        ",
        )
        .unwrap();
    }

    let http_client = reqwest::Client::default_client_with_name(Some(HeaderValue::from_static(
        "project-lily-server",
    )))
    .expect("Could not create default client");

    Router::new()
        .route("/twitch/auth/{state}", get(twitch_redirect))
        .route("/twitch/callback", get(twitch::auth_callback))
        .route("/streamlabs/auth/{state}", get(streamlabs_redirect))
        .route("/streamlabs/callback", get(streamlabs::auth_callback))
        .route("/", get(handler))
        .route("/ws", any(ws_handler))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            db,
        ))
        .layer(Extension(http_client))
        .with_state(AppState {
            rate_limiter: RateLimitHolder {
                twitch: Arc::new(Mutex::new(interval(Duration::from_secs(5)))),
                streamlabs: Arc::new(Mutex::new(interval(Duration::from_secs(5)))),
                new_user: Arc::new(Mutex::new(interval(Duration::from_secs(1)))),
            },
            connection_table: connection_table.clone(),
        })
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello!</h1>")
}

async fn twitch_redirect(Path(state): Path<String>) -> impl IntoResponse {
    // URL encode the scopes
    let scopes = env::var("TWITCH_SCOPES")
        .expect("You must provide a TWITCH_SCOPES env var")
        .replace(' ', "%20");

    Redirect::temporary(&format!(
        "https://id.twitch.tv/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        env::var("TWITCH_CLIENT").expect("You must provide a TWITCH_CLIENT env var"),
        env::var("TWITCH_REDIRECT").expect("You must provide a TWITCH_REDIRECT env var"),
        scopes,
        state
    ))
}

async fn streamlabs_redirect(Path(state): Path<String>) -> impl IntoResponse {
    // URL encode the scopes
    let scopes = env::var("STREAMLABS_SCOPES")
        .expect("You must provide a STREAMLABS_SCOPES env var")
        .replace(' ', "%20");

    Redirect::temporary(&format!(
        "https://streamlabs.com/api/v2.0/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        env::var("STREAMLABS_CLIENT").expect("You must provide a STREAMLABS_CLIENT env var"),
        env::var("STREAMLABS_REDIRECT").expect("You must provide a STREAMLABS_REDIRECT env var"),
        scopes,
        state
    ))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(database): Extension<Database>,
    Extension(http_client): Extension<reqwest::Client>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_client(database, http_client, socket, addr, app_state))
}
