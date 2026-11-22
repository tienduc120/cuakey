//! The notification-area icon and the menu that opens from it.

/// An icon resource of the running executable, by its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayIcon(pub u16);

/// What the user did with the icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// The icon was double-clicked, or chosen with Enter or Space while it had the keyboard focus. A single click does
    /// nothing.
    Activate,
    /// The user asked for the icon's menu, with a right click or from the keyboard.
    MenuRequested,
}

/// One entry of the icon's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    /// An entry the user can choose; `Tray::show_menu` returns its `id`, which must be from 1 to `i32::MAX`, or the
    /// menu is refused. `checked: None` draws no check mark, as `Some(false)` does.
    Command {
        id: u32,
        label: String,
        checked: Option<bool>,
        enabled: bool,
    },
    Submenu {
        label: String,
        items: Vec<Self>,
    },
    Separator,
}

/// `WM_LBUTTONDBLCLK`: the icon was double-clicked.
#[cfg(any(windows, test))]
const WM_LBUTTONDBLCLK: u32 = 0x0203;
/// `NIN_KEYSELECT`: the icon was chosen with Enter or Space while it had the keyboard focus.
#[cfg(any(windows, test))]
const NIN_KEYSELECT: u32 = 0x0401;
/// `WM_CONTEXTMENU`: the user asked for the icon's menu.
#[cfg(any(windows, test))]
const WM_CONTEXTMENU: u32 = 0x007b;

/// `MF_GRAYED`: the entry is drawn dimmed and cannot be chosen.
#[cfg(any(windows, test))]
const MF_GRAYED: u32 = 0x0001;
/// `MF_CHECKED`: the entry has a check mark.
#[cfg(any(windows, test))]
const MF_CHECKED: u32 = 0x0008;
/// `MF_POPUP`: the entry opens a submenu.
#[cfg(any(windows, test))]
const MF_POPUP: u32 = 0x0010;
/// `MF_SEPARATOR`: the entry is a dividing line.
#[cfg(any(windows, test))]
const MF_SEPARATOR: u32 = 0x0800;

/// Room for the tooltip in `NOTIFYICONDATAW::szTip`, in UTF-16 units, its terminating NUL included.
#[cfg(any(windows, test))]
const TIP_UNITS: usize = 128;

/// Whether the notification area shows the icon. It does not when the app starts before Explorer has made the
/// taskbar, as it can at sign-in. Until it does, a new icon or tooltip is kept, and the icon is added with them once
/// Explorer says it has made the taskbar.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Presence {
    Shown,
    Pending,
}

/// How a new icon or tooltip reaches the notification area.
#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeCall {
    /// The shown icon is updated, and a failure is an error.
    Update,
    /// The icon is offered again, with the change, and still waiting is not an error. An add that timed out while
    /// Explorer was busy can have taken effect without a word, and no taskbar announcement may follow, so this is
    /// how such an icon is found shown.
    Offer,
}

#[cfg(any(windows, test))]
impl Presence {
    /// The presence after the icon is offered to the notification area, which the tray does when it starts, each time
    /// Explorer says it has made the taskbar, shown or not, and on a change while pending: `add` adds the icon, and
    /// only when that fails, `update` updates it in case the notification area has it after all, so a shown icon is
    /// never taken for a pending one. Neither working means there is no notification area yet. An icon found shown
    /// either way is asked for version-4 callbacks with `ask_version`.
    fn after_offer(add: impl FnOnce() -> bool, update: impl FnOnce() -> bool, ask_version: impl FnOnce()) -> Self {
        if add() || update() {
            ask_version();
            Self::Shown
        } else {
            Self::Pending
        }
    }

    /// How a new icon or tooltip reaches the notification area.
    const fn change_call(self) -> ChangeCall {
        match self {
            Self::Shown => ChangeCall::Update,
            Self::Pending => ChangeCall::Offer,
        }
    }
}

