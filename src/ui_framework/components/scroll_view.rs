//! Reusable Native Win32 ScrollView Component
//!
//! Two-window container architecture:
//! - Outer Viewport (`KoraScrollViewport`): acts as the hardware clipping frame (WS_CLIPCHILDREN),
//!   renders the Fluent/macOS scrollbar thumb in the right gutter, and handles mouse wheel / thumb drag.
//! - Inner Canvas (`KoraScrollContent`): hosts all child controls at permanent fixed (x, y) coordinates.
//!   Renders double-buffered background and rounded card containers without WS_CLIPCHILDREN,
//!   eliminating label ghosting and smearing.
//! - Automatic message bubbling: WM_COMMAND, WM_NOTIFY, WM_MEASUREITEM, WM_DRAWITEM, WM_CTLCOLOR*
//!   are seamlessly routed to the parent window (h_panel).

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::tokens::{borders, layout, radii};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

pub struct ScrollViewState {
    pub h_viewport: isize,
    pub h_content: isize,
    pub viewport_w: i32,
    pub viewport_h: i32,
    pub content_w: i32,
    pub content_h: i32,
    pub scroll_y: i32,
    pub max_scroll: i32,
    pub is_dragging_scroll: bool,
    pub drag_start_y: i32,
    pub drag_start_scroll: i32,
    pub cards: Vec<(i32, i32)>,
    pub on_paint_custom: Option<Arc<dyn Fn(isize, &ThemePalette) + Send + Sync>>,
}

#[derive(Clone)]
pub struct ScrollView {
    state: Arc<Mutex<ScrollViewState>>,
}

static REGISTER_SCROLL_CLASSES_ONCE: Once = Once::new();

fn ensure_classes_registered() {
    REGISTER_SCROLL_CLASSES_ONCE.call_once(|| unsafe {
        let class_vp = to_wide("KoraScrollViewport");
        let mut wc_vp: WNDCLASSEXW = std::mem::zeroed();
        wc_vp.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc_vp.lpfn_wnd_proc = Some(scroll_viewport_wndproc);
        wc_vp.lpsz_class_name = class_vp.as_ptr();
        wc_vp.h_cursor = LoadCursorW(0, IDC_ARROW);
        RegisterClassExW(&wc_vp);

        let class_cnt = to_wide("KoraScrollContent");
        let mut wc_cnt: WNDCLASSEXW = std::mem::zeroed();
        wc_cnt.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc_cnt.lpfn_wnd_proc = Some(scroll_content_wndproc);
        wc_cnt.lpsz_class_name = class_cnt.as_ptr();
        wc_cnt.h_cursor = LoadCursorW(0, IDC_ARROW);
        RegisterClassExW(&wc_cnt);
    });
}

