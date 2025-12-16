use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod config;
mod error;
mod events;
mod sparkle;

pub use config::{Config, ConfigError};
pub use error::{Error, Result};

use sparkle::SparkleUpdater;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sparkle-updater APIs.
pub trait SparkleUpdaterExt<R: Runtime> {
    fn sparkle_updater(&self) -> &SparkleUpdater<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SparkleUpdaterExt<R> for T {
    fn sparkle_updater(&self) -> &SparkleUpdater<R> {
        self.state::<SparkleUpdater<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R, Config> {
    Builder::<R, Config>::new("sparkle-updater")
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
        ])
        .setup(|app, api| {
            let config = api.config().clone();
            config.validate()?;

            let sparkle_updater = sparkle::init(app, api)?;
            app.manage(sparkle_updater);
            Ok(())
        })
        .build()
}
