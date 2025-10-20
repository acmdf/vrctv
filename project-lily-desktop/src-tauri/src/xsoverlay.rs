// Code adapted from https://github.com/bluskript/xsoverlay-notifier/blob/master/src/main.rs

use anyhow::Context;
use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Manager;
use tokio::{net::UdpSocket, sync::mpsc};

#[derive(Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct XSOverlayMessage {
    /// 1 = Notification Popup, 2 = MediaPlayer Information, will be extended later on.
    pub message_type: i32,
    /// Only used for Media Player, changes the icon on the wrist.
    pub index: i32,
    /// How long the notification will stay on screen for in seconds
    pub timeout: f32,
    /// Height notification will expand to if it has content other than a title. Default is 175
    pub height: f32,
    /// Opacity of the notification, to make it less intrusive. Setting to 0 will set to 1.
    pub opacity: f32,
    /// Notification sound volume.
    pub volume: f32,
    /// File path to .ogg audio file. Can be "default", "error", or "warning". Notification will be silent if left empty.
    pub audio_path: String,
    /// Notification title, supports Rich Text Formatting
    pub title: String,
    /// Notification content, supports Rich Text Formatting, if left empty, notification will be small.
    pub content: String,
    /// Set to true if using Base64 for the icon image
    pub use_base64_icon: bool,
    /// Base64 Encoded image, or file path to image. Can also be "default", "error", or "warning"
    pub icon: String,
    /// Somewhere to put your app name for debugging purposes
    pub source_app: String,
}

async fn connect_udp(host: &String, port: usize) -> anyhow::Result<UdpSocket> {
    // using port 0 so the OS allocates a available port automatically
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind to local UDP port")?;
    socket
        .connect(format!("{host}:{port}"))
        .await
        .context("Failed to connect to XSOverlay Notification Daemon")?;
    Ok(socket)
}

pub async fn xsoverlay_notifier(
    rx: &mut mpsc::UnboundedReceiver<XSOverlayMessage>,
    host: &String,
    port: usize,
) -> anyhow::Result<()> {
    let socket = connect_udp(&host, port).await?;
    while let Some(msg) = rx.recv().await {
        info!("Sending notification from {}", msg.source_app);
        let data = serde_json::to_string(&msg)?;
        socket
            .send(data.as_bytes())
            .await
            .context("Failed to send notification to XSOverlay UDP socket")?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn send_notification(
    app: tauri::AppHandle,
    message: XSOverlayMessage,
) -> Result<(), String> {
    let tx = app
        .state::<mpsc::UnboundedSender<XSOverlayMessage>>()
        .clone();
    tx.send(message).map_err(|e| e.to_string())
}
