use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::error;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, ClassType, DeclaredClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSArray, NSDictionary, NSMutableSet, NSNumber, NSSet, NSString, NSURL};
use serde::Serialize;
use serde_json::Value;

use super::bindings::SPUAppcastItem;
use crate::events::UpdateInfo;
use crate::events::{
    DownloadFailedInfo, EmptyPayload, ErrorPayload, NoUpdateInfo, NoUpdateReason, ScheduleInfo,
    UpdateCycleInfo, UserChoiceInfo, VersionInfo, EVENT_DID_ABORT_WITH_ERROR,
    EVENT_DID_DOWNLOAD_UPDATE, EVENT_DID_EXTRACT_UPDATE, EVENT_DID_FIND_VALID_UPDATE,
    EVENT_DID_FINISH_LOADING_APPCAST, EVENT_DID_FINISH_UPDATE_CYCLE, EVENT_DID_NOT_FIND_UPDATE,
    EVENT_FAILED_TO_DOWNLOAD_UPDATE, EVENT_USER_DID_CANCEL_DOWNLOAD, EVENT_USER_DID_MAKE_CHOICE,
    EVENT_WILL_DOWNLOAD_UPDATE, EVENT_WILL_EXTRACT_UPDATE, EVENT_WILL_INSTALL_UPDATE,
    EVENT_WILL_INSTALL_UPDATE_ON_QUIT, EVENT_WILL_NOT_SCHEDULE_UPDATE_CHECK,
    EVENT_WILL_RELAUNCH_APPLICATION, EVENT_WILL_SCHEDULE_UPDATE_CHECK,
};

pub type EventEmitter = Arc<dyn Fn(&str, Value) + Send + Sync>;
pub type EventCallback = Arc<dyn Fn(&str, &Value) + Send + Sync>;

pub struct DelegateIvars {
    emitter: RefCell<Option<EventEmitter>>,
    event_callback: RefCell<Option<EventCallback>>,
    allowed_channels: RefCell<Option<Vec<String>>>,
    feed_url_override: RefCell<Option<String>>,
    feed_parameters: RefCell<Option<HashMap<String, String>>>,
    should_download_release_notes: RefCell<bool>,
    should_relaunch: RefCell<bool>,
    may_check_for_updates: RefCell<bool>,
    should_proceed_with_update: RefCell<bool>,
    decryption_password: RefCell<Option<String>>,
    last_found_update: RefCell<Option<UpdateInfo>>,
    download_request_headers: RefCell<Option<HashMap<String, String>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TauriSparkleDelegate"]
    #[ivars = DelegateIvars]
    pub struct SparkleDelegate;

    impl SparkleDelegate {
        #[unsafe(method(updater:didFinishLoadingAppcast:))]
        fn updater_did_finish_loading_appcast(
            &self,
            _updater: &NSObject,
            _appcast: &NSObject,
        ) {
            self.emit(EVENT_DID_FINISH_LOADING_APPCAST, &EmptyPayload {});
        }

        #[unsafe(method(updater:didFindValidUpdate:))]
        fn updater_did_find_valid_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            let update_info = update_info_from_item(item);

            *self.ivars().last_found_update.borrow_mut() = Some(update_info.clone());
            self.emit(EVENT_DID_FIND_VALID_UPDATE, &update_info);
        }

        // Sparkle prefers this over `updaterDidNotFindUpdate:`, and it is the
        // only variant that carries why no update was found. Implementing the
        // bare variant instead silently reduces every outcome -- OS too old,
        // newer than the feed, nothing eligible -- to "up to date".
        #[unsafe(method(updaterDidNotFindUpdate:error:))]
        fn updater_did_not_find_update(&self, _updater: &NSObject, error: &NSObject) {
            self.emit(
                EVENT_DID_NOT_FIND_UPDATE,
                &no_update_info(error).unwrap_or_default(),
            );
        }

