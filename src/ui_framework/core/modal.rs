//! Declarative Modal Base Window Scaffold
//!
//! Web-like Modal architecture for native desktop:
//! - [`ModalHandle<M>`]: Handle to a running modal with `.close()`, `.render(new_view)`, and `.dispatch(msg)`.
//! - [`open_modal`]: The core modal container hosting an arbitrary [`Element<M>`] virtual tree.
//! - Centered over parent (or screen), modal barrier (disables background interaction), dark/light styling.
//! - Built-in keyboard shortcuts: Escape to cancel/close.
//! - Zero domain logic or hardcoded content: an empty floating shell for UI dialogs to fill.

use super::containers::{col, row, spacer};
use super::element::{Element, IntoElement, Length};
use super::surface::Surface;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

const MODAL_CLASS_NAME: &str = "KoraDeclarativeModalClass";
const WM_NCDESTROY: u32 = 0x0082;
static REGISTER_MODAL_ONCE: Once = Once::new();

fn ensure_modal_class_registered() {
    REGISTER_MODAL_ONCE.call_once(|| unsafe {
        let class_name_w = to_wide(MODAL_CLASS_NAME);
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfn_wnd_proc = Some(modal_wnd_proc);
        wc.lpsz_class_name = class_name_w.as_ptr();
        wc.h_cursor = LoadCursorW(0, IDC_ARROW);
        RegisterClassExW(&wc);
    });
}

struct ModalInstanceData {
    parent: isize,
    raw_handler: Option<Box<dyn FnMut(isize, u32, usize, isize) -> Option<isize>>>,
}

