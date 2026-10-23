//! Single-HWND Declarative Surface Engine (`KoraDeclarativeSurface`)
//!
//! Renders an [`Element<M>`] tree onto a double-buffered GDI [`BackBuffer`] at 120 FPS
//! with zero child HWNDs, built-in smooth scrolling, hover tracking, slider dragging,
//! and a top-layer floating dropdown overlay for [`WidgetKind::Select`].

use super::element::*;
use super::layout_engine::*;
use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::tokens::{radii, spacing};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

struct OpenSelectOverlay<M> {
    key: u32,
    rect_doc: RECT,
    selected: usize,
    options: Vec<String>,
    hover_idx: Option<usize>,
    on_select: Option<Arc<dyn Fn(usize) -> M + Send + Sync>>,
}

struct ActiveSliderDrag<M> {
    node_id: usize,
    rect_doc: RECT,
    min: i32,
    max: i32,
    last_val: i32,
    on_change: Option<Arc<dyn Fn(i32, bool) -> M + Send + Sync>>,
}

struct SurfaceInner<M> {
    hwnd: isize,
    height: i32,
    content_w: i32,
    content_h: i32,
    scroll_y: i32,
    max_scroll: i32,
    is_dragging_scroll: bool,
    drag_start_y: i32,
    drag_start_scroll: i32,
    hover_node_id: Option<usize>,
    tracking_mouse: bool,
    layout_root: Option<LayoutNode<M>>,
    open_select: Option<OpenSelectOverlay<M>>,
    active_slider: Option<ActiveSliderDrag<M>>,
    on_msg: Option<Arc<dyn Fn(M) + Send + Sync>>,
    bg: BgStyle,
}

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
const SURFACE_CLASS_NAME: &str = "KoraDeclarativeSurface";
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
    pub fn render(&self, root: Element<M>) {
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

#[inline]
fn pt_in_rect(x: i32, y: i32, rc: &RECT) -> bool {
    x >= rc.left && x < rc.right && y >= rc.top && y < rc.bottom
}

#[inline]
fn offset_rect(rc: &RECT, dy: i32) -> RECT {
    RECT {
        left: rc.left,
        top: rc.top + dy,
        right: rc.right,
        bottom: rc.bottom + dy,
    }
}

const SELECT_ITEM_H: i32 = 28;

fn compute_dropdown_menu_rect<M>(sel: &OpenSelectOverlay<M>, scroll_y: i32, viewport_h: i32) -> RECT {
    let trig = offset_rect(&sel.rect_doc, -scroll_y);
    let menu_h = (sel.options.len() as i32 * SELECT_ITEM_H + 4).clamp(SELECT_ITEM_H + 4, 240);
    let below_bottom = trig.bottom + 2 + menu_h;
    let (top, bottom) = if below_bottom > viewport_h && trig.top - 2 - menu_h >= 0 {
        (trig.top - 2 - menu_h, trig.top - 2)
    } else {
        (trig.bottom + 2, below_bottom)
    };
    RECT {
        left: trig.left,
        top,
        right: trig.right,
        bottom,
    }
}

fn slider_value_from_x(x: i32, rc: &RECT, min: i32, max: i32) -> i32 {
    let pad = 8;
    let track_l = rc.left + pad;
    let track_r = (rc.right - pad).max(track_l + 1);
    let ratio = ((x - track_l) as f32 / (track_r - track_l) as f32).clamp(0.0, 1.0);
    (min as f32 + ratio * (max - min) as f32).round() as i32
}

/// Finds the deepest interactive node under document coordinates `(doc_x, doc_y)`.
fn hit_test_node<'a, M>(node: &'a LayoutNode<M>, doc_x: i32, doc_y: i32) -> Option<&'a LayoutNode<M>> {
    if !pt_in_rect(doc_x, doc_y, &node.rect) {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(found) = hit_test_node(child, doc_x, doc_y) {
            return Some(found);
        }
    }
    let is_interactive = node.element.on_click.is_some() || match &node.element.kind {
        WidgetKind::Button { disabled, on_click, .. } => !*disabled && on_click.is_some(),
        WidgetKind::Badge { disabled, on_click, .. } => !*disabled && on_click.is_some(),
        WidgetKind::Text { on_click, .. } => on_click.is_some(),
        WidgetKind::Switch { disabled, on_toggle, .. } => !*disabled && on_toggle.is_some(),
        WidgetKind::Checkbox { disabled, on_toggle, .. } => !*disabled && on_toggle.is_some(),
        WidgetKind::Select { disabled, .. } => !*disabled,
        WidgetKind::Slider { disabled, .. } => !*disabled,
        _ => false,
    };
    if is_interactive {
        Some(node)
    } else {
        None
    }
}