        #[unsafe(method(updater:willDownloadUpdate:withRequest:))]
        fn updater_will_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
            request: &NSObject,
        ) {
            if let Some(ref headers) = *self.ivars().download_request_headers.borrow() {
                for (key, value) in headers {
                    let ns_value = NSString::from_str(value);
                    let ns_field = NSString::from_str(key);
                    let _: () =
                        unsafe { msg_send![request, setValue: &*ns_value, forHTTPHeaderField: &*ns_field] };
                }
            }

            self.emit(EVENT_WILL_DOWNLOAD_UPDATE, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:didDownloadUpdate:))]
        fn updater_did_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            self.emit(EVENT_DID_DOWNLOAD_UPDATE, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:willInstallUpdate:))]
        fn updater_will_install_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            self.emit(EVENT_WILL_INSTALL_UPDATE, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:didAbortWithError:))]
        fn updater_did_abort_with_error(
            &self,
            _updater: &NSObject,
            ns_error: &NSObject,
        ) {
            self.emit(EVENT_DID_ABORT_WITH_ERROR, &error_payload(ns_error));
        }

        #[unsafe(method(updater:didFinishUpdateCycleForUpdateCheck:error:))]
        fn updater_did_finish_update_cycle(
            &self,
            _updater: &NSObject,
            update_check: isize,
            error: Option<&NSObject>,
        ) {
            let update_check_str = match update_check {
                0 => "userInitiated",
                1 => "background",
                _ => "information",
            };
            self.emit(EVENT_DID_FINISH_UPDATE_CYCLE, &UpdateCycleInfo {
                update_check: update_check_str.to_string(),
                error: error.map(error_payload),
            });
        }

        #[unsafe(method(updater:failedToDownloadUpdate:error:))]
        fn updater_failed_to_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
            ns_error: &NSObject,
        ) {
            self.emit(EVENT_FAILED_TO_DOWNLOAD_UPDATE, &DownloadFailedInfo {
                version: item.display_version_string().to_string(),
                error: error_payload(ns_error),
            });
        }

        #[unsafe(method(userDidCancelDownload:))]
        fn user_did_cancel_download(&self, _updater: &NSObject) {
            self.emit(EVENT_USER_DID_CANCEL_DOWNLOAD, &EmptyPayload {});
        }

        #[unsafe(method(updater:willExtractUpdate:))]
        fn updater_will_extract_update(&self, _updater: &NSObject, item: &SPUAppcastItem) {
            self.emit(EVENT_WILL_EXTRACT_UPDATE, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:didExtractUpdate:))]
        fn updater_did_extract_update(&self, _updater: &NSObject, item: &SPUAppcastItem) {
            self.emit(EVENT_DID_EXTRACT_UPDATE, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updaterWillRelaunchApplication:))]
        fn updater_will_relaunch_application(&self, _updater: &NSObject) {
            self.emit(EVENT_WILL_RELAUNCH_APPLICATION, &EmptyPayload {});
        }

        #[unsafe(method(updater:userDidMakeChoice:forUpdate:state:))]
        fn updater_user_did_make_choice(
            &self,
            _updater: &NSObject,
            choice: isize,
            item: &SPUAppcastItem,
            state: isize,
        ) {
            let choice_str = match choice {
                0 => "skip",
                1 => "install",
                _ => "dismiss",
            };
            let stage_str = match state {
                0 => "notDownloaded",
                1 => "downloaded",
                _ => "installing",
            };
            self.emit(EVENT_USER_DID_MAKE_CHOICE, &UserChoiceInfo {
                choice: choice_str.to_string(),
                version: item.display_version_string().to_string(),
                stage: stage_str.to_string(),
            });
        }

        #[unsafe(method(updater:willScheduleUpdateCheckAfterDelay:))]
        fn updater_will_schedule_update_check(&self, _updater: &NSObject, delay: f64) {
            self.emit(EVENT_WILL_SCHEDULE_UPDATE_CHECK, &ScheduleInfo { delay });
        }

        #[unsafe(method(updaterWillNotScheduleUpdateCheck:))]
        fn updater_will_not_schedule_update_check(&self, _updater: &NSObject) {
            self.emit(EVENT_WILL_NOT_SCHEDULE_UPDATE_CHECK, &EmptyPayload {});
        }

        #[unsafe(method(updaterShouldPromptForPermissionToCheckForUpdates:))]
        fn updater_should_prompt_for_permission(&self, _updater: &NSObject) -> bool {
            true
        }

        #[unsafe(method(updater:willInstallUpdateOnQuit:immediateInstallationBlock:))]
        fn updater_will_install_update_on_quit(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
            _handler: &NSObject,
        ) -> bool {
            self.emit(EVENT_WILL_INSTALL_UPDATE_ON_QUIT, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
            true
        }

        #[unsafe(method(allowedChannelsForUpdater:))]
        fn allowed_channels_for_updater(
            &self,
            _updater: &NSObject,
        ) -> *mut NSSet<NSString> {
            let channels = self.ivars().allowed_channels.borrow();
            match channels.as_ref() {
                Some(ch) => {
                    let set = NSMutableSet::<NSString>::new();
                    for channel in ch {
                        let ns_str = NSString::from_str(channel);
                        let _: () = unsafe { msg_send![&set, addObject: &*ns_str] };
                    }
                    Retained::autorelease_return(Retained::into_super(set))
                }
                None => std::ptr::null_mut(),
            }
        }

        #[unsafe(method(feedURLStringForUpdater:))]
        fn feed_url_string_for_updater(
            &self,
            _updater: &NSObject,
        ) -> *mut NSString {
            let url = self.ivars().feed_url_override.borrow();
            match url.as_ref() {
                Some(u) => Retained::autorelease_return(NSString::from_str(u)),
                None => std::ptr::null_mut(),
            }
        }

        #[unsafe(method(feedParametersForUpdater:sendingSystemProfile:))]
        fn feed_parameters_for_updater(
            &self,
            _updater: &NSObject,
            _sending_profile: bool,
        ) -> *mut NSArray<NSDictionary<NSString, NSString>> {
            let params = self.ivars().feed_parameters.borrow();
            let array = match params.as_ref() {
                Some(p) if !p.is_empty() => {
                    let mut dicts: Vec<Retained<NSDictionary<NSString, NSString>>> = Vec::new();
                    for (key, value) in p {
                        let key_str = NSString::from_str("key");
                        let value_str = NSString::from_str("value");
                        let k = NSString::from_str(key);
                        let v = NSString::from_str(value);
                        let dict = NSDictionary::from_slices(
                            &[&*key_str, &*value_str],
                            &[&*k, &*v],
                        );
                        dicts.push(dict);
                    }
                    let refs: Vec<&NSDictionary<NSString, NSString>> =
                        dicts.iter().map(|d| d.as_ref()).collect();
                    NSArray::from_slice(&refs)
                }
                _ => NSArray::new(),
            };
            Retained::autorelease_return(array)
        }

        #[unsafe(method(updater:shouldDownloadReleaseNotesForUpdate:))]
        fn updater_should_download_release_notes(
            &self,
            _updater: &NSObject,
            _item: &SPUAppcastItem,
        ) -> bool {
            *self.ivars().should_download_release_notes.borrow()
        }

        #[unsafe(method(updaterShouldRelaunchApplication:))]
        fn updater_should_relaunch_application(&self, _updater: &NSObject) -> bool {
            *self.ivars().should_relaunch.borrow()
        }

        #[unsafe(method(updater:mayPerformUpdateCheck:error:))]
        fn updater_may_perform_update_check(
            &self,
            _updater: &NSObject,
            _update_check: isize,
            _error: *mut *mut NSObject,
        ) -> bool {
            *self.ivars().may_check_for_updates.borrow()
        }

        #[unsafe(method(updater:shouldProceedWithUpdate:updateCheck:error:))]
        fn updater_should_proceed_with_update(
            &self,
            _updater: &NSObject,
            _item: &SPUAppcastItem,
            _update_check: isize,
            _error: *mut *mut NSObject,
        ) -> bool {
            *self.ivars().should_proceed_with_update.borrow()
        }

        #[unsafe(method(decryptionPasswordForUpdater:))]
        fn decryption_password_for_updater(
            &self,
            _updater: &NSObject,
        ) -> *mut NSString {
            let password = self.ivars().decryption_password.borrow();
            match password.as_ref() {
                Some(p) => Retained::autorelease_return(NSString::from_str(p)),
                None => std::ptr::null_mut(),
            }
        }
    }
);