unsafe extern "system" fn modal_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    unsafe {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ModalInstanceData;
        if !ptr.is_null() {
            let instance = &mut *ptr;
            if let Some(ref mut handler) = instance.raw_handler {
                if let Some(res) = handler(hwnd, msg, wparam, lparam) {
                    return res;
                }
            }

            match msg {
                WM_GETMINMAXINFO => {
                    let mmi = &mut *(lparam as *mut MINMAXINFO);
                    mmi.ptMinTrackSize.x = 240;
                    mmi.ptMinTrackSize.y = 80;
                    0
                }
                WM_KEYDOWN => {
                    if wparam == 0x1B /* VK_ESCAPE */ {
                        if instance.parent != 0 {
                            EnableWindow(instance.parent, 1);
                            SetForegroundWindow(instance.parent);
                        }
                        DestroyWindow(hwnd);
                        return 0;
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_CLOSE => {
                    if instance.parent != 0 {
                        EnableWindow(instance.parent, 1);
                        SetForegroundWindow(instance.parent);
                    }
                    DestroyWindow(hwnd);
                    0
                }
                WM_NCDESTROY => {
                    let parent = instance.parent;
                    if parent != 0 {
                        EnableWindow(parent, 1);
                    }
                    drop(Box::from_raw(ptr));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        } else {
            if msg == WM_GETMINMAXINFO {
                let mmi = &mut *(lparam as *mut MINMAXINFO);
                mmi.ptMinTrackSize.x = 240;
                mmi.ptMinTrackSize.y = 80;
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }
}

/// Options for configuring modal geometry and title.
#[derive(Debug, Clone)]
pub struct ModalConfig {
    pub title: String,
    pub width: i32,
    pub height: i32,
}

impl ModalConfig {
    pub fn new(title: impl Into<String>, width: i32, height: i32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }
}

/// A handle to a live Modal dialog.
#[derive(Clone)]
pub struct ModalHandle<M: Clone + Send + Sync + 'static> {
    pub hwnd: isize,
    pub parent: isize,
    pub surface: Surface<M>,
    dispatcher: Arc<dyn Fn(M) + Send + Sync>,
}

impl<M: Clone + Send + Sync + 'static> ModalHandle<M> {
    /// Re-renders the modal with a new virtual [`Element<M>`] tree.
    pub fn render(&self, mut view: Element<M>) {
        let mut cr = RECT::default();
        let client_h = unsafe {
            GetClientRect(self.hwnd, &mut cr);
            cr.bottom - cr.top
        };
        if client_h > 0 {
            view.style.height = Length::Fixed(client_h);
        }
        self.surface.render(view);
    }

    /// Dispatches a message to the modal's reactive update pipeline.
    pub fn dispatch(&self, msg: M) {
        (self.dispatcher)(msg);
    }

    /// Closes the modal, re-enables the parent window, and restores focus.
    pub fn close(&self) {
        unsafe {
            PostMessageW(self.hwnd, WM_CLOSE, 0, 0);
        }
    }
}

/// A declarative 3-section Modal Scaffold (Header, Body, Footer).
///
/// Follows modern web modal architectures (like Bootstrap/Radix/Chakra):
/// - `header`: (Optional) Header bar at top with bottom border separator.
/// - `body`: Main content area, expands to fill available space (`flex_1`).
/// - `footer`: (Optional) Action bar pinned to bottom, compact vertical padding,
///   distinct sidebar background, top border separator.
pub struct ModalDialog<M> {
    pub header: Option<Element<M>>,
    pub body: Option<Element<M>>,
    pub footer: Option<Element<M>>,
}

impl<M> Default for ModalDialog<M> {
    fn default() -> Self {
        Self {
            header: None,
            body: None,
            footer: None,
        }
    }
}

/// Creates a new 3-section modal layout builder.
pub fn modal_dialog<M: Clone + Send + Sync + 'static>() -> ModalDialog<M> {
    ModalDialog {
        header: None,
        body: None,
        footer: None,
    }
}

impl<M: Clone + Send + Sync + 'static> ModalDialog<M> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the optional modal header bar.
    pub fn header(mut self, header: impl IntoElement<M>) -> Self {
        self.header = Some(header.into_element());
        self
    }

    /// Sets the main modal body content.
    pub fn body(mut self, body: impl IntoElement<M>) -> Self {
        self.body = Some(body.into_element());
        self
    }

    /// Sets the optional modal footer bar.
    pub fn footer(mut self, footer: impl IntoElement<M>) -> Self {
        self.footer = Some(footer.into_element());
        self
    }

    /// Builds the 3-section virtual element tree.
    pub fn build(self) -> Element<M> {
        let has_header = self.header.is_some();
        let has_footer = self.footer.is_some();
        let mut sections = Vec::with_capacity(3);

        if let Some(h) = self.header {
            sections.push(
                col([h])
                    .px(4)
                    .pt(3)
                    .pb(2),
            );
        }

        if let Some(b) = self.body {
            sections.push(
                col([b])
                    .px(4)
                    .pt(if has_header { 2 } else { 3 })
                    .pb(if has_footer { 2 } else { 3 })
                    .h_flex(1),
            );
        } else {
            sections.push(col([]).h_flex(1));
        }

        if let Some(f) = self.footer {
            sections.push(
                col([f])
                    .px(4)
                    .pt(1)
                    .pb(3),
            );
        }

        col(sections).w_full().h_flex(1)
    }
}

impl<M: Clone + Send + Sync + 'static> IntoElement<M> for ModalDialog<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        self.build()
    }
}

/// Helper to construct a standard modal footer action bar with right-aligned buttons.
pub fn modal_footer<M: Clone + Send + Sync + 'static, I, E>(actions: I) -> Element<M>
where
    I: IntoIterator<Item = E>,
    E: IntoElement<M>,
{
    let mut items = vec![spacer()];
    items.extend(actions.into_iter().map(IntoElement::into_element));
    row(items).gap(2).items_center()
}