/// The event a version-4 notification-area callback reports in the low word of its `lParam`, or `None` for one the
/// icon does not act on. The high word is the icon's id. A single click (`NIN_SELECT`) is not one: it also comes
/// around every double click.
#[cfg(any(windows, test))]
fn callback_event(lparam: isize) -> Option<TrayEvent> {
    match u32::try_from(lparam & 0xffff).ok()? {
        WM_LBUTTONDBLCLK | NIN_KEYSELECT => Some(TrayEvent::Activate),
        WM_CONTEXTMENU => Some(TrayEvent::MenuRequested),
        _ => None,
    }
}

/// The `MF_*` flags that draw `item`. `MF_STRING`, `MF_UNCHECKED` and `MF_ENABLED` are zero, so a plain entry needs
/// none.
#[cfg(any(windows, test))]
fn menu_flags(item: &MenuItem) -> u32 {
    match item {
        MenuItem::Command { checked, enabled, .. } => {
            let mark = if *checked == Some(true) { MF_CHECKED } else { 0 };
            let dimmed = if *enabled { 0 } else { MF_GRAYED };
            mark | dimmed
        }
        MenuItem::Submenu { .. } => MF_POPUP,
        MenuItem::Separator => MF_SEPARATOR,
    }
}

/// `id` as a menu entry's identifier, or `None` for one the menu could not hand back: zero, since `TrackPopupMenu`
/// answers zero for a dismissed menu, and anything above `i32::MAX`, since it answers in an `i32`.
#[cfg(any(windows, test))]
fn command_id(id: u32) -> Option<usize> {
    let id = i32::try_from(id).ok().filter(|&id| id != 0)?;
    usize::try_from(id).ok()
}

/// `text` as `szTip` holds it: UTF-16, cut after the last whole character that leaves room for the terminating NUL,
/// so a surrogate pair is never split, and padded with NULs.
#[cfg(any(windows, test))]
fn tooltip_units(text: &str) -> [u16; TIP_UNITS] {
    let mut used = 0;
    let kept: String = text
        .chars()
        .take_while(|character| {
            used += character.len_utf16();
            used < TIP_UNITS
        })
        .collect();
    let mut units = [0; TIP_UNITS];
    for (slot, unit) in units.iter_mut().zip(kept.encode_utf16()) {
        *slot = unit;
    }
    units
}

#[cfg(windows)]
pub use self::shell::{Tray, TrayError};

#[cfg(windows)]
mod shell {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use windows::Win32::Foundation::{E_INVALIDARG, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Shell::{
        NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
        NOTIFY_ICON_MESSAGE, NOTIFYICON_VERSION_4, NOTIFYICONDATAW, NOTIFYICONDATAW_0, Shell_NotifyIconW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CS_DBLCLKS, ChangeWindowMessageFilterEx, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
        DestroyIcon, DestroyMenu, DestroyWindow, GWLP_USERDATA, GetCursorPos, GetSystemMetrics, GetWindowLongPtrW,
        HICON, HMENU, IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW, MENU_ITEM_FLAGS, MSGFLT_ALLOW, PostMessageW,
        RegisterClassExW, RegisterWindowMessageW, SM_CXSMICON, SM_CYSMICON, SetForegroundWindow, SetWindowLongPtrW,
        TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, UnregisterClassW, WM_APP, WM_CLOSE, WM_NCDESTROY,
        WM_NULL, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_POPUP,
    };
    #[cfg(test)]
    use windows::Win32::UI::WindowsAndMessaging::{CopyIcon, IDI_APPLICATION, LR_SHARED, WM_KEYDOWN};
    use windows::core::{HSTRING, PCWSTR, w};

    use super::{
        ChangeCall, MenuItem, Presence, TIP_UNITS, TrayEvent, TrayIcon, callback_event, command_id, menu_flags,
        tooltip_units,
    };

    /// The class of the tray's hidden window.
    const CLASS_NAME: PCWSTR = w!("CuaKey.Tray");
    /// What Explorer broadcasts to every top-level window once it has made the taskbar, as after it restarts.
    const TASKBAR_CREATED: PCWSTR = w!("TaskbarCreated");
    /// The message the notification area sends the window for the icon's events.
    const CALLBACK_MESSAGE: u32 = WM_APP;
    /// The icon's id among the window's icons; it has only this one.
    const ICON_ID: u32 = 1;

