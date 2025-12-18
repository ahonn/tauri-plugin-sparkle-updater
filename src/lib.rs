use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod error;
mod events;
mod sparkle;

pub use error::{Error, Result};

use sparkle::SparkleUpdater;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sparkle-updater APIs.
pub trait SparkleUpdaterExt<R: Runtime> {
    /// Returns the SparkleUpdater if available.
    ///
    /// Returns `None` when running outside a valid macOS bundle (e.g., during `tauri dev`).
    fn sparkle_updater(&self) -> Option<tauri::State<'_, SparkleUpdater<R>>>;
}

impl<R: Runtime, T: Manager<R>> crate::SparkleUpdaterExt<R> for T {
    fn sparkle_updater(&self) -> Option<tauri::State<'_, SparkleUpdater<R>>> {
        self.try_state::<SparkleUpdater<R>>()
    }
}

/// Initializes the plugin.
///
/// Sparkle configuration is read from the app's Info.plist:
/// - `SUFeedURL` - Appcast feed URL
/// - `SUPublicEDKey` - Ed25519 public key for signature verification
/// - `SUEnableAutomaticChecks` - Enable automatic update checks (default: true)
/// - `SUAutomaticallyUpdate` - Automatically download and install updates (default: false)
/// - `SUScheduledCheckInterval` - Check interval in seconds (default: 86400)
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("sparkle-updater")
        .invoke_handler(tauri::generate_handler![
            commands::check_for_updates,
            commands::check_for_updates_in_background,
            commands::can_check_for_updates,
            commands::current_version,
            commands::feed_url,
            commands::set_feed_url,
            commands::automatically_checks_for_updates,
            commands::set_automatically_checks_for_updates,
            commands::automatically_downloads_updates,
            commands::set_automatically_downloads_updates,
            commands::last_update_check_date,
            commands::reset_update_cycle,
            commands::update_check_interval,
            commands::set_update_check_interval,
            commands::check_for_update_information,
            commands::session_in_progress,
            commands::http_headers,
            commands::set_http_headers,
            commands::user_agent_string,
            commands::set_user_agent_string,
            commands::sends_system_profile,
            commands::set_sends_system_profile,
            commands::clear_feed_url_from_user_defaults,
            commands::reset_update_cycle_after_short_delay,
            commands::allowed_channels,
            commands::set_allowed_channels,
            commands::feed_url_override,
            commands::set_feed_url_override,
            commands::feed_parameters,
            commands::set_feed_parameters,
            commands::should_download_release_notes,
            commands::set_should_download_release_notes,
            commands::should_relaunch_application,
            commands::set_should_relaunch_application,
            commands::may_check_for_updates_config,
            commands::set_may_check_for_updates_config,
            commands::should_proceed_with_update,
            commands::set_should_proceed_with_update,
            commands::decryption_password,
            commands::set_decryption_password,
            commands::last_found_update,
        ])
        .setup(|app, _api| {
            if let Some(sparkle_updater) = sparkle::init(app)? {
                app.manage(sparkle_updater);
            }
            Ok(())
        })
        .build()
}
