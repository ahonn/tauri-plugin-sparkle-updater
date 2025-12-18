use serde::Serialize;

pub const EVENT_DID_FINISH_LOADING_APPCAST: &str = "sparkle://did-finish-loading-appcast";
pub const EVENT_DID_FIND_VALID_UPDATE: &str = "sparkle://did-find-valid-update";
pub const EVENT_DID_NOT_FIND_UPDATE: &str = "sparkle://did-not-find-update";
pub const EVENT_WILL_DOWNLOAD_UPDATE: &str = "sparkle://will-download-update";
pub const EVENT_DID_DOWNLOAD_UPDATE: &str = "sparkle://did-download-update";
pub const EVENT_WILL_INSTALL_UPDATE: &str = "sparkle://will-install-update";
pub const EVENT_DID_ABORT_WITH_ERROR: &str = "sparkle://did-abort-with-error";
pub const EVENT_DID_FINISH_UPDATE_CYCLE: &str = "sparkle://did-finish-update-cycle";
pub const EVENT_FAILED_TO_DOWNLOAD_UPDATE: &str = "sparkle://failed-to-download-update";
pub const EVENT_USER_DID_CANCEL_DOWNLOAD: &str = "sparkle://user-did-cancel-download";
pub const EVENT_WILL_EXTRACT_UPDATE: &str = "sparkle://will-extract-update";
pub const EVENT_DID_EXTRACT_UPDATE: &str = "sparkle://did-extract-update";
pub const EVENT_WILL_RELAUNCH_APPLICATION: &str = "sparkle://will-relaunch-application";
pub const EVENT_USER_DID_MAKE_CHOICE: &str = "sparkle://user-did-make-choice";
pub const EVENT_WILL_SCHEDULE_UPDATE_CHECK: &str = "sparkle://will-schedule-update-check";
pub const EVENT_WILL_NOT_SCHEDULE_UPDATE_CHECK: &str = "sparkle://will-not-schedule-update-check";
pub const EVENT_SHOULD_PROMPT_FOR_PERMISSION: &str = "sparkle://should-prompt-for-permission";
pub const EVENT_WILL_INSTALL_UPDATE_ON_QUIT: &str = "sparkle://will-install-update-on-quit";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_notes_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_system_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<f64>,
    pub is_critical: bool,
    pub is_major_upgrade: bool,
    pub is_information_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_system_version: Option<String>,
    pub minimum_os_version_ok: bool,
    pub maximum_os_version_ok: bool,
    pub installation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phased_rollout_interval: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_release_notes_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_autoupdate_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_skipped_upgrades_below_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_description_format: Option<String>,
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCycleInfo {
    pub update_check: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorPayload>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFailedInfo {
    pub version: String,
    pub error: ErrorPayload,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserChoiceInfo {
    pub choice: String,
    pub version: String,
    pub stage: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInfo {
    pub delay: f64,
}