    /// Why the tray could not start, change its icon, or show its menu.
    #[derive(Debug, thiserror::Error)]
    pub enum TrayError {
        #[error("could not register the taskbar message")]
        RegisterMessage(#[source] windows::core::Error),
        #[error("could not register the tray window class")]
        RegisterClass(#[source] windows::core::Error),
        #[error("could not create the tray window")]
        CreateWindow(#[source] windows::core::Error),
        #[error("could not let Explorer's messages through to the tray window")]
        AllowMessage(#[source] windows::core::Error),
        #[error("could not load the icon")]
        LoadIcon(#[source] windows::core::Error),
        #[error("could not update the icon in the notification area")]
        Update(#[source] windows::core::Error),
        #[error("could not build the menu")]
        BuildMenu(#[source] windows::core::Error),
        #[error("could not read the cursor position")]
        Cursor(#[source] windows::core::Error),
    }

    type Handler = Box<dyn FnMut(TrayEvent)>;

    /// The icon in the notification area, with a hidden window of its own that receives its events. Dropping it
    /// removes the icon. Every method takes `&self`, since a call can come from a handler that runs while another
    /// holds the tray, as one dispatched by the menu's loop.
    #[must_use = "dropping the tray removes its icon"]
    pub struct Tray {
        hwnd: HWND,
        instance: HINSTANCE,
        /// What the window procedure reads. `Rc` keeps the type on the thread that owns the window.
        shared: Rc<Shared>,
        /// The icon shown, whose handle `shared` holds. It is the last field, so it is destroyed after the icon is
        /// removed.
        icon: Cell<Icon>,
    }

    impl std::fmt::Debug for Tray {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Tray")
                .field("hwnd", &self.hwnd)
                .field("shared", &self.shared)
                .finish_non_exhaustive()
        }
    }

    /// What the window procedure shares with the tray. The window holds a count of its own, from [`lend`] until
    /// `WM_NCDESTROY`. A handler can run while a tray method is part way through, so no field is borrowed across a call
    /// that may pump messages.
    struct Shared {
        /// The registered `"TaskbarCreated"` message.
        taskbar_created: u32,
        /// The icon and tooltip for the notification area, kept to add the icon with whenever Explorer makes the
        /// taskbar.
        icon: Cell<HICON>,
        tip: Cell<[u16; TIP_UNITS]>,
        presence: Cell<Presence>,
        on_event: RefCell<Handler>,
    }

    impl std::fmt::Debug for Shared {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Shared")
                .field("taskbar_created", &self.taskbar_created)
                .field("icon", &self.icon.get())
                .field("presence", &self.presence.get())
                .finish_non_exhaustive()
        }
    }

    impl Tray {
        /// Adds `icon` to the notification area with `tooltip`. When there is no notification area yet, as when the app
        /// starts before Explorer at sign-in, the tray still starts, and adds the icon once Explorer says it has made the
        /// taskbar; [`Self::is_shown`] tells which. Only a window or icon that cannot be made is an error.
        ///
        /// The tray makes a hidden window on this thread, and `on_event` runs there, from whichever loop pumps the
        /// thread's messages, such as [`crate::MessageWindow::run_loop`]; the tray has no loop of its own. An event that
        /// arrives while `on_event` is still running, as while it shows the menu, is dropped. A panic in `on_event` ends
        /// the process, since it cannot unwind through the system's call. The window class is the same for every tray,
        /// so a process has one at a time.
        pub fn add(
            icon: TrayIcon,
            tooltip: &str,
            on_event: impl FnMut(TrayEvent) + 'static,
        ) -> Result<Self, TrayError> {
            Self::create(IconSource::Executable(icon), tooltip, Box::new(on_event))
        }

        fn create(source: IconSource, tooltip: &str, on_event: Handler) -> Result<Self, TrayError> {
            // SAFETY: the name is a terminated string, and registering the same name again returns the same id.
            let taskbar_created = unsafe { RegisterWindowMessageW(TASKBAR_CREATED) };
            if taskbar_created == 0 {
                return Err(TrayError::RegisterMessage(windows::core::Error::from_thread()));
            }
            let instance = executable().map_err(TrayError::CreateWindow)?;
            let icon = Icon::load(source, instance).map_err(TrayError::LoadIcon)?;
            let hwnd = create_window(instance)?;
            let shared = Rc::new(Shared {
                taskbar_created,
                icon: Cell::new(icon.0),
                tip: Cell::new(tooltip_units(tooltip)),
                presence: Cell::new(Presence::Pending),
                on_event: RefCell::new(on_event),
            });
            lend(hwnd, &shared);
            // From here a failure drops the tray, which destroys the window.
            let tray = Self {
                hwnd,
                instance,
                shared,
                icon: Cell::new(icon),
            };
            // Explorer sends both its taskbar announcement and the icon's events.
            allow_from_explorer(hwnd, taskbar_created)?;
            allow_from_explorer(hwnd, CALLBACK_MESSAGE)?;
            offer(hwnd, &tray.shared);
            Ok(tray)
        }

        /// Whether the notification area shows the icon, rather than the tray waiting for Explorer to make the taskbar.
        pub fn is_shown(&self) -> bool {
            self.shared.presence.get() == Presence::Shown
        }

        /// Shows `icon` instead. While the icon waits for the taskbar, or when the notification area cannot take the
        /// change (which is an error), the new icon is kept, and appears when the icon is next added.
        pub fn set_icon(&self, icon: TrayIcon) -> Result<(), TrayError> {
            let loaded = Icon::load(IconSource::Executable(icon), self.instance).map_err(TrayError::LoadIcon)?;
            self.shared.icon.set(loaded.0);
            // The notification area may use the old icon until the update returns, so it is destroyed only then.
            let previous = self.icon.replace(loaded);
            let updated = self.update();
            drop(previous);
            updated
        }

        /// Shows `text` as the tooltip, cut to the 127 UTF-16 units the notification area holds, never inside a
        /// character. As with [`Self::set_icon`], a tooltip the notification area cannot take yet is kept for the next
        /// add.
        pub fn set_tooltip(&self, text: &str) -> Result<(), TrayError> {
            self.shared.tip.set(tooltip_units(text));
            self.update()
        }

        /// Shows `items` as a menu at the cursor and waits for the user. Returns the id of the entry chosen, or `None`
        /// when the menu was dismissed. Messages keep being dispatched while the menu is open.
        pub fn show_menu(&self, items: &[MenuItem]) -> Result<Option<u32>, TrayError> {
            let menu = PopupMenu::build(items).map_err(TrayError::BuildMenu)?;
            let mut cursor = POINT::default();
            // SAFETY: `cursor` is a local the call fills.
            unsafe { GetCursorPos(&raw mut cursor) }.map_err(TrayError::Cursor)?;
            // Microsoft documents this for a notification-area menu: unless the owner is in front, the menu stays open
            // after a click elsewhere.
            // SAFETY: the window is this thread's own and alive.
            if !unsafe { SetForegroundWindow(self.hwnd) }.as_bool() {
                tracing::debug!("could not bring the tray window to the front");
            }
            // SAFETY: the menu and the window are this thread's own and alive for the call, and no rectangle is given.
            let chosen = unsafe {
                TrackPopupMenu(
                    menu.handle,
                    TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
                    cursor.x,
                    cursor.y,
                    None,
                    self.hwnd,
                    None,
                )
            };
            // The documented companion: without a task switch, the next menu opens and closes at once.
            // SAFETY: the message carries no pointers, and the window is this thread's own.
            if let Err(error) = unsafe { PostMessageW(Some(self.hwnd), WM_NULL, WPARAM(0), LPARAM(0)) } {
                tracing::warn!(error = ?error, "could not post the message that follows the tray menu");
            }
            // Zero is both a dismissed menu and a failure, which `TrackPopupMenu` does not tell apart.
            Ok(u32::try_from(chosen.0).ok().filter(|&id| id != 0))
        }

        /// Hands the kept icon and tooltip to the notification area.
        fn update(&self) -> Result<(), TrayError> {
            match self.shared.presence.get().change_call() {
                ChangeCall::Update => {
                    notify(NIM_MODIFY, &icon_data(self.hwnd, &self.shared)).map_err(TrayError::Update)
                }
                ChangeCall::Offer => {
                    offer(self.hwnd, &self.shared);
                    Ok(())
                }
            }
        }
    }

    impl Drop for Tray {
        fn drop(&mut self) {
            // Removed even while pending, since an add that timed out can have taken effect; only a shown icon that
            // cannot be removed is worth a warning.
            if let Err(error) = notify(NIM_DELETE, &identity(self.hwnd))
                && self.is_shown()
            {
                tracing::warn!(error = ?error, "could not remove the notification-area icon");
            }
            // SAFETY: the window was made on this thread, which is where the type stays, and is destroyed once, here.
            if let Err(error) = unsafe { DestroyWindow(self.hwnd) } {
                tracing::warn!(error = ?error, "could not destroy the tray window");
            }
            unregister_class(self.instance);
        }
    }

    /// Where an icon is loaded from.
    #[derive(Debug, Clone, Copy)]
    enum IconSource {
        Executable(TrayIcon),
        /// The system's application icon, for a test, whose executable carries no icon.
        #[cfg(test)]
        StockApplication,
    }

    /// An icon this program loaded, or copied, and so destroys on drop.
    #[derive(Debug)]
    struct Icon(HICON);

    impl Icon {
        /// The icon at the size of a small icon, which is the size the notification area draws.
        fn load(source: IconSource, instance: HINSTANCE) -> windows::core::Result<Self> {
            // SAFETY: the call takes a metric index and reads a system setting.
            let width = unsafe { GetSystemMetrics(SM_CXSMICON) };
            // SAFETY: the call takes a metric index and reads a system setting.
            let height = unsafe { GetSystemMetrics(SM_CYSMICON) };
            match source {
                IconSource::Executable(TrayIcon(id)) => {
                    // A resource id travels in place of the name, as `MAKEINTRESOURCEW` puts it.
                    let name = PCWSTR(std::ptr::without_provenance(usize::from(id)));
                    // SAFETY: the module is this program's executable, which stays loaded while it runs, and the name
                    // is a resource id, which the call reads as a number; without `LR_SHARED` the icon is ours to
                    // destroy.
                    let handle =
                        unsafe { LoadImageW(Some(instance), name, IMAGE_ICON, width, height, LR_DEFAULTCOLOR) }?;
                    Ok(Self(HICON(handle.0)))
                }
                #[cfg(test)]
                IconSource::StockApplication => {
                    // SAFETY: with no module the name is a system icon's id, which the call loads shared, as it must.
                    let stock = unsafe { LoadImageW(None, IDI_APPLICATION, IMAGE_ICON, width, height, LR_SHARED) }?;
                    // A shared icon must never be destroyed, so the tray keeps a copy of its own.
                    // SAFETY: the shared icon stays loaded for the life of the process.
                    let copy = unsafe { CopyIcon(HICON(stock.0)) }?;
                    Ok(Self(copy))
                }
            }
        }
    }

    impl Drop for Icon {
        fn drop(&mut self) {
            // SAFETY: the icon was loaded without `LR_SHARED` or copied, so it is this value's, destroyed once, here.
            if let Err(error) = unsafe { DestroyIcon(self.0) } {
                tracing::warn!(error = ?error, "could not destroy a tray icon");
            }
        }
    }

    /// A popup menu, destroyed on drop with its submenus, unless it was attached to another menu, which then destroys
    /// it along with itself.
    #[derive(Debug)]
    struct PopupMenu {
        handle: HMENU,
        attached: bool,
    }

    impl PopupMenu {
        /// A menu of `items`, with their submenus attached.
        fn build(items: &[MenuItem]) -> windows::core::Result<Self> {
            // SAFETY: the call takes nothing; the menu it returns is destroyed by `Drop` or along with its parent.
            let handle = unsafe { CreatePopupMenu() }?;
            let menu = Self {
                handle,
                attached: false,
            };
            for item in items {
                menu.append(item)?;
            }
            Ok(menu)
        }

        fn append(&self, item: &MenuItem) -> windows::core::Result<()> {
            let flags = MENU_ITEM_FLAGS(menu_flags(item));
            match item {
                MenuItem::Command { id, label, .. } => {
                    let id = command_id(*id).ok_or_else(refused_id)?;
                    self.append_entry(flags, id, Some(&HSTRING::from(label.as_str())))
                }
                MenuItem::Submenu { label, items } => {
                    let mut submenu = Self::build(items)?;
                    let id = submenu.handle.0 as usize;
                    self.append_entry(flags, id, Some(&HSTRING::from(label.as_str())))?;
                    // From here this menu destroys the submenu along with itself.
                    submenu.attached = true;
                    Ok(())
                }
                MenuItem::Separator => self.append_entry(flags, 0, None),
            }
        }

        /// Appends an entry with `text`, or none for a separator. For a submenu, `id` is the submenu's handle.
        fn append_entry(&self, flags: MENU_ITEM_FLAGS, id: usize, text: Option<&HSTRING>) -> windows::core::Result<()> {
            let text = text.map_or(PCWSTR::null(), |text| PCWSTR(text.as_ptr()));
            // SAFETY: the menu is this value's and alive, and the text is null or a terminated string that outlives
            // the call, which copies it.
            unsafe { AppendMenuW(self.handle, flags, id, text) }
        }
    }

    /// Why an entry is refused whose id the menu could not hand back.
    fn refused_id() -> windows::core::Error {
        windows::core::Error::new(E_INVALIDARG, "a menu entry's id must be from 1 to i32::MAX")
    }

    impl Drop for PopupMenu {
        fn drop(&mut self) {
            if self.attached {
                return;
            }
            // SAFETY: the menu is this value's, not attached to another, and destroyed once, here.
            if let Err(error) = unsafe { DestroyMenu(self.handle) } {
                tracing::warn!(error = ?error, "could not destroy the tray menu");
            }
        }
    }

    fn executable() -> windows::core::Result<HINSTANCE> {
        // SAFETY: a null name asks for the program's own executable, which stays loaded while it runs.
        let module = unsafe { GetModuleHandleW(PCWSTR::null()) }?;
        Ok(HINSTANCE(module.0))
    }

    /// Registers the class and makes the hidden window: a top-level one, since a message-only window does not hear
    /// Explorer's broadcast.
    fn create_window(instance: HINSTANCE) -> Result<HWND, TrayError> {
        let class = WNDCLASSEXW {
            cbSize: u32::try_from(size_of::<WNDCLASSEXW>()).unwrap_or(0),
            // Only a class with this style receives double clicks. Whether the notification area applies that rule to
            // the double clicks it reports to the icon's window is not documented, so the class has it.
            style: CS_DBLCLKS,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };
        // SAFETY: `class` is fully set for this call; the system copies the class name, and the procedure is a
        // function of this program.
        if unsafe { RegisterClassExW(&raw const class) } == 0 {
            return Err(TrayError::RegisterClass(windows::core::Error::from_thread()));
        }
        // SAFETY: the class was registered just above; the window has no parent, menu or creation parameter, and is
        // never shown.
        let created = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                CLASS_NAME,
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance),
                None,
            )
        };
        created.map_err(|source| {
            unregister_class(instance);
            TrayError::CreateWindow(source)
        })
    }

    fn unregister_class(instance: HINSTANCE) {
        // SAFETY: the class was registered by this program and no window of it is left.
        if let Err(error) = unsafe { UnregisterClassW(CLASS_NAME, Some(instance)) } {
            tracing::warn!(error = ?error, "could not unregister the tray window class");
        }
    }

    /// Lets `message` from Explorer reach a tray that runs as administrator, which would otherwise not hear a program
    /// with fewer rights.
    fn allow_from_explorer(hwnd: HWND, message: u32) -> Result<(), TrayError> {
        // SAFETY: the window is this thread's own and alive, and no out structure is asked for.
        unsafe { ChangeWindowMessageFilterEx(hwnd, message, MSGFLT_ALLOW, None) }.map_err(TrayError::AllowMessage)
    }

    /// Gives the window a count of `shared`, which its procedure reads and `WM_NCDESTROY` gives back.
    fn lend(hwnd: HWND, shared: &Rc<Shared>) {
        let address = Rc::into_raw(Rc::clone(shared));
        // SAFETY: the window is this thread's own and alive, and the value is only an address, which the procedure
        // turns back into the count it stands for.
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, address as isize) };
    }

