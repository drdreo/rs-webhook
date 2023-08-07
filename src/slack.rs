use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlVerificationEvent {
    pub token: String,
    pub challenge: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkSharedEvent {
    pub token: String,
    #[serde(rename = "team_id")]
    pub team_id: Option<String>,
    #[serde(rename = "api_app_id")]
    pub api_app_id: String,
    pub event: Event,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "authed_users")]
    pub authed_users: Option<Vec<String>>,
    #[serde(rename = "event_id")]
    pub event_id: String,
    #[serde(rename = "event_time")]
    pub event_time: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "type")]
    pub type_field: String,
    pub channel: String,
    #[serde(rename = "is_bot_user_member")]
    pub is_bot_user_member: bool,
    pub user: Option<String>,
    #[serde(rename = "message_ts")]
    pub message_ts: String,
    #[serde(rename = "unfurl_id")]
    pub unfurl_id: String,
    #[serde(rename = "thread_ts")]
    pub thread_ts: Option<String>,
    pub source: String,
    pub links: Vec<Link>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub domain: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlackEvent {
    UrlVerification(UrlVerificationEvent),
    LinkShared(LinkSharedEvent),
}
