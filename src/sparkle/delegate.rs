use std::cell::RefCell;
use std::sync::Arc;

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly, DeclaredClass};
use objc2_foundation::NSString;

use super::bindings::SPUAppcastItem;
use crate::events::*;

pub type EventEmitter = Arc<dyn Fn(&str, &str) + Send + Sync>;

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
            self.emit_event(EVENT_CHECKING, "{}");
        }

        #[unsafe(method(updater:didFindValidUpdate:))]
        fn updater_did_find_valid_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            let version = item.display_version_string().to_string();
            let notes = item.item_description().map(|s| s.to_string());
            let payload = serde_json::json!({
                "version": version,
                "releaseNotes": notes
            });
            self.emit_event(EVENT_UPDATE_AVAILABLE, &payload.to_string());
        }

        #[unsafe(method(updaterDidNotFindUpdate:))]
        fn updater_did_not_find_update(&self, _updater: &NSObject) {
            self.emit_event(EVENT_UPDATE_NOT_AVAILABLE, "{}");
        }

        #[unsafe(method(updater:willDownloadUpdate:withRequest:))]
        fn updater_will_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
            _request: &NSObject,
        ) {
            let version = item.display_version_string().to_string();
            let payload = serde_json::json!({ "version": version });
            self.emit_event(EVENT_DOWNLOADING, &payload.to_string());
        }

        #[unsafe(method(updater:didDownloadUpdate:))]
        fn updater_did_download_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            let version = item.display_version_string().to_string();
            let payload = serde_json::json!({ "version": version });
            self.emit_event(EVENT_DOWNLOADED, &payload.to_string());
        }

        #[unsafe(method(updater:willInstallUpdate:))]
        fn updater_will_install_update(
            &self,
            _updater: &NSObject,
            item: &SPUAppcastItem,
        ) {
            let version = item.display_version_string().to_string();
            let payload = serde_json::json!({ "version": version });
            self.emit_event(EVENT_INSTALLING, &payload.to_string());
        }

        #[unsafe(method(updater:didAbortWithError:))]
        fn updater_did_abort_with_error(
            &self,
            _updater: &NSObject,
            error: &NSObject,
        ) {
            let description: Retained<NSString> =
                unsafe { msg_send![error, localizedDescription] };
            let code: i64 = unsafe { objc2::msg_send![error, code] };
            let domain: Retained<NSString> = unsafe { msg_send![error, domain] };
            let payload = serde_json::json!({
                "message": description.to_string(),
                "code": code,
                "domain": domain.to_string()
            });
            self.emit_event(EVENT_ERROR, &payload.to_string());
        }
    }
);

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

    fn emit_event(&self, event: &str, payload: &str) {
        if let Some(ref emitter) = *self.ivars().emitter.borrow() {
            emitter(event, payload);
        }
    }
}
