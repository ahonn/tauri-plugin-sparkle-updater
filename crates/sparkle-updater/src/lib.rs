#![doc = include_str!("../README.md")]
#![cfg(target_os = "macos")]

mod bindings;
mod callbacks;
mod delegate;
mod error;
pub mod events;
mod updater;

pub use callbacks::{GentleReminders, RelaunchContinuation, RelaunchHandler};
pub use delegate::EventCallback;
pub use error::{Error, Result};
pub use events::{UpdateEvent, UserUpdateStage, UserUpdateState};
pub use objc2::MainThreadMarker;
pub use updater::{init, SparkleUpdater, UpdaterConfig};

#[cfg(test)]
mod tests {
    static_assertions::assert_not_impl_any!(super::SparkleUpdater: Send, Sync);
    static_assertions::assert_not_impl_any!(super::RelaunchContinuation: Send, Sync);
}
