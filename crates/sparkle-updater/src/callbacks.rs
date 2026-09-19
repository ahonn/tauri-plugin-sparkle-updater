//! Host decisions that cannot be represented by informational update events.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use block2::{Block, RcBlock};
use objc2::MainThreadMarker;

use crate::events::UpdateInfo;
pub use crate::events::UserUpdateState;

/// Called when Sparkle permits delaying its relaunch while the host saves work.
///
/// Retain the continuation and resume it on the main thread after saving succeeds.
/// Dropping it does not approve installation. This callback is not a universal quit
/// veto: Sparkle does not invoke it on every termination path, so hosts still need
/// their normal application termination handling.
pub type RelaunchHandler = Rc<dyn Fn(UpdateInfo, RelaunchContinuation)>;

/// A main-thread-only, one-shot continuation for a postponed Sparkle relaunch.
///
/// Dropping this value without resuming leaves that relaunch postponed.
#[must_use = "retain and resume the continuation after saving, or deliberately leave relaunch postponed"]
pub struct RelaunchContinuation {
    state: Rc<ContinuationState>,
}

impl RelaunchContinuation {
    /// Allow Sparkle to continue with installation and relaunch.
    ///
    /// Calling this synchronously inside the relaunch handler allows the original
    /// delegate call to proceed without invoking Sparkle's block reentrantly.
    pub fn resume(self, _main_thread: MainThreadMarker) {
        self.state.resume();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContinuationPhase {
    InCallback,
    Postponed,
    Resumed,
}

struct ContinuationState {
    phase: Cell<ContinuationPhase>,
    block: RefCell<Option<RcBlock<dyn Fn()>>>,
}

impl ContinuationState {
    fn new(block: &Block<dyn Fn()>) -> Rc<Self> {
        Rc::new(Self {
            phase: Cell::new(ContinuationPhase::InCallback),
            block: RefCell::new(Some(block.copy())),
        })
    }

    fn resume(&self) {
        let previous = self.phase.replace(ContinuationPhase::Resumed);
        let block = self.block.borrow_mut().take();
        if previous == ContinuationPhase::Postponed {
            if let Some(block) = block {
                block.call(());
            }
        }
    }

    fn finish_callback(&self) -> bool {
        if self.phase.get() == ContinuationPhase::Resumed {
            false
        } else {
            self.phase.set(ContinuationPhase::Postponed);
            true
        }
    }
}

pub(crate) fn postpone_relaunch(
    handler: &RelaunchHandler,
    update: UpdateInfo,
    block: &Block<dyn Fn()>,
) -> bool {
    let state = ContinuationState::new(block);
    handler(
        update,
        RelaunchContinuation {
            state: state.clone(),
        },
    );
    state.finish_callback()
}

/// Host-provided gentle reminders for scheduled updates.
///
/// Install this delegate before starting the updater. All callbacks run on the
/// main thread. User-initiated checks always use Sparkle's standard interface.
pub trait GentleReminders {
    /// Return `true` to let Sparkle present the scheduled update (the default).
    /// Return `false` to take responsibility for presenting a reminder from
    /// [`Self::will_show_update`]. Keep this decision free of side effects.
    fn should_show_scheduled_update(&self, _update: &UpdateInfo, _immediate_focus: bool) -> bool {
        true
    }

    /// Present any custom reminder when `handled_by_sparkle` is false. Calling
    /// the updater's `check_for_updates` method brings Sparkle's dialog forward.
    fn will_show_update(
        &self,
        handled_by_sparkle: bool,
        update: &UpdateInfo,
        state: UserUpdateState,
    );

    /// Dismiss attention indicators once the user interacts with the update.
    fn did_receive_user_attention(&self, _update: &UpdateInfo) {}

    /// Remove session indicators after an update is dismissed or fails.
    fn will_finish_update_session(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_continuation_calls_copied_block_once() {
        let calls = Rc::new(Cell::new(0));
        let captured = calls.clone();
        let block = RcBlock::new(move || captured.set(captured.get() + 1));
        let state = ContinuationState::new(&block);
        drop(block);
        assert!(state.finish_callback());
        state.resume();
        state.resume();
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn synchronous_resume_does_not_call_sparkle_block_reentrantly() {
        let calls = Rc::new(Cell::new(0));
        let captured = calls.clone();
        let block = RcBlock::new(move || captured.set(captured.get() + 1));
        let state = ContinuationState::new(&block);
        state.resume();
        assert!(!state.finish_callback());
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn dropping_postponed_continuation_does_not_resume() {
        let calls = Rc::new(Cell::new(0));
        let captured = calls.clone();
        let block = RcBlock::new(move || captured.set(captured.get() + 1));
        let state = ContinuationState::new(&block);
        assert!(state.finish_callback());
        drop(RelaunchContinuation { state });
        assert_eq!(calls.get(), 0);
    }
}
