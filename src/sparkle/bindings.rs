use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{extern_class, extern_methods, MainThreadOnly};
use objc2_foundation::{NSDate, NSError, NSString, NSURL};

extern_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUStandardUpdaterController"]
    #[derive(Debug)]
    pub struct SPUStandardUpdaterController;
);

impl SPUStandardUpdaterController {
    extern_methods!(
        #[unsafe(method(initWithStartingUpdater:updaterDelegate:userDriverDelegate:))]
        pub fn init_with_starting_updater(
            this: objc2::rc::Allocated<Self>,
            starting_updater: bool,
            updater_delegate: Option<&NSObject>,
            user_driver_delegate: Option<&NSObject>,
        ) -> Retained<Self>;

        #[unsafe(method(updater))]
        pub fn updater(&self) -> Retained<SPUUpdater>;

        #[unsafe(method(checkForUpdates:))]
        pub fn check_for_updates(&self, sender: Option<&NSObject>);
    );
}

extern_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUUpdater"]
    #[derive(Debug)]
    pub struct SPUUpdater;
);

impl SPUUpdater {
    extern_methods!(
        #[unsafe(method(canCheckForUpdates))]
        pub fn can_check_for_updates(&self) -> bool;

        #[unsafe(method(checkForUpdates))]
        pub fn check_for_updates(&self);

        #[unsafe(method(checkForUpdatesInBackground))]
        pub fn check_for_updates_in_background(&self);

        #[unsafe(method(feedURL))]
        pub fn feed_url(&self) -> Option<Retained<NSURL>>;

        #[unsafe(method(setFeedURL:))]
        pub fn set_feed_url(&self, url: Option<&NSURL>);

        #[unsafe(method(automaticallyChecksForUpdates))]
        pub fn automatically_checks_for_updates(&self) -> bool;

        #[unsafe(method(setAutomaticallyChecksForUpdates:))]
        pub fn set_automatically_checks_for_updates(&self, enabled: bool);

        #[unsafe(method(automaticallyDownloadsUpdates))]
        pub fn automatically_downloads_updates(&self) -> bool;

        #[unsafe(method(setAutomaticallyDownloadsUpdates:))]
        pub fn set_automatically_downloads_updates(&self, enabled: bool);

        #[unsafe(method(lastUpdateCheckDate))]
        pub fn last_update_check_date(&self) -> Option<Retained<NSDate>>;

        #[unsafe(method(resetUpdateCycle))]
        pub fn reset_update_cycle(&self);

        #[unsafe(method(updateCheckInterval))]
        pub fn update_check_interval(&self) -> f64;

        #[unsafe(method(setUpdateCheckInterval:))]
        pub fn set_update_check_interval(&self, interval: f64);

        #[unsafe(method(checkForUpdateInformation))]
        pub fn check_for_update_information(&self);

        #[unsafe(method(sessionInProgress))]
        pub fn session_in_progress(&self) -> bool;

        #[unsafe(method(startUpdater:))]
        pub fn start_updater(&self, error: *mut *mut NSError) -> bool;

        #[unsafe(method(setDelegate:))]
        pub fn set_delegate(&self, delegate: Option<&NSObject>);
    );
}

extern_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUAppcastItem"]
    #[derive(Debug)]
    pub struct SPUAppcastItem;
);

impl SPUAppcastItem {
    extern_methods!(
        #[unsafe(method(displayVersionString))]
        pub fn display_version_string(&self) -> Retained<NSString>;

        #[unsafe(method(versionString))]
        pub fn version_string(&self) -> Retained<NSString>;

        #[unsafe(method(itemDescription))]
        pub fn item_description(&self) -> Option<Retained<NSString>>;

        #[unsafe(method(fileURL))]
        pub fn file_url(&self) -> Option<Retained<NSURL>>;

        #[unsafe(method(contentLength))]
        pub fn content_length(&self) -> u64;

        #[unsafe(method(title))]
        pub fn title(&self) -> Option<Retained<NSString>>;

        #[unsafe(method(releaseNotesURL))]
        pub fn release_notes_url(&self) -> Option<Retained<NSURL>>;

        #[unsafe(method(infoURL))]
        pub fn info_url(&self) -> Option<Retained<NSURL>>;

        #[unsafe(method(minimumSystemVersion))]
        pub fn minimum_system_version(&self) -> Option<Retained<NSString>>;

        #[unsafe(method(isCriticalUpdate))]
        pub fn is_critical_update(&self) -> bool;

        #[unsafe(method(isMajorUpgrade))]
        pub fn is_major_upgrade(&self) -> bool;

        #[unsafe(method(channel))]
        pub fn channel(&self) -> Option<Retained<NSString>>;

        #[unsafe(method(isInformationOnlyUpdate))]
        pub fn is_information_only_update(&self) -> bool;

        #[unsafe(method(date))]
        pub fn date(&self) -> Option<Retained<NSDate>>;
    );
}
