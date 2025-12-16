use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod config;
mod error;
mod models;

pub use config::{Config, ConfigError};
pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::SparkleUpdater;
#[cfg(mobile)]
use mobile::SparkleUpdater;

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
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      let config = api.config().clone();
      config.validate()?;

      #[cfg(mobile)]
      let sparkle_updater = mobile::init(app, api)?;
      #[cfg(desktop)]
      let sparkle_updater = desktop::init(app, api)?;
      app.manage(sparkle_updater);
      Ok(())
    })
    .build()
}
