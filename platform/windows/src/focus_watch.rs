//! Watches which control has the keyboard focus and classifies each one, so the keyboard hook knows whether to type.

#[cfg(any(windows, test))]
use std::fmt;
#[cfg(any(windows, test))]
use std::sync::Arc;

#[cfg(any(windows, test))]
use platform_clock::Timestamp;

use crate::FocusClass;
#[cfg(any(windows, test))]
use crate::{ElementProps, FocusCell};

/// One classification of the focused control, as the watch hands it on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusChange {
    /// The lower-cased executable file name of the app whose top-level window holds the focus, or empty when it cannot
    /// be read. A Microsoft Store app's top-level window is its frame, so it is `applicationframehost.exe`.
    pub app: String,
    /// The app is not the one of the previous change; the first change always counts as a new app.
    pub app_changed: bool,
    pub class: FocusClass,
}

/// The change for a classification of a control in `app`, given the app of the change before it.
#[cfg(any(windows, test))]
fn next_change(previous_app: Option<&str>, app: String, class: FocusClass) -> FocusChange {
    FocusChange {
        app_changed: previous_app != Some(app.as_str()),
        app,
        class,
    }
}

/// The name an app goes by: its executable's file name in lower case, so `C:\Games\Game.EXE` is `game.exe`.
#[cfg(any(windows, test))]
fn app_name(image_path: &str) -> String {
    image_path.rsplit(['\\', '/']).next().unwrap_or_default().to_lowercase()
}

/// The window whose app a classification is about: the window a focus change happened in; for a recheck, which has
/// none, the window with the focus in the thread in front (`focus`), or else the window in front (`front`). The window
/// in front can still be the previous app's just after a switch, so it comes last, and each lookup is made only when
/// the one before it found nothing.
#[cfg(any(windows, test))]
fn app_window<W>(event: Option<W>, focus: impl FnOnce() -> Option<W>, front: impl FnOnce() -> Option<W>) -> Option<W> {
    event.or_else(focus).or_else(front)
}

/// One line of the focus log: the time, the app, what UI Automation said about the control (or why it could not be
/// read), and the class it was given.
#[cfg(any(windows, test))]
fn log_line<E: fmt::Display + ?Sized>(
    time: Timestamp,
    app: &str,
    reading: Result<&ElementProps, &E>,
    class: FocusClass,
) -> String {
    match reading {
        Ok(props) => format!(
            "{time} process={app:?} class_name={:?} control_type={} password={} enabled={} text_edit={} \
             value_read_only={:?} es_password={} result={class:?}\n",
            props.class_name,
            props.control_type,
            props.flags.password,
            props.flags.enabled,
            props.flags.text_edit,
            props.value_read_only,
            props.es_password,
        ),
        Err(error) => format!(
            "{time} process={app:?} error={:?} result={class:?}\n",
            error.to_string()
        ),
    }
}

/// Marks the focus pending, then hands the generation the answer must carry to `post`. The order matters: the keyboard
/// hook sees pending from the moment it asks, before the watch thread has heard of the recheck.
#[cfg(any(windows, test))]
fn request_recheck<E>(cell: &FocusCell, post: impl FnOnce(u64) -> Result<(), E>) -> Result<(), E> {
    post(cell.invalidate())
}

/// The focus changes that arrive while the watch is still classifying an earlier one, kept apart from the rest of the
/// watch's state so the callback can reach them while that state is in use.
#[cfg(any(windows, test))]
struct MissedFocus<W> {
    cell: Arc<FocusCell>,
    /// The window of the latest missed change, when the system said where it happened.
    latest: Option<W>,
    /// A message to answer the missed changes is already queued.
    queued: bool,
}

#[cfg(any(windows, test))]
impl<W> MissedFocus<W> {
    fn new(cell: Arc<FocusCell>) -> Self {
        Self {
            cell,
            latest: None,
            queued: false,
        }
    }

    /// Records a focus change, in `window` when known, that cannot be answered yet. The focus is marked pending at
    /// once, so the answer in progress is refused rather than recorded for a control the focus has left. Returns
    /// whether a message must be queued to answer it: only for the first change missed since the last answer, so a
    /// burst of changes costs one more read, not one each.
    fn record(&mut self, window: Option<W>) -> bool {
        self.cell.invalidate();
        self.latest = window;
        !std::mem::replace(&mut self.queued, true)
    }

