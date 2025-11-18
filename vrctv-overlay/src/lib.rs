use std::{collections::HashMap, time::Duration};

use axum::{
    Extension, Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{Html, IntoResponse},
    routing::{any, get},
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::{
    net::TcpListener,
    signal,
    sync::{broadcast, watch},
};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct OverlayItem {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub visible: bool,
}

// Implement Debug manually to avoid printing long URLs in logs
impl std::fmt::Debug for OverlayItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OverlayItem")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("visible", &self.visible)
            .field("url", &&self.url[..20]) // Print only the first 20 characters of the URL
            .finish()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(tag = "type", content = "data")]
pub enum ServerCommand {
    UpdateData(OverlayItem),
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(tag = "type", content = "data")]
pub enum ClientRequest {
    GetOverlay(i32),
    SubscribeOverlay(i32),
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub lookup_table: watch::Receiver<Vec<OverlayItem>>,
}

pub async fn start_server(
    tx: broadcast::Sender<ServerCommand>,
    state: AppState,
) -> anyhow::Result<()> {
    // HTTP Server with a WebSocket endpoint to send commands to overlays

    // build our application with a route
    let app = app(tx, state);

    let listener = TcpListener::bind("0.0.0.0:2678").await?;

    // run it
    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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

fn app(tx: broadcast::Sender<ServerCommand>, app_state: AppState) -> Router {
    Router::new()
        .route("/ws", any(ws_handler))
        .route("/overlay/{id}", get(overlay_handler))
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ))
        .layer(Extension(tx))
        .with_state(app_state)
}

async fn overlay_handler(Path(id): Path<i32>) -> impl IntoResponse {
    // Return the overlay HTML from the static file
    Html(include_str!("../static/overlay.html").replace("{{OVERLAY_ID}}", &id.to_string()))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<broadcast::Sender<ServerCommand>>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let rx = tx.subscribe();
    let receiver = app_state.lookup_table.clone();

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket: axum::extract::ws::WebSocket| handle_client(socket, rx, receiver))
}

async fn handle_client(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<ServerCommand>,
    mut receiver: watch::Receiver<Vec<OverlayItem>>,
) {
    info!("WebSocket connection established");

    let overlay_table = receiver.borrow().clone();
    for overlay in overlay_table {
        let response = ServerCommand::UpdateData(overlay.clone());
        if let Err(e) = socket
            .send(Message::Text(
                serde_json::to_string(&response).unwrap().into(),
            ))
            .await
        {
            eprintln!(
                "Error sending updated overlay data to WebSocket client: {}",
                e
            );
            break;
        }
    }

    let mut subscribed_overlays: Vec<i32> = Vec::new();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(cmd) => {
                        if let Err(e) = socket.send(Message::Text(serde_json::to_string(&cmd).unwrap().into())).await {
                            eprintln!("Error sending message to WebSocket client: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving command: {}", e);
                        break;
                    }
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(axum::extract::ws::Message::Text(text)) => {
                        info!("Received message from client: {}", text);

                        match serde_json::from_str::<ClientRequest>(&text) {
                            Ok(ClientRequest::GetOverlay(overlay_id)) => {
                                let current_data = {
                                    let lookup = receiver.borrow();
                                    lookup.iter().find(|o| o.id == overlay_id).cloned()
                                };
                                if let Some(overlay) = current_data {
                                    let response = ServerCommand::UpdateData(overlay.clone());
                                    if let Err(e) = socket.send(Message::Text(serde_json::to_string(&response).unwrap().into())).await {
                                        error!("Error sending overlay data to WebSocket client: {}", e);
                                        break;
                                    }
                                } else {
                                    error!("Overlay with ID {} not found", overlay_id);
                                }
                            }
                            Ok(ClientRequest::SubscribeOverlay(overlay_id)) => {
                                if !subscribed_overlays.contains(&overlay_id) {
                                    subscribed_overlays.push(overlay_id);
                                    info!("Client subscribed to overlay ID {}", overlay_id);
                                }
                            }
                            Err(e) => {
                                error!("Error parsing client request: {}", e);
                            }
                        }
                    }
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Err(e) => {
                        error!("Error receiving message from WebSocket client: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            Ok(_) = receiver.changed() => {
                let new_table = receiver.borrow().clone();
                info!("Lookup table updated: {:?}", new_table);
                for value in new_table {
                    if subscribed_overlays.len() != 0 && !subscribed_overlays.contains(&value.id) {
                        continue;
                    }

                    let response = ServerCommand::UpdateData(value.clone());
                    if let Err(e) = socket.send(Message::Text(serde_json::to_string(&response).unwrap().into())).await {
                        error!("Error sending updated overlay data to WebSocket client: {}", e);
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket connection closed");
}
