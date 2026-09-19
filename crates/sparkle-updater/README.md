# sparkle-updater

A macOS-only Rust API for Sparkle 2.9.6. It has no Tauri, GPUI, Tokio, or other application-runtime dependency. The repository's root Tauri plugin is an adapter over this crate.

## Build and bundle

Download the official framework explicitly. In this repository:

```sh
bash scripts/download-sparkle.sh
export SPARKLE_FRAMEWORK_PATH="$PWD"
export DYLD_FRAMEWORK_PATH="$PWD" # Only for local tests and examples.
cargo test --workspace
cargo run -p sparkle-updater --example native
```

`SPARKLE_FRAMEWORK_PATH` is the directory **containing** `Sparkle.framework`. The build script links the framework; it never downloads, bundles, signs, or notarizes it. The repository helper pins the archive version and SHA256. For local development it also searches ancestors of Cargo's output and manifest directories.

The host packaging pipeline must copy the complete framework, including its helpers and XPC services, into `YourApp.app/Contents/Frameworks`, add an `@executable_path/../Frameworks` runtime search path, sign nested code and the application, and notarize the release. Follow [Sparkle's distribution guide](https://sparkle-project.org/documentation/).

Configure the host's `Info.plist` with `CFBundleIdentifier`, `CFBundleVersion`, `CFBundleShortVersionString`, `SUFeedURL`, and `SUPublicEDKey`. Sparkle owns update verification, download, installation, and its native UI. Set automatic-check preferences explicitly when your product needs a particular first-run policy.

## Host integration

```rust,no_run
use sparkle_updater::{MainThreadMarker, SparkleUpdater, UpdaterConfig};
use std::rc::Rc;

let mtm = MainThreadMarker::new().expect("Initialize on the AppKit main thread");
let updater = SparkleUpdater::new(mtm, UpdaterConfig {
    event_callback: Some(Rc::new(|event| println!("{event:?}"))),
    ..Default::default()
})?;
// Retain updater for the host's lifetime and run the host's AppKit event loop.
# Ok::<(), sparkle_updater::Error>(())
```

`SparkleUpdater` is `!Send` and `!Sync`: initialize, use, and release it on the main thread. `new` installs callbacks before starting Sparkle and returns `None` outside an `.app` bundle. A running AppKit application/event loop is required for real update operations. The library does not create one for you.

Events are typed `UpdateEvent` values. Event callbacks are informational; they do not decide whether an update may proceed. Callbacks run synchronously on the main thread: keep them brief, never panic, and marshal background work through the host's own executor. Avoid capturing a strong reference to the updater inside a callback it owns.

## Relaunch and reminder decisions

`UpdaterConfig::relaunch_handler` receives the update and a `RelaunchContinuation`. Store the continuation on the main thread while the host finishes saving, then consume it with `resume(mtm)`. It cannot be cloned or sent across threads. Dropping it does not resume installation. Never synchronously wait on background work that itself needs the main thread.

This hook is not a universal termination veto: Sparkle may install after an ordinary application quit without invoking it. Integrate saving with the host's regular quit lifecycle too. `set_should_relaunch_application(false)` controls relaunch; it does not cancel termination.

`UpdaterConfig::gentle_reminders` accepts an `Rc<dyn GentleReminders>`. Configure it before startup. Returning `true` from `should_show_scheduled_update` keeps Sparkle's native reminder UI. Returning `false` transfers reminder presentation to the host; implement the trait's lifecycle notifications, present an accessible update affordance, and call `check_for_updates()` when the user activates it.

The low-level Objective-C bindings remain private. The public API intentionally wraps the subset needed by the Rust and Tauri integrations rather than mirroring every Sparkle symbol.
