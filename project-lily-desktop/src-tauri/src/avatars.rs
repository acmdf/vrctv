use std::sync::Arc;

use glob::glob;
use log::info;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use vrchat_osc::rosc::{self, OscMessage, OscPacket};

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Avatar {
    pub id: String,
    pub name: String,
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_avatars() -> Result<Vec<Avatar>, String> {
    // List all Files in the C:\Users\*user*\AppData\LocalLow\VRChat\VRChat\OSC\ directory
    let mut avatars = Vec::new();
    let app_dirs = AppDirs::new(None, true).unwrap();
    let path = app_dirs
        .cache_dir
        .parent()
        .unwrap()
        .join("LocalLow/VRChat/VRChat/OSC"); // C:\Users\*user*\AppData\Local\..\LocalLow\VRChat\VRChat\OSC/

    info!("Looking for Avatars in {:?}", path);

    for path in glob(format!("{}/**/*.json", path.display()).as_str()).map_err(|e| e.to_string())? {
        match path {
            Ok(path) => {
                // Pull
                //   "id": "avtr_LyumaAv3Emulator_A",
                //   "name": "Cyber FT SFW Quest",
                let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let id_line = content
                    .lines()
                    .find(|line| line.trim_start().starts_with("\"id\":"));
                let name_line = content
                    .lines()
                    .find(|line| line.trim_start().starts_with("\"name\":"));

                if let (Some(id_line), Some(name_line)) = (id_line, name_line) {
                    let id = id_line
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .trim()
                        .trim_matches(',')
                        .trim_matches('"')
                        .to_string();
                    let name = name_line
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .trim()
                        .trim_matches(',')
                        .trim_matches('"')
                        .to_string();
                    avatars.push(Avatar { id, name });
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(avatars)
}

#[tauri::command]
#[specta::specta]
pub async fn change_avatar(app: AppHandle, avatar_id: &str) -> Result<(), String> {
    let osc = app.state::<Arc<vrchat_osc::VRChatOSC>>();
    let packet = OscPacket::Message(OscMessage {
        addr: "/avatar/change".into(),
        args: vec![rosc::OscType::String(avatar_id.into())],
    });

    let opener = app.opener();
    info!(
        "Open info: {:?}",
        opener.open_url(format!("vrcx://switchavatar/{}", avatar_id), None::<&str>)
    );

    // Send OSC Message to change avatar
    osc.send(packet.clone(), "VRChat-Client-*")
        .await
        .map_err(|e| e.to_string())
}
