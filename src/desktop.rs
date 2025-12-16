use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<SparkleUpdater<R>> {
  Ok(SparkleUpdater(app.clone()))
}

/// Access to the sparkle-updater APIs.
pub struct SparkleUpdater<R: Runtime>(AppHandle<R>);

impl<R: Runtime> SparkleUpdater<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
