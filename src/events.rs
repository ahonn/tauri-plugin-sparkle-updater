//! Event constants and payload types for Sparkle updater events.

use serde::Serialize;

// Event names
pub const EVENT_CHECKING: &str = "sparkle://checking";
pub const EVENT_UPDATE_AVAILABLE: &str = "sparkle://update-available";
pub const EVENT_UPDATE_NOT_AVAILABLE: &str = "sparkle://update-not-available";
pub const EVENT_DOWNLOADING: &str = "sparkle://downloading";
pub const EVENT_DOWNLOADED: &str = "sparkle://downloaded";
pub const EVENT_INSTALLING: &str = "sparkle://installing";
pub const EVENT_ERROR: &str = "sparkle://error";

/// Information about an available update.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
}

/// Error information.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorPayload {
    pub message: String,
}
