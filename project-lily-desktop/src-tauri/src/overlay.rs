use std::collections::HashMap;

use project_lily_overlay::ServerCommand;
use tauri::Manager;
use tokio::sync::{broadcast, watch};

#[tauri::command]
#[specta::specta]
pub async fn send_overlay_command(
    app: tauri::AppHandle,
    message: ServerCommand,
) -> Result<(), String> {
    let tx = app.state::<broadcast::Sender<ServerCommand>>().clone();

    tx.send(message).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_overlays(
    app: tauri::AppHandle,
    message: Vec<project_lily_overlay::OverlayItem>,
) -> Result<(), String> {
    let tx = app
        .state::<watch::Sender<Vec<project_lily_overlay::OverlayItem>>>()
        .clone();

    tx.send(message).map_err(|e| e.to_string())?;

    Ok(())
}
