//! A message-only window: the place the other `CuaKey` process posts its messages to, and where the notification-area
//! icon will send its callbacks.

use std::cell::RefCell;
use std::marker::PhantomData;

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, ChangeWindowMessageFilterEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    FindWindowExW, GWLP_USERDATA, GetMessageW, GetWindowLongPtrW, HWND_MESSAGE, MSG, MSGFLT_ALLOW, PostMessageW,
    PostQuitMessage, RegisterClassExW, RegisterWindowMessageW, SetWindowLongPtrW, TranslateMessage, UnregisterClassW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_NCCREATE, WM_NCDESTROY, WNDCLASSEXW,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::AppMessage;

/// The registered window message every [`AppMessage`] travels as, with the message in `wParam`.
const MESSAGE_NAME: PCWSTR = w!("CuaKey.AppMessage");

/// Why the window could not be made.
#[derive(Debug, thiserror::Error)]
pub enum CreateMessageWindowError {
    #[error("could not register the window message")]
    RegisterMessage(#[source] windows::core::Error),
    #[error("could not register the window class")]
    RegisterClass(#[source] windows::core::Error),
    #[error("could not create the window")]
    Create(#[source] windows::core::Error),
    #[error("could not let a program with fewer rights post to the window")]
    AllowMessage(#[source] windows::core::Error),
}

/// Why a message could not be posted to the other process.
#[derive(Debug, thiserror::Error)]
pub enum PostError {
    /// No window has that class, so `CuaKey` is not running.
    #[error("no window of that class is running")]
    NotRunning,
    #[error("could not register the window message")]
    RegisterMessage(#[source] windows::core::Error),
    #[error("could not post the message")]
    Post(#[source] windows::core::Error),
}

type Handler = Box<dyn FnMut(AppMessage)>;

/// What the window procedure needs, at an address that stays put while the window lives.
struct State {
    message_id: u32,
    on_message: RefCell<Handler>,
}

/// A message-only window of the given class. Messages posted to it by [`post_to`] reach the handler, on the thread
/// that made the window. Dropping the window destroys it and unregisters its class.
#[derive(Debug)]
pub struct MessageWindow {
    hwnd: HWND,
    class: HSTRING,
    instance: HINSTANCE,
    /// The window procedure reads this through a pointer, so it is dropped only after the window is destroyed.
    _state: Box<State>,
    /// A window belongs to its thread, so this type is neither `Send` nor `Sync`.
    thread_bound: PhantomData<*const ()>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("message_id", &self.message_id)
            .finish_non_exhaustive()
    }
}

fn app_message_id() -> Result<u32, windows::core::Error> {
    // SAFETY: the name is a terminated string, and registering the same name again returns the same id.
    let id = unsafe { RegisterWindowMessageW(MESSAGE_NAME) };
    if id == 0 {
        return Err(windows::core::Error::from_thread());
    }
    Ok(id)
}

impl MessageWindow {
    /// Makes the window of class `class` on this thread. `on_message` runs on this thread, from [`Self::run_loop`] or
    /// whichever loop pumps this thread's messages.
    pub fn new(class: &str, on_message: impl FnMut(AppMessage) + 'static) -> Result<Self, CreateMessageWindowError> {
        let message_id = app_message_id().map_err(CreateMessageWindowError::RegisterMessage)?;
        let state = Box::new(State {
            message_id,
            on_message: RefCell::new(Box::new(on_message)),
        });
        let class = HSTRING::from(class);
        // SAFETY: a null name asks for the program's own executable, which stays loaded while it runs.
        let instance = HINSTANCE(
            unsafe { GetModuleHandleW(PCWSTR::null()) }
                .map_err(CreateMessageWindowError::Create)?
                .0,
        );
        let description = WNDCLASSEXW {
            cbSize: u32::try_from(size_of::<WNDCLASSEXW>()).unwrap_or(0),
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        // SAFETY: `description` is fully set for this call; the system copies the class name, and the procedure is a
        // function of this program.
        if unsafe { RegisterClassExW(&raw const description) } == 0 {
            return Err(CreateMessageWindowError::RegisterClass(
                windows::core::Error::from_thread(),
            ));
        }
        let state_ptr: *const State = &raw const *state;
        // SAFETY: the class was registered just above; the parent is the message-only parent; the creation parameter
        // is the state's address, which the window procedure borrows only while the window lives, and `Self` keeps
        // the state alive until after the window is destroyed.
        let created = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                &class,
                PCWSTR::null(),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(instance),
                Some(state_ptr.cast()),
            )
        };
        match created {
            Ok(hwnd) => {
                // From here a failure drops the window, which destroys it.
                let window = Self {
                    hwnd,
                    class,
                    instance,
                    _state: state,
                    thread_bound: PhantomData,
                };
                allow_lower_integrity(hwnd, message_id)?;
                Ok(window)
            }
            Err(source) => {
                unregister_class(&class, instance);
                Err(CreateMessageWindowError::Create(source))
            }
        }
    }

    /// The window's handle, for a notification-area icon to send its callbacks to.
    #[must_use]
    pub const fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Handles this thread's messages until [`AppMessage::Quit`] arrives or something posts a quit message, then
    /// destroys the window.
    pub fn run_loop(self) {
        pump_messages();
        drop(self);
    }
}

/// Takes this thread's messages and dispatches them until the quit message or a failure.
fn pump_messages() {
    let mut message = MSG::default();
    loop {
        // SAFETY: `message` is a local the call fills; with no window filter it takes any message of this thread.
        let got = unsafe { GetMessageW(&raw mut message, None, 0, 0) };
        if got.0 == 0 {
            return;
        }
        if got.0 < 0 {
            tracing::warn!(error = ?windows::core::Error::from_thread(), "the message loop failed");
            return;
        }
        // SAFETY: `message` was just filled by `GetMessageW`.
        let _translated = unsafe { TranslateMessage(&raw const message) };
        // SAFETY: `message` was just filled by `GetMessageW`.
        unsafe { DispatchMessageW(&raw const message) };
    }
}

impl Drop for MessageWindow {
    fn drop(&mut self) {
        // SAFETY: the window was made on this thread, which is where the type stays, and is destroyed once, here.
        if let Err(error) = unsafe { DestroyWindow(self.hwnd) } {
            tracing::warn!(error = ?error, "could not destroy the message window");
        }
        unregister_class(&self.class, self.instance);
    }
}

/// Lets a process with fewer rights post the app message to the window. Without it, a `CuaKey` that runs as administrator
/// would ignore a second launch from the Start menu.
fn allow_lower_integrity(hwnd: HWND, message_id: u32) -> Result<(), CreateMessageWindowError> {
    // SAFETY: the window is this thread's own and alive, and no out structure is asked for.
    unsafe { ChangeWindowMessageFilterEx(hwnd, message_id, MSGFLT_ALLOW, None) }
        .map_err(CreateMessageWindowError::AllowMessage)
}

fn unregister_class(class: &HSTRING, instance: HINSTANCE) {
    // SAFETY: the class was registered by this program and no window of it is left.
    if let Err(error) = unsafe { UnregisterClassW(class, Some(instance)) } {
        tracing::warn!(error = ?error, "could not unregister the message window class");
    }
}

/// The window procedure: hands a `CuaKey` message to the handler and leaves everything else to the system.
///
/// # Safety
///
/// Only the system calls it. The creation parameter of the window it serves is a pointer to a [`State`] that outlives
/// the window.
unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: for this message `lparam` points to the creation data, valid during the call.
        let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        // SAFETY: the window is being created on this thread; the value is the state's address.
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
    } else if message == WM_NCDESTROY {
        // SAFETY: the window is on this thread; after this message nothing reads the value.
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
    } else if let Some(state) = state_of(hwnd)
        && message == state.message_id
    {
        deliver(state, wparam);
        return LRESULT(0);
    }
    // SAFETY: the arguments are the ones the system passed this procedure, handed on unchanged.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// The state of the window, or `None` before it is set up.
fn state_of<'a>(hwnd: HWND) -> Option<&'a State> {
    // SAFETY: the window belongs to this thread; the value is zero or the address `WM_NCCREATE` stored.
    let address = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    // SAFETY: a non-zero value is the state's address, which stays valid until `MessageWindow` drops, after the
    // window is destroyed, and the procedure runs only while the window exists.
    unsafe { (address as *const State).as_ref() }
}

fn deliver(state: &State, wparam: WPARAM) {
    let Some(message) = AppMessage::decode(wparam.0) else {
        // A code from another version of the program: nothing to do with it.
        return;
    };
    // A handler that is already running and posts to its own window would otherwise borrow twice.
    if let Ok(mut handler) = state.on_message.try_borrow_mut() {
        handler(message);
    }
    if message == AppMessage::Quit {
        // SAFETY: the call only posts a quit message to this thread's queue.
        unsafe { PostQuitMessage(0) };
    }
}

/// Posts `message` to the window of class `class`, in this or another process.
pub fn post_to(class: &str, message: AppMessage) -> Result<(), PostError> {
    let id = app_message_id().map_err(PostError::RegisterMessage)?;
    // SAFETY: the class is a terminated string that outlives the call; no parent window but the message-only one.
    let hwnd =
        unsafe { FindWindowExW(Some(HWND_MESSAGE), None, &HSTRING::from(class), PCWSTR::null()) }.map_err(|error| {
            tracing::debug!(error = ?error, "no window of that class was found");
            PostError::NotRunning
        })?;
    // SAFETY: the message carries a number only, no pointers, so it is sound whichever process owns the window.
    unsafe { PostMessageW(Some(hwnd), id, WPARAM(message.encode()), LPARAM(0)) }.map_err(PostError::Post)
}

#[cfg(test)]
mod tests;
