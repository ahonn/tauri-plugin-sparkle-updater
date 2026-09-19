use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

use log::warn;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{msg_send, ClassType, MainThreadMarker};
use objc2_foundation::{NSBundle, NSDictionary, NSError, NSString, NSURL};

use super::bindings::{SPUStandardUpdaterController, SPUUpdater};
use super::delegate::{EventCallback, SparkleDelegate};
use crate::events::UpdateInfo;
use crate::{Error, GentleReminders, RelaunchHandler, Result};

fn is_valid_bundle() -> bool {
    unsafe {
        let bundle = NSBundle::mainBundle();
        let identifier: Option<Retained<NSString>> = msg_send![&bundle, bundleIdentifier];
        let path = bundle.bundlePath().to_string();
        if !std::path::Path::new(&path)
            .extension()
            .is_some_and(|ext| ext == "app")
        {
            return false;
        }
        match identifier {
            Some(id) => {
                let id_str = id.to_string();
                !id_str.is_empty() && id_str != "com.apple.dt.Xcode.tool"
            }
            None => false,
        }
    }
}

/// Callbacks installed before Sparkle starts its update cycle.
#[derive(Default)]
pub struct UpdaterConfig {
    pub event_callback: Option<EventCallback>,
    pub relaunch_handler: Option<RelaunchHandler>,
    pub gentle_reminders: Option<Rc<dyn GentleReminders>>,
}

/// Starts Sparkle with an optional event listener. Returns `None` outside an app bundle.
pub fn init(
    mtm: MainThreadMarker,
    callback: Option<EventCallback>,
) -> Result<Option<SparkleUpdater>> {
    SparkleUpdater::new(
        mtm,
        UpdaterConfig {
            event_callback: callback,
            ..Default::default()
        },
    )
}