fn nserror_description(error: &NSObject) -> String {
    let desc: Retained<NSString> = unsafe { msg_send![error, localizedDescription] };
    desc.to_string()
}

fn nserror_domain(error: &NSObject) -> String {
    let domain: Retained<NSString> = unsafe { msg_send![error, domain] };
    domain.to_string()
}

fn nserror_recovery_suggestion(error: &NSObject) -> Option<String> {
    let suggestion: Option<Retained<NSString>> =
        unsafe { msg_send![error, localizedRecoverySuggestion] };
    suggestion.map(|s| s.to_string())
}

// Sparkle exports these user info keys; linking them keeps the lookup in step
// with the framework instead of hardcoding its string values.
#[link(name = "Sparkle", kind = "framework")]
extern "C" {
    static SPUNoUpdateFoundReasonKey: &'static NSString;
    static SPULatestAppcastItemFoundKey: &'static NSString;
    static SPUNoUpdateFoundUserInitiatedKey: &'static NSString;
}

fn user_info_value(
    user_info: &NSDictionary<NSString, AnyObject>,
    key: &NSString,
) -> Option<Retained<AnyObject>> {
    unsafe { msg_send![user_info, objectForKey: key] }
}

fn user_info_number(user_info: &NSDictionary<NSString, AnyObject>, key: &NSString) -> Option<i64> {
    let value = user_info_value(user_info, key)?;
    let is_number: bool = unsafe { msg_send![&*value, isKindOfClass: NSNumber::class()] };
    is_number.then(|| unsafe { msg_send![&*value, longLongValue] })
}

