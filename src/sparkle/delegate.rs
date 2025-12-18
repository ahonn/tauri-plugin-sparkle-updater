use std::cell::RefCell;
use std::sync::Arc;

use log::error;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, DeclaredClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::NSString;
use serde::Serialize;

use super::bindings::SPUAppcastItem;
use crate::events::{
    DownloadFailedInfo, EmptyPayload, ErrorPayload, ScheduleInfo, UpdateCycleInfo, UpdateInfo,
    UserChoiceInfo, VersionInfo, EVENT_DID_ABORT_WITH_ERROR, EVENT_DID_DOWNLOAD_UPDATE,
    EVENT_DID_EXTRACT_UPDATE, EVENT_DID_FIND_VALID_UPDATE, EVENT_DID_FINISH_LOADING_APPCAST,
    EVENT_DID_FINISH_UPDATE_CYCLE, EVENT_DID_NOT_FIND_UPDATE, EVENT_FAILED_TO_DOWNLOAD_UPDATE,
    EVENT_SHOULD_PROMPT_FOR_PERMISSION, EVENT_USER_DID_CANCEL_DOWNLOAD, EVENT_USER_DID_MAKE_CHOICE,
    EVENT_WILL_DOWNLOAD_UPDATE, EVENT_WILL_EXTRACT_UPDATE, EVENT_WILL_INSTALL_UPDATE,
    EVENT_WILL_INSTALL_UPDATE_ON_QUIT, EVENT_WILL_NOT_SCHEDULE_UPDATE_CHECK,
    EVENT_WILL_RELAUNCH_APPLICATION, EVENT_WILL_SCHEDULE_UPDATE_CHECK,
};

pub type EventEmitter = Arc<dyn Fn(&str, String) + Send + Sync>;

pub struct DelegateIvars {
    emitter: RefCell<Option<EventEmitter>>,
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
            self.emit(EVENT_DID_FIND_VALID_UPDATE, &UpdateInfo {
                version: item.display_version_string().to_string(),
                release_notes: item.item_description().map(|s| s.to_string()),
            });
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
            self.emit(EVENT_SHOULD_PROMPT_FOR_PERMISSION, &EmptyPayload {});
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
        });
        unsafe { msg_send![super(this), init] }
    }

    pub fn set_emitter(&self, emitter: EventEmitter) {
        *self.ivars().emitter.borrow_mut() = Some(emitter);
    }

    fn emit<T: Serialize>(&self, event: &str, payload: &T) {
        if let Some(ref emitter) = *self.ivars().emitter.borrow() {
            match serde_json::to_string(payload) {
                Ok(json) => emitter(event, json),
                Err(e) => error!("Failed to serialize event payload: {}", e),
            }
        }
    }
}
