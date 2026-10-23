//! Single-HWND Declarative Surface Engine (`KoraDeclarativeSurface`)
//!
//! Renders an [`Element<M>`] tree onto a double-buffered GDI [`BackBuffer`] at 120 FPS
//! with zero child HWNDs, built-in smooth scrolling, hover tracking, slider dragging,
//! child Win32 native `EDIT` overlay synchronization, and floating select dropdowns.

pub mod events;
pub mod input_overlay;
pub mod paint;
pub mod types;

pub use types::*;

use crate::ui_framework::core::element::IntoElement;
use crate::ui_framework::core::layout_engine::compute_layout;
use crate::ui_framework::core::style::BgStyle;
use crate::ui_framework::win32::*;
use events::surface_wndproc;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

/// A reusable Single-HWND Declarative UI Surface parameterized over message type `M`.
pub struct Surface<M: Clone + Send + Sync + 'static> {
    state: Arc<Mutex<SurfaceInner<M>>>,
}

impl<M: Clone + Send + Sync + 'static> Clone for Surface<M> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

static REGISTER_SURFACE_CLASS: Once = Once::new();
const GWLP_WNDPROC: i32 = -4;

fn ensure_surface_class() {
    REGISTER_SURFACE_CLASS.call_once(|| unsafe {
        let class_w = to_wide(SURFACE_CLASS_NAME);
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfn_wnd_proc = Some(DefWindowProcW);
        wc.h_instance = GetModuleHandleW(null_mut());
        wc.h_cursor = LoadCursorW(0, IDC_ARROW);
        wc.lpsz_class_name = class_w.as_ptr();
        RegisterClassExW(&wc);
    });
}

impl<M: Clone + Send + Sync + 'static> Surface<M> {
    /// Creates a new Single-HWND Declarative Surface inside `parent`.
    pub fn create(parent: isize, x: i32, y: i32, width: i32, height: i32) -> Option<Self> {
        ensure_surface_class();
        let class_w = to_wide(SURFACE_CLASS_NAME);
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_w.as_ptr(),
                null_mut(),
                WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
                x,
                y,
                width,
                height,
                parent,
                0,
                GetModuleHandleW(null_mut()),
                null_mut(),
            )
        };
        if hwnd == 0 {
            return None;
        }

        let content_w = (width - 10).max(100);
        let state = Arc::new(Mutex::new(SurfaceInner {
            hwnd,
            height,
            content_w,
            content_h: height,
            scroll_y: 0,
            max_scroll: 0,
            is_dragging_scroll: false,
            drag_start_y: 0,
            drag_start_scroll: 0,
            hover_node_id: None,
            tracking_mouse: false,
            layout_root: None,
            open_select: None,
            active_slider: None,
            inputs: Vec::new(),
            focused_node_id: None,
            edit_bg_brush: 0,
            edit_bg_color: 0,
            on_msg: None,
            bg: BgStyle::Window,
        }));

        let raw_ptr = Arc::into_raw(state.clone());
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw_ptr as isize);
            SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                surface_wndproc::<M> as *const () as usize as isize,
            );
        }

        Some(Self { state })
    }

    #[inline]
    pub fn hwnd(&self) -> isize {
        self.state.lock().map(|s| s.hwnd).unwrap_or(0)
    }

    /// Sets the available layout content width (default is `width - 10` for scrollbars).
    #[inline]
    pub fn set_content_width(&self, w: i32) {
        if let Ok(mut guard) = self.state.lock() {
            guard.content_w = w;
        }
    }

    /// Sets the background style of the surface (`Window`, `Sidebar`, `Card`, etc.).
    #[inline]
    pub fn set_bg(&self, bg: BgStyle) {
        if let Ok(mut guard) = self.state.lock() {
            guard.bg = bg;
        }
    }

    /// Registers the unidirectional message handler invoked when user interactions emit `M`.
    pub fn set_on_msg<F>(&self, handler: F)
    where
        F: Fn(M) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.state.lock() {
            guard.on_msg = Some(Arc::new(handler));
        }
    }

    /// Mounts and lays out a new [`Element<M>`] tree, clamping scroll bounds and scheduling a 120 FPS repaint.
    pub fn render(&self, root: impl IntoElement<M>) {
        let root = root.into_element();
        let hwnd = {
            let mut guard = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let hdc = unsafe { GetDC(guard.hwnd) };
            let layout_tree = compute_layout(hdc, root, 0, 0, guard.content_w);
            if hdc != 0 {
                unsafe {
                    ReleaseDC(guard.hwnd, hdc);
                }
            }
            let content_bottom = layout_tree.rect.bottom;
            let total_h = if content_bottom > guard.height {
                content_bottom + 8
            } else {
                guard.height
            };
            guard.content_h = total_h;
            guard.max_scroll = total_h.saturating_sub(guard.height);
            guard.scroll_y = guard.scroll_y.clamp(0, guard.max_scroll);
            guard.layout_root = Some(layout_tree);
            guard.sync_inputs();
            guard.hwnd
        };
        if hwnd != 0 {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    }

    pub fn scroll_to_top(&self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.scroll_y = 0;
            guard.open_select = None;
            guard.update_input_scroll();
            if guard.hwnd != 0 {
                unsafe {
                    InvalidateRect(guard.hwnd, null_mut(), 0);
                }
            }
        }
    }

    pub fn set_visible(&self, visible: bool) {
        if let Ok(mut guard) = self.state.lock() {
            if !visible {
                guard.open_select = None;
            }
            if guard.hwnd != 0 {
                unsafe {
                    if visible {
                        SetWindowPos(
                            guard.hwnd,
                            HWND_TOP,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                        );
                        guard.update_input_scroll();
                        InvalidateRect(guard.hwnd, null_mut(), 0);
                    } else {
                        ShowWindow(guard.hwnd, SW_HIDE);
                    }
                }
            }
        }
    }

    pub fn invalidate(&self) {
        if let Ok(guard) = self.state.lock()
            && guard.hwnd != 0
        {
            unsafe {
                InvalidateRect(guard.hwnd, null_mut(), 0);
            }
        }
    }
}

