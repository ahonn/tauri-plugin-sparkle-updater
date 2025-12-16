//! SparkleUpdater - Rust wrapper around Sparkle framework
//!
//! Provides a safe Rust interface to the macOS Sparkle update framework.

use std::ptr;
use std::sync::Arc;

use dispatch::Queue;
use log::warn;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{msg_send, ClassType, MainThreadMarker};
use objc2_foundation::{NSBundle, NSDictionary, NSError, NSString, NSURL};
use tauri::{plugin::PluginApi, AppHandle, Emitter, Runtime};

use super::bindings::{SPUStandardUpdaterController, SPUUpdater};
use super::delegate::SparkleDelegate;
use crate::config::Config;
use crate::{Error, Result};

/// Wrapper for raw pointer that can be sent across threads.
/// Safety: The pointer is only dereferenced on the main thread via dispatch.
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

    /// Safety: Must only be called on the main thread.
    unsafe fn as_ref(&self) -> &T {
        &*self.0
    }
}

/// Initialize the Sparkle updater plugin.
///
/// Note: This function must be called on the main thread (which is the case
/// for Tauri plugin setup).
pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Config>,
) -> Result<SparkleUpdater<R>> {
    let config = api.config();

    // Get main thread marker - this function is called during Tauri plugin setup on main thread
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| Error::SparkleInit("Must be called on main thread".to_string()))?;

    // Create the delegate for event handling
    let delegate = SparkleDelegate::new(mtm);

    // Set up the event emitter
    let app_clone = app.clone();
    delegate.set_emitter(Arc::new(move |event: &str, payload: &str| {
        let _ = app_clone.emit(event, payload);
    }));

    // Create the controller with the delegate
    // This runs on main thread during Tauri plugin setup
    let controller = unsafe {
        let alloc: objc2::rc::Allocated<SPUStandardUpdaterController> =
            objc2::msg_send![SPUStandardUpdaterController::class(), alloc];
        // Pass delegate to the controller during initialization
        // SparkleDelegate inherits from NSObject via define_class!, so Deref works
        let delegate_obj: &NSObject = &*delegate;
        SPUStandardUpdaterController::init_with_starting_updater(
            alloc,
            false,
            Some(delegate_obj),
            None,
        )
    };

    let updater: Retained<SPUUpdater> = controller.updater();

    // Apply configuration
    // 1. Set feed URL
    let ns_string = NSString::from_str(&config.feed_url);
    let ns_url: Option<Retained<NSURL>> =
        unsafe { objc2::msg_send![NSURL::class(), URLWithString: &*ns_string] };
    if let Some(url) = ns_url {
        updater.set_feed_url(Some(&url));
    }

    // 2. Set automatic check
    updater.set_automatically_checks_for_updates(config.automatically_checks_for_updates);

    // 3. Set automatic download
    updater.set_automatically_downloads_updates(config.automatically_downloads_updates);

    // 4. Set update check interval (in seconds)
    updater.set_update_check_interval(config.update_check_interval as f64);

    // 5. Start updater
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

    // Store raw pointer for dispatch operations in command handlers
    let controller_ptr = SendPtr::new(Retained::as_ptr(&controller));

    // Check for SUPublicEDKey in Info.plist and warn if missing
    check_info_plist_keys();

    Ok(SparkleUpdater {
        app: app.clone(),
        _controller: controller,
        controller_ptr,
        _delegate: delegate,
    })
}

/// Checks for required Sparkle keys in Info.plist and logs warnings if missing.
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
                     Sparkle will not be able to verify update signatures. \
                     Add SUPublicEDKey to your bundle.macOS.infoPlist in tauri.conf.json."
                );
            }
        }
    }
}

/// Access to the Sparkle updater APIs.
pub struct SparkleUpdater<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    /// Retained reference to keep the controller alive.
    _controller: Retained<SPUStandardUpdaterController>,
    /// Raw pointer for main-thread dispatch operations.
    controller_ptr: SendPtr<SPUStandardUpdaterController>,
    /// Retained reference to keep the delegate alive (Sparkle holds a weak reference).
    _delegate: Retained<SparkleDelegate>,
}

// Safety: All SPUStandardUpdaterController operations are dispatched to the main thread
// using GCD. The raw pointer is only dereferenced within exec_sync on the main thread.
unsafe impl<R: Runtime> Send for SparkleUpdater<R> {}
unsafe impl<R: Runtime> Sync for SparkleUpdater<R> {}

impl<R: Runtime> SparkleUpdater<R> {
    /// Check for updates and show the standard update UI.
    ///
    /// This will display the Sparkle update dialog with release notes,
    /// download progress, and installation options.
    pub fn check_for_updates(&self) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.check_for_updates(None);
        });
        Ok(())
    }

    /// Check for updates silently in the background.
    ///
    /// This will not show any UI unless an update is found and ready
    /// for installation.
    pub fn check_for_updates_in_background(&self) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().check_for_updates_in_background();
        });
        Ok(())
    }

    /// Returns whether the updater can currently check for updates.
    ///
    /// This may return false if:
    /// - The feed URL is not configured
    /// - An update check is already in progress
    /// - The updater has not been started
    pub fn can_check_for_updates(&self) -> Result<bool> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().can_check_for_updates()
        }))
    }

    /// Returns the current application version.
    pub fn current_version(&self) -> Result<String> {
        Ok(self.app.package_info().version.to_string())
    }

    /// Returns the current feed URL.
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

    /// Sets the feed URL.
    ///
    /// This can be used to change the appcast URL at runtime.
    pub fn set_feed_url(&self, url: &str) -> Result<()> {
        // Validate URL format first
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

    /// Returns whether automatic update checks are enabled.
    pub fn automatically_checks_for_updates(&self) -> Result<bool> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().automatically_checks_for_updates()
        }))
    }

    /// Sets whether automatic update checks are enabled.
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

    /// Returns whether updates are automatically downloaded.
    pub fn automatically_downloads_updates(&self) -> Result<bool> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().automatically_downloads_updates()
        }))
    }

    /// Sets whether updates are automatically downloaded.
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

    /// Returns the date of the last update check as Unix timestamp (milliseconds since epoch).
    ///
    /// Returns None if no update check has been performed yet.
    pub fn last_update_check_date(&self) -> Result<Option<f64>> {
        let ptr = self.controller_ptr;
        Ok(Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            let date = controller.updater().last_update_check_date();
            match date {
                Some(date) => {
                    let seconds: f64 = unsafe { objc2::msg_send![&date, timeIntervalSince1970] };
                    Some(seconds * 1000.0) // Convert to milliseconds
                }
                None => None,
            }
        }))
    }

    /// Resets the update cycle.
    ///
    /// This allows checking for updates immediately regardless of the
    /// configured check interval. Useful for testing.
    pub fn reset_update_cycle(&self) -> Result<()> {
        let ptr = self.controller_ptr;
        Queue::main().exec_sync(move || {
            let controller = unsafe { ptr.as_ref() };
            controller.updater().reset_update_cycle();
        });
        Ok(())
    }
}