/// Spawns a modal dialog hosting a declarative [`Element<M>`] virtual tree with an optional
/// raw Win32 message interception hook (`raw_wndproc`).
pub fn open_modal_with_wndproc<M: Clone + Send + Sync + 'static>(
    parent: isize,
    config: ModalConfig,
    view: Element<M>,
    on_msg: impl Fn(ModalHandle<M>, M) + Send + Sync + 'static,
    raw_wndproc: impl Fn(&ModalHandle<M>, u32, usize, isize) -> Option<isize> + 'static,
) -> Option<ModalHandle<M>> {
    ensure_modal_class_registered();

    let style = WS_CAPTION | WS_SYSMENU | WS_VISIBLE;
    let ex_style = WS_EX_DLGMODALFRAME | WS_EX_CONTROLPARENT;

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: config.width,
        bottom: config.height,
    };
    unsafe {
        AdjustWindowRectEx(&mut rect, style, 0, ex_style);
    }
    let win_w = rect.right - rect.left;
    let win_h = rect.bottom - rect.top;

    let (x, y) = if parent != 0 {
        let mut rc = RECT::default();
        unsafe { GetWindowRect(parent, &mut rc) };
        let cx = rc.left + ((rc.right - rc.left) - win_w) / 2;
        let cy = rc.top + ((rc.bottom - rc.top) - win_h) / 2;
        (cx, cy)
    } else {
        let screen_w = unsafe { GetSystemMetrics(0) };
        let screen_h = unsafe { GetSystemMetrics(1) };
        ((screen_w - win_w) / 2, (screen_h - win_h) / 2)
    };

    let title_w = to_wide(&config.title);
    let class_w = to_wide(MODAL_CLASS_NAME);

    let hwnd = unsafe {
        if parent != 0 {
            EnableWindow(parent, 0);
        }
        CreateWindowExW(
            ex_style,
            class_w.as_ptr(),
            title_w.as_ptr(),
            style,
            x,
            y,
            win_w,
            win_h,
            parent,
            0,
            0,
            null_mut(),
        )
    };

    if hwnd == 0 {
        if parent != 0 {
            unsafe {
                EnableWindow(parent, 1);
            }
        }
        return None;
    }

    let mut cr = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut cr);
    }
    let client_w = (cr.right - cr.left).max(config.width);
    let client_h = (cr.bottom - cr.top).max(config.height);

    let surface = match Surface::create(hwnd, 0, 0, client_w, client_h) {
        Some(s) => s,
        None => {
            if parent != 0 {
                unsafe {
                    EnableWindow(parent, 1);
                }
            }
            unsafe {
                DestroyWindow(hwnd);
            }
            return None;
        }
    };
    surface.set_content_width(client_w);
    let mut view = view;
    view.style.height = Length::Fixed(client_h);
    surface.render(view);

    let on_msg = Arc::new(on_msg);
    let handle_cell = Arc::new(Mutex::new(None::<ModalHandle<M>>));
    let handle_cell_for_disp = Arc::clone(&handle_cell);
    let on_msg_for_disp = Arc::clone(&on_msg);
    let dispatcher: Arc<dyn Fn(M) + Send + Sync> = Arc::new(move |msg: M| {
        if let Some(ref h) = *handle_cell_for_disp.lock().unwrap() {
            on_msg_for_disp(h.clone(), msg);
        }
    });

    let handle = ModalHandle {
        hwnd,
        parent,
        surface: surface.clone(),
        dispatcher: Arc::clone(&dispatcher),
    };
    *handle_cell.lock().unwrap() = Some(handle.clone());

    let disp_for_surface = Arc::clone(&dispatcher);
    surface.set_on_msg(move |msg| {
        disp_for_surface(msg);
    });

    let handle_for_raw = handle.clone();
    let instance = Box::new(ModalInstanceData {
        parent,
        raw_handler: Some(Box::new(move |_hwnd, msg, wparam, lparam| {
            raw_wndproc(&handle_for_raw, msg, wparam, lparam)
        })),
    });

    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(instance) as isize);
        apply_modern_window_styling(hwnd, is_current_dark());
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
    }

    Some(handle)
}

/// Spawns a modal dialog hosting a declarative [`Element<M>`] virtual tree.
/// Whenever an interaction in the tree produces a message `M`, `on_msg` is invoked with `(modal_handle, msg)`.
pub fn open_modal<M: Clone + Send + Sync + 'static>(
    parent: isize,
    config: ModalConfig,
    view: Element<M>,
    on_msg: impl Fn(ModalHandle<M>, M) + Send + Sync + 'static,
) -> Option<ModalHandle<M>> {
    open_modal_with_wndproc(parent, config, view, on_msg, |_, _, _, _| None)
}