fn user_info_appcast_item(
    user_info: &NSDictionary<NSString, AnyObject>,
    key: &NSString,
) -> Option<UpdateInfo> {
    let value = user_info_value(user_info, key)?;
    let is_item: bool = unsafe { msg_send![&*value, isKindOfClass: SPUAppcastItem::class()] };
    if !is_item {
        return None;
    }

    // Checked above, so the object really is an appcast item.
    let item: &SPUAppcastItem = unsafe { &*Retained::as_ptr(&value).cast() };
    Some(update_info_from_item(item))
}

/// Reads Sparkle's no-update context out of an `NSError`.
///
/// Returns `None` for anything that is not a no-update outcome: the reason key
/// is what identifies one, so callers never have to match on error code 1001.
fn no_update_info(error: &NSObject) -> Option<NoUpdateInfo> {
    let user_info: Option<Retained<NSDictionary<NSString, AnyObject>>> =
        unsafe { msg_send![error, userInfo] };
    let user_info = user_info?;

    let reason_code = user_info_number(&user_info, unsafe { SPUNoUpdateFoundReasonKey })?;

    Some(NoUpdateInfo {
        reason: NoUpdateReason::from_raw(reason_code),
        reason_code,
        user_initiated: user_info_number(&user_info, unsafe { SPUNoUpdateFoundUserInitiatedKey })
            .is_some_and(|value| value != 0),
        latest_item: user_info_appcast_item(&user_info, unsafe { SPULatestAppcastItemFoundKey }),
        recovery_suggestion: nserror_recovery_suggestion(error),
    })
}

fn error_payload(error: &NSObject) -> ErrorPayload {
    ErrorPayload {
        message: nserror_description(error),
        code: unsafe { msg_send![error, code] },
        domain: nserror_domain(error),
        no_update: no_update_info(error),
    }
}

fn update_info_from_item(item: &SPUAppcastItem) -> UpdateInfo {
    let url_to_string = |url: &NSURL| -> String {
        let abs: Option<Retained<NSString>> = unsafe { msg_send![url, absoluteString] };
        abs.map(|s| s.to_string()).unwrap_or_default()
    };

    let number_to_f64 = |num: &NSNumber| -> f64 { unsafe { msg_send![num, doubleValue] } };

    UpdateInfo {
        version: item.display_version_string().to_string(),
        release_notes: item.item_description().map(|s| s.to_string()),
        title: item.title().map(|s| s.to_string()),
        release_notes_url: item.release_notes_url().map(|u| url_to_string(&u)),
        info_url: item.info_url().map(|u| url_to_string(&u)),
        minimum_system_version: item.minimum_system_version().map(|s| s.to_string()),
        channel: item.channel().map(|s| s.to_string()),
        date: item.date().map(|d| {
            let seconds: f64 = unsafe { msg_send![&d, timeIntervalSince1970] };
            seconds * 1000.0
        }),
        is_critical: item.is_critical_update(),
        is_major_upgrade: item.is_major_upgrade(),
        is_information_only: item.is_information_only_update(),
        maximum_system_version: item.maximum_system_version().map(|s| s.to_string()),
        minimum_os_version_ok: item.minimum_operating_system_version_is_ok(),
        maximum_os_version_ok: item.maximum_operating_system_version_is_ok(),
        installation_type: item.installation_type().to_string(),
        phased_rollout_interval: item.phased_rollout_interval().map(|n| number_to_f64(&n)),
        full_release_notes_url: item.full_release_notes_url().map(|u| url_to_string(&u)),
        minimum_autoupdate_version: item.minimum_autoupdate_version().map(|s| s.to_string()),
        ignore_skipped_upgrades_below_version: item
            .ignore_skipped_upgrades_below_version()
            .map(|s| s.to_string()),
        date_string: item.date_string().map(|s| s.to_string()),
        item_description_format: item.item_description_format().map(|s| s.to_string()),
    }
}

