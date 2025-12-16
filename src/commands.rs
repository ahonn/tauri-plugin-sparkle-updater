use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::SparkleUpdaterExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.sparkle_updater().ping(payload)
}