unsafe extern "system" fn surface_wndproc<M: Clone + Send + Sync + 'static>(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let _dpi_guard = ScopedThreadDpiContext::for_window(hwnd);

    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<SurfaceInner<M>> };
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

    match msg {
        WM_ERASEBKGND => 1,
        WM_SETCURSOR => {
            let is_hand = state_arc
                .lock()
                .ok()
                .map(|g| {
                    g.hover_node_id.is_some()
                        || g.open_select
                            .as_ref()
                            .and_then(|s| s.hover_idx)
                            .is_some()
                })
                .unwrap_or(false);
            unsafe {
                SetCursor(LoadCursorW(0, if is_hand { IDC_HAND } else { IDC_ARROW }));
            }
            1
        }
        WM_PAINT => {
            unsafe {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let w = rc.right - rc.left;
                    let h = rc.bottom - rc.top;
                    let palette = ThemePalette::get(is_current_dark());
                    if w > 0 && h > 0 {
                        if let Some(bb) = BackBuffer::new(hdc, w, h) {
                            paint_surface(bb.hdc(), state_arc, palette, &rc);
                        } else {
                            paint_surface(hdc, state_arc, palette, &rc);
                        }
                    }
                    EndPaint(hwnd, &ps);
                }
            }
            0
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) as i16) as i32;
            let mut should_repaint = false;
            if let Ok(mut guard) = state_arc.lock() {
                if guard.open_select.is_some() {
                    guard.open_select = None;
                    should_repaint = true;
                }
                if guard.max_scroll > 0 {
                    let step = 48;
                    let target = (guard.scroll_y - (delta / 120) * step).clamp(0, guard.max_scroll);
                    if target != guard.scroll_y {
                        guard.scroll_y = target;
                        should_repaint = true;
                    }
                }
            }
            if should_repaint {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            let x = (lparam as i16) as i32;
            let y = ((lparam >> 16) as i16) as i32;
            let mut pending_msg: Option<(Arc<dyn Fn(M) + Send + Sync>, M)> = None;
            let mut needs_repaint = false;

            if let Ok(mut guard) = state_arc.lock() {
                if !guard.tracking_mouse {
                    let mut tme = TRACKMOUSEEVENT {
                        cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dw_flags: TME_LEAVE,
                        hwnd_track: hwnd,
                        dw_hover_time: 0,
                    };
                    unsafe {
                        TrackMouseEvent(&mut tme);
                    }
                    guard.tracking_mouse = true;
                }

                // 1. Active Scrollbar Thumb Drag
                if guard.is_dragging_scroll && guard.max_scroll > 0 {
                    let track_h = (guard.height - 8).max(1);
                    let total_h = guard.content_h.max(guard.height);
                    let min_thumb = 28.0f32.min(track_h as f32);
                    let thumb_h = ((guard.height as f32 / total_h as f32) * track_h as f32)
                        .clamp(min_thumb, track_h as f32) as i32;
                    let travel = (track_h - thumb_h).max(1);
                    let delta_y = y - guard.drag_start_y;
                    let scroll_delta =
                        ((delta_y as f32 / travel as f32) * guard.max_scroll as f32) as i32;
                    let target = (guard.drag_start_scroll + scroll_delta).clamp(0, guard.max_scroll);
                    if target != guard.scroll_y {
                        guard.scroll_y = target;
                        needs_repaint = true;
                    }
                }
                // 2. Active Slider Drag
                else if let Some(ref mut sl) = guard.active_slider {
                    let new_val = slider_value_from_x(x, &sl.rect_doc, sl.min, sl.max);
                    if new_val != sl.last_val {
                        sl.last_val = new_val;
                        let cb_opt = sl.on_change.clone();
                        let nid = sl.node_id;
                        if let Some(ref mut root) = guard.layout_root {
                            update_slider_in_tree(root, nid, new_val);
                        }
                        if let (Some(cb), Some(handler)) = (cb_opt, guard.on_msg.clone()) {
                            pending_msg = Some((handler, cb(new_val, false)));
                        }
                        needs_repaint = true;
                    }
                }
                // 3. Open Select Dropdown Hover
                else if guard.open_select.is_some() {
                    let scroll_y = guard.scroll_y;
                    let vh = guard.height;
                    if let Some(ref mut sel) = guard.open_select {
                        let menu_rc = compute_dropdown_menu_rect(sel, scroll_y, vh);
                        let new_hover = if pt_in_rect(x, y, &menu_rc) {
                            let idx = ((y - (menu_rc.top + 2)) / SELECT_ITEM_H) as usize;
                            (idx < sel.options.len()).then_some(idx)
                        } else {
                            None
                        };
                        if new_hover != sel.hover_idx {
                            sel.hover_idx = new_hover;
                            needs_repaint = true;
                        }
                    }
                }
                // 4. Standard Widget Hover
                else {
                    let doc_y = y + guard.scroll_y;
                    let new_hover = guard
                        .layout_root
                        .as_ref()
                        .and_then(|root| hit_test_node(root, x, doc_y))
                        .map(|n| n.id);
                    if new_hover != guard.hover_node_id {
                        guard.hover_node_id = new_hover;
                        needs_repaint = true;
                    }
                }
            }

            if let Some((handler, m)) = pending_msg {
                handler(m);
            }
            if needs_repaint {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            if let Ok(mut guard) = state_arc.lock() {
                guard.tracking_mouse = false;
                if guard.hover_node_id.is_some() {
                    guard.hover_node_id = None;
                    unsafe {
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                }
            }
            0
        }
        WM_LBUTTONDOWN => {
            let x = (lparam as i16) as i32;
            let y = ((lparam >> 16) as i16) as i32;
            let mut pending_msg: Option<(Arc<dyn Fn(M) + Send + Sync>, M)> = None;
            let mut should_capture = false;
            let mut needs_repaint = false;

            if let Ok(mut guard) = state_arc.lock() {
                // 1. Check Open Select Dropdown Overlay first
                if let Some(sel) = guard.open_select.take() {
                    let menu_rc = compute_dropdown_menu_rect(&sel, guard.scroll_y, guard.height);
                    if pt_in_rect(x, y, &menu_rc) {
                        let idx = ((y - (menu_rc.top + 2)) / SELECT_ITEM_H) as usize;
                        if idx < sel.options.len()
                            && let (Some(cb), Some(handler)) =
                                (sel.on_select.as_ref(), guard.on_msg.clone())
                        {
                            pending_msg = Some((handler, cb(idx)));
                        }
                    }
                    needs_repaint = true;
                }
                // 2. Check Scrollbar Gutter Click
                else if guard.max_scroll > 0 && x >= guard.content_w {
                    guard.is_dragging_scroll = true;
                    guard.drag_start_y = y;
                    guard.drag_start_scroll = guard.scroll_y;
                    should_capture = true;
                    needs_repaint = true;
                }
                // 3. Check Interactive Widget Hit
                else {
                    let doc_y = y + guard.scroll_y;
                    let hit_action = guard
                        .layout_root
                        .as_ref()
                        .and_then(|root| hit_test_node(root, x, doc_y))
                        .map(|node| (node.id, node.rect, &node.element.kind, node.element.on_click.clone()));

                    if let Some((node_id, node_rect, kind, elem_click)) = hit_action {
                        if let Some(m) = elem_click {
                            if let Some(h) = guard.on_msg.clone() {
                                pending_msg = Some((h, m));
                            }
                        } else {
                        match kind {
                            WidgetKind::Button { on_click, .. }
                            | WidgetKind::Badge { on_click, .. }
                            | WidgetKind::Text { on_click, .. } => {
                                if let (Some(m), Some(h)) = (on_click.clone(), guard.on_msg.clone()) {
                                    pending_msg = Some((h, m));
                                }
                            }
                            WidgetKind::Switch { on_toggle, .. }
                            | WidgetKind::Checkbox { on_toggle, .. } => {
                                if let (Some(m), Some(h)) = (on_toggle.clone(), guard.on_msg.clone()) {
                                    pending_msg = Some((h, m));
                                }
                            }
                            WidgetKind::Select {
                                key,
                                selected,
                                options,
                                on_select,
                                ..
                            } => {
                                guard.open_select = Some(OpenSelectOverlay {
                                    key: *key,
                                    rect_doc: node_rect,
                                    selected: *selected,
                                    options: options.clone(),
                                    hover_idx: Some(*selected),
                                    on_select: on_select.clone(),
                                });
                                needs_repaint = true;
                            }
                            WidgetKind::Slider {
                                min,
                                max,
                                on_change,
                                ..
                            } => {
                                let new_val = slider_value_from_x(x, &node_rect, *min, *max);
                                let cb_clone = on_change.clone();
                                let min_v = *min;
                                let max_v = *max;
                                if let Some(ref mut root) = guard.layout_root {
                                    update_slider_in_tree(root, node_id, new_val);
                                }
                                if let (Some(cb), Some(h)) = (cb_clone.as_ref(), guard.on_msg.clone()) {
                                    pending_msg = Some((h, cb(new_val, false)));
                                }
                                guard.active_slider = Some(ActiveSliderDrag {
                                    node_id,
                                    rect_doc: node_rect,
                                    min: min_v,
                                    max: max_v,
                                    last_val: new_val,
                                    on_change: cb_clone,
                                });
                                should_capture = true;
                                needs_repaint = true;
                            }
                            _ => {}
                        }
                        }
                    }
                }
            }

            if should_capture {
                unsafe {
                    SetCapture(hwnd);
                }
            }
            if let Some((handler, m)) = pending_msg {
                handler(m);
            }
            if needs_repaint {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
            }
            0
        }
        WM_LBUTTONUP => {
            let mut pending_msg: Option<(Arc<dyn Fn(M) + Send + Sync>, M)> = None;
            let mut should_release = false;
            if let Ok(mut guard) = state_arc.lock() {
                if guard.is_dragging_scroll {
                    guard.is_dragging_scroll = false;
                    should_release = true;
                }
                if let Some(sl) = guard.active_slider.take() {
                    should_release = true;
                    if let (Some(cb), Some(h)) = (sl.on_change, guard.on_msg.clone()) {
                        pending_msg = Some((h, cb(sl.last_val, true)));
                    }
                }
            }
            if should_release {
                unsafe {
                    ReleaseCapture();
                    InvalidateRect(hwnd, null_mut(), 0);
                }
            }
            if let Some((handler, m)) = pending_msg {
                handler(m);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn update_slider_in_tree<M>(node: &mut LayoutNode<M>, target_id: usize, new_val: i32) -> bool {
    if node.id == target_id {
        if let WidgetKind::Slider { value, .. } = &mut node.element.kind {
            *value = new_val;
            return true;
        }
    }
    for c in &mut node.children {
        if update_slider_in_tree(c, target_id, new_val) {
            return true;
        }
    }
    false
}

fn paint_surface<M>(
    hdc: isize,
    state_arc: &Mutex<SurfaceInner<M>>,
    palette: &ThemePalette,
    client_rc: &RECT,
) {
    let Ok(guard) = state_arc.lock() else { return };

    // 1. Fill window background
    if let Some(bg_color) = guard.bg.resolve(palette) {
        fill_rect_color(hdc, client_rc, bg_color);
    }

    let scroll_y = guard.scroll_y;
    let viewport_h = client_rc.bottom - client_rc.top;
    let hover_id = guard.hover_node_id;
    let open_select_key = guard.open_select.as_ref().map(|s| s.key);

    // 2. Paint layout tree
    if let Some(ref root) = guard.layout_root {
        paint_node(hdc, root, scroll_y, viewport_h, hover_id, open_select_key, palette);
    }

    // 3. Paint slim modern scrollbar thumb (only if scrollable and viewport has enough height)
    if guard.max_scroll > 0 && viewport_h >= 40 {
        let track_top = 4;
        let track_bottom = viewport_h - 4;
        let track_h = (track_bottom - track_top).max(1);
        let total_h = guard.content_h.max(viewport_h);
        let min_thumb = 28.0f32.min(track_h as f32);
        let thumb_h = ((viewport_h as f32 / total_h as f32) * track_h as f32)
            .clamp(min_thumb, track_h as f32) as i32;
        let travel = (track_h - thumb_h).max(1);
        let thumb_top =
            track_top + ((scroll_y as f32 / guard.max_scroll as f32) * travel as f32) as i32;
        let thumb_color = if guard.is_dragging_scroll {
            palette.accent
        } else {
            palette.border_separator
        };
        let thumb_rc = RECT {
            left: guard.content_w + 3,
            top: thumb_top,
            right: guard.content_w + 7,
            bottom: thumb_top + thumb_h,
        };
        round_rect(hdc, &thumb_rc, 4, thumb_color, None);
    }

    // 4. Paint top-layer open Select dropdown overlay (z-index: top)
    if let Some(ref sel) = guard.open_select {
        let menu_rc = compute_dropdown_menu_rect(sel, scroll_y, viewport_h);
        round_rect(
            hdc,
            &menu_rc,
            radii::RADIUS_MD,
            palette.bg_card,
            Some(palette.accent),
        );

        unsafe {
            SetBkMode(hdc, TRANSPARENT);
            let font_norm = resolve_font_handle(FontScale::Normal);
            let font_bold = resolve_font_handle(FontScale::Bold);

            for (i, opt) in sel.options.iter().enumerate() {
                let item_top = menu_rc.top + 2 + i as i32 * SELECT_ITEM_H;
                if item_top + SELECT_ITEM_H > menu_rc.bottom {
                    break;
                }
                let item_rc = RECT {
                    left: menu_rc.left + 2,
                    top: item_top,
                    right: menu_rc.right - 2,
                    bottom: item_top + SELECT_ITEM_H,
                };
                let is_sel = i == sel.selected;
                let is_hov = sel.hover_idx == Some(i);

                if is_hov {
                    round_rect(hdc, &item_rc, 4, palette.accent, None);
                } else if is_sel {
                    round_rect(hdc, &item_rc, 4, palette.tab_active_bg, None);
                }

                let txt_color = if is_hov {
                    palette.text_white
                } else if is_sel {
                    palette.accent
                } else {
                    palette.text_primary
                };
                SetTextColor(hdc, txt_color);
                SelectObject(hdc, if is_sel { font_bold } else { font_norm });

                let mut text_rc = RECT {
                    left: item_rc.left + 10,
                    top: item_rc.top,
                    right: item_rc.right - 24,
                    bottom: item_rc.bottom,
                };
                let wide = to_wide(opt);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut text_rc,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                );

                if is_sel {
                    let mut chk_rc = RECT {
                        left: item_rc.right - 22,
                        top: item_rc.top,
                        right: item_rc.right - 6,
                        bottom: item_rc.bottom,
                    };
                    let chk_w = to_wide("✓");
                    DrawTextW(
                        hdc,
                        chk_w.as_ptr(),
                        -1,
                        &mut chk_rc,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                }
            }
        }
    }
}

fn paint_node<M>(
    hdc: isize,
    node: &LayoutNode<M>,
    scroll_y: i32,
    viewport_h: i32,
    hover_id: Option<usize>,
    open_select_key: Option<u32>,
    palette: &ThemePalette,
) {
    let rc = offset_rect(&node.rect, -scroll_y);
    if rc.bottom < 0 || rc.top > viewport_h {
        return;
    }

    let style = &node.element.style;
    let is_hover = hover_id == Some(node.id);

    // 1. Paint Container Box Background & Borders
    let is_clickable_container = node.element.on_click.is_some();
    let bg_opt = if is_hover && is_clickable_container && style.bg != BgStyle::TabActive {
        Some(palette.tab_hover_bg)
    } else {
        style.bg.resolve(palette)
    };
    let border_opt = if is_hover && is_clickable_container && style.bg != BgStyle::TabActive {
        Some((palette.border, false))
    } else {
        style.border.resolve(palette)
    };
    let radius = if style.radius > 0 {
        style.radius
    } else if is_clickable_container {
        6
    } else {
        0
    };

    if bg_opt.is_some() || border_opt.is_some() {
        let fill = bg_opt.unwrap_or(palette.bg_card);
        if let Some((b_col, is_dashed)) = border_opt {
            if is_dashed {
                unsafe {
                    round_rect(hdc, &rc, radius, fill, None);
                    let pen = CreatePen(PS_DASH, 1, b_col);
                    let op = SelectObject(hdc, pen);
                    let ob = SelectObject(hdc, GetStockObject(NULL_BRUSH));
                    RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
                    SelectObject(hdc, ob);
                    SelectObject(hdc, op);
                    DeleteObject(pen);
                }
            } else {
                round_rect(hdc, &rc, radius, fill, Some(b_col));
            }
        } else {
            round_rect(hdc, &rc, radius, fill, None);
        }
    }

    // Left accent bar (e.g. Alert Banner)
    if let Some((bar_w, variant)) = style.border_left_accent {
        let bar_col = variant.colors(palette, false).2;
        let bar_rc = RECT {
            left: rc.left,
            top: rc.top + 2,
            right: (rc.left + bar_w).min(rc.right),
            bottom: rc.bottom - 2,
        };
        round_rect(hdc, &bar_rc, 4, bar_col, None);
    }

    if style.border_bottom {
        draw_line(
            hdc,
            rc.left,
            rc.bottom - 1,
            rc.right,
            rc.bottom - 1,
            palette.border_separator,
        );
    }
    if style.border_top {
        draw_line(
            hdc,
            rc.left,
            rc.top,
            rc.right,
            rc.top,
            palette.border_separator,
        );
    }

    // 2. Paint Atomic Widget Content
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        match &node.element.kind {
            WidgetKind::Empty => {}
            WidgetKind::Divider => {
                draw_line(hdc, rc.left, rc.top, rc.right, rc.top, palette.border_separator);
            }
            WidgetKind::Text {
                content,
                font,
                tone,
                align,
                wrap,
                on_click,
            } => {
                let hf = resolve_font_handle(*font);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                let color = if on_click.is_some() && is_hover {
                    palette.accent_hover
                } else {
                    tone.resolve(palette)
                };
                SetTextColor(hdc, color);
                let mut text_rc = rc;
                let align_flag = match align {
                    TextAlign::Left => DT_LEFT,
                    TextAlign::Center => DT_CENTER,
                    TextAlign::Right => DT_RIGHT,
                };
                let wrap_flag = if *wrap {
                    DT_WORDBREAK
                } else {
                    DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS
                };
                let wide = to_wide(content);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut text_rc,
                    align_flag | wrap_flag | DT_NOPREFIX,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Label {
                content,
                font,
                tone,
                align,
            } => {
                let hf = resolve_font_handle(*font);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                let is_clickable = node.element.on_click.is_some();
                let color = if is_clickable && is_hover {
                    palette.accent_hover
                } else if is_clickable {
                    palette.accent
                } else {
                    tone.resolve(palette)
                };
                SetTextColor(hdc, color);
                let mut text_rc = rc;
                let align_flag = match align {
                    TextAlign::Left => DT_LEFT,
                    TextAlign::Center => DT_CENTER,
                    TextAlign::Right => DT_RIGHT,
                };
                let wide = to_wide(content);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut text_rc,
                    align_flag | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Badge {
                text,
                variant,
                is_pill,
                disabled,
                ..
            } => {
                let (bg, border, fg) = match variant {
                    Variant::Primary => (
                        palette.bg_input,
                        if is_hover { palette.accent } else { palette.border },
                        if *disabled {
                            palette.text_secondary
                        } else {
                            palette.accent
                        },
                    ),
                    v => v.colors(palette, is_hover),
                };
                let rad = if *is_pill {
                    radii::RADIUS_PILL
                } else {
                    radii::RADIUS_MD
                };
                round_rect(hdc, &rc, rad, bg, Some(border));
                let hf = resolve_font_handle(FontScale::Bold);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                SetTextColor(hdc, if *disabled { palette.text_secondary } else { fg });
                let mut trc = rc;
                let wide = to_wide(text);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut trc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Button {
                label,
                variant,
                disabled,
                ..
            } => {
                let (bg, border, fg) = if *disabled {
                    (palette.bg_input, palette.border_separator, palette.text_secondary)
                } else {
                    variant.colors(palette, is_hover)
                };
                round_rect(hdc, &rc, radii::RADIUS_MD, bg, Some(border));
                let hf = resolve_font_handle(FontScale::Bold);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                SetTextColor(hdc, fg);
                let mut trc = rc;
                let wide = to_wide(label);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut trc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Switch {
                checked,
                label,
                note,
                disabled,
                ..
            } => {
                let sw_w = 34;
                let sw_h = 18;
                let sw_top = rc.top + ((rc.bottom - rc.top) - sw_h) / 2;
                let sw_rc = RECT {
                    left: rc.left,
                    top: sw_top,
                    right: rc.left + sw_w,
                    bottom: sw_top + sw_h,
                };
                let (track_bg, track_border, thumb_col) = if *disabled {
                    (palette.bg_input, palette.border_separator, palette.text_secondary)
                } else if *checked {
                    let c = if is_hover {
                        palette.accent_hover
                    } else {
                        palette.accent
                    };
                    (c, c, palette.text_white)
                } else {
                    (
                        palette.bg_input,
                        if is_hover {
                            palette.accent
                        } else {
                            palette.border_hover
                        },
                        palette.text_secondary,
                    )
                };
                round_rect(hdc, &sw_rc, sw_h, track_bg, Some(track_border));

                let thumb_d = 12;
                let thumb_left = if *checked {
                    sw_rc.right - thumb_d - 3
                } else {
                    sw_rc.left + 3
                };
                let thumb_rc = RECT {
                    left: thumb_left,
                    top: sw_rc.top + 3,
                    right: thumb_left + thumb_d,
                    bottom: sw_rc.top + 3 + thumb_d,
                };
                fill_ellipse(hdc, &thumb_rc, thumb_col);

                let mut cur_x = sw_rc.right + 8;
                if !label.is_empty() {
                    let hf = resolve_font_handle(FontScale::Bold);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(
                        hdc,
                        if *disabled {
                            palette.text_secondary
                        } else {
                            palette.text_primary
                        },
                    );
                    let (lw, _) = measure_text_rect(hdc, label, FontScale::Bold, 1000, false);
                    let mut lrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: (cur_x + lw + 2).min(rc.right),
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(label);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut lrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                    cur_x = lrc.right + 6;
                }
                if !note.is_empty() && cur_x < rc.right {
                    let hf = resolve_font_handle(FontScale::Small);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(hdc, palette.text_secondary);
                    let mut nrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: rc.right,
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(note);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut nrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                }
            }
            WidgetKind::Checkbox {
                checked,
                indeterminate,
                label,
                note,
                disabled,
                ..
            } => {
                let box_sz = 15;
                let box_top = rc.top + ((rc.bottom - rc.top) - box_sz) / 2;
                let box_rc = RECT {
                    left: rc.left,
                    top: box_top,
                    right: rc.left + box_sz,
                    bottom: box_top + box_sz,
                };
                let is_active = *checked || *indeterminate;
                let (bg, border) = if *disabled {
                    (palette.bg_input, palette.border_separator)
                } else if is_active {
                    let c = if is_hover {
                        palette.accent_hover
                    } else {
                        palette.accent
                    };
                    (c, c)
                } else {
                    (
                        palette.bg_input,
                        if is_hover {
                            palette.accent
                        } else {
                            palette.border_hover
                        },
                    )
                };
                round_rect(hdc, &box_rc, 3, bg, Some(border));

                if *indeterminate {
                    let dash_rc = RECT {
                        left: box_rc.left + 3,
                        top: box_rc.top + 6,
                        right: box_rc.right - 3,
                        bottom: box_rc.top + 9,
                    };
                    fill_rect_color(hdc, &dash_rc, palette.text_white);
                } else if *checked {
                    let pen = CreatePen(PS_SOLID, 2, palette.text_white);
                    let op = SelectObject(hdc, pen);
                    MoveToEx(hdc, box_rc.left + 3, box_rc.top + 7, null_mut());
                    LineTo(hdc, box_rc.left + 6, box_rc.top + 10);
                    LineTo(hdc, box_rc.left + 12, box_rc.top + 4);
                    SelectObject(hdc, op);
                    DeleteObject(pen);
                }

                let mut cur_x = box_rc.right + 8;
                if !label.is_empty() {
                    let hf = resolve_font_handle(FontScale::Normal);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(
                        hdc,
                        if *disabled {
                            palette.text_secondary
                        } else {
                            palette.text_primary
                        },
                    );
                    let (lw, _) = measure_text_rect(hdc, label, FontScale::Normal, 1000, false);
                    let mut lrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: (cur_x + lw + 2).min(rc.right),
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(label);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut lrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                    cur_x = lrc.right + 6;
                }
                if !note.is_empty() && cur_x < rc.right {
                    let hf = resolve_font_handle(FontScale::Small);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(hdc, palette.text_secondary);
                    let mut nrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: rc.right,
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(note);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut nrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                }
            }
            WidgetKind::Select {
                key,
                selected,
                options,
                disabled,
                ..
            } => {
                let is_open = open_select_key == Some(*key);
                let border_col = if *disabled {
                    palette.border_separator
                } else if is_open || is_hover {
                    palette.accent
                } else {
                    palette.border
                };
                round_rect(hdc, &rc, radii::RADIUS_MD, palette.bg_input, Some(border_col));

                let sel_text = options.get(*selected).map(|s| s.as_str()).unwrap_or("");
                let hf = resolve_font_handle(FontScale::Normal);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                SetTextColor(
                    hdc,
                    if *disabled {
                        palette.text_secondary
                    } else {
                        palette.text_primary
                    },
                );
                let mut trc = RECT {
                    left: rc.left + spacing::SPACE_8,
                    top: rc.top,
                    right: rc.right - 24,
                    bottom: rc.bottom,
                };
                let wide = to_wide(sel_text);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut trc,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                );

                // Draw chevron arrow `▾`
                SetTextColor(hdc, palette.text_secondary);
                let mut crc = RECT {
                    left: rc.right - 22,
                    top: rc.top,
                    right: rc.right - 6,
                    bottom: rc.bottom,
                };
                let chev = to_wide(if is_open { "▴" } else { "▾" });
                DrawTextW(
                    hdc,
                    chev.as_ptr(),
                    -1,
                    &mut crc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Slider {
                value,
                min,
                max,
                disabled,
                ..
            } => {
                let pad = 8;
                let track_l = rc.left + pad;
                let track_r = (rc.right - pad).max(track_l + 4);
                let cy = rc.top + (rc.bottom - rc.top) / 2;
                let track_rc = RECT {
                    left: track_l,
                    top: cy - 3,
                    right: track_r,
                    bottom: cy + 3,
                };
                round_rect(hdc, &track_rc, 6, palette.bg_input, Some(palette.border));

                let ratio = ((*value - *min) as f32 / (*max - *min).max(1) as f32).clamp(0.0, 1.0);
                let thumb_x = track_l + (ratio * (track_r - track_l) as f32).round() as i32;
                let active_col = if *disabled {
                    palette.border_hover
                } else {
                    palette.accent
                };

                if thumb_x > track_l {
                    let fill_rc = RECT {
                        left: track_l,
                        top: cy - 3,
                        right: thumb_x,
                        bottom: cy + 3,
                    };
                    round_rect(hdc, &fill_rc, 6, active_col, None);
                }

                let r = if is_hover && !*disabled { 8 } else { 7 };
                let thumb_rc = RECT {
                    left: thumb_x - r,
                    top: cy - r,
                    right: thumb_x + r,
                    bottom: cy + r,
                };
                fill_ellipse(hdc, &thumb_rc, active_col);
            }
            WidgetKind::Row(_) | WidgetKind::Col(_) | WidgetKind::Grid { .. } => {}
            WidgetKind::TabGroup { options, selected, .. } => {
                round_rect(
                    hdc,
                    &rc,
                    radii::RADIUS_PILL,
                    palette.tab_track_bg,
                    Some(palette.tab_track_border),
                );

                let num_tabs = options.len();
                if num_tabs > 0 {
                    let track_padding = 3;
                    let available_w = (rc.right - rc.left) - (track_padding * 2);
                    let tab_w = available_w / num_tabs as i32;

                    for (i, title) in options.iter().enumerate() {
                        let i_i32 = i as i32;
                        let item_left = rc.left + track_padding + i_i32 * tab_w;
                        let item_right = if i == num_tabs - 1 {
                            rc.right - track_padding
                        } else {
                            item_left + tab_w
                        };
                        let item_top = rc.top + track_padding;
                        let item_bottom = rc.bottom - track_padding;

                        let mut item_rc = RECT {
                            left: item_left,
                            top: item_top,
                            right: item_right,
                            bottom: item_bottom,
                        };

                        let is_item_hover = if let Some(hid) = hover_id {
                            node.children.get(i).map(|c| c.id == hid).unwrap_or(false)
                        } else {
                            false
                        };

                        if i == *selected {
                            round_rect(
                                hdc,
                                &item_rc,
                                radii::RADIUS_CARD,
                                palette.tab_active_bg,
                                Some(palette.tab_active_border),
                            );

                            let hf = resolve_font_handle(FontScale::Bold);
                            let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                            SetTextColor(hdc, palette.accent);

                            let line_rc = RECT {
                                left: item_rc.left + 4,
                                top: item_rc.bottom - 2,
                                right: item_rc.right - 4,
                                bottom: item_rc.bottom,
                            };
                            let line_br = CreateSolidBrush(palette.accent);
                            FillRect(hdc, &line_rc, line_br);
                            DeleteObject(line_br);

                            let wide = to_wide(title);
                            DrawTextW(
                                hdc,
                                wide.as_ptr(),
                                -1,
                                &mut item_rc,
                                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                            );
                            if old_f != 0 {
                                SelectObject(hdc, old_f);
                            }
                        } else {
                            if is_item_hover {
                                round_rect(
                                    hdc,
                                    &item_rc,
                                    radii::RADIUS_CARD,
                                    palette.tab_hover_bg,
                                    None,
                                );
                            }
                            let hf = resolve_font_handle(FontScale::Normal);
                            let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                            SetTextColor(hdc, palette.text_secondary);

                            let wide = to_wide(title);
                            DrawTextW(
                                hdc,
                                wide.as_ptr(),
                                -1,
                                &mut item_rc,
                                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                            );
                            if old_f != 0 {
                                SelectObject(hdc, old_f);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Paint Child Nodes
    if !matches!(node.element.kind, WidgetKind::TabGroup { .. }) {
        for child in &node.children {
            paint_node(
                hdc,
                child,
                scroll_y,
                viewport_h,
                hover_id,
                open_select_key,
                palette,
            );
        }
    }
}
