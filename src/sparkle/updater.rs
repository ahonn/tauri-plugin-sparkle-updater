use std::ptr;
use std::sync::Arc;

use dispatch::Queue;
use log::warn;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{msg_send, ClassType, MainThreadMarker};
use objc2_foundation::{NSBundle, NSDictionary, NSError, NSString, NSURL};
use tauri::{AppHandle, Emitter, Runtime};

use super::bindings::{SPUStandardUpdaterController, SPUUpdater};
use super::delegate::SparkleDelegate;
use crate::{Error, Result};

/// Pointer wrapper for cross-thread dispatch. Only dereference on main thread.
#[repr(transparent)]
struct SendPtr<T>(*const T);

unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SendPtr<T> {}

impl<T> SendPtr<T> {
    fn new(ptr: *const T) -> Self {
        SendPtr(ptr)
    }

    unsafe fn as_ref(&self) -> &T {
        &*self.0
    }
}

fn is_valid_bundle() -> bool {
    unsafe {
        let bundle = NSBundle::mainBundle();
        let identifier: Option<Retained<NSString>> = msg_send![&bundle, bundleIdentifier];
        match identifier {
            Some(id) => {
                let id_str = id.to_string();
                !id_str.is_empty() && id_str != "com.apple.dt.Xcode.tool"
            }
            None => false,
        }
    }
}

/// Returns `None` if running outside a valid macOS bundle (e.g., during `tauri dev`).
pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<Option<SparkleUpdater<R>>> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| Error::SparkleInit("Must be called on main thread".to_string()))?;

    if !is_valid_bundle() {
        warn!(
            "Sparkle updater disabled: not running inside a valid macOS bundle. \
             This is expected during development (tauri dev). \
             Sparkle will work in release builds (tauri build)."
        );
        return Ok(None);
    }

    check_info_plist_keys();

    let delegate = SparkleDelegate::new(mtm);
    let app_clone = app.clone();
    delegate.set_emitter(Arc::new(move |event: &str, payload: &str| {
        let _ = app_clone.emit(event, payload);
    }));

    let controller = unsafe {
        let alloc: objc2::rc::Allocated<SPUStandardUpdaterController> =
            objc2::msg_send![SPUStandardUpdaterController::class(), alloc];
        let delegate_obj: &NSObject = &*delegate;
        SPUStandardUpdaterController::init_with_starting_updater(
            alloc,
            false,
            Some(delegate_obj),
            None,
        )
    };

    let updater: Retained<SPUUpdater> = controller.updater();
    let mut error: *mut NSError = ptr::null_mut();
    let success = updater.start_updater(&mut error);

    if !success {
        if !error.is_null() {
            let ns_error = unsafe { &*error };
            let description: Retained<NSString> =
                unsafe { objc2::msg_send![ns_error, localizedDescription] };
            return Err(Error::SparkleInit(description.to_string()));
        }
        return Err(Error::SparkleInit("Failed to start updater".to_string()));
    }

    let controller_ptr = SendPtr::new(Retained::as_ptr(&controller));

    Ok(Some(SparkleUpdater {
        app: app.clone(),
        _controller: controller,
        controller_ptr,
        _delegate: delegate,
    }))
}

fn check_info_plist_keys() {
    unsafe {
        let bundle = NSBundle::mainBundle();
        let info_dict: Option<Retained<NSDictionary>> = msg_send![&bundle, infoDictionary];

        if let Some(dict) = info_dict {
            let key = NSString::from_str("SUPublicEDKey");
            let value: Option<Retained<NSObject>> = msg_send![&dict, objectForKey: &*key];
            if value.is_none() {
                warn!(
                    "SUPublicEDKey not found in Info.plist. \
                     Sparkle will not be able to verify update signatures."
                );
            }

            let key = NSString::from_str("SUFeedURL");
            let value: Option<Retained<NSObject>> = msg_send![&dict, objectForKey: &*key];
            if value.is_none() {
                warn!(
                    "SUFeedURL not found in Info.plist. \
                     You must set a feed URL before checking for updates."
                );
            }
        }
    }
}

pub struct SparkleUpdater<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    _controller: Retained<SPUStandardUpdaterController>,
    controller_ptr: SendPtr<SPUStandardUpdaterController>,
    _delegate: Retained<SparkleDelegate>,
}

// All operations dispatched to main thread via GCD
unsafe impl<R: Runtime> Send for SparkleUpdater<R> {}
unsafe impl<R: Runtime> Sync for SparkleUpdater<R> {}

impl<R: Runtime> SparkleUpdater<R> {
    pub fn check_for_updates(&self) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.check_for_updates(None);
        });
        Ok(())
    }

    pub fn check_for_updates_in_background(&self) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().check_for_updates_in_background();
        });
        Ok(())
    }

    pub fn can_check_for_updates(&self) -> Result<bool> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().can_check_for_updates()
        }))
    }

    pub fn current_version(&self) -> Result<String> {
        Ok(self.app.package_info().version.to_string())
    }

    pub fn feed_url(&self) -> Result<Option<String>> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            let url = controller.updater().feed_url();
            match url {
                Some(url) => {
                    let abs_string: Option<Retained<NSString>> =
                        unsafe { objc2::msg_send![&url, absoluteString] };
                    abs_string.map(|s| s.to_string())
                }
                None => None,
            }
        }))
    }

    pub fn set_feed_url(&self, url: &str) -> Result<()> {
        url::Url::parse(url).map_err(|_| Error::InvalidFeedUrl(url.to_string()))?;

        let url_string = url.to_string();
        let ptr = self.controller_ptr;

        let result: Result<()> = Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            let ns_string = NSString::from_str(&url_string);
            let ns_url: Option<Retained<NSURL>> =
                unsafe { objc2::msg_send![NSURL::class(), URLWithString: &*ns_string] };

            match ns_url {
                Some(url) => {
                    controller.updater().set_feed_url(Some(&url));
                    Ok(())
                }
                None => Err(Error::InvalidFeedUrl(url_string)),
            }
        });

        result
    }

    pub fn automatically_checks_for_updates(&self) -> Result<bool> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().automatically_checks_for_updates()
        }))
    }

    pub fn set_automatically_checks_for_updates(&self, enabled: bool) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller
                .updater()
                .set_automatically_checks_for_updates(enabled);
        });
        Ok(())
    }

    pub fn automatically_downloads_updates(&self) -> Result<bool> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().automatically_downloads_updates()
        }))
    }

    pub fn set_automatically_downloads_updates(&self, enabled: bool) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller
                .updater()
                .set_automatically_downloads_updates(enabled);
        });
        Ok(())
    }

    /// Returns Unix timestamp in milliseconds. None if never checked.
    pub fn last_update_check_date(&self) -> Result<Option<f64>> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            let date = controller.updater().last_update_check_date();
            match date {
                Some(date) => {
                    let seconds: f64 = unsafe { objc2::msg_send![&date, timeIntervalSince1970] };
                    Some(seconds * 1000.0)
                }
                None => None,
            }
        }))
    }

    pub fn reset_update_cycle(&self) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().reset_update_cycle();
        });
        Ok(())
    }
}
