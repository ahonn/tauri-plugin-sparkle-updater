use std::cell::RefCell;
use std::sync::Arc;

use log::error;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, DeclaredClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::NSString;
use serde::Serialize;

use super::bindings::SPUAppcastItem;
use crate::events::*;

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
            self.emit(EVENT_CHECKING, &EmptyPayload {});
        }

        #[unsafe(method(updater:didFindValidUpdate:))]
        fn updater_did_find_valid_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            self.emit(EVENT_UPDATE_AVAILABLE, &UpdateInfo {
                version: item.display_version_string().to_string(),
                release_notes: item.item_description().map(|s| s.to_string()),
            });
        }

        #[unsafe(method(updaterDidNotFindUpdate:))]
        fn updater_did_not_find_update(&self, _updater: &NSObject) {
            self.emit(EVENT_UPDATE_NOT_AVAILABLE, &EmptyPayload {});
        }

        #[unsafe(method(updater:willDownloadUpdate:withRequest:))]
        fn updater_will_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
            _request: &NSObject,
        ) {
            self.emit(EVENT_DOWNLOADING, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:didDownloadUpdate:))]
        fn updater_did_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            self.emit(EVENT_DOWNLOADED, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:willInstallUpdate:))]
        fn updater_will_install_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            self.emit(EVENT_INSTALLING, &VersionInfo {
                version: item.display_version_string().to_string(),
            });
        }

        #[unsafe(method(updater:didAbortWithError:))]
        fn updater_did_abort_with_error(
            &self,
            _updater: &NSObject,
            ns_error: &NSObject,
        ) {
            self.emit(EVENT_ERROR, &ErrorPayload {
                message: nserror_description(ns_error),
                code: unsafe { msg_send![ns_error, code] },
                domain: nserror_domain(ns_error),
            });
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