impl ScrollView {
    /// Creates a scrollable container with an outer viewport and inner content canvas.
    pub fn create(parent: isize, x: i32, y: i32, width: i32, height: i32) -> Option<Self> {
        ensure_classes_registered();

        let class_vp = to_wide("KoraScrollViewport");
        let h_viewport = unsafe {
            CreateWindowExW(
                WS_EX_CONTROLPARENT,
                class_vp.as_ptr(),
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
        if h_viewport == 0 {
            return None;
        }

        let content_w = layout::CARD_WIDTH;
        let class_cnt = to_wide("KoraScrollContent");
        let h_content = unsafe {
            CreateWindowExW(
                WS_EX_CONTROLPARENT,
                class_cnt.as_ptr(),
                null_mut(),
                WS_CHILD | WS_VISIBLE,
                0,
                0,
                content_w,
                height,
                h_viewport,
                0,
                GetModuleHandleW(null_mut()),
                null_mut(),
            )
        };
        if h_content == 0 {
            unsafe {
                DestroyWindow(h_viewport);
            }
            return None;
        }

        let state = Arc::new(Mutex::new(ScrollViewState {
            h_viewport,
            h_content,
            viewport_w: width,
            viewport_h: height,
            content_w,
            content_h: height,
            scroll_y: 0,
            max_scroll: 0,
            is_dragging_scroll: false,
            drag_start_y: 0,
            drag_start_scroll: 0,
            cards: Vec::new(),
            on_paint_custom: None,
        }));

        let raw_vp = Arc::into_raw(state.clone());
        let raw_cnt = Arc::into_raw(state.clone());
        unsafe {
            SetWindowLongPtrW(h_viewport, GWLP_USERDATA, raw_vp as isize);
            SetWindowLongPtrW(h_content, GWLP_USERDATA, raw_cnt as isize);
        }

        Some(Self { state })
    }

    /// Returns the HWND of the outer viewport (used for focus/clipping).
    pub fn viewport_hwnd(&self) -> isize {
        self.state.lock().map(|g| g.h_viewport).unwrap_or(0)
    }

    /// Returns the HWND of the inner content canvas.
    /// ALL CHILD CONTROLS (and CardLayout) should be created with this HWND as parent!
    pub fn content_hwnd(&self) -> isize {
        self.state.lock().map(|g| g.h_content).unwrap_or(0)
    }

    /// Sets the card container bounds `[(top, bottom)]` to be automatically drawn as rounded rectangles.
    pub fn set_cards(&self, cards: Vec<(i32, i32)>) {
        if let Ok(mut state) = self.state.lock() {
            state.cards = cards;
        }
    }

    /// Updates the total content height. Recalculates max_scroll, clamps scroll_y,
    /// resizes h_content, and refreshes the scrollbar visibility.
    pub fn set_content_height(&self, height: i32) {
        let (h_content, h_viewport) = {
            let mut state = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            state.content_h = height;
            let total_h = height.max(state.viewport_h);
            let max_s = (total_h - state.viewport_h).max(0);
            state.max_scroll = max_s;
            let target = state.scroll_y.clamp(0, max_s);
            state.scroll_y = target;

            unsafe {
                SetWindowPos(
                    state.h_content,
                    0,
                    0,
                    -target,
                    state.content_w,
                    total_h,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            (state.h_content, state.h_viewport)
        };
        unsafe {
            InvalidateRect(h_viewport, null_mut(), 0);
            InvalidateRect(h_content, null_mut(), 0);
        }
    }

    /// Programmatically scrolls to a specific Y offset.
    pub fn scroll_to(&self, new_scroll: i32) {
        let (h_content, h_viewport, target) = {
            let mut state = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let target = new_scroll.clamp(0, state.max_scroll);
            if target == state.scroll_y {
                return;
            }
            state.scroll_y = target;
            (state.h_content, state.h_viewport, target)
        };
        unsafe {
            SetWindowPos(
                h_content,
                0,
                0,
                -target,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            InvalidateRect(h_viewport, null_mut(), 0);
        }
    }

    /// Returns the current scroll offset.
    pub fn scroll_y(&self) -> i32 {
        self.state.lock().map(|g| g.scroll_y).unwrap_or(0)
    }

    /// Returns the maximum scroll offset.
    pub fn max_scroll(&self) -> i32 {
        self.state.lock().map(|g| g.max_scroll).unwrap_or(0)
    }

    /// Sets a custom background/card painting callback for the inner content canvas.
    pub fn set_on_paint<F: Fn(isize, &ThemePalette) + Send + Sync + 'static>(&self, f: F) {
        if let Ok(mut state) = self.state.lock() {
            state.on_paint_custom = Some(Arc::new(f));
        }
    }

    /// Shows or hides the scroll view.
    pub fn set_visible(&self, visible: bool) {
        if let Ok(state) = self.state.lock() {
            unsafe {
                if visible {
                    SetWindowPos(
                        state.h_viewport,
                        HWND_TOP,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                    );
                } else {
                    ShowWindow(state.h_viewport, SW_HIDE);
                }
            }
        }
    }

    /// Invalidates both viewport and content so they repaint cleanly.
    pub fn invalidate(&self) {
        if let Ok(state) = self.state.lock() {
            unsafe {
                InvalidateRect(state.h_viewport, null_mut(), 0);
                InvalidateRect(state.h_content, null_mut(), 0);
            }
        }
    }
}

fn scroll_by_delta(state_arc: &Mutex<ScrollViewState>, delta: i32) {
    let (h_content, h_viewport, new_scroll) = {
        let mut state = match state_arc.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if state.max_scroll <= 0 {
            return;
        }
        let step = 48;
        let target = (state.scroll_y - (delta / 120) * step).clamp(0, state.max_scroll);
        if target == state.scroll_y {
            return;
        }
        state.scroll_y = target;
        (state.h_content, state.h_viewport, target)
    };
    unsafe {
        SetWindowPos(
            h_content,
            0,
            0,
            -new_scroll,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        InvalidateRect(h_viewport, null_mut(), 0);
    }
}

unsafe extern "system" fn scroll_viewport_wndproc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let _dpi_guard = ScopedThreadDpiContext::for_window(hwnd);

    let state_ptr =
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<ScrollViewState> };
    if msg == WM_DESTROY {
        if !state_ptr.is_null() {
            unsafe {
                drop(Arc::from_raw(state_ptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    if state_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    let state_arc = unsafe { &*state_ptr };

    unsafe {
        match msg {
            WM_ERASEBKGND => {
                let hdc = wparam as isize;
                if hdc != 0 {
                    let palette = ThemePalette::get(is_current_dark());
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let bg_brush = CreateSolidBrush(palette.bg_window);
                    FillRect(hdc, &rc, bg_brush);
                    DeleteObject(bg_brush);
                }
                1
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let palette = ThemePalette::get(is_current_dark());
                    if let Ok(state) = state_arc.lock() {
                        let mut rc = RECT::default();
                        GetClientRect(hwnd, &mut rc);
                        // Fill exposed viewport area with window background color.
                        // WS_CLIPCHILDREN ensures child windows (h_content) are automatically
                        // excluded from this paint, so there is no flicker or overdraw.
                        let bg_brush = CreateSolidBrush(palette.bg_window);
                        FillRect(hdc, &rc, bg_brush);
                        DeleteObject(bg_brush);

                        if state.max_scroll > 0 {
                            let viewport_h = rc.bottom - rc.top;
                            let track_top = 4;
                            let track_bottom = viewport_h - 4;
                            let track_h = (track_bottom - track_top).max(20);
                            let total_h = state.content_h.max(viewport_h);
                            let thumb_h = ((viewport_h as f32 / total_h as f32) * track_h as f32)
                                .clamp(28.0, track_h as f32)
                                as i32;
                            let travel = (track_h - thumb_h).max(1);
                            let thumb_top = track_top
                                + ((state.scroll_y as f32 / state.max_scroll as f32)
                                    * travel as f32) as i32;

                            let thumb_color = if state.is_dragging_scroll {
                                palette.accent
                            } else {
                                palette.border_separator
                            };
                            let thumb_br = CreateSolidBrush(thumb_color);
                            let thumb_rc = RECT {
                                left: state.content_w + 2,
                                top: thumb_top,
                                right: state.content_w + 6,
                                bottom: thumb_top + thumb_h,
                            };
                            FillRect(hdc, &thumb_rc, thumb_br);
                            DeleteObject(thumb_br);
                        }
                    }
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) as i16) as i32;
                scroll_by_delta(state_arc, delta);
                0
            }
            WM_LBUTTONDOWN => {
                let x = (lparam as i16) as i32;
                let y = ((lparam >> 16) as i16) as i32;
                let mut should_capture = false;
                if let Ok(mut state) = state_arc.lock() {
                    if state.max_scroll > 0 && x >= state.content_w {
                        state.is_dragging_scroll = true;
                        state.drag_start_y = y;
                        state.drag_start_scroll = state.scroll_y;
                        should_capture = true;
                    }
                }
                if should_capture {
                    SetCapture(hwnd);
                    InvalidateRect(hwnd, null_mut(), 0);
                    return 0;
                }
                0
            }
            WM_MOUSEMOVE => {
                let y = ((lparam >> 16) as i16) as i32;
                let mut update_data = None;
                if let Ok(mut state) = state_arc.lock() {
                    if state.is_dragging_scroll && state.max_scroll > 0 {
                        let mut rc = RECT::default();
                        GetClientRect(hwnd, &mut rc);
                        let viewport_h = rc.bottom - rc.top;
                        let track_top = 4;
                        let track_bottom = viewport_h - 4;
                        let track_h = (track_bottom - track_top).max(20);
                        let total_h = state.content_h.max(viewport_h);
                        let thumb_h = ((viewport_h as f32 / total_h as f32) * track_h as f32)
                            .clamp(28.0, track_h as f32)
                            as i32;
                        let travel = (track_h - thumb_h).max(1);

                        let delta_y = y - state.drag_start_y;
                        let scroll_delta =
                            ((delta_y as f32 / travel as f32) * state.max_scroll as f32) as i32;
                        let target =
                            (state.drag_start_scroll + scroll_delta).clamp(0, state.max_scroll);
                        if target != state.scroll_y {
                            state.scroll_y = target;
                            update_data = Some((state.h_content, target));
                        }
                    }
                }
                if let Some((h_content, target)) = update_data {
                    SetWindowPos(
                        h_content,
                        0,
                        0,
                        -target,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                    InvalidateRect(hwnd, null_mut(), 0);
                    return 0;
                }
                0
            }
            WM_LBUTTONUP => {
                let mut should_release = false;
                if let Ok(mut state) = state_arc.lock() {
                    if state.is_dragging_scroll {
                        state.is_dragging_scroll = false;
                        should_release = true;
                    }
                }
                if should_release {
                    ReleaseCapture();
                    InvalidateRect(hwnd, null_mut(), 0);
                    return 0;
                }
                0
            }
            WM_COMMAND | WM_NOTIFY | WM_MEASUREITEM | WM_DRAWITEM => {
                let parent = GetParent(hwnd);
                if parent != 0 {
                    return SendMessageW(parent, msg, wparam, lparam);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe extern "system" fn scroll_content_wndproc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let _dpi_guard = ScopedThreadDpiContext::for_window(hwnd);

    let state_ptr =
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<ScrollViewState> };
    if msg == WM_DESTROY {
        if !state_ptr.is_null() {
            unsafe {
                drop(Arc::from_raw(state_ptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    if state_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    let state_arc = unsafe { &*state_ptr };

    unsafe {
        match msg {
            WM_ERASEBKGND => {
                let hdc = wparam as isize;
                if hdc != 0 {
                    let palette = ThemePalette::get(is_current_dark());
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let bg_brush = CreateSolidBrush(palette.bg_window);
                    FillRect(hdc, &rc, bg_brush);
                    DeleteObject(bg_brush);
                }
                1
            }

            WM_DRAWITEM => {
                let dis = &*(lparam as *const DRAWITEMSTRUCT);
                if dis.hwnd_item != 0 {
                    let res = SendMessageW(dis.hwnd_item, OCM_DRAWITEM, wparam, lparam);
                    if res != 0 {
                        return 1;
                    }
                }
                let parent = GetParent(hwnd);
                if parent != 0 {
                    return SendMessageW(parent, msg, wparam, lparam);
                }
                0
            }
            WM_COMMAND | WM_NOTIFY | WM_MEASUREITEM => {
                let parent = GetParent(hwnd);
                if parent != 0 {
                    return SendMessageW(parent, msg, wparam, lparam);
                }
                0
            }
            WM_CTLCOLORSTATIC => crate::ui_framework::win32::ctlcolor::handle_ctlcolor_static(
                hwnd, msg, wparam, lparam,
            ),
            WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                crate::ui_framework::win32::ctlcolor::handle_ctlcolor_edit_listbox(
                    hwnd, msg, wparam, lparam,
                )
            }
            WM_SETCURSOR => {
                crate::ui_framework::win32::ctlcolor::handle_setcursor(hwnd, msg, wparam, lparam)
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let width = rc.right - rc.left;
                    let height = rc.bottom - rc.top;
                    let palette = ThemePalette::get(is_current_dark());
                    if width > 0 && height > 0 {
                        if let Some(bb) = BackBuffer::new(hdc, width, height) {
                            paint_content_surface(bb.hdc(), state_arc, palette, &rc);
                        } else {
                            paint_content_surface(hdc, state_arc, palette, &rc);
                        }
                    } else {
                        paint_content_surface(hdc, state_arc, palette, &rc);
                    }
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_MOUSEWHEEL => {
                let parent = GetParent(hwnd);
                if parent != 0 {
                    return SendMessageW(parent, msg, wparam, lparam);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn paint_content_surface(
    hdc: isize,
    state_arc: &Mutex<ScrollViewState>,
    palette: &ThemePalette,
    rc: &RECT,
) {
    unsafe {
        let bg_brush = CreateSolidBrush(palette.bg_window);
        FillRect(hdc, rc, bg_brush);
        DeleteObject(bg_brush);

        let (cards, custom_cb) = if let Ok(state) = state_arc.lock() {
            (state.cards.clone(), state.on_paint_custom.clone())
        } else {
            (Vec::new(), None)
        };

        if !cards.is_empty() {
            let b_br = CreateSolidBrush(palette.bg_card);
            let b_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
            let ob = SelectObject(hdc, b_br);
            let op = SelectObject(hdc, b_pen);
            for &(top, bottom) in &cards {
                RoundRect(
                    hdc,
                    0,
                    top,
                    layout::CARD_WIDTH,
                    bottom,
                    radii::RADIUS_CARD,
                    radii::RADIUS_CARD,
                );
            }
            SelectObject(hdc, ob);
            SelectObject(hdc, op);
            DeleteObject(b_br);
            DeleteObject(b_pen);
        }

        if let Some(cb) = custom_cb {
            cb(hdc, palette);
        }
    }
}
