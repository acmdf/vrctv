use std::time::Duration;

use log::{error, LevelFilter};
use project_lily_overlay::start_server;
use serde::{Deserialize, Serialize};
use specta::Type;
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder, Event};
use tokio::{
    sync::{broadcast, mpsc, watch},
    time::sleep,
};

use crate::{
    avatars::{change_avatar, fetch_avatar_osc, fetch_avatars, set_osc},
    osc::osc_message_broadcaster,
    overlay::{send_overlay_command, update_overlays},
    xsoverlay::{send_notification, xsoverlay_notifier},
};

mod avatars;
mod osc;
mod overlay;
mod xsoverlay;

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct OscChangeEvent {
    pub address: String,
    pub value: OscValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub enum OscValue {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct ServiceStatusEvent {
    pub service: Service,
    pub status: ServiceStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub enum Service {
    Osc,
    Overlay,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub enum ServiceStatus {
    Started,
    Stopped,
    Error(String),
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            fetch_avatars,
            fetch_avatar_osc,
            change_avatar,
            set_osc,
            send_notification,
            send_overlay_command,
            update_overlays,
        ])
        .events(collect_events![OscChangeEvent, ServiceStatusEvent]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Info)
                .level_for("vrchat_osc::mdns::task", LevelFilter::Warn)
                .level_for("tauri_runtime_wry", LevelFilter::Off)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Webview,
                ))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // This is also required if you want to use events
            builder.mount_events(app);

            let (tx, _) = broadcast::channel(16);
            let (watch_tx, watch_rx) = watch::channel(Vec::new());
            let state = project_lily_overlay::AppState {
                lookup_table: watch_rx,
            };

            app.manage(watch_tx);
            app.manage(tx.clone());

            let overlay_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;

                loop {
                    ServiceStatusEvent {
                        service: Service::Overlay,
                        status: ServiceStatus::Started,
                    }
                    .emit(&overlay_handle)
                    .unwrap_or_else(|e| {
                        error!("Failed to emit service status event: {}", e);
                    });

                    match start_server(tx.clone(), state.clone()).await {
                        Ok(_) => {
                            ServiceStatusEvent {
                                service: Service::Overlay,
                                status: ServiceStatus::Stopped,
                            }
                            .emit(&overlay_handle)
                            .unwrap_or_else(|e| {
                                error!("Failed to emit service status event: {}", e);
                            });
                        }
                        Err(e) => {
                            error!("Overlay server encountered an error: {}", e);

                            ServiceStatusEvent {
                                service: Service::Overlay,
                                status: ServiceStatus::Error(format!(
                                    "Overlay server encountered an error: {}",
                                    e
                                )),
                            }
                            .emit(&overlay_handle)
                            .unwrap_or_else(|e| {
                                error!("Failed to emit service status event: {}", e);
                            });
                        }
                    }

                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            });

            let osc_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let osc_listener =
                    match osc::setup_osc_listener(vec!["/avatar/change".into()]).await {
                        Ok(listener) => listener,
                        Err(e) => {
                            error!("Failed to set up OSC listener: {}", e);

                            ServiceStatusEvent {
                                service: Service::Osc,
                                status: ServiceStatus::Error(format!(
                                    "Failed to set up OSC listener: {}",
                                    e
                                )),
                            }
                            .emit(&osc_handle)
                            .unwrap_or_else(|e| {
                                error!("Failed to emit service status event: {}", e);
                            });

                            return;
                        }
                    };

                ServiceStatusEvent {
                    service: Service::Osc,
                    status: ServiceStatus::Started,
                }
                .emit(&osc_handle)
                .unwrap_or_else(|e| {
                    error!("Failed to emit service status event: {}", e);
                });

                osc_handle.manage(osc_listener);

                osc_message_broadcaster(&osc_handle)
                    .await
                    .unwrap_or_else(move |e| {
                        error!("OSC message broadcaster encountered an error: {}", e);

                        ServiceStatusEvent {
                            service: Service::Osc,
                            status: ServiceStatus::Error(format!(
                                "OSC message broadcaster encountered an error: {}",
                                e
                            )),
                        }
                        .emit(&osc_handle)
                        .unwrap_or_else(|e| {
                            error!("Failed to emit service status event: {}", e);
                        });
                    });

                log::info!("OSC Service registered.");
            });

            let (tx, mut rx) = mpsc::unbounded_channel();

            tauri::async_runtime::spawn(async move {
                loop {
                    let res =
                        xsoverlay_notifier(&mut rx, &"127.0.0.1".to_string(), 42069, 42070).await;
                    error!(
                        "XSOverlay notification sender died unexpectedly: {:?}, restarting sender",
                        res
                    );
                    sleep(Duration::from_secs(2)).await;
                }
            });
            app.manage(tx);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
