use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct ConnectRequest {
    /// The random code given to the client to identify itself
    pub state_token: String,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct CodeRequest {}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct CodeResponse {
    /// The random code given to the client to identify itself
    pub state_token: String,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct ConnectResponse {
    pub has_twitch: bool,
    pub has_streamlabs: bool,
    pub twitch_id: Option<i32>,
    pub twitch_name: Option<String>,
    pub streamlabs_name: Option<String>,
    pub streamlabs_id: Option<String>,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct CustomReward {
    pub title: String,
    pub prompt: String,
    pub cost: u32,
    pub is_enabled: bool,
    pub is_global_cooldown_enabled: bool,
    pub global_cooldown_seconds: u32,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub enum TwitchTriggerRequest {
    ChannelPointsFulfill {
        request_id: i32,
        reward_id: String,
        redemption_id: String,
    },
    ChannelPointsCancel {
        request_id: i32,
        reward_id: String,
        redemption_id: String,
    },
    UpdateCustomRewards {
        request_id: i32,
        rewards: Vec<CustomReward>,
    },
    GetCustomRewards {
        request_id: i32,
    },
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct CustomRewardResponse {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub cost: u32,
    pub is_enabled: bool,
    pub is_global_cooldown_enabled: bool,
    pub global_cooldown_seconds: u32,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct Notify {
    pub title: String,
    pub message: String,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct ChangeAvatar {
    pub id: String,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct TwitchEvent {
    pub user_id: String,
    pub user_name: String,
    pub event: TwitchEventSource,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
#[serde(tag = "type")]
pub enum TwitchEventSource {
    ChannelPoints { reward_id: String, reward_name: String },
    BitDonation { amount: u32, message: Option<String>, emojis: Option<Vec<String>> },
    Whisper { message: String },
    Message { message: String },
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct StreamLabsEvent {
    pub event_key: String,
    pub event_source: StreamLabsEventSource,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub enum StreamLabsEventSource {
    Donation,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct ErrorMessage {
    pub request_id: i32,
    pub source: String,
    pub message: String,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
pub struct TaskResponse {
    pub request_id: i32,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ServerMessage {
    ConnectResponse(ConnectResponse),
    CodeResponse(CodeResponse),
    CustomRewards {
        rewards: Vec<CustomRewardResponse>
    },
    Notify(Notify),
    ChangeAvatar(ChangeAvatar),
    TwitchEvent(TwitchEvent),
    StreamLabsEvent(StreamLabsEvent),
    Error(ErrorMessage),
    TaskResponse(TaskResponse),
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ClientMessage {
    Connect(ConnectRequest),
    CodeRequest(CodeRequest),
    TwitchTrigger(TwitchTriggerRequest),
}
