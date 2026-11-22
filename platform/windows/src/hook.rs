//! The low-level keyboard and mouse hook: every real key and click, seen before the app in front sees it.

use crate::SendOp;

/// One real key event, as the hook saw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawKey {
    /// The virtual-key code.
    pub vk: u16,
    /// The hardware scan code.
    pub scan: u16,
    /// Pressed, or repeating while held, rather than released.
    pub down: bool,
    /// On the extended part of the keyboard: the arrows, the navigation block, right Ctrl and Alt.
    pub extended: bool,
    pub mods: RawMods,
}

/// The modifiers held and Caps Lock at one key, before that key itself takes effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawMods {
    pub shift: bool,
    /// Ctrl, Alt or Win held: the key is a shortcut, never text.
    pub ctrl_alt_win: bool,
    pub caps_lock: bool,
}

/// What the hook hands the handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    Key(RawKey),
    /// A mouse button went down, which usually moves the focus.
    MouseDown,
}

/// What the handler answers for one event. The default lets the event through and sends nothing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HookReply {
    /// The app in front must not see the event.
    pub swallow: bool,
    /// What to type, in order, before the next real key is handled.
    pub ops: Vec<SendOp>,
    /// The app in front is a Microsoft Store app and the support setting is on, so each backspace is also posted as a
    /// character.
    pub store_app: bool,
}

/// `LLKHF_EXTENDED`: the key is on the extended part of the keyboard.
#[cfg(any(windows, test))]
const EXTENDED_FLAG: u32 = 0x01;
/// `LLKHF_UP`: the key is being released.
#[cfg(any(windows, test))]
const RELEASED_FLAG: u32 = 0x80;

/// One keyboard event as the system reports it to a low-level hook (`KBDLLHOOKSTRUCT`), before it is judged.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy)]
struct HookedKey {
    vk: u32,
    scan: u32,
    flags: u32,
    extra_info: usize,
}

/// The key the handler sees for `hooked`, or `None` for one the app sent itself, which passes through untouched.
/// `read_mods` is only called for a key the handler sees.
#[cfg(any(windows, test))]
fn key_for_handler(hooked: HookedKey, read_mods: impl FnOnce() -> RawMods) -> Option<RawKey> {
    if crate::is_injected(hooked.extra_info) {
        return None;
    }
    Some(RawKey {
        vk: u16::try_from(hooked.vk).ok()?,
        scan: u16::try_from(hooked.scan).ok()?,
        down: hooked.flags & RELEASED_FLAG == 0,
        extended: hooked.flags & EXTENDED_FLAG != 0,
        mods: read_mods(),
    })
}

#[cfg(windows)]
pub use self::listen::{KeyboardHook, StartHookError, StopHookError};

#[cfg(windows)]
mod listen {
    use std::cell::RefCell;
    use std::io;
    use std::sync::mpsc;

