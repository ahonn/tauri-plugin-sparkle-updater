use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::error;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, DeclaredClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSArray, NSDictionary, NSMutableSet, NSNumber, NSSet, NSString, NSURL};
use serde::Serialize;
use serde_json::Value;

use super::bindings::SPUAppcastItem;
use crate::events::UpdateInfo;
use crate::events::{
    DownloadFailedInfo, EmptyPayload, ErrorPayload, ScheduleInfo, UpdateCycleInfo, UserChoiceInfo,
    VersionInfo, EVENT_DID_ABORT_WITH_ERROR, EVENT_DID_DOWNLOAD_UPDATE, EVENT_DID_EXTRACT_UPDATE,
    EVENT_DID_FIND_VALID_UPDATE, EVENT_DID_FINISH_LOADING_APPCAST, EVENT_DID_FINISH_UPDATE_CYCLE,
    EVENT_DID_NOT_FIND_UPDATE, EVENT_FAILED_TO_DOWNLOAD_UPDATE, EVENT_USER_DID_CANCEL_DOWNLOAD,
    EVENT_USER_DID_MAKE_CHOICE, EVENT_WILL_DOWNLOAD_UPDATE, EVENT_WILL_EXTRACT_UPDATE,
    EVENT_WILL_INSTALL_UPDATE, EVENT_WILL_INSTALL_UPDATE_ON_QUIT,
    EVENT_WILL_NOT_SCHEDULE_UPDATE_CHECK, EVENT_WILL_RELAUNCH_APPLICATION,
    EVENT_WILL_SCHEDULE_UPDATE_CHECK,
};

pub type EventEmitter = Arc<dyn Fn(&str, Value) + Send + Sync>;

pub struct DelegateIvars {
    emitter: RefCell<Option<EventEmitter>>,
    allowed_channels: RefCell<Option<Vec<String>>>,
    feed_url_override: RefCell<Option<String>>,
    feed_parameters: RefCell<Option<HashMap<String, String>>>,
    should_download_release_notes: RefCell<bool>,
    should_relaunch: RefCell<bool>,
    may_check_for_updates: RefCell<bool>,
    should_proceed_with_update: RefCell<bool>,
    decryption_password: RefCell<Option<String>>,
    last_found_update: RefCell<Option<UpdateInfo>>,
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
            let url_to_string = |url: &NSURL| -> String {
                let abs: Option<Retained<NSString>> = unsafe { msg_send![url, absoluteString] };
                abs.map(|s| s.to_string()).unwrap_or_default()
            };

            let number_to_f64 = |num: &NSNumber| -> f64 {
                unsafe { msg_send![num, doubleValue] }
            };

            let update_info = UpdateInfo {
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
                ignore_skipped_upgrades_below_version: item.ignore_skipped_upgrades_below_version().map(|s| s.to_string()),
                date_string: item.date_string().map(|s| s.to_string()),
                item_description_format: item.item_description_format().map(|s| s.to_string()),
            };

            *self.ivars().last_found_update.borrow_mut() = Some(update_info.clone());
            self.emit(EVENT_DID_FIND_VALID_UPDATE, &update_info);
        }

        #[unsafe(method(updaterDidNotFindUpdate:))]
        fn updater_did_not_find_update(&self, _updater: &NSObject) {
            self.emit(EVENT_DID_NOT_FIND_UPDATE, &EmptyPayload {});
        }

        #[unsafe(method(updater:willDownloadUpdate:withRequest:))]
        fn updater_will_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
            _request: &NSObject,
        ) {
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
            self.emit(EVENT_DID_ABORT_WITH_ERROR, &ErrorPayload {
                message: nserror_description(ns_error),
                code: unsafe { msg_send![ns_error, code] },
                domain: nserror_domain(ns_error),
            });
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
                error: error.map(|e| ErrorPayload {
                    message: nserror_description(e),
                    code: unsafe { msg_send![e, code] },
                    domain: nserror_domain(e),
                }),
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
                error: ErrorPayload {
                    message: nserror_description(ns_error),
                    code: unsafe { msg_send![ns_error, code] },
                    domain: nserror_domain(ns_error),
                },
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

impl SparkleDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm);
        let this = this.set_ivars(DelegateIvars {
            emitter: RefCell::new(None),
            allowed_channels: RefCell::new(None),
            feed_url_override: RefCell::new(None),
            feed_parameters: RefCell::new(None),
            should_download_release_notes: RefCell::new(true),
            should_relaunch: RefCell::new(true),
            may_check_for_updates: RefCell::new(true),
            should_proceed_with_update: RefCell::new(true),
            decryption_password: RefCell::new(None),
            last_found_update: RefCell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    pub fn set_emitter(&self, emitter: EventEmitter) {
        *self.ivars().emitter.borrow_mut() = Some(emitter);
    }

    fn emit<T: Serialize>(&self, event: &str, payload: &T) {
        if let Some(ref emitter) = *self.ivars().emitter.borrow() {
            match serde_json::to_value(payload) {
                Ok(value) => emitter(event, value),
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
}
