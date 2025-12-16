use tauri::{command, AppHandle, Runtime};

use crate::Result;
use crate::SparkleUpdaterExt;

#[command]
pub(crate) async fn check_for_updates<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.sparkle_updater().check_for_updates()
}

#[command]
pub(crate) async fn check_for_updates_in_background<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.sparkle_updater().check_for_updates_in_background()
}

#[command]
pub(crate) async fn can_check_for_updates<R: Runtime>(app: AppHandle<R>) -> Result<bool> {
    app.sparkle_updater().can_check_for_updates()
}

#[command]
pub(crate) async fn current_version<R: Runtime>(app: AppHandle<R>) -> Result<String> {
    app.sparkle_updater().current_version()
}

#[command]
pub(crate) async fn feed_url<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>> {
    app.sparkle_updater().feed_url()
}

#[command]
pub(crate) async fn set_feed_url<R: Runtime>(app: AppHandle<R>, url: String) -> Result<()> {
    app.sparkle_updater().set_feed_url(&url)
}

#[command]
pub(crate) async fn automatically_checks_for_updates<R: Runtime>(
    app: AppHandle<R>,
) -> Result<bool> {
    app.sparkle_updater().automatically_checks_for_updates()
}

#[command]
pub(crate) async fn set_automatically_checks_for_updates<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<()> {
    app.sparkle_updater()
        .set_automatically_checks_for_updates(enabled)
}

#[command]
pub(crate) async fn automatically_downloads_updates<R: Runtime>(
    app: AppHandle<R>,
) -> Result<bool> {
    app.sparkle_updater().automatically_downloads_updates()
}

#[command]
pub(crate) async fn set_automatically_downloads_updates<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<()> {
    app.sparkle_updater()
        .set_automatically_downloads_updates(enabled)
}

#[command]
pub(crate) async fn last_update_check_date<R: Runtime>(app: AppHandle<R>) -> Result<Option<f64>> {
    app.sparkle_updater().last_update_check_date()
}

#[command]
pub(crate) async fn reset_update_cycle<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.sparkle_updater().reset_update_cycle()
}