impl SparkleDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm);
        let this = this.set_ivars(DelegateIvars {
            emitter: RefCell::new(None),
            event_callback: RefCell::new(None),
            allowed_channels: RefCell::new(None),
            feed_url_override: RefCell::new(None),
            feed_parameters: RefCell::new(None),
            should_download_release_notes: RefCell::new(true),
            should_relaunch: RefCell::new(true),
            may_check_for_updates: RefCell::new(true),
            should_proceed_with_update: RefCell::new(true),
            decryption_password: RefCell::new(None),
            last_found_update: RefCell::new(None),
            download_request_headers: RefCell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    pub fn set_emitter(&self, emitter: EventEmitter) {
        *self.ivars().emitter.borrow_mut() = Some(emitter);
    }

    pub fn set_event_callback(&self, callback: Option<EventCallback>) {
        *self.ivars().event_callback.borrow_mut() = callback;
    }

    fn emit<T: Serialize>(&self, event: &str, payload: &T) {
        if let Some(ref emitter) = *self.ivars().emitter.borrow() {
            match serde_json::to_value(payload) {
                Ok(value) => {
                    if let Some(ref callback) = *self.ivars().event_callback.borrow() {
                        callback(event, &value);
                    }
                    emitter(event, value)
                }
                Err(e) => error!("Failed to serialize event payload: {}", e),
            }
        }
    }

    pub fn allowed_channels(&self) -> Option<Vec<String>> {
        self.ivars().allowed_channels.borrow().clone()
    }

    pub fn set_allowed_channels(&self, channels: Option<Vec<String>>) {
        *self.ivars().allowed_channels.borrow_mut() = channels;
    }

    pub fn feed_url_override(&self) -> Option<String> {
        self.ivars().feed_url_override.borrow().clone()
    }

    pub fn set_feed_url_override(&self, url: Option<String>) {
        *self.ivars().feed_url_override.borrow_mut() = url;
    }

    pub fn feed_parameters(&self) -> Option<HashMap<String, String>> {
        self.ivars().feed_parameters.borrow().clone()
    }

    pub fn set_feed_parameters(&self, params: Option<HashMap<String, String>>) {
        *self.ivars().feed_parameters.borrow_mut() = params;
    }

    pub fn should_download_release_notes(&self) -> bool {
        *self.ivars().should_download_release_notes.borrow()
    }

    pub fn set_should_download_release_notes(&self, enabled: bool) {
        *self.ivars().should_download_release_notes.borrow_mut() = enabled;
    }

    pub fn should_relaunch(&self) -> bool {
        *self.ivars().should_relaunch.borrow()
    }

    pub fn set_should_relaunch(&self, enabled: bool) {
        *self.ivars().should_relaunch.borrow_mut() = enabled;
    }

    pub fn may_check_for_updates(&self) -> bool {
        *self.ivars().may_check_for_updates.borrow()
    }

    pub fn set_may_check_for_updates(&self, enabled: bool) {
        *self.ivars().may_check_for_updates.borrow_mut() = enabled;
    }

    pub fn should_proceed_with_update(&self) -> bool {
        *self.ivars().should_proceed_with_update.borrow()
    }

    pub fn set_should_proceed_with_update(&self, enabled: bool) {
        *self.ivars().should_proceed_with_update.borrow_mut() = enabled;
    }

    pub fn decryption_password(&self) -> Option<String> {
        self.ivars().decryption_password.borrow().clone()
    }

    pub fn set_decryption_password(&self, password: Option<String>) {
        *self.ivars().decryption_password.borrow_mut() = password;
    }

    pub fn last_found_update(&self) -> Option<UpdateInfo> {
        self.ivars().last_found_update.borrow().clone()
    }

    pub fn download_request_headers(&self) -> Option<HashMap<String, String>> {
        self.ivars().download_request_headers.borrow().clone()
    }

    pub fn set_download_request_headers(&self, headers: Option<HashMap<String, String>>) {
        *self.ivars().download_request_headers.borrow_mut() = headers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_foundation::{NSError, NSMutableDictionary};

    fn number(value: i64) -> Retained<NSNumber> {
        unsafe { msg_send![NSNumber::class(), numberWithLongLong: value] }
    }

    fn error_with(
        entries: &[(&NSString, &AnyObject)],
        domain: &str,
        code: isize,
    ) -> Retained<NSObject> {
        let user_info: Retained<NSMutableDictionary<NSString, AnyObject>> = unsafe {
            msg_send![
                NSMutableDictionary::<NSString, AnyObject>::class(),
                dictionary
            ]
        };
        for (key, value) in entries {
            unsafe {
                let _: () = msg_send![&*user_info, setObject: *value, forKey: *key];
            }
        }

        let domain = NSString::from_str(domain);
        unsafe {
            msg_send![NSError::class(), errorWithDomain: &*domain, code: code, userInfo: &*user_info]
        }
    }

    #[test]
    fn reads_reason_from_sparkle_user_info_keys() {
        let reason = number(3);
        let user_initiated = number(1);
        let suggestion = NSString::from_str("At least macOS 14 is required.");
        let suggestion_key = NSString::from_str("NSLocalizedRecoverySuggestion");

        let error = error_with(
            &[
                (unsafe { SPUNoUpdateFoundReasonKey }, &reason),
                (unsafe { SPUNoUpdateFoundUserInitiatedKey }, &user_initiated),
                (&suggestion_key, &suggestion),
            ],
            "SUSparkleErrorDomain",
            1001,
        );

        let info = no_update_info(&error).expect("a no-update error carries its reason");

        assert_eq!(info.reason, NoUpdateReason::SystemIsTooOld);
        assert_eq!(info.reason_code, 3);
        assert!(info.user_initiated);
        assert!(info.latest_item.is_none());
        assert_eq!(
            info.recovery_suggestion.as_deref(),
            Some("At least macOS 14 is required.")
        );
    }

    #[test]
    fn ignores_errors_without_a_no_update_reason() {
        let error = error_with(&[], "NSURLErrorDomain", -1005);

        assert!(no_update_info(&error).is_none());

        let payload = error_payload(&error);
        assert_eq!(payload.code, -1005);
        assert_eq!(payload.domain, "NSURLErrorDomain");
        assert!(payload.no_update.is_none());
    }

    #[test]
    fn error_payload_attaches_no_update_context() {
        let reason = number(2);
        let error = error_with(
            &[(unsafe { SPUNoUpdateFoundReasonKey }, &reason)],
            "SUSparkleErrorDomain",
            1001,
        );

        let payload = error_payload(&error);
        let info = payload.no_update.expect("1001 keeps its reason");

        assert_eq!(payload.code, 1001);
        assert_eq!(info.reason, NoUpdateReason::OnNewerThanLatestVersion);
        assert!(!info.user_initiated);
    }

    #[test]
    fn rejects_a_latest_item_that_is_not_an_appcast_item() {
        let reason = number(1);
        let impostor = NSString::from_str("not an appcast item");
        let error = error_with(
            &[
                (unsafe { SPUNoUpdateFoundReasonKey }, &reason),
                (unsafe { SPULatestAppcastItemFoundKey }, &impostor),
            ],
            "SUSparkleErrorDomain",
            1001,
        );

        let info = no_update_info(&error).expect("reason is still present");

        assert_eq!(info.reason, NoUpdateReason::OnLatestVersion);
        assert!(info.latest_item.is_none());
    }
}
