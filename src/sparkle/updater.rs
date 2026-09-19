use crate::events::UpdateInfo;
use crate::{Error, Result};
use dispatch::Queue;
use objc2::MainThreadMarker;
use std::{
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};
use tauri::{AppHandle, Emitter, Runtime};

pub type EventCallback = Arc<dyn Fn(&str, &serde_json::Value) + Send + Sync>;
type CallbackSlot = Arc<RwLock<Option<EventCallback>>>;

thread_local! {
    static UPDATERS: RefCell<HashMap<u64, Rc<sparkle_updater::SparkleUpdater>>> = RefCell::new(HashMap::new());
}
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A thread-safe Tauri handle. Native objects live exclusively on the main thread.
pub struct SparkleUpdater<R: Runtime> {
    id: u64,
    version: String,
    callback: CallbackSlot,
    runtime: PhantomData<fn() -> R>,
}

pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<Option<SparkleUpdater<R>>> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| Error::SparkleInit("Must initialize on the main thread".into()))?;
    let callback: CallbackSlot = Arc::new(RwLock::new(None));
    let callback_slot = callback.clone();
    let handle = app.clone();
    let updater = sparkle_updater::init(
        mtm,
        Some(Rc::new(move |event| {
            let name = event.name();
            let payload = event.payload();
            if let Err(error) = handle.emit(name, &payload) {
                log::error!("Failed to emit {name}: {error}");
            }
            let callback = callback_slot.read().unwrap().clone();
            if let Some(callback) = callback {
                callback(name, &payload);
            }
        })),
    )?;
    let Some(updater) = updater else {
        return Ok(None);
    };
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    UPDATERS.with(|registry| registry.borrow_mut().insert(id, Rc::new(updater)));
    Ok(Some(SparkleUpdater {
        id,
        version: app.package_info().version.to_string(),
        callback,
        runtime: PhantomData,
    }))
}

impl<R: Runtime> SparkleUpdater<R> {
    fn dispatch<T: Send>(
        &self,
        f: impl FnOnce(&sparkle_updater::SparkleUpdater) -> Result<T> + Send,
    ) -> Result<T> {
        let id = self.id;
        let action = move || {
            // Release the registry borrow before callbacks can reenter this handle.
            let updater = UPDATERS.with(|registry| registry.borrow().get(&id).cloned());
            updater.as_deref().ok_or(Error::UpdaterNotReady).and_then(f)
        };
        if MainThreadMarker::new().is_some() {
            action()
        } else {
            Queue::main().exec_sync(action)
        }
    }

    pub fn current_version(&self) -> Result<String> {
        Ok(self.version.clone())
    }
    pub fn set_event_callback(&self, callback: Option<EventCallback>) {
        *self.callback.write().unwrap() = callback;
    }

    pub(crate) fn shutdown(&self) {
        let id = self.id;
        let remove = move || {
            // During process teardown the thread-local registry may already be
            // destroying its values (which can release Tauri's managed state).
            let updater = UPDATERS.try_with(|registry| registry.borrow_mut().remove(&id));
            drop(updater);
        };
        if MainThreadMarker::new().is_some() {
            remove();
        } else {
            Queue::main().exec_async(remove);
        }
    }
    pub fn check_for_updates(&self) -> Result<()> {
        self.dispatch(move |updater| updater.check_for_updates())
    }

    pub fn check_for_updates_in_background(&self) -> Result<()> {
        self.dispatch(move |updater| updater.check_for_updates_in_background())
    }