fn initialize(mtm: MainThreadMarker, config: UpdaterConfig) -> Result<Option<SparkleUpdater>> {
    if !is_valid_bundle() {
        warn!(
            "Sparkle updater disabled: not running inside a valid macOS bundle. \
             Bundle the application before enabling updates."
        );
        return Ok(None);
    }

    check_info_plist_keys();

    let delegate = SparkleDelegate::new(mtm);
    delegate.set_event_callback(config.event_callback);
    delegate.set_relaunch_handler(config.relaunch_handler);
    delegate.set_gentle_reminders(config.gentle_reminders);

    let controller = unsafe {
        let alloc: objc2::rc::Allocated<SPUStandardUpdaterController> =
            objc2::msg_send![SPUStandardUpdaterController::class(), alloc];
        let delegate_obj: &NSObject = &delegate;
        SPUStandardUpdaterController::init_with_starting_updater(
            alloc,
            false,
            Some(delegate_obj),
            Some(delegate_obj),
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

    Ok(Some(SparkleUpdater {
        controller,
        delegate,
    }))
}

const PLIST_KEY_VALIDATIONS: &[(&str, &str)] = &[
    (
        "SUPublicEDKey",
        "Sparkle will not be able to verify update signatures.",
    ),
    (
        "SUFeedURL",
        "You must set a feed URL before checking for updates.",
    ),
];

fn check_info_plist_keys() {
    unsafe {
        let bundle = NSBundle::mainBundle();
        let info_dict: Option<Retained<NSDictionary>> = msg_send![&bundle, infoDictionary];

        if let Some(dict) = info_dict {
            for (key_name, warning) in PLIST_KEY_VALIDATIONS {
                let key = NSString::from_str(key_name);
                let value: Option<Retained<NSObject>> = msg_send![&dict, objectForKey: &*key];
                if value.is_none() {
                    warn!("{} not found in Info.plist. {}", key_name, warning);
                }
            }
        }
    }
}

/// Owns Sparkle's controller and delegates on the macOS main thread.
///
/// This type is neither `Send` nor `Sync`. Keep it alive for the application's
/// lifetime; callers must provide an AppKit event loop and a signed app bundle.
pub struct SparkleUpdater {
    controller: Retained<SPUStandardUpdaterController>,
    delegate: Retained<SparkleDelegate>,
}

impl SparkleUpdater {
    /// Creates and starts the updater. Configure Info.plist and callbacks before calling.
    /// Returns `None` when the executable is not inside an application bundle.
    pub fn new(mtm: MainThreadMarker, config: UpdaterConfig) -> Result<Option<Self>> {
        initialize(mtm, config)
    }

    /// Installs a host callback that postpones relaunch until its continuation is resumed.
    /// This is not a universal quit veto: also save state through the host's quit lifecycle.
    pub fn set_relaunch_handler(&self, handler: Option<RelaunchHandler>) {
        self.delegate.set_relaunch_handler(handler);
    }

    fn with_controller<T>(&self, f: impl FnOnce(&SPUStandardUpdaterController) -> T) -> T {
        f(&self.controller)
    }

    fn with_delegate<T>(&self, f: impl FnOnce(&SparkleDelegate) -> T) -> T {
        f(&self.delegate)
    }

    pub fn check_for_updates(&self) -> Result<()> {
        self.with_controller(|c| c.check_for_updates(None));
        Ok(())
    }

    pub fn check_for_updates_in_background(&self) -> Result<()> {
        self.with_controller(|c| c.updater().check_for_updates_in_background());
        Ok(())
    }

    pub fn can_check_for_updates(&self) -> Result<bool> {
        Ok(self.with_controller(|c| c.updater().can_check_for_updates()))
    }

    pub fn current_version(&self) -> Result<String> {
        let bundle = NSBundle::mainBundle();
        let key = NSString::from_str("CFBundleShortVersionString");
        let version = bundle
            .objectForInfoDictionaryKey(&key)
            .and_then(|value| value.downcast::<NSString>().ok())
            .ok_or_else(|| Error::SparkleInit("Missing CFBundleShortVersionString".into()))?;
        Ok(version.to_string())
    }

    pub fn feed_url(&self) -> Result<Option<String>> {
        Ok(self.with_controller(|c| {
            c.updater().feed_url().and_then(|url| {
                let abs: Option<Retained<NSString>> =
                    unsafe { objc2::msg_send![&url, absoluteString] };
                abs.map(|s| s.to_string())
            })
        }))
    }

    pub fn set_feed_url(&self, url: &str) -> Result<()> {
        url::Url::parse(url).map_err(|_| Error::InvalidFeedUrl(url.to_string()))?;
        let url_string = url.to_string();

        self.with_controller(move |c| {
            let ns_string = NSString::from_str(&url_string);
            let ns_url: Option<Retained<NSURL>> =
                unsafe { objc2::msg_send![NSURL::class(), URLWithString: &*ns_string] };
            if let Some(url) = ns_url {
                c.updater().set_feed_url(Some(&url));
            }
        });
        Ok(())
    }

    pub fn automatically_checks_for_updates(&self) -> Result<bool> {
        Ok(self.with_controller(|c| c.updater().automatically_checks_for_updates()))
    }

    pub fn set_automatically_checks_for_updates(&self, enabled: bool) -> Result<()> {
        self.with_controller(|c| c.updater().set_automatically_checks_for_updates(enabled));
        Ok(())
    }

    pub fn automatically_downloads_updates(&self) -> Result<bool> {
        Ok(self.with_controller(|c| c.updater().automatically_downloads_updates()))
    }

    pub fn set_automatically_downloads_updates(&self, enabled: bool) -> Result<()> {
        self.with_controller(|c| c.updater().set_automatically_downloads_updates(enabled));
        Ok(())
    }

    pub fn last_update_check_date(&self) -> Result<Option<f64>> {
        Ok(self.with_controller(|c| {
            c.updater().last_update_check_date().map(|date| {
                let seconds: f64 = unsafe { objc2::msg_send![&date, timeIntervalSince1970] };
                seconds * 1000.0
            })
        }))
    }

    pub fn reset_update_cycle(&self) -> Result<()> {
        self.with_controller(|c| c.updater().reset_update_cycle());
        Ok(())
    }

    pub fn update_check_interval(&self) -> Result<f64> {
        Ok(self.with_controller(|c| c.updater().update_check_interval()))
    }

    pub fn set_update_check_interval(&self, interval: f64) -> Result<()> {
        self.with_controller(|c| c.updater().set_update_check_interval(interval));
        Ok(())
    }

    pub fn check_for_update_information(&self) -> Result<()> {
        self.with_controller(|c| c.updater().check_for_update_information());
        Ok(())
    }

    pub fn session_in_progress(&self) -> Result<bool> {
        Ok(self.with_controller(|c| c.updater().session_in_progress()))
    }

    pub fn http_headers(&self) -> Result<Option<HashMap<String, String>>> {
        Ok(self.with_controller(|c| {
            c.updater().http_headers().map(|dict| {
                let mut map = HashMap::new();
                let count: usize = unsafe { objc2::msg_send![&dict, count] };
                if count > 0 {
                    let keys: Retained<objc2_foundation::NSArray<NSString>> =
                        unsafe { objc2::msg_send![&dict, allKeys] };
                    for i in 0..count {
                        let key: &NSString = unsafe { objc2::msg_send![&keys, objectAtIndex: i] };
                        let value: Option<Retained<NSString>> =
                            unsafe { objc2::msg_send![&dict, objectForKey: key] };
                        if let Some(v) = value {
                            map.insert(key.to_string(), v.to_string());
                        }
                    }
                }
                map
            })
        }))
    }

    pub fn set_http_headers(&self, headers: Option<HashMap<String, String>>) -> Result<()> {
        self.with_controller(move |c| {
            let ns_dict = headers.map(|h| {
                let keys: Vec<Retained<NSString>> =
                    h.keys().map(|k| NSString::from_str(k)).collect();
                let values: Vec<Retained<NSString>> =
                    h.values().map(|v| NSString::from_str(v)).collect();
                let key_refs: Vec<&NSString> = keys.iter().map(|k| k.as_ref()).collect();
                let value_refs: Vec<&NSString> = values.iter().map(|v| v.as_ref()).collect();
                NSDictionary::from_slices(&key_refs, &value_refs)
            });
            c.updater().set_http_headers(ns_dict.as_deref());
        });
        Ok(())
    }

    pub fn user_agent_string(&self) -> Result<String> {
        Ok(self.with_controller(|c| c.updater().user_agent_string().to_string()))
    }

    pub fn set_user_agent_string(&self, user_agent: &str) -> Result<()> {
        let ua = user_agent.to_string();
        self.with_controller(move |c| {
            let ns_string = NSString::from_str(&ua);
            c.updater().set_user_agent_string(&ns_string);
        });
        Ok(())
    }

    pub fn sends_system_profile(&self) -> Result<bool> {
        Ok(self.with_controller(|c| c.updater().sends_system_profile()))
    }

    pub fn set_sends_system_profile(&self, sends: bool) -> Result<()> {
        self.with_controller(|c| c.updater().set_sends_system_profile(sends));
        Ok(())
    }

    pub fn clear_feed_url_from_user_defaults(&self) -> Result<Option<String>> {
        Ok(self.with_controller(|c| {
            c.updater()
                .clear_feed_url_from_user_defaults()
                .and_then(|url| {
                    let abs: Option<Retained<NSString>> =
                        unsafe { objc2::msg_send![&url, absoluteString] };
                    abs.map(|s| s.to_string())
                })
        }))
    }

    pub fn reset_update_cycle_after_short_delay(&self) -> Result<()> {
        self.with_controller(|c| c.updater().reset_update_cycle_after_short_delay());
        Ok(())
    }

    pub fn allowed_channels(&self) -> Result<Option<Vec<String>>> {
        Ok(self.with_delegate(|d| d.allowed_channels()))
    }

    pub fn set_allowed_channels(&self, channels: Option<Vec<String>>) -> Result<()> {
        self.with_delegate(|d| d.set_allowed_channels(channels));
        Ok(())
    }

    pub fn feed_url_override(&self) -> Result<Option<String>> {
        Ok(self.with_delegate(|d| d.feed_url_override()))
    }

    pub fn set_feed_url_override(&self, url: Option<String>) -> Result<()> {
        self.with_delegate(|d| d.set_feed_url_override(url));
        Ok(())
    }

    pub fn feed_parameters(&self) -> Result<Option<HashMap<String, String>>> {
        Ok(self.with_delegate(|d| d.feed_parameters()))
    }

    pub fn set_feed_parameters(&self, params: Option<HashMap<String, String>>) -> Result<()> {
        self.with_delegate(|d| d.set_feed_parameters(params));
        Ok(())
    }

    pub fn should_download_release_notes(&self) -> Result<bool> {
        Ok(self.with_delegate(|d| d.should_download_release_notes()))
    }

    pub fn set_should_download_release_notes(&self, enabled: bool) -> Result<()> {
        self.with_delegate(|d| d.set_should_download_release_notes(enabled));
        Ok(())
    }

    pub fn should_relaunch_application(&self) -> Result<bool> {
        Ok(self.with_delegate(|d| d.should_relaunch()))
    }

    pub fn set_should_relaunch_application(&self, enabled: bool) -> Result<()> {
        self.with_delegate(|d| d.set_should_relaunch(enabled));
        Ok(())
    }

    pub fn may_check_for_updates_config(&self) -> Result<bool> {
        Ok(self.with_delegate(|d| d.may_check_for_updates()))
    }

    pub fn set_may_check_for_updates_config(&self, enabled: bool) -> Result<()> {
        self.with_delegate(|d| d.set_may_check_for_updates(enabled));
        Ok(())
    }

    pub fn should_proceed_with_update(&self) -> Result<bool> {
        Ok(self.with_delegate(|d| d.should_proceed_with_update()))
    }

    pub fn set_should_proceed_with_update(&self, enabled: bool) -> Result<()> {
        self.with_delegate(|d| d.set_should_proceed_with_update(enabled));
        Ok(())
    }

    pub fn decryption_password(&self) -> Result<Option<String>> {
        Ok(self.with_delegate(|d| d.decryption_password()))
    }

    pub fn set_decryption_password(&self, password: Option<String>) -> Result<()> {
        self.with_delegate(|d| d.set_decryption_password(password));
        Ok(())
    }

    pub fn last_found_update(&self) -> Result<Option<UpdateInfo>> {
        Ok(self.with_delegate(|d| d.last_found_update()))
    }

    pub fn set_event_callback(&self, callback: Option<EventCallback>) {
        self.with_delegate(|d| d.set_event_callback(callback))
    }

    pub fn download_request_headers(&self) -> Result<Option<HashMap<String, String>>> {
        Ok(self.with_delegate(|d| d.download_request_headers()))
    }

    pub fn set_download_request_headers(
        &self,
        headers: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.with_delegate(|d| d.set_download_request_headers(headers));
        Ok(())
    }
}
