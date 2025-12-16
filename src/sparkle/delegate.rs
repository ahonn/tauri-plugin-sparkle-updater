//! SPUUpdaterDelegate implementation for Sparkle event handling.
//!
//! This module provides a Rust implementation of the SPUUpdaterDelegate protocol
//! that emits Tauri events when update-related callbacks are triggered.

use std::cell::RefCell;
use std::sync::Arc;

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly, DeclaredClass};
use objc2_foundation::NSString;

use super::bindings::SPUAppcastItem;
use crate::events::*;

/// Type alias for the event emitter function.
pub type EventEmitter = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Instance variables for SparkleDelegate.
pub struct DelegateIvars {
    emitter: RefCell<Option<EventEmitter>>,
}

define_class!(
    /// Rust implementation of SPUUpdaterDelegate protocol.
    ///
    /// This delegate receives callbacks from Sparkle and emits corresponding
    /// Tauri events.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TauriSparkleDelegate"]
    #[ivars = DelegateIvars]
    pub struct SparkleDelegate;

    // SPUUpdaterDelegate protocol methods - defined as instance methods
    impl SparkleDelegate {
        /// Called when the appcast has been downloaded and parsed.
        ///
        /// Note: This triggers the `sparkle://checking` event. Despite the event name,
        /// it fires AFTER the appcast is loaded (not when checking starts), as Sparkle
        /// does not provide a "will start checking" callback. This is the earliest
        /// meaningful event in the update check flow.
        #[unsafe(method(updater:didFinishLoadingAppcast:))]
        fn updater_did_finish_loading_appcast(
            &self,
            _updater: &NSObject,
            _appcast: &NSObject,
        ) {
            self.emit_event(EVENT_CHECKING, "{}");
        }

        /// Called when a valid update is found.
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

        /// Called when no update is found.
        #[unsafe(method(updaterDidNotFindUpdate:))]
        fn updater_did_not_find_update(&self, _updater: &NSObject) {
            self.emit_event(EVENT_UPDATE_NOT_AVAILABLE, "{}");
        }

        /// Called when the updater will download an update.
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

        /// Called when the updater finished downloading an update.
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

        /// Called when the updater will install an update.
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

        /// Called when the updater fails with an error.
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
    /// Creates a new SparkleDelegate instance.
    ///
    /// Must be called on the main thread.
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm);
        let this = this.set_ivars(DelegateIvars {
            emitter: RefCell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    /// Sets the event emitter function.
    ///
    /// The emitter will be called with (event_name, payload_json) when
    /// Sparkle triggers delegate callbacks.
    pub fn set_emitter(&self, emitter: EventEmitter) {
        *self.ivars().emitter.borrow_mut() = Some(emitter);
    }

    /// Emits an event through the configured emitter.
    fn emit_event(&self, event: &str, payload: &str) {
        if let Some(ref emitter) = *self.ivars().emitter.borrow() {
            emitter(event, payload);
        }
    }
}