    use platform_task::{TaskError, ThreadHandle};
    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, GetKeyState, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, HC_ACTION, HHOOK, HOOKPROC, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT,
        PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL, WINDOWS_HOOK_ID,
        WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_QUIT, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
    };
    use windows::core::PCWSTR;

    use super::{HookEvent, HookReply, HookedKey, RawMods, key_for_handler};
    use crate::is_injected;
    use crate::send::send;

    /// Why the hook could not start.
    #[derive(Debug, thiserror::Error)]
    pub enum StartHookError {
        #[error("could not start the keyboard hook thread")]
        Spawn(#[source] io::Error),
        #[error("could not install the low-level {hook} hook")]
        Install {
            hook: &'static str,
            #[source]
            source: windows::core::Error,
        },
        /// The thread ended without reporting, which only a panic does; the source is the panic when there was one.
        #[error("the keyboard hook thread ended before it was ready")]
        Ended(#[source] Option<TaskError>),
    }

    /// Why the hook did not stop cleanly.
    #[derive(Debug, thiserror::Error)]
    pub enum StopHookError {
        #[error("could not ask the keyboard hook thread to stop")]
        Signal(#[source] windows::core::Error),
        #[error("the keyboard hook thread failed")]
        Thread(#[source] TaskError),
    }

    /// The low-level keyboard and mouse hooks, on a thread of their own that runs their message loop. Every real key
    /// and every mouse button going down reaches the handler there; input the app sent itself never does. Dropping the
    /// hook removes it, as [`KeyboardHook::stop`] does, and only logs a failure.
    #[derive(Debug)]
    #[must_use = "dropping the hook removes it"]
    pub struct KeyboardHook {
        /// The hook thread, until it is stopped.
        running: Option<HookThread>,
    }

    #[derive(Debug)]
    struct HookThread {
        id: u32,
        handle: ThreadHandle,
    }

    impl KeyboardHook {
        /// Installs both hooks and returns once they are in, or with the reason they are not. The handler runs on the
        /// hook thread and must answer quickly: Windows skips a hook that keeps a key waiting too long. A panic in it
        /// ends the process, since it cannot unwind through the system's call.
        pub fn start(handler: impl FnMut(HookEvent) -> HookReply + Send + 'static) -> Result<Self, StartHookError> {
            let (ready, report) = mpsc::channel();
            let handle = platform_task::spawn_thread(THREAD_NAME, move || run(Box::new(handler), &ready))
                .map_err(StartHookError::Spawn)?;
            match report.recv() {
                Ok(Ok(id)) => Ok(Self {
                    running: Some(HookThread { id, handle }),
                }),
                Ok(Err(error)) => {
                    reap(handle);
                    Err(error)
                }
                Err(mpsc::RecvError) => Err(StartHookError::Ended(handle.join().err())),
            }
        }

        /// Removes both hooks and waits for their thread to end.
        pub fn stop(mut self) -> Result<(), StopHookError> {
            self.running.take().map_or(Ok(()), HookThread::stop)
        }
    }

    impl Drop for KeyboardHook {
        fn drop(&mut self) {
            if let Some(thread) = self.running.take()
                && let Err(error) = thread.stop()
            {
                tracing::warn!(error = ?error, "the keyboard hook did not stop cleanly");
            }
        }
    }

    impl HookThread {
        fn stop(self) -> Result<(), StopHookError> {
            // SAFETY: the message carries no pointers; the id is the hook thread's, and once that thread has ended the
            // call fails rather than faults.
            unsafe { PostThreadMessageW(self.id, WM_QUIT, WPARAM(0), LPARAM(0)) }.map_err(StopHookError::Signal)?;
            self.handle.join().map_err(StopHookError::Thread)
        }
    }

    /// Waits for a hook thread that reported a failure and is returning.
    fn reap(handle: ThreadHandle) {
        if let Err(error) = handle.join() {
            tracing::warn!(error = ?error, "the keyboard hook thread failed after reporting");
        }
    }

    const THREAD_NAME: &str = "keyboard-hook";

    type Handler = Box<dyn FnMut(HookEvent) -> HookReply>;

    thread_local! {
        /// The handler of the hooks on this thread. The system calls the hook procedures as plain functions, so this
        /// is where they find it.
        static HANDLER: RefCell<Option<Handler>> = const { RefCell::new(None) };
    }

    /// The hook thread: installs the hooks, reports, and runs the message loop that calls them until `stop`.
    fn run(handler: Handler, ready: &mpsc::Sender<Result<u32, StartHookError>>) {
        HANDLER.set(Some(handler));
        match install_hooks() {
            Ok(hooks) => {
                // SAFETY: the call takes nothing and returns this thread's id.
                let id = unsafe { GetCurrentThreadId() };
                // Installing the hooks gave this thread its message queue, so the quit message from `stop` can reach
                // it. If `start` is gone, nobody could stop the hook, so it comes down at once.
                if ready.send(Ok(id)).is_ok() {
                    pump_until_quit();
                }
                drop(hooks);
            }
            // `start` waits on the other end for this; were it gone, there would be nobody left to tell.
            Err(error) => drop(ready.send(Err(error))),
        }
        HANDLER.set(None);
    }

    fn install_hooks() -> Result<(Hook, Hook), StartHookError> {
        let keyboard =
            Hook::install(WH_KEYBOARD_LL, Some(keyboard_proc)).map_err(|source| StartHookError::Install {
                hook: "keyboard",
                source,
            })?;
        let mouse = Hook::install(WH_MOUSE_LL, Some(mouse_proc))
            .map_err(|source| StartHookError::Install { hook: "mouse", source })?;
        Ok((keyboard, mouse))
    }

    /// Waits for messages, during which the system calls the hook procedures, until the quit message from `stop`.
    fn pump_until_quit() {
        let mut message = MSG::default();
        let ended = loop {
            // SAFETY: `message` is a local the call fills; with no window filter it takes any message of this thread.
            let got = unsafe { GetMessageW(&raw mut message, None, 0, 0) };
            if got.0 <= 0 {
                break got.0;
            }
        };
        // Zero is the quit message; below zero is a failure, which a valid message and no window filter rule out.
        if ended < 0 {
            let error = windows::core::Error::from_thread();
            tracing::warn!(error = ?error, "the keyboard hook stopped on a message loop failure");
        }
    }

    /// An installed hook, removed on drop.
    struct Hook(HHOOK);

    impl Hook {
        fn install(kind: WINDOWS_HOOK_ID, procedure: HOOKPROC) -> windows::core::Result<Self> {
            // SAFETY: a null name asks for the program's own executable, which stays loaded while it runs.
            let module = unsafe { GetModuleHandleW(PCWSTR::null()) }?;
            // SAFETY: `procedure` is a function of that executable, so it outlives the hook, and a low-level hook is
            // called on this thread, which runs a message loop until the hook is removed.
            let hook = unsafe { SetWindowsHookExW(kind, procedure, Some(HINSTANCE(module.0)), 0) }?;
            Ok(Self(hook))
        }
    }

    impl Drop for Hook {
        fn drop(&mut self) {
            // SAFETY: the handle came from `SetWindowsHookExW` and is removed once, here.
            let removed = unsafe { UnhookWindowsHookEx(self.0) };
            if let Err(error) = removed {
                tracing::warn!(error = ?error, "could not remove a low-level hook");
            }
        }
    }

    /// A hook procedure's answer that keeps the event from the app in front.
    const SWALLOW: LRESULT = LRESULT(1);

    /// The `WH_KEYBOARD_LL` hook procedure.
    ///
    /// # Safety
    ///
    /// Only the system calls it, with `lparam` pointing to a `KBDLLHOOKSTRUCT` when `code` is `HC_ACTION`.
    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if is_action(code) {
            // SAFETY: for HC_ACTION the system passes a pointer to the event, valid until this procedure returns.
            let event = unsafe { (lparam.0 as *const KBDLLHOOKSTRUCT).read() };
            let hooked = HookedKey {
                vk: event.vkCode,
                scan: event.scanCode,
                flags: event.flags.0,
                extra_info: event.dwExtraInfo,
            };
            if key_for_handler(hooked, read_mods).is_some_and(|key| handle(HookEvent::Key(key))) {
                return SWALLOW;
            }
        }
        // SAFETY: the arguments are the ones the system passed this procedure, handed on unchanged.
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    /// The `WH_MOUSE_LL` hook procedure.
    ///
    /// # Safety
    ///
    /// Only the system calls it, with `lparam` pointing to a `MSLLHOOKSTRUCT` when `code` is `HC_ACTION`.
    unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if is_action(code) && is_button_down(wparam) {
            // SAFETY: for HC_ACTION the system passes a pointer to the event, valid until this procedure returns.
            let event = unsafe { (lparam.0 as *const MSLLHOOKSTRUCT).read() };
            if !is_injected(event.dwExtraInfo) && handle(HookEvent::MouseDown) {
                return SWALLOW;
            }
        }
        // SAFETY: the arguments are the ones the system passed this procedure, handed on unchanged.
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    /// Whether a hook procedure is called with an event to act on; otherwise it only passes the call on.
    fn is_action(code: i32) -> bool {
        u32::try_from(code) == Ok(HC_ACTION)
    }

    /// Whether a low-level mouse message is a button going down; moves and wheel turns are left alone.
    fn is_button_down(message: WPARAM) -> bool {
        u32::try_from(message.0).is_ok_and(|message| {
            matches!(
                message,
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
            )
        })
    }

    /// Runs the handler on `event`, types its replacement, and says whether the event is swallowed. The replacement
    /// goes out before this returns, so it lands before the next real key, and after the handler is released, since
    /// sending can call the hook again. When the replacement failed before any key went out, the event passes
    /// through, so the user sees the plain key rather than losing it.
    fn handle(event: HookEvent) -> bool {
        let Some(reply) = dispatch(event) else {
            return false;
        };
        match send(&reply.ops, reply.store_app) {
            Ok(()) => reply.swallow,
            Err(error) => {
                tracing::warn!(error = ?error, "could not type the replacement");
                reply.swallow && !error.sent_nothing()
            }
        }
    }

    /// The handler's reply to `event`, or `None` when there is no handler or it is already running because the hook
    /// was called again while it worked; the event then passes through.
    fn dispatch(event: HookEvent) -> Option<HookReply> {
        HANDLER
            .try_with(|slot| {
                let mut handler = slot.try_borrow_mut().ok()?;
                handler.as_mut().map(|handler| handler(event))
            })
            .ok()
            .flatten()
    }

    /// The modifiers as the system holds them now. They are read at each key rather than tracked, so a key-up the
    /// hook never saw (the lock screen, an elevated window in front) cannot leave one stuck.
    fn read_mods() -> RawMods {
        RawMods {
            shift: held(VK_SHIFT),
            ctrl_alt_win: [VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN].into_iter().any(held),
            caps_lock: caps_lock_on(),
        }
    }

    /// Whether `key` is physically down.
    fn held(key: VIRTUAL_KEY) -> bool {
        // SAFETY: the call takes a key code and only reads the system's key state.
        let state = unsafe { GetAsyncKeyState(i32::from(key.0)) };
        // The high bit, which makes the value negative, says the key is down.
        state < 0
    }

    /// Whether Caps Lock is toggled on. The toggle is not part of the physical state `GetAsyncKeyState` reads, so it
    /// comes from this thread's key state, which Windows keeps in step with the system's for a thread that owns no
    /// window and takes no keyboard input; the value is as it stood before the key being handled.
    fn caps_lock_on() -> bool {
        // SAFETY: the call takes a key code and only reads this thread's key state.
        let state = unsafe { GetKeyState(i32::from(VK_CAPITAL.0)) };
        state & 1 != 0
    }
}

#[cfg(test)]
mod tests;
