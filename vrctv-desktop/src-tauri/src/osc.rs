use std::sync::Arc;

use anyhow::Result;
use log::{debug, error, info};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use vrchat_osc::{
    models::OscRootNode,
    rosc::{self, OscPacket},
    ServiceType, VRChatOSC,
};

use crate::{OscChangeEvent, OscValue, Service, ServiceStatus, ServiceStatusEvent};

pub async fn setup_osc_listener(paths: Vec<String>) -> Result<Arc<VRChatOSC>> {
    let vrchat_osc = VRChatOSC::new().await?;

    info!("Starting VRChat OSC client...");
    let osc_clone = vrchat_osc.clone();
    vrchat_osc
        .on_connect(move |res| match res {
            ServiceType::Osc(name, addr) => {
                error!("Connected to OSC server: {} at {}", name, addr);
            }
            ServiceType::OscQuery(name, addr) => {
                error!("Connected to OSCQuery server: {} at {}", name, addr);
                let osc_clone = osc_clone.clone();
                let paths = paths.clone();
                tauri::async_runtime::spawn(async move {
                    for path in paths {
                        // NOTE: When actually retrieving parameters, you should implement retry logic here.
                        // If VRChat has just started, it is possible that valid values may not be returned immediately.
                        match osc_clone.get_parameter_from_addr(&path, addr).await {
                            Err(e) => {
                                log::error!("Error retrieving parameter {}: {:?}", path, e);
                            }
                            Ok(params) => log::info!("Received parameter {}: {:?}", path, params),
                        }
                    }
                });
            }
        })
        .await;

    Ok(vrchat_osc)
}

pub async fn osc_message_broadcaster(app: &AppHandle) -> Result<()> {
    let osc_status_handle = app.clone();
    let osc_callback_handle = app.clone();
    let osc = app.state::<Arc<VRChatOSC>>();

    let handle_osc_message = move |msg: rosc::OscMessage| {
        let address = msg.addr.clone();
        if msg.args.is_empty() {
            log::warn!("OSC message has no arguments: {:?}", msg);
            return;
        }

        let value = match &msg.args[0] {
            rosc::OscType::Int(i) => OscValue::Int(*i),
            rosc::OscType::Float(f) => OscValue::Float(*f),
            rosc::OscType::String(s) => OscValue::String(s.clone()),
            rosc::OscType::Bool(b) => OscValue::Bool(*b),
            _ => {
                log::warn!("Unsupported OSC argument type: {:?}", msg.args[0]);
                return;
            }
        };

        debug!("OSC Message - Address: {}, Value: {:?}", address, value);

        OscChangeEvent { address, value }
            .emit(&osc_callback_handle)
            .unwrap_or_else(|e| {
                error!("Failed to emit OSC change event: {}", e);
            });
    };

    let root_node = OscRootNode::new().with_avatar();

    osc.register(
        "vrctv_desktop",
        root_node,
        move |packet| match packet {
            OscPacket::Message(msg) => {
                debug!("Received OSC message: {:?}", msg);

                handle_osc_message(msg);
            }
            OscPacket::Bundle(bundle) => {
                debug!("Received OSC bundle: {:?}", bundle);

                for msg in &bundle.content {
                    if let OscPacket::Message(msg) = msg {
                        handle_osc_message(msg.clone());
                    }
                }
            }
        },
    )
    .await
    .unwrap_or_else(|e| {
        error!("Failed to register OSC service: {}", e);
        ServiceStatusEvent {
            service: Service::Osc,
            status: ServiceStatus::Error(format!("Failed to register OSC service: {}", e)),
        }
        .emit(&osc_status_handle)
        .unwrap_or_else(|e| {
            error!("Failed to emit service status event: {}", e);
        });
    });

    Ok(())
}
