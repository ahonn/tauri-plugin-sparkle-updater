//! objc2 FFI bindings for Sparkle framework
//!
//! This module provides Rust bindings to the Sparkle macOS update framework.

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{extern_class, extern_methods, MainThreadOnly};
use objc2_foundation::{NSDate, NSError, NSString, NSURL};

// MARK: - SPUStandardUpdaterController

extern_class!(
    /// The standard user interface based updater controller.
    ///
    /// This is the main entry point for using Sparkle with a standard UI.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUStandardUpdaterController"]
    #[derive(Debug)]
    pub struct SPUStandardUpdaterController;
);

impl SPUStandardUpdaterController {
    extern_methods!(
        /// Initializes a new updater controller.
        ///
        /// - `starting_updater`: If true, starts the updater immediately.
        /// - `updater_delegate`: Optional delegate for updater callbacks.
        /// - `user_driver_delegate`: Optional delegate for user driver callbacks.
        #[unsafe(method(initWithStartingUpdater:updaterDelegate:userDriverDelegate:))]
        pub fn init_with_starting_updater(
            this: objc2::rc::Allocated<Self>,
            starting_updater: bool,
            updater_delegate: Option<&NSObject>,
            user_driver_delegate: Option<&NSObject>,
        ) -> Retained<Self>;

        /// Returns the underlying SPUUpdater instance.
        #[unsafe(method(updater))]
        pub fn updater(&self) -> Retained<SPUUpdater>;

        /// Triggers an update check with UI.
        ///
        /// This will show the standard Sparkle update dialog.
        #[unsafe(method(checkForUpdates:))]
        pub fn check_for_updates(&self, sender: Option<&NSObject>);
    );
}

// MARK: - SPUUpdater

extern_class!(
    /// The core updater class that manages update checking and downloading.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUUpdater"]
    #[derive(Debug)]
    pub struct SPUUpdater;
);

impl SPUUpdater {
    extern_methods!(
        /// Returns whether the updater can check for updates.
        ///
        /// This can be false if the feed URL is not configured or if an update
        /// check is already in progress.
        #[unsafe(method(canCheckForUpdates))]
        pub fn can_check_for_updates(&self) -> bool;

        /// Checks for updates and shows the standard update UI.
        #[unsafe(method(checkForUpdates))]
        pub fn check_for_updates(&self);

        /// Checks for updates silently in the background.
        ///
        /// This will not show any UI unless an update is found.
        #[unsafe(method(checkForUpdatesInBackground))]
        pub fn check_for_updates_in_background(&self);

        /// Returns the current feed URL.
        #[unsafe(method(feedURL))]
        pub fn feed_url(&self) -> Option<Retained<NSURL>>;

        /// Sets the feed URL.
        #[unsafe(method(setFeedURL:))]
        pub fn set_feed_url(&self, url: Option<&NSURL>);

        /// Returns whether automatic update checks are enabled.
        #[unsafe(method(automaticallyChecksForUpdates))]
        pub fn automatically_checks_for_updates(&self) -> bool;

        /// Sets whether automatic update checks are enabled.
        #[unsafe(method(setAutomaticallyChecksForUpdates:))]
        pub fn set_automatically_checks_for_updates(&self, enabled: bool);

        /// Returns whether updates are automatically downloaded.
        #[unsafe(method(automaticallyDownloadsUpdates))]
        pub fn automatically_downloads_updates(&self) -> bool;

        /// Sets whether updates are automatically downloaded.
        #[unsafe(method(setAutomaticallyDownloadsUpdates:))]
        pub fn set_automatically_downloads_updates(&self, enabled: bool);

        /// Returns the date of the last update check.
        #[unsafe(method(lastUpdateCheckDate))]
        pub fn last_update_check_date(&self) -> Option<Retained<NSDate>>;

        /// Resets the update cycle.
        ///
        /// This can be used to force an immediate check regardless of the
        /// configured check interval.
        #[unsafe(method(resetUpdateCycle))]
        pub fn reset_update_cycle(&self);

        /// Returns the update check interval in seconds.
        #[unsafe(method(updateCheckInterval))]
        pub fn update_check_interval(&self) -> f64;

        /// Sets the update check interval in seconds.
        #[unsafe(method(setUpdateCheckInterval:))]
        pub fn set_update_check_interval(&self, interval: f64);

        /// Starts the updater.
        ///
        /// Returns true if the updater started successfully, false otherwise.
        /// On failure, the error parameter will contain the error information.
        #[unsafe(method(startUpdater:))]
        pub fn start_updater(&self, error: *mut *mut NSError) -> bool;

        /// Sets the updater delegate.
        ///
        /// The delegate receives callbacks for update events.
        /// Note: Sparkle holds a weak reference to the delegate.
        #[unsafe(method(setDelegate:))]
        pub fn set_delegate(&self, delegate: Option<&NSObject>);
    );
}

// MARK: - SPUAppcastItem

extern_class!(
    /// Represents an item in the appcast feed.
    ///
    /// Contains information about a specific update version.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUAppcastItem"]
    #[derive(Debug)]
    pub struct SPUAppcastItem;
);

impl SPUAppcastItem {
    extern_methods!(
        /// Returns the display version string (e.g., "1.2.3").
        #[unsafe(method(displayVersionString))]
        pub fn display_version_string(&self) -> Retained<NSString>;

        /// Returns the internal version string (CFBundleVersion).
        #[unsafe(method(versionString))]
        pub fn version_string(&self) -> Retained<NSString>;

        /// Returns the release notes HTML content.
        #[unsafe(method(itemDescription))]
        pub fn item_description(&self) -> Option<Retained<NSString>>;

        /// Returns the download URL for this update.
        #[unsafe(method(fileURL))]
        pub fn file_url(&self) -> Option<Retained<NSURL>>;

        /// Returns the content length of the download.
        #[unsafe(method(contentLength))]
        pub fn content_length(&self) -> u64;
    );
}
