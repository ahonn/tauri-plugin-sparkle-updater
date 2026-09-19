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

/// Informational notifications from Sparkle. Decision callbacks are configured separately.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum UpdateEvent {
    DidFinishLoadingAppcast,
    DidFindValidUpdate(UpdateInfo),
    DidNotFindUpdate(NoUpdateInfo),
    WillDownloadUpdate(VersionInfo),
    DidDownloadUpdate(VersionInfo),
    WillInstallUpdate(VersionInfo),
    DidAbortWithError(ErrorPayload),
    DidFinishUpdateCycle(UpdateCycleInfo),
    FailedToDownloadUpdate(DownloadFailedInfo),
    UserDidCancelDownload,
    WillExtractUpdate(VersionInfo),
    DidExtractUpdate(VersionInfo),
    WillRelaunchApplication,
    UserDidMakeChoice(UserChoiceInfo),
    WillScheduleUpdateCheck(ScheduleInfo),
    WillNotScheduleUpdateCheck,
    WillInstallUpdateOnQuit(VersionInfo),
}

impl UpdateEvent {
    /// The stable event name used by the Tauri adapter.
    pub fn name(&self) -> &'static str {
        match self {
            Self::DidFinishLoadingAppcast => EVENT_DID_FINISH_LOADING_APPCAST,
            Self::DidFindValidUpdate(..) => EVENT_DID_FIND_VALID_UPDATE,
            Self::DidNotFindUpdate(..) => EVENT_DID_NOT_FIND_UPDATE,
            Self::WillDownloadUpdate(..) => EVENT_WILL_DOWNLOAD_UPDATE,
            Self::DidDownloadUpdate(..) => EVENT_DID_DOWNLOAD_UPDATE,
            Self::WillInstallUpdate(..) => EVENT_WILL_INSTALL_UPDATE,
            Self::DidAbortWithError(..) => EVENT_DID_ABORT_WITH_ERROR,
            Self::DidFinishUpdateCycle(..) => EVENT_DID_FINISH_UPDATE_CYCLE,
            Self::FailedToDownloadUpdate(..) => EVENT_FAILED_TO_DOWNLOAD_UPDATE,
            Self::UserDidCancelDownload => EVENT_USER_DID_CANCEL_DOWNLOAD,
            Self::WillExtractUpdate(..) => EVENT_WILL_EXTRACT_UPDATE,
            Self::DidExtractUpdate(..) => EVENT_DID_EXTRACT_UPDATE,
            Self::WillRelaunchApplication => EVENT_WILL_RELAUNCH_APPLICATION,
            Self::UserDidMakeChoice(..) => EVENT_USER_DID_MAKE_CHOICE,
            Self::WillScheduleUpdateCheck(..) => EVENT_WILL_SCHEDULE_UPDATE_CHECK,
            Self::WillNotScheduleUpdateCheck => EVENT_WILL_NOT_SCHEDULE_UPDATE_CHECK,
            Self::WillInstallUpdateOnQuit(..) => EVENT_WILL_INSTALL_UPDATE_ON_QUIT,
        }
    }

    /// Serialize only the payload, preserving the adapter's existing wire format.
    pub fn payload(&self) -> serde_json::Value {
        match self {
            Self::DidFinishLoadingAppcast => serde_json::json!({}),
            Self::DidFindValidUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::DidNotFindUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::WillDownloadUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::DidDownloadUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::WillInstallUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::DidAbortWithError(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::DidFinishUpdateCycle(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::FailedToDownloadUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::UserDidCancelDownload => serde_json::json!({}),
            Self::WillExtractUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::DidExtractUpdate(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::WillRelaunchApplication => serde_json::json!({}),
            Self::UserDidMakeChoice(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::WillScheduleUpdateCheck(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
            Self::WillNotScheduleUpdateCheck => serde_json::json!({}),
            Self::WillInstallUpdateOnQuit(payload) => {
                serde_json::to_value(payload).expect("event payloads are serializable")
            }
        }
    }
}

/// The update's current stage, including values introduced by future Sparkle versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserUpdateStage {
    NotDownloaded,
    Downloaded,
    Installing,
    Unknown(isize),
}

impl UserUpdateStage {
    pub(crate) fn from_raw(value: isize) -> Self {
        match value {
            0 => Self::NotDownloaded,
            1 => Self::Downloaded,
            2 => Self::Installing,
            value => Self::Unknown(value),
        }
    }

    pub(crate) fn wire_name(self) -> &'static str {
        match self {
            Self::NotDownloaded => "notDownloaded",
            Self::Downloaded => "downloaded",
            Self::Installing => "installing",
            Self::Unknown(_) => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserUpdateState {
    pub stage: UserUpdateStage,
    pub user_initiated: bool,
}

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
    /// A newer version exists but requires an Apple silicon Mac, and this is
    /// an Intel Mac.
    HardwareDoesNotSupportArm64,
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
            5 => Self::HardwareDoesNotSupportArm64,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_suggestion: Option<String>,
    /// The errors this one wraps, outermost first. Sparkle reports most
    /// installer failures as a generic `SUInstallationError` (4005) whose
    /// actual cause is only found here.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub underlying: Vec<UnderlyingError>,
}

/// One link of an `NSError`'s `NSUnderlyingErrorKey` chain.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnderlyingError {
    pub message: String,
    pub code: i64,
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
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
        assert_eq!(
            NoUpdateReason::from_raw(5),
            NoUpdateReason::HardwareDoesNotSupportArm64
        );
    }

    #[test]
    fn unmapped_reasons_stay_distinguishable_from_up_to_date() {
        // Sparkle adds reasons over time; an unknown one must not be reported
        // as "on the latest version".
        assert_eq!(NoUpdateReason::from_raw(6), NoUpdateReason::Unknown);
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

        // The digits must stay attached to "Arm" so the string matches the
        // guest-js `NoUpdateReason` union.
        assert_eq!(
            serde_json::to_value(NoUpdateReason::HardwareDoesNotSupportArm64).unwrap(),
            "hardwareDoesNotSupportArm64"
        );
    }

    #[test]
    fn error_payload_omits_no_update_for_unrelated_errors() {
        let json = serde_json::to_value(ErrorPayload {
            message: "The network connection was lost.".to_string(),
            code: -1005,
            domain: "NSURLErrorDomain".to_string(),
            no_update: None,
            failure_reason: None,
            recovery_suggestion: None,
            underlying: vec![],
        })
        .unwrap();

        assert_eq!(json["code"], -1005);
        assert!(json.get("noUpdate").is_none());
        assert!(json.get("failureReason").is_none());
        assert!(json.get("recoverySuggestion").is_none());
        assert!(json.get("underlying").is_none());
    }

    #[test]
    fn error_payload_serializes_its_underlying_chain() {
        let json = serde_json::to_value(ErrorPayload {
            message: "An error occurred while running the updater.".to_string(),
            code: 4005,
            domain: "SUSparkleErrorDomain".to_string(),
            no_update: None,
            failure_reason: None,
            recovery_suggestion: Some("Please try again later.".to_string()),
            underlying: vec![UnderlyingError {
                message: "You don't have permission.".to_string(),
                code: 513,
                domain: "NSCocoaErrorDomain".to_string(),
                failure_reason: Some("The folder is not writable.".to_string()),
            }],
        })
        .unwrap();

        assert_eq!(json["recoverySuggestion"], "Please try again later.");
        assert_eq!(json["underlying"][0]["code"], 513);
        assert_eq!(json["underlying"][0]["domain"], "NSCocoaErrorDomain");
        assert_eq!(
            json["underlying"][0]["failureReason"],
            "The folder is not writable."
        );
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
            failure_reason: None,
            recovery_suggestion: None,
            underlying: vec![],
        })
        .unwrap();

        assert_eq!(json["noUpdate"]["reason"], "onNewerThanLatestVersion");
        assert_eq!(json["noUpdate"]["userInitiated"], false);
    }
}

#[cfg(test)]
mod typed_event_tests {
    use super::*;

    #[test]
    fn typed_events_preserve_adapter_names_and_payloads() {
        let event = UpdateEvent::DidDownloadUpdate(VersionInfo {
            version: "2.0".into(),
        });
        assert_eq!(event.name(), EVENT_DID_DOWNLOAD_UPDATE);
        assert_eq!(event.payload(), serde_json::json!({"version": "2.0"}));
        assert_eq!(
            UpdateEvent::UserDidCancelDownload.payload(),
            serde_json::json!({})
        );
    }

    #[test]
    fn unknown_update_stages_are_not_reported_as_installing() {
        assert_eq!(UserUpdateStage::from_raw(19), UserUpdateStage::Unknown(19));
        assert_eq!(UserUpdateStage::from_raw(19).wire_name(), "unknown");
    }
}
