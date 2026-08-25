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

/// Why Sparkle reported that no update is available.
///
/// Mirrors Sparkle's `SPUNoUpdateFoundReason`. A `SUNoUpdateError` (code 1001)
/// on its own only means "no eligible update in the effective update context",
/// so consumers must read this before telling a user they are up to date.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoUpdateReason {
    /// Sparkle could not attribute the outcome, or reported a reason this
    /// crate does not map yet. Check [`NoUpdateInfo::reason_code`].
    #[default]
    Unknown,
    /// The host is on the newest version the feed offers.
    OnLatestVersion,
    /// The host is newer than anything the feed offers.
    OnNewerThanLatestVersion,
    /// A newer version exists but requires a newer macOS.
    SystemIsTooOld,
    /// A newer version exists but does not support this macOS.
    SystemIsTooNew,
}

impl NoUpdateReason {
    /// Maps a raw `SPUNoUpdateFoundReason` value, keeping unmapped reasons
    /// (added by Sparkle versions newer than the bundled framework) distinct
    /// from a confirmed "you are up to date".
    pub(crate) fn from_raw(raw: i64) -> Self {
        match raw {
            1 => Self::OnLatestVersion,
            2 => Self::OnNewerThanLatestVersion,
            3 => Self::SystemIsTooOld,
            4 => Self::SystemIsTooNew,
            _ => Self::Unknown,
        }
    }
}

/// Sparkle's explanation for a no-update outcome, taken from the
/// `SUNoUpdateError` user info.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoUpdateInfo {
    pub reason: NoUpdateReason,
    /// Raw `SPUNoUpdateFoundReason` value, preserved so reasons this crate
    /// does not map yet stay diagnosable.
    pub reason_code: i64,
    /// Whether the check that produced this outcome was started by the user.
    pub user_initiated: bool,
    /// Newest item Sparkle could still see after channel filtering, including
    /// items rejected for OS requirements. `None` when the feed offered no
    /// applicable item at all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_item: Option<UpdateInfo>,
    /// Sparkle's localized explanation, e.g. which version needs which macOS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_suggestion: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub message: String,
    pub code: i64,
    pub domain: String,
    /// Present only for `SUNoUpdateError`, so consumers can tell "already on
    /// the newest version" apart from "no eligible update was found".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_update: Option<NoUpdateInfo>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_documented_sparkle_reason() {
        assert_eq!(NoUpdateReason::from_raw(0), NoUpdateReason::Unknown);
        assert_eq!(NoUpdateReason::from_raw(1), NoUpdateReason::OnLatestVersion);
        assert_eq!(
            NoUpdateReason::from_raw(2),
            NoUpdateReason::OnNewerThanLatestVersion
        );
        assert_eq!(NoUpdateReason::from_raw(3), NoUpdateReason::SystemIsTooOld);
        assert_eq!(NoUpdateReason::from_raw(4), NoUpdateReason::SystemIsTooNew);
    }

    #[test]
    fn unmapped_reasons_stay_distinguishable_from_up_to_date() {
        // Sparkle adds reasons over time; an unknown one must not be reported
        // as "on the latest version".
        assert_eq!(NoUpdateReason::from_raw(5), NoUpdateReason::Unknown);
        assert_eq!(NoUpdateReason::from_raw(-1), NoUpdateReason::Unknown);
    }

    #[test]
    fn serializes_reason_as_camel_case() {
        let json = serde_json::to_value(NoUpdateInfo {
            reason: NoUpdateReason::SystemIsTooOld,
            reason_code: 3,
            user_initiated: true,
            latest_item: None,
            recovery_suggestion: Some("At least macOS 14 is required.".to_string()),
        })
        .unwrap();

        assert_eq!(json["reason"], "systemIsTooOld");
        assert_eq!(json["reasonCode"], 3);
        assert_eq!(json["userInitiated"], true);
        assert_eq!(json["recoverySuggestion"], "At least macOS 14 is required.");
        assert!(json.get("latestItem").is_none());
    }

    #[test]
    fn error_payload_omits_no_update_for_unrelated_errors() {
        let json = serde_json::to_value(ErrorPayload {
            message: "The network connection was lost.".to_string(),
            code: -1005,
            domain: "NSURLErrorDomain".to_string(),
            no_update: None,
        })
        .unwrap();

        assert_eq!(json["code"], -1005);
        assert!(json.get("noUpdate").is_none());
    }

    #[test]
    fn error_payload_carries_no_update_context() {
        let json = serde_json::to_value(ErrorPayload {
            message: "You’re up to date!".to_string(),
            code: 1001,
            domain: "SUSparkleErrorDomain".to_string(),
            no_update: Some(NoUpdateInfo {
                reason: NoUpdateReason::OnNewerThanLatestVersion,
                reason_code: 2,
                ..Default::default()
            }),
        })
        .unwrap();

        assert_eq!(json["noUpdate"]["reason"], "onNewerThanLatestVersion");
        assert_eq!(json["noUpdate"]["userInitiated"], false);
    }
}
