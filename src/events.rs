use serde::Serialize;

pub const EVENT_CHECKING: &str = "sparkle://checking";
pub const EVENT_UPDATE_AVAILABLE: &str = "sparkle://update-available";
pub const EVENT_UPDATE_NOT_AVAILABLE: &str = "sparkle://update-not-available";
pub const EVENT_DOWNLOADING: &str = "sparkle://downloading";
pub const EVENT_DOWNLOADED: &str = "sparkle://downloaded";
pub const EVENT_INSTALLING: &str = "sparkle://installing";
pub const EVENT_ERROR: &str = "sparkle://error";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionInfo {
    pub version: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorPayload {
    pub message: String,
    pub code: i64,
    pub domain: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct EmptyPayload {}
