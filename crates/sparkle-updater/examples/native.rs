//! Run from an app bundle to test native initialization without a UI framework.
use sparkle_updater::{MainThreadMarker, SparkleUpdater, UpdaterConfig};
use std::rc::Rc;

fn main() -> sparkle_updater::Result<()> {
    let mtm = MainThreadMarker::new().expect("Run on the macOS main thread");
    let _application = objc2_app_kit::NSApplication::sharedApplication(mtm);
    let updater = SparkleUpdater::new(
        mtm,
        UpdaterConfig {
            event_callback: Some(Rc::new(|event| println!("{event:?}"))),
            ..Default::default()
        },
    )?;
    let Some(updater) = updater else {
        println!("No application bundle: updater correctly disabled");
        return Ok(());
    };
    println!("Version: {}", updater.current_version()?);
    println!("Feed: {:?}", updater.feed_url()?);
    println!("Can check: {}", updater.can_check_for_updates()?);
    // A real host retains the updater and runs its AppKit event loop here.
    Ok(())
}