    pub fn can_check_for_updates(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.can_check_for_updates())
    }

    pub fn feed_url(&self) -> Result<Option<String>> {
        self.dispatch(move |updater| updater.feed_url())
    }

    pub fn set_feed_url(&self, url: &str) -> Result<()> {
        let url = url.to_owned();
        self.dispatch(move |updater| updater.set_feed_url(&url))
    }

    pub fn automatically_checks_for_updates(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.automatically_checks_for_updates())
    }

    pub fn set_automatically_checks_for_updates(&self, enabled: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_automatically_checks_for_updates(enabled))
    }

    pub fn automatically_downloads_updates(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.automatically_downloads_updates())
    }

    pub fn set_automatically_downloads_updates(&self, enabled: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_automatically_downloads_updates(enabled))
    }

    pub fn last_update_check_date(&self) -> Result<Option<f64>> {
        self.dispatch(move |updater| updater.last_update_check_date())
    }

    pub fn reset_update_cycle(&self) -> Result<()> {
        self.dispatch(move |updater| updater.reset_update_cycle())
    }

    pub fn update_check_interval(&self) -> Result<f64> {
        self.dispatch(move |updater| updater.update_check_interval())
    }

    pub fn set_update_check_interval(&self, interval: f64) -> Result<()> {
        self.dispatch(move |updater| updater.set_update_check_interval(interval))
    }

    pub fn check_for_update_information(&self) -> Result<()> {
        self.dispatch(move |updater| updater.check_for_update_information())
    }

    pub fn session_in_progress(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.session_in_progress())
    }

    pub fn http_headers(&self) -> Result<Option<HashMap<String, String>>> {
        self.dispatch(move |updater| updater.http_headers())
    }

    pub fn set_http_headers(&self, headers: Option<HashMap<String, String>>) -> Result<()> {
        self.dispatch(move |updater| updater.set_http_headers(headers))
    }

    pub fn user_agent_string(&self) -> Result<String> {
        self.dispatch(move |updater| updater.user_agent_string())
    }

    pub fn set_user_agent_string(&self, user_agent: &str) -> Result<()> {
        let user_agent = user_agent.to_owned();
        self.dispatch(move |updater| updater.set_user_agent_string(&user_agent))
    }

    pub fn sends_system_profile(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.sends_system_profile())
    }

    pub fn set_sends_system_profile(&self, sends: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_sends_system_profile(sends))
    }

    pub fn clear_feed_url_from_user_defaults(&self) -> Result<Option<String>> {
        self.dispatch(move |updater| updater.clear_feed_url_from_user_defaults())
    }

    pub fn reset_update_cycle_after_short_delay(&self) -> Result<()> {
        self.dispatch(move |updater| updater.reset_update_cycle_after_short_delay())
    }

    pub fn allowed_channels(&self) -> Result<Option<Vec<String>>> {
        self.dispatch(move |updater| updater.allowed_channels())
    }

    pub fn set_allowed_channels(&self, channels: Option<Vec<String>>) -> Result<()> {
        self.dispatch(move |updater| updater.set_allowed_channels(channels))
    }

    pub fn feed_url_override(&self) -> Result<Option<String>> {
        self.dispatch(move |updater| updater.feed_url_override())
    }

    pub fn set_feed_url_override(&self, url: Option<String>) -> Result<()> {
        self.dispatch(move |updater| updater.set_feed_url_override(url))
    }

    pub fn feed_parameters(&self) -> Result<Option<HashMap<String, String>>> {
        self.dispatch(move |updater| updater.feed_parameters())
    }

    pub fn set_feed_parameters(&self, params: Option<HashMap<String, String>>) -> Result<()> {
        self.dispatch(move |updater| updater.set_feed_parameters(params))
    }

    pub fn should_download_release_notes(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.should_download_release_notes())
    }

    pub fn set_should_download_release_notes(&self, enabled: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_should_download_release_notes(enabled))
    }

    pub fn should_relaunch_application(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.should_relaunch_application())
    }

    pub fn set_should_relaunch_application(&self, enabled: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_should_relaunch_application(enabled))
    }

    pub fn may_check_for_updates_config(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.may_check_for_updates_config())
    }

    pub fn set_may_check_for_updates_config(&self, enabled: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_may_check_for_updates_config(enabled))
    }

    pub fn should_proceed_with_update(&self) -> Result<bool> {
        self.dispatch(move |updater| updater.should_proceed_with_update())
    }

    pub fn set_should_proceed_with_update(&self, enabled: bool) -> Result<()> {
        self.dispatch(move |updater| updater.set_should_proceed_with_update(enabled))
    }

    pub fn decryption_password(&self) -> Result<Option<String>> {
        self.dispatch(move |updater| updater.decryption_password())
    }

    pub fn set_decryption_password(&self, password: Option<String>) -> Result<()> {
        self.dispatch(move |updater| updater.set_decryption_password(password))
    }

    pub fn last_found_update(&self) -> Result<Option<UpdateInfo>> {
        self.dispatch(move |updater| updater.last_found_update())
    }

    pub fn download_request_headers(&self) -> Result<Option<HashMap<String, String>>> {
        self.dispatch(move |updater| updater.download_request_headers())
    }

    pub fn set_download_request_headers(
        &self,
        headers: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.dispatch(move |updater| updater.set_download_request_headers(headers))
    }
}

impl<R: Runtime> Drop for SparkleUpdater<R> {
    fn drop(&mut self) {
        self.shutdown();
    }
}