    /// The fields that name the icon, which is all `NIM_DELETE` needs.
    fn identity(hwnd: HWND) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: u32::try_from(size_of::<NOTIFYICONDATAW>()).unwrap_or(0),
            hWnd: hwnd,
            uID: ICON_ID,
            ..Default::default()
        }
    }

    /// The icon as the notification area shows it, and where its events go.
    fn icon_data(hwnd: HWND, shared: &Shared) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            // With version 4 the standard tooltip shows only when asked for.
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: CALLBACK_MESSAGE,
            hIcon: shared.icon.get(),
            szTip: shared.tip.get(),
            ..identity(hwnd)
        }
    }

    /// Offers the icon to the notification area, with the icon and tooltip kept, and records whether it is shown.
    fn offer(hwnd: HWND, shared: &Shared) {
        let presence = Presence::after_offer(
            || add(hwnd, shared),
            || notify(NIM_MODIFY, &icon_data(hwnd, shared)).is_ok(),
            || ask_version_4(hwnd),
        );
        if presence == Presence::Pending {
            tracing::info!("the notification area is not there yet, so the icon waits for the taskbar");
        }
        shared.presence.set(presence);
    }

    /// Adds the icon, and says whether it was added.
    fn add(hwnd: HWND, shared: &Shared) -> bool {
        let added = notify(NIM_ADD, &icon_data(hwnd, shared));
        if let Err(error) = &added {
            tracing::debug!(error = ?error, "the notification area did not take the icon");
        }
        added.is_ok()
    }

    /// Asks for version-4 callbacks, whose `lParam` [`callback_event`] reads. Without them, a right click sends no
    /// menu request, so a failure is worth a warning.
    fn ask_version_4(hwnd: HWND) {
        let version = NOTIFYICONDATAW {
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            ..identity(hwnd)
        };
        if let Err(error) = notify(NIM_SETVERSION, &version) {
            tracing::warn!(error = ?error, "the notification area did not take the version-4 callbacks the tray reads");
        }
    }

    fn notify(message: NOTIFY_ICON_MESSAGE, data: &NOTIFYICONDATAW) -> windows::core::Result<()> {
        // SAFETY: `data` is a whole structure with its size set, valid for the call, which keeps no pointer to it.
        unsafe { Shell_NotifyIconW(message, data) }.ok()
    }

    /// The window procedure: hands the icon's events to the handler, adds the icon again when Explorer restarts, and
    /// leaves everything else to the system.
    ///
    /// # Safety
    ///
    /// Only the system calls it, for the window [`create_window`] makes.
    unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if message == WM_NCDESTROY {
            give_back(hwnd);
        } else if let Some(shared) = shared_of(hwnd)
            && handle(hwnd, &shared, message, lparam)
        {
            return LRESULT(0);
        }
        // SAFETY: the arguments are the ones the system passed this procedure, handed on unchanged.
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }

    /// Acts on the tray's own messages, and says whether `message` was one.
    fn handle(hwnd: HWND, shared: &Shared, message: u32, lparam: LPARAM) -> bool {
        if message == CALLBACK_MESSAGE {
            if let Some(event) = callback_event(lparam.0) {
                deliver(shared, event);
            }
            true
        } else if message == shared.taskbar_created {
            // A new taskbar, at sign-in or after Explorer restarts, has none of the icons, so the icon is offered
            // whether or not it was shown.
            offer(hwnd, shared);
            true
        } else {
            // The system would destroy the window on a request to close, which any program can post to a top-level
            // window, and the tray would lose its icon unawares. Only dropping the tray destroys it.
            message == WM_CLOSE
        }
    }

    /// The shared state with a count held for the length of one message, so a handler that drops the tray does not
    /// free the state under the procedure; `None` before [`lend`] and after `WM_NCDESTROY`.
    fn shared_of(hwnd: HWND) -> Option<Rc<Shared>> {
        // SAFETY: the window belongs to this thread; the value is zero or the address `lend` stored.
        let address = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
        let pointer = address as *const Shared;
        if pointer.is_null() {
            return None;
        }
        // SAFETY: a non-zero value came from `Rc::into_raw` and stands for the window's count, which `WM_NCDESTROY`
        // gives back only after setting the value to zero, so the state is alive.
        unsafe { Rc::increment_strong_count(pointer) };
        // SAFETY: the count taken just above is this `Rc`'s to give back.
        Some(unsafe { Rc::from_raw(pointer) })
    }

    /// Gives back the window's count of the shared state.
    fn give_back(hwnd: HWND) {
        // SAFETY: the window is this thread's own, and after this message nothing reads the value.
        let address = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
        if address != 0 {
            // SAFETY: the value came from `Rc::into_raw` in `lend` and stands for one count, given back once, since the
            // value is now zero.
            drop(unsafe { Rc::from_raw(address as *const Shared) });
        }
    }

    fn deliver(shared: &Shared, event: TrayEvent) {
        // The handler is already running when a call it made pumps messages, as the menu does; it cannot be borrowed
        // twice, so the event is dropped.
        if let Ok(mut handler) = shared.on_event.try_borrow_mut() {
            handler(event);
        } else {
            tracing::debug!(event = ?event, "the tray handler is busy, so the event is dropped");
        }
    }

    #[cfg(test)]
    impl Tray {
        /// A tray showing the system's application icon, since a test executable carries no icon of its own.
        pub(super) fn add_stock(tooltip: &str, on_event: impl FnMut(TrayEvent) + 'static) -> Result<Self, TrayError> {
            Self::create(IconSource::StockApplication, tooltip, Box::new(on_event))
        }

        pub(super) const fn hwnd(&self) -> HWND {
            self.hwnd
        }

        /// Posts the window the callback the notification area sends for `event`.
        pub(super) fn post_callback(&self, event: u32) -> windows::core::Result<()> {
            let lparam = isize::try_from((ICON_ID << 16) | event)
                .map_err(|error| windows::core::Error::new(E_INVALIDARG, error.to_string()))?;
            // SAFETY: the message carries numbers only, and the window is this thread's own.
            unsafe { PostMessageW(Some(self.hwnd), CALLBACK_MESSAGE, WPARAM(0), LPARAM(lparam)) }
        }

        /// Queues a press of `key` for the window, which the menu's loop reads as the user's.
        pub(super) fn post_key(&self, key: u16) -> windows::core::Result<()> {
            // SAFETY: the message carries numbers only, and the window is this thread's own.
            unsafe { PostMessageW(Some(self.hwnd), WM_KEYDOWN, WPARAM(usize::from(key)), LPARAM(0)) }
        }

        /// Posts the window a request to close, as `taskkill` without `/F` sends one.
        pub(super) fn post_close(&self) -> windows::core::Result<()> {
            // SAFETY: the message carries no pointers, and the window is this thread's own.
            unsafe { PostMessageW(Some(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        }

        /// Removes the icon behind the tray's back and posts the window Explorer's broadcast, as a restart of
        /// Explorer does.
        pub(super) fn act_out_explorer_restart(&self) -> windows::core::Result<()> {
            notify(NIM_DELETE, &identity(self.hwnd))?;
            // SAFETY: the message carries no pointers, and the window is this thread's own.
            unsafe { PostMessageW(Some(self.hwnd), self.shared.taskbar_created, WPARAM(0), LPARAM(0)) }
        }
    }

    /// Whether the notification area has the icon of `hwnd`: an update that changes nothing succeeds only for an icon
    /// it has.
    #[cfg(test)]
    pub(super) fn notification_area_has(hwnd: HWND) -> bool {
        notify(NIM_MODIFY, &identity(hwnd)).is_ok()
    }
}

#[cfg(test)]
mod tests;
