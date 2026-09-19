//! Runs on the process main thread and invokes the real Objective-C selectors.
#![allow(dead_code)]

mod bindings;
// A custom harness does not register #[test] functions in these shared modules.
#[allow(unused_imports)]
mod callbacks;
mod delegate;
#[allow(unused_imports)]
mod events;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{msg_send, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSDictionary, NSString};

use bindings::{SPUAppcastItem, SPUUserUpdateState};
use callbacks::{GentleReminders, RelaunchContinuation};
use delegate::SparkleDelegate;
use events::{UpdateEvent, UpdateInfo, UserUpdateStage, UserUpdateState};

fn update_item(mtm: MainThreadMarker) -> Retained<SPUAppcastItem> {
    let version_key = NSString::from_str("sparkle:version");
    let url_key = NSString::from_str("url");
    let version = NSString::from_str("2.0");
    let url = NSString::from_str("https://example.com/App_2.0.zip");
    let enclosure = NSDictionary::from_slices(&[&*version_key, &*url_key], &[&*version, &*url]);
    let enclosure_key = NSString::from_str("enclosure");
    let dict = NSDictionary::<NSString, AnyObject>::from_slices(&[&*enclosure_key], &[&*enclosure]);
    // The deprecated dictionary initializer is confined to this fixture; it
    // constructs a real Sparkle item without requiring a network update cycle.
    unsafe { msg_send![SPUAppcastItem::alloc(mtm), initWithDictionary: &*dict] }
}

struct Reminders {
    shown: Cell<bool>,
    attention: Cell<bool>,
    finished: Cell<bool>,
}

impl GentleReminders for Reminders {
    fn should_show_scheduled_update(&self, update: &UpdateInfo, immediate_focus: bool) -> bool {
        assert_eq!(update.version, "2.0");
        assert!(!immediate_focus);
        false
    }

    fn will_show_update(
        &self,
        handled_by_sparkle: bool,
        update: &UpdateInfo,
        state: UserUpdateState,
    ) {
        assert!(!handled_by_sparkle);
        assert_eq!(update.version, "2.0");
        assert_eq!(state.stage, UserUpdateStage::Downloaded);
        assert!(state.user_initiated);
        self.shown.set(true);
    }

    fn did_receive_user_attention(&self, _update: &UpdateInfo) {
        self.attention.set(true);
    }

    fn will_finish_update_session(&self) {
        self.finished.set(true);
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("native harness runs on main thread");
    objc2::rc::autoreleasepool(|_| {
        let delegate = SparkleDelegate::new(mtm);
        let updater = NSObject::new();
        let item = update_item(mtm);
        // Sparkle's private initializer is used only by this ABI regression
        // fixture so the state can have a known nonzero stage.
        let state: Retained<SPUUserUpdateState> = unsafe {
            msg_send![SPUUserUpdateState::alloc(mtm), initWithStage: 1isize, userInitiated: true]
        };
        let seen = Rc::new(Cell::new(false));
        let captured = seen.clone();
        let reentrant_delegate = delegate.clone();
        delegate.set_event_callback(Some(Rc::new(move |event| {
            if let UpdateEvent::UserDidMakeChoice(info) = event {
                assert_eq!(info.stage, "downloaded");
                assert_eq!(info.choice, "install");
                captured.set(true);
                reentrant_delegate.set_event_callback(None);
            }
        })));
        unsafe {
            let _: () = msg_send![&*delegate, updater: &*updater, userDidMakeChoice: 1isize, forUpdate: &*item, state: &*state];
        }
        assert!(seen.get());

        let reminders = Rc::new(Reminders {
            shown: Cell::new(false),
            attention: Cell::new(false),
            finished: Cell::new(false),
        });
        let supported: bool =
            unsafe { msg_send![&*delegate, supportsGentleScheduledUpdateReminders] };
        assert!(!supported);
        delegate.set_gentle_reminders(Some(reminders.clone()));
        let supported: bool =
            unsafe { msg_send![&*delegate, supportsGentleScheduledUpdateReminders] };
        assert!(supported);
        let show: bool = unsafe {
            msg_send![&*delegate, standardUserDriverShouldHandleShowingScheduledUpdate: &*item, andInImmediateFocus: false]
        };
        assert!(!show);
        unsafe {
            let _: () = msg_send![&*delegate, standardUserDriverWillHandleShowingUpdate: false, forUpdate: &*item, state: &*state];
            let _: () =
                msg_send![&*delegate, standardUserDriverDidReceiveUserAttentionForUpdate: &*item];
            let _: () = msg_send![&*delegate, standardUserDriverWillFinishUpdateSession];
        }
        assert!(reminders.shown.get() && reminders.attention.get() && reminders.finished.get());

        let continuation: Rc<RefCell<Option<RelaunchContinuation>>> = Rc::new(RefCell::new(None));
        let captured = continuation.clone();
        delegate.set_relaunch_handler(Some(Rc::new(move |_, resume| {
            *captured.borrow_mut() = Some(resume);
        })));
        let calls = Rc::new(Cell::new(0));
        let captured = calls.clone();
        let block = RcBlock::new(move || captured.set(captured.get() + 1));
        let postponed: bool = unsafe {
            msg_send![&*delegate, updater: &*updater, shouldPostponeRelaunchForUpdate: &*item, untilInvokingBlock: &*block]
        };
        assert!(postponed);
        assert_eq!(calls.get(), 0);
        drop(block);
        continuation.borrow_mut().take().unwrap().resume(mtm);
        assert_eq!(calls.get(), 1);

        delegate.set_relaunch_handler(Some(Rc::new(move |_, continuation| {
            continuation.resume(mtm)
        })));
        let captured = calls.clone();
        let block = RcBlock::new(move || captured.set(captured.get() + 1));
        let postponed: bool = unsafe {
            msg_send![&*delegate, updater: &*updater, shouldPostponeRelaunchForUpdate: &*item, untilInvokingBlock: &*block]
        };
        assert!(!postponed);
        assert_eq!(calls.get(), 1);
        println!("native selectors: state object, gentle reminders, deferred and synchronous relaunch passed");
    });
}