    /// The window of the latest missed change, taken to answer it; the next missed change queues a message again.
    fn take(&mut self) -> Option<W> {
        self.queued = false;
        self.latest.take()
    }
}

#[cfg(windows)]
pub use self::watch::{FocusWatch, StartFocusWatchError, StopFocusWatchError};

#[cfg(windows)]
mod watch {
    use std::cell::RefCell;
    use std::fmt;
    use std::fs::{File, OpenOptions};
    use std::io::{self, Write as _};
    use std::marker::PhantomData;
    use std::path::Path;
    use std::sync::{Arc, mpsc};

    use platform_clock::Clock;
    use platform_task::{TaskError, ThreadHandle};
    use windows::Win32::Foundation::{CloseHandle, E_FAIL, E_INVALIDARG, HANDLE, HWND, LPARAM, WPARAM};
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    };
    use windows::Win32::System::Threading::{
        GetCurrentThreadId, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::System::Variant::VT_BOOL;
    use windows::Win32::UI::Accessibility::{
        AutomationElementMode_None, CUIAutomation8, HWINEVENTHOOK, IUIAutomation2, IUIAutomationCacheRequest,
        IUIAutomationElement, SetWinEventHook, UIA_ClassNamePropertyId, UIA_ControlTypePropertyId,
        UIA_IsEnabledPropertyId, UIA_IsPasswordPropertyId, UIA_IsTextEditPatternAvailablePropertyId,
        UIA_NativeWindowHandlePropertyId, UIA_PROPERTY_ID, UIA_ValueIsReadOnlyPropertyId, UnhookWinEvent,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, EVENT_OBJECT_FOCUS, GA_ROOT, GUITHREADINFO, GWL_STYLE, GetAncestor, GetForegroundWindow,
        GetGUIThreadInfo, GetMessageW, GetWindowLongPtrW, GetWindowThreadProcessId, MSG, PM_NOREMOVE, PeekMessageW,
        PostThreadMessageW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP, WM_QUIT,
    };
    use windows::core::PWSTR;

    use super::{FocusChange, MissedFocus, app_name, app_window, log_line, next_change, request_recheck};
    use crate::focus_rules::style_says_password;
    use crate::{ElementFlags, ElementProps, FocusCell, FocusClass, classify};

    /// Why the focus watch could not start.
    #[derive(Debug, thiserror::Error)]
    pub enum StartFocusWatchError {
        #[error("could not start the focus watch thread")]
        Spawn(#[source] io::Error),
        #[error("could not initialise COM on the focus watch thread")]
        Com(#[source] windows::core::Error),
        #[error("could not create the UI Automation client")]
        Automation(#[source] windows::core::Error),
        #[error("could not listen for focus changes")]
        Listen(#[source] windows::core::Error),
        /// The thread ended without reporting, which only a panic does; the source is the panic when there was one.
        #[error("the focus watch thread ended before it was ready")]
        Ended(#[source] Option<TaskError>),
    }

    /// Why the focus watch did not stop cleanly.
    #[derive(Debug, thiserror::Error)]
    pub enum StopFocusWatchError {
        #[error("could not ask the focus watch thread to stop")]
        Signal(#[source] windows::core::Error),
        #[error("the focus watch thread failed")]
        Thread(#[source] TaskError),
    }

    /// Classifies the control with the keyboard focus each time it moves, on a thread of its own, and records the
    /// class in the [`FocusCell`] the keyboard hook reads. A UI Automation error or timeout gives
    /// [`FocusClass::Unknown`], and the watch carries on. Dropping the watch stops it, as [`FocusWatch::stop`] does,
    /// and only logs a failure.
    #[derive(Debug)]
    #[must_use = "dropping the watch stops it"]
    pub struct FocusWatch {
        cell: Arc<FocusCell>,
        /// The watch thread, until it is stopped.
        running: Option<WatchThread>,
    }

    #[derive(Debug)]
    struct WatchThread {
        id: u32,
        handle: ThreadHandle,
    }

    impl FocusWatch {
        /// Starts watching and returns once the watch listens for focus changes, or with the reason it cannot.
        /// `on_change` runs on the watch thread after each classification that is still current. With a `log` path,
        /// one line per classification is appended to that file, timed by `clock`; the log never stops the watch, so a
        /// file that cannot be opened or written is only reported through tracing.
        pub fn start(
            cell: Arc<FocusCell>,
            on_change: impl Fn(FocusChange) + Send + 'static,
            log: Option<&Path>,
            clock: impl Clock + 'static,
        ) -> Result<Self, StartFocusWatchError> {
            let setup = Setup {
                cell: Arc::clone(&cell),
                on_change: Box::new(on_change),
                log: log.and_then(|path| FocusLog::open(path, Box::new(clock))),
            };
            let (ready, report) = mpsc::channel();
            let handle = platform_task::spawn_thread(THREAD_NAME, move || run(setup, &ready))
                .map_err(StartFocusWatchError::Spawn)?;
            match report.recv() {
                Ok(Ok(id)) => Ok(Self {
                    cell,
                    running: Some(WatchThread { id, handle }),
                }),
                Ok(Err(error)) => {
                    reap(handle);
                    Err(error)
                }
                Err(mpsc::RecvError) => Err(StartFocusWatchError::Ended(handle.join().err())),
            }
        }

        /// Marks the focus pending at once, on the caller's thread, and asks the watch thread to classify it again.
        /// It never waits, so the keyboard hook calls it after a click or a key that may move the focus: until the
        /// answer lands, the hook sees pending and lets keys through.
        pub fn recheck(&self) {
            let posted = request_recheck(&self.cell, |generation| {
                self.running
                    .as_ref()
                    .map_or(Ok(()), |thread| thread.post_recheck(generation))
            });
            if let Err(error) = posted {
                tracing::warn!(error = ?error, "could not ask the focus watch to classify the focus again");
            }
        }

        /// Stops listening, releases UI Automation, and waits for the watch thread to end.
        pub fn stop(mut self) -> Result<(), StopFocusWatchError> {
            self.running.take().map_or(Ok(()), WatchThread::stop)
        }
    }

    impl Drop for FocusWatch {
        fn drop(&mut self) {
            if let Some(thread) = self.running.take()
                && let Err(error) = thread.stop()
            {
                tracing::warn!(error = ?error, "the focus watch did not stop cleanly");
            }
        }
    }

    impl WatchThread {
        fn post_recheck(&self, generation: u64) -> windows::core::Result<()> {
            // Never fails on a 64-bit target; elsewhere a generation too large for the parameter is refused, not cut.
            let word = usize::try_from(generation)
                .map_err(|error| windows::core::Error::new(E_INVALIDARG, error.to_string()))?;
            post(self.id, RECHECK_MESSAGE, WPARAM(word))
        }

        fn stop(self) -> Result<(), StopFocusWatchError> {
            post(self.id, WM_QUIT, WPARAM(0)).map_err(StopFocusWatchError::Signal)?;
            self.handle.join().map_err(StopFocusWatchError::Thread)
        }
    }

    /// Waits for a watch thread that reported a failure and is returning.
    fn reap(handle: ThreadHandle) {
        if let Err(error) = handle.join() {
            tracing::warn!(error = ?error, "the focus watch thread failed after reporting");
        }
    }

    const THREAD_NAME: &str = "focus-watch";

    /// The thread message that asks for a recheck, with the generation its answer must carry in `wParam`.
    const RECHECK_MESSAGE: u32 = WM_APP;
    /// The thread message that answers the focus changes missed while another was being classified; [`MISSED`] holds
    /// the latest one's window.
    const MISSED_FOCUS_MESSAGE: u32 = WM_APP + 1;

    /// How long UI Automation may take to reach an app, and then to get its answer, before the read fails and the
    /// control is Unknown. Both keep a hung app from holding up the classifications after it.
    const CONNECTION_TIMEOUT_MS: u32 = 200;
    const TRANSACTION_TIMEOUT_MS: u32 = 500;

    /// Everything [`read_props`] reads, fetched in the one cross-process call that finds the focused control.
    const CACHED_PROPERTIES: [UIA_PROPERTY_ID; 7] = [
        UIA_ControlTypePropertyId,
        UIA_IsPasswordPropertyId,
        UIA_IsEnabledPropertyId,
        UIA_IsTextEditPatternAvailablePropertyId,
        UIA_ValueIsReadOnlyPropertyId,
        UIA_ClassNamePropertyId,
        UIA_NativeWindowHandlePropertyId,
    ];

    /// Room for an executable's path, in UTF-16 units; an app with a longer one goes unnamed.
    const IMAGE_PATH_CAPACITY: usize = 1024;

    /// What `start` hands the watch thread.
    struct Setup {
        cell: Arc<FocusCell>,
        on_change: Box<dyn Fn(FocusChange) + Send>,
        log: Option<FocusLog>,
    }

    /// The focus log: one line per classification, for working out how an app describes its controls.
    struct FocusLog {
        file: File,
        clock: Box<dyn Clock>,
    }

    impl FocusLog {
        fn open(path: &Path, clock: Box<dyn Clock>) -> Option<Self> {
            match OpenOptions::new().create(true).append(true).open(path) {
                Ok(file) => Some(Self { file, clock }),
                Err(error) => {
                    tracing::warn!(
                        error = ?error,
                        path = %path.display(),
                        "could not open the focus log; the watch runs without it"
                    );
                    None
                }
            }
        }

        fn write<E: fmt::Display + ?Sized>(
            &mut self,
            app: &str,
            reading: Result<&ElementProps, &E>,
            class: FocusClass,
        ) {
            let line = log_line(self.clock.now(), app, reading, class);
            if let Err(error) = self.file.write_all(line.as_bytes()) {
                tracing::warn!(error = ?error, "could not write the focus log");
            }
        }
    }

    /// The watch thread's state.
    struct Watcher {
        cell: Arc<FocusCell>,
        on_change: Box<dyn Fn(FocusChange) + Send>,
        automation: Automation,
        log: Option<FocusLog>,
        /// The app of the last change handed on, which the next one is compared with.
        last_app: Option<String>,
    }

    impl Watcher {
        /// A new generation, and its answer, for a focus change in `window` when the system said where.
        fn answer_afresh(&mut self, window: Option<HWND>) {
            let generation = self.cell.invalidate();
            self.answer(generation, window);
        }

        /// Classifies the focused control as the answer for `generation`, records and logs it, and hands the change
        /// on while it is still current. `event_window` is the window a focus change happened in; a recheck has none.
        fn answer(&mut self, generation: u64, event_window: Option<HWND>) {
            let app = app_of(event_window);
            let reading = self.automation.read_focused(app.clone());
            let class = reading.as_ref().map_or(FocusClass::Unknown, classify);
            let current = self.cell.set(class, generation);
            if let Some(log) = &mut self.log {
                log.write(&app, reading.as_ref(), class);
            }
            // A newer invalidation is waiting for its own answer; handing this one on would report a control the
            // focus has already left.
            if current {
                let change = next_change(self.last_app.as_deref(), app, class);
                self.last_app = Some(change.app.clone());
                (self.on_change)(change);
            }
        }
    }

    thread_local! {
        /// The watch thread's state. The system calls the `WinEvent` callback as a plain function, so this is where it
        /// finds it.
        static WATCHER: RefCell<Option<Watcher>> = const { RefCell::new(None) };

        /// The focus changes missed while [`WATCHER`] is busy, which the callback can still reach then.
        static MISSED: RefCell<Option<MissedFocus<HWND>>> = const { RefCell::new(None) };
    }

    /// The watch thread: sets up, reports, and runs the message loop that delivers focus changes and rechecks until
    /// `stop`.
    fn run(setup: Setup, ready: &mpsc::Sender<Result<u32, StartFocusWatchError>>) {
        match Running::begin(setup) {
            Ok(running) => {
                // The thread has its message queue, so rechecks and the quit message from `stop` can reach it. If
                // `start` is gone, nobody could stop the watch, so it comes down at once.
                if ready.send(Ok(current_thread_id())).is_ok() {
                    pump_until_quit();
                }
                drop(running);
            }
            // `start` waits on the other end for this; were it gone, there would be nobody left to tell.
            Err(error) => drop(ready.send(Err(error))),
        }
    }

    /// What the watch thread holds while it runs. The fields drop in order: the hook stops the callbacks, then the
    /// watcher's UI Automation objects go, and COM is uninitialised last, once nothing uses it.
    struct Running {
        _hook: FocusEventHook,
        _watcher: InstalledWatcher,
        _com: ComApartment,
    }

    impl Running {
        fn begin(setup: Setup) -> Result<Self, StartFocusWatchError> {
            let com = ComApartment::enter().map_err(StartFocusWatchError::Com)?;
            let automation = Automation::create().map_err(StartFocusWatchError::Automation)?;
            let watcher = InstalledWatcher::install(Watcher {
                cell: setup.cell,
                on_change: setup.on_change,
                automation,
                log: setup.log,
                last_app: None,
            });
            // Peeking can run callbacks, so the queue comes first: no focus change is handled before `start` returns.
            ensure_message_queue();
            let hook = FocusEventHook::install().map_err(StartFocusWatchError::Listen)?;
            Ok(Self {
                _hook: hook,
                _watcher: watcher,
                _com: com,
            })
        }
    }

    /// COM on this thread, in the multithreaded apartment UI Automation recommends for its clients, until the guard
    /// drops. COM is uninitialised on the thread that initialised it, so the guard cannot leave the thread.
    struct ComApartment(PhantomData<*const ()>);

    impl ComApartment {
        fn enter() -> windows::core::Result<Self> {
            // SAFETY: the reserved argument is null as required, and the guard uninitialises COM once, on this thread.
            unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok()?;
            Ok(Self(PhantomData))
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            // SAFETY: the guard exists only once `CoInitializeEx` succeeded on this thread, and it drops after every
            // COM object the thread made.
            unsafe { CoUninitialize() };
        }
    }

    /// The UI Automation client and the one cache request every read uses, both made once, so a focus change pays for
    /// neither.
    struct Automation {
        client: IUIAutomation2,
        cache: IUIAutomationCacheRequest,
    }

    impl Automation {
        fn create() -> windows::core::Result<Self> {
            // SAFETY: COM is initialised on this thread, and the class id names the system's UI Automation client.
            let client: IUIAutomation2 = unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }?;
            // SAFETY: `client` is a live UI Automation client, and the call takes a number of milliseconds.
            unsafe { client.SetConnectionTimeout(CONNECTION_TIMEOUT_MS) }?;
            // SAFETY: `client` is a live UI Automation client, and the call takes a number of milliseconds.
            unsafe { client.SetTransactionTimeout(TRANSACTION_TIMEOUT_MS) }?;
            let cache = cache_request(&client)?;
            Ok(Self { client, cache })
        }

        /// What the control with the keyboard focus says about itself, in one cross-process call.
        fn read_focused(&self, process: String) -> windows::core::Result<ElementProps> {
            // SAFETY: the client and the request are live objects of this thread's apartment.
            let element = unsafe { self.client.GetFocusedElementBuildCache(&self.cache) }?;
            read_props(&element, process)
        }
    }

    /// The cache request every read uses: exactly the properties [`read_props`] reads, for the focused element alone.
    fn cache_request(client: &IUIAutomation2) -> windows::core::Result<IUIAutomationCacheRequest> {
        // SAFETY: `client` is a live UI Automation client, and the call only makes an empty request.
        let cache = unsafe { client.CreateCacheRequest() }?;
        for property in CACHED_PROPERTIES {
            // SAFETY: `cache` is a live request, and each id is a property UI Automation defines.
            unsafe { cache.AddProperty(property) }?;
        }
        // The raw view caches the focused element even when it is not part of the control view.
        // SAFETY: `client` is a live UI Automation client, and the call only returns the condition every element meets.
        let every_element = unsafe { client.RawViewCondition() }?;
        // SAFETY: both objects are live, and the request keeps its own reference to the condition.
        unsafe { cache.SetTreeFilter(&every_element) }?;
        // Only cached values are read, so the element needs no live link back to its app.
        // SAFETY: `cache` is a live request, and the mode is one UI Automation defines.
        unsafe { cache.SetAutomationElementMode(AutomationElementMode_None) }?;
        Ok(cache)
    }

    /// The properties [`classify`] judges a control by, from the element's cache.
    fn read_props(element: &IUIAutomationElement, process: String) -> windows::core::Result<ElementProps> {
        // SAFETY: `element` is a live element, and its control type is in its cache.
        let control_type = unsafe { element.CachedControlType() }?.0;
        // SAFETY: `element` is a live element, and `IsPassword` is in its cache.
        let password = unsafe { element.CachedIsPassword() }?.as_bool();
        // SAFETY: `element` is a live element, and `IsEnabled` is in its cache.
        let enabled = unsafe { element.CachedIsEnabled() }?.as_bool();
        // SAFETY: `element` is a live element, and its class name is in its cache.
        let class_name = unsafe { element.CachedClassName() }?.to_string();
        // SAFETY: `element` is a live element, and its window handle is in its cache.
        let window = unsafe { element.CachedNativeWindowHandle() }?;
        let text_edit = cached_flag(element, UIA_IsTextEditPatternAvailablePropertyId)?.unwrap_or(false);
        Ok(ElementProps {
            control_type,
            flags: ElementFlags {
                password,
                enabled,
                text_edit,
            },
            value_read_only: cached_flag(element, UIA_ValueIsReadOnlyPropertyId)?,
            class_name,
            process,
            es_password: !window.is_invalid() && style_says_password(control_type, window_style(window)),
        })
    }

    /// A yes-or-no property from the element's cache, or `None` when the control does not report it, as a control
    /// without the Value pattern does not report whether its value is read-only.
    fn cached_flag(element: &IUIAutomationElement, property: UIA_PROPERTY_ID) -> windows::core::Result<Option<bool>> {
        // SAFETY: `element` is a live element, and the property is in its cache. Skipping default values makes a
        // property the control does not report come back as not supported rather than as false.
        let value = unsafe { element.GetCachedPropertyValueEx(property, true) }?;
        if value.vt() == VT_BOOL {
            bool::try_from(&value).map(Some)
        } else {
            Ok(None)
        }
    }

    /// The style bits of `window`, which may belong to another app.
    fn window_style(window: HWND) -> isize {
        // SAFETY: the call only reads a window's style, and for a window that no longer exists it fails and returns
        // zero.
        unsafe { GetWindowLongPtrW(window, GWL_STYLE) }
    }

    /// The watcher and its record of missed changes, installed for this thread until the guard drops and releases them
    /// with the UI Automation objects. The slots are the thread's own, so the guard cannot leave the thread.
    struct InstalledWatcher(PhantomData<*const ()>);

    impl InstalledWatcher {
        fn install(watcher: Watcher) -> Self {
            MISSED.set(Some(MissedFocus::new(Arc::clone(&watcher.cell))));
            WATCHER.set(Some(watcher));
            Self(PhantomData)
        }
    }

    impl Drop for InstalledWatcher {
        fn drop(&mut self) {
            WATCHER.set(None);
            MISSED.set(None);
        }
    }

    /// The `WinEvent` hook that reports focus changes in every other process, removed on drop. The system calls its
    /// callback on this thread while the thread waits for messages, and it must be removed from the same thread; the
    /// raw handle keeps the guard there.
    struct FocusEventHook(HWINEVENTHOOK);

    impl FocusEventHook {
        fn install() -> windows::core::Result<Self> {
            // SAFETY: an out-of-context hook needs no module, and `focus_event` is a function of this program, so it
            // outlives the hook.
            let handle = unsafe {
                SetWinEventHook(
                    EVENT_OBJECT_FOCUS,
                    EVENT_OBJECT_FOCUS,
                    None,
                    Some(focus_event),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                )
            };
            if handle.is_invalid() {
                return Err(failure_from_thread());
            }
            Ok(Self(handle))
        }
    }

    impl Drop for FocusEventHook {
        fn drop(&mut self) {
            // SAFETY: the handle came from `SetWinEventHook` on this thread, which the guard cannot leave, and is
            // removed once, here.
            let removed = unsafe { UnhookWinEvent(self.0) };
            if !removed.as_bool() {
                tracing::warn!("could not remove the focus event hook");
            }
        }
    }

    /// The `WinEvent` callback for a focus change in another process, in `window`.
    extern "system" fn focus_event(
        _hook: HWINEVENTHOOK,
        _event: u32,
        window: HWND,
        _object: i32,
        _child: i32,
        _thread: u32,
        _time: u32,
    ) {
        let window = present(window);
        if !with_watcher(|watcher| watcher.answer_afresh(window)) {
            miss_focus_change(window);
        }
    }

    /// A focus change found the watcher busy. Recording it marks the focus pending at once; the first change missed
    /// since the last answer queues the one message that answers the latest of them.
    fn miss_focus_change(window: Option<HWND>) {
        if with_missed(|missed| missed.record(window)) != Some(true) {
            return;
        }
        if let Err(error) = post(current_thread_id(), MISSED_FOCUS_MESSAGE, WPARAM(0)) {
            // No message will answer the record, so it is cleared for the next missed change to queue one; until
            // then the focus stays pending and keys pass through.
            with_missed(MissedFocus::take);
            tracing::warn!(error = ?error, "could not queue a missed focus change for later");
        }
    }

    /// Runs `work` on the record of missed changes, or returns `None` when the thread's state is gone.
    fn with_missed<T>(work: impl FnOnce(&mut MissedFocus<HWND>) -> T) -> Option<T> {
        MISSED
            .try_with(|slot| {
                let mut missed = slot.try_borrow_mut().ok()?;
                missed.as_mut().map(work)
            })
            .ok()
            .flatten()
    }

    /// Runs `work` on the watcher and returns `true`, or returns `false` when the watcher is busy: the system called
    /// back into this thread while it was already classifying, as a message wait inside `on_change` would let it.
    fn with_watcher(work: impl FnOnce(&mut Watcher)) -> bool {
        WATCHER
            .try_with(|slot| {
                let Ok(mut watcher) = slot.try_borrow_mut() else {
                    return false;
                };
                if let Some(watcher) = watcher.as_mut() {
                    work(watcher);
                }
                true
            })
            // The thread is ending and its state is gone, so there is nothing left to classify for.
            .unwrap_or(true)
    }

    /// Waits for messages, during which the system calls the `WinEvent` callback, until the quit message from `stop`.
    fn pump_until_quit() {
        let mut message = MSG::default();
        let ended = loop {
            // SAFETY: `message` is a local the call fills; with no window filter it takes any message of this thread.
            let got = unsafe { GetMessageW(&raw mut message, None, 0, 0) };
            if got.0 <= 0 {
                break got.0;
            }
            if message.hwnd.is_invalid() {
                handle_thread_message(message.message, message.wParam);
            } else {
                // SAFETY: `message` was just filled by `GetMessageW` on this thread and goes to its window unchanged.
                unsafe { DispatchMessageW(&raw const message) };
            }
        };
        // Zero is the quit message; below zero is a failure, which a valid message and no window filter rule out.
        if ended < 0 {
            let error = windows::core::Error::from_thread();
            tracing::warn!(error = ?error, "the focus watch stopped on a message loop failure");
        }
    }

    /// Handles a message posted to the thread itself. The message loop holds no borrow of the watcher, so it is free
    /// here.
    fn handle_thread_message(message: u32, word: WPARAM) {
        match message {
            RECHECK_MESSAGE => {
                // Only a generation `recheck` posted arrives here, and it always fits.
                if let Ok(generation) = u64::try_from(word.0) {
                    with_watcher(|watcher| watcher.answer(generation, None));
                }
            }
            MISSED_FOCUS_MESSAGE => {
                let window = with_missed(MissedFocus::take).flatten();
                with_watcher(|watcher| watcher.answer_afresh(window));
            }
            _ => {}
        }
    }

    /// Makes sure this thread has a message queue, so a message posted to it is not refused.
    fn ensure_message_queue() {
        let mut message = MSG::default();
        // SAFETY: `message` is a local the call fills, and `PM_NOREMOVE` leaves any message where it is.
        let _any_waiting = unsafe { PeekMessageW(&raw mut message, None, 0, 0, PM_NOREMOVE) };
    }

    fn current_thread_id() -> u32 {
        // SAFETY: the call takes nothing and returns this thread's id.
        unsafe { GetCurrentThreadId() }
    }

    fn post(thread: u32, message: u32, word: WPARAM) -> windows::core::Result<()> {
        // SAFETY: the message carries no pointers; once the thread has ended, the call fails rather than faults.
        unsafe { PostThreadMessageW(thread, message, word, LPARAM(0)) }
    }

    /// The thread's last error, or a generic failure when the call that failed set none.
    fn failure_from_thread() -> windows::core::Error {
        let error = windows::core::Error::from_thread();
        if error.code().is_ok() {
            windows::core::Error::from(E_FAIL)
        } else {
            error
        }
    }

    /// The name of the app a classification is about, or empty when it cannot be read: the app owning the top-level
    /// window of the window [`app_window`] picks.
    fn app_of(event_window: Option<HWND>) -> String {
        app_window(event_window, focus_window_in_front, window_in_front)
            .and_then(|window| image_path_of(top_level(window)))
            .map(|path| app_name(&path))
            .unwrap_or_default()
    }

    /// `window`, unless it is null.
    fn present(window: HWND) -> Option<HWND> {
        (!window.is_invalid()).then_some(window)
    }

    fn window_in_front() -> Option<HWND> {
        // SAFETY: the call takes nothing and returns a window handle, null when no window is in front.
        present(unsafe { GetForegroundWindow() })
    }

    /// The window with the keyboard focus in the thread that owns the window in front, if it names one.
    fn focus_window_in_front() -> Option<HWND> {
        let front = window_in_front()?;
        // SAFETY: with no process id asked for, the call only returns the window's thread id, zero for a window that
        // has just closed.
        let thread = unsafe { GetWindowThreadProcessId(front, None) };
        if thread == 0 {
            return None;
        }
        let mut info = GUITHREADINFO {
            cbSize: u32::try_from(size_of::<GUITHREADINFO>()).ok()?,
            ..GUITHREADINFO::default()
        };
        // SAFETY: `info` is a local the call fills, with its size set as the call requires.
        unsafe { GetGUIThreadInfo(thread, &raw mut info) }.ok()?;
        present(info.hwndFocus)
    }

    /// The top-level window `window` belongs to, or `window` itself when it has none or no longer exists.
    fn top_level(window: HWND) -> HWND {
        // SAFETY: the call only walks a window's parents, and for a window that no longer exists it returns null.
        present(unsafe { GetAncestor(window, GA_ROOT) }).unwrap_or(window)
    }

    /// The full path of the executable of the process that owns `window`.
    fn image_path_of(window: HWND) -> Option<String> {
        let mut process_id = 0;
        // SAFETY: `process_id` is a local the call fills; for a window that has just closed it fails and returns zero.
        let thread = unsafe { GetWindowThreadProcessId(window, Some(&raw mut process_id)) };
        if thread == 0 {
            return None;
        }
        ProcessHandle::open(process_id)?.image_path()
    }

    /// A process opened only to read its executable's path, closed on drop.
    struct ProcessHandle(HANDLE);

    impl ProcessHandle {
        fn open(id: u32) -> Option<Self> {
            // SAFETY: the call takes plain values and returns a new handle, which this guard closes.
            let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, id) }.ok()?;
            Some(Self(handle))
        }

        fn image_path(&self) -> Option<String> {
            let mut buffer = [0_u16; IMAGE_PATH_CAPACITY];
            let mut length = u32::try_from(buffer.len()).ok()?;
            // SAFETY: the handle is open with the access the call needs; `buffer` holds `length` units, and the call
            // writes no more than that and sets `length` to the units it wrote.
            let read = unsafe {
                QueryFullProcessImageNameW(self.0, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &raw mut length)
            };
            read.ok()?;
            let written = buffer.get(..usize::try_from(length).ok()?)?;
            Some(String::from_utf16_lossy(written))
        }
    }

    impl Drop for ProcessHandle {
        fn drop(&mut self) {
            // SAFETY: the handle came from `OpenProcess` and is closed once, here.
            let closed = unsafe { CloseHandle(self.0) };
            if let Err(error) = closed {
                tracing::warn!(error = ?error, "could not close a process handle");
            }
        }
    }
}

#[cfg(test)]
mod tests;
