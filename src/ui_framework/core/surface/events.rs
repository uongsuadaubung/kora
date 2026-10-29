//! Native Win32 Surface Message Loop, Interaction Dispatch, and Hit Testing

use super::paint::paint_surface;
use super::types::*;
use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::core::element::WidgetKind;
use crate::ui_framework::core::layout_engine::LayoutNode;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

pub(crate) fn hit_test_node_layer<'a, M>(
    node: &'a LayoutNode<M>,
    doc_x: i32,
    doc_y: i32,
    parent_z: i32,
    target_z: i32,
) -> Option<&'a LayoutNode<M>> {
    let my_z = if node.element.style.z_index != 0 {
        node.element.style.z_index
    } else {
        parent_z
    };

    for child in node.children.iter().rev() {
        if let Some(found) = hit_test_node_layer(child, doc_x, doc_y, my_z, target_z) {
            return Some(found);
        }
    }

    if my_z == target_z && pt_in_rect(doc_x, doc_y, &node.rect) {
        let is_interactive = node.element.on_click.is_some()
            || match &node.element.kind {
                WidgetKind::Button {
                    disabled, on_click, ..
                } => !*disabled && on_click.is_some(),
                WidgetKind::Badge {
                    disabled, on_click, ..
                } => !*disabled && on_click.is_some(),
                WidgetKind::Text { on_click, .. } => on_click.is_some(),
                WidgetKind::Switch {
                    disabled,
                    on_toggle,
                    ..
                } => !*disabled && on_toggle.is_some(),
                WidgetKind::Checkbox {
                    disabled,
                    on_toggle,
                    ..
                } => !*disabled && on_toggle.is_some(),
                WidgetKind::Select { disabled, .. } => !*disabled,
                WidgetKind::Slider { disabled, .. } => !*disabled,
                WidgetKind::Input { disabled, .. } => !*disabled,
                _ => false,
            };
        if is_interactive {
            return Some(node);
        }
    }
    None
}

/// Finds the deepest interactive node under document coordinates `(doc_x, doc_y)`
/// prioritizing higher `z_index` layers over lower layers.
pub fn hit_test_node<'a, M>(
    node: &'a LayoutNode<M>,
    doc_x: i32,
    doc_y: i32,
) -> Option<&'a LayoutNode<M>> {
    let mut z_levels = Vec::with_capacity(4);
    collect_z_levels(node, 0, &mut z_levels);
    z_levels.sort_unstable();
    z_levels.dedup();

    for &z in z_levels.iter().rev() {
        if let Some(hit) = hit_test_node_layer(node, doc_x, doc_y, 0, z) {
            return Some(hit);
        }
    }
    None
}

pub(crate) fn update_slider_in_tree<M>(
    node: &mut LayoutNode<M>,
    target_id: usize,
    new_val: i32,
) -> bool {
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

/// Fallback brush for edit controls when surface state is temporarily held during render()
fn get_cached_edit_brush(color: u32) -> isize {
    use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
    static CACHED_COLOR: AtomicU32 = AtomicU32::new(0xFFFFFFFF);
    static CACHED_BRUSH: AtomicIsize = AtomicIsize::new(0);

    let cur = CACHED_COLOR.load(Ordering::Relaxed);
    let brush = CACHED_BRUSH.load(Ordering::Relaxed);
    if cur == color && brush != 0 {
        return brush;
    }
    let new_brush = unsafe { CreateSolidBrush(color) };
    let old_brush = CACHED_BRUSH.swap(new_brush, Ordering::Relaxed);
    CACHED_COLOR.store(color, Ordering::Relaxed);
    if old_brush != 0 {
        unsafe {
            DeleteObject(old_brush);
        }
    }
    new_brush
}

pub(crate) unsafe extern "system" fn surface_wndproc<M: Clone + Send + Sync + 'static>(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let _dpi_guard = ScopedThreadDpiContext::for_window(hwnd);

    let state_ptr =
        unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<SurfaceInner<M>> };
    if msg == WM_DESTROY {
        if !state_ptr.is_null() {
            unsafe {
                if let Ok(mut g) = (*state_ptr).lock() {
                    if g.edit_bg_brush != 0 {
                        DeleteObject(g.edit_bg_brush);
                        g.edit_bg_brush = 0;
                    }
                    for inp in g.inputs.drain(..) {
                        DestroyWindow(inp.hwnd);
                    }
                }
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
                        || g.open_select.as_ref().and_then(|s| s.hover_idx).is_some()
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
                    let clip_rc = if ps.rc_paint.right > ps.rc_paint.left
                        && ps.rc_paint.bottom > ps.rc_paint.top
                    {
                        ps.rc_paint
                    } else {
                        rc
                    };
                    if w > 0 && h > 0 {
                        if let Some(bb) = BackBuffer::new(hdc, w, h) {
                            paint_surface(bb.hdc(), state_arc, palette, &rc, &clip_rc);
                        } else {
                            paint_surface(hdc, state_arc, palette, &rc, &clip_rc);
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
            let mut update_inputs = false;
            let mut handled = false;
            if let Ok(mut guard) = state_arc.lock() {
                if guard.open_select.is_some() {
                    guard.open_select = None;
                    should_repaint = true;
                    update_inputs = true;
                }
                if guard.max_scroll > 0 {
                    handled = true;
                    let step = 48;
                    let target = (guard.scroll_y - (delta / 120) * step).clamp(0, guard.max_scroll);
                    if target != guard.scroll_y {
                        guard.scroll_y = target;
                        should_repaint = true;
                        update_inputs = true;
                    }
                }
                if update_inputs {
                    guard.update_input_scroll();
                }
            }
            if should_repaint {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
            }
            if !handled {
                let parent = unsafe { GetParent(hwnd) };
                if parent != 0 {
                    unsafe {
                        SendMessageW(parent, WM_MOUSEWHEEL, wparam, lparam);
                    }
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
                    let target =
                        (guard.drag_start_scroll + scroll_delta).clamp(0, guard.max_scroll);
                    if target != guard.scroll_y {
                        guard.scroll_y = target;
                        guard.update_input_scroll();
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
                            let mut d_rc = inflate_rect(&menu_rc, 2, 2);
                            unsafe {
                                InvalidateRect(hwnd, &mut d_rc, 0);
                            }
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
                        let old_id = guard.hover_node_id;
                        guard.hover_node_id = new_hover;
                        let mut damage_rc: Option<RECT> = None;
                        if let Some(ref root) = guard.layout_root {
                            let old_rc = old_id.and_then(|id| find_node_rect(root, id));
                            let new_rc = new_hover.and_then(|id| find_node_rect(root, id));
                            damage_rc = match (old_rc, new_rc) {
                                (Some(o), Some(n)) => Some(rect_union(&o, &n)),
                                (Some(o), None) => Some(o),
                                (None, Some(n)) => Some(n),
                                (None, None) => None,
                            };
                        }
                        if let Some(d_rc) = damage_rc {
                            let mut vp_rc =
                                inflate_rect(&offset_rect(&d_rc, -guard.scroll_y), 4, 4);
                            unsafe {
                                InvalidateRect(hwnd, &mut vp_rc, 0);
                            }
                        } else {
                            needs_repaint = true;
                        }
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
                if let Some(old_id) = guard.hover_node_id.take() {
                    let old_rc = guard
                        .layout_root
                        .as_ref()
                        .and_then(|root| find_node_rect(root, old_id));
                    if let Some(rc) = old_rc {
                        let mut vp_rc = inflate_rect(&offset_rect(&rc, -guard.scroll_y), 4, 4);
                        unsafe {
                            InvalidateRect(hwnd, &mut vp_rc, 0);
                        }
                    } else {
                        unsafe {
                            InvalidateRect(hwnd, null_mut(), 0);
                        }
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
                    guard.update_input_scroll();
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
                        .map(|node| {
                            (
                                node.id,
                                node.rect,
                                &node.element.kind,
                                node.element.on_click.clone(),
                            )
                        });

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
                                    if let (Some(m), Some(h)) =
                                        (on_click.clone(), guard.on_msg.clone())
                                    {
                                        pending_msg = Some((h, m));
                                    }
                                }
                                WidgetKind::Switch { on_toggle, .. }
                                | WidgetKind::Checkbox { on_toggle, .. } => {
                                    if let (Some(m), Some(h)) =
                                        (on_toggle.clone(), guard.on_msg.clone())
                                    {
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
                                    guard.update_input_scroll();
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
                                    if let (Some(cb), Some(h)) =
                                        (cb_clone.as_ref(), guard.on_msg.clone())
                                    {
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
                                WidgetKind::Input { .. } => {
                                    if let Some(inp) =
                                        guard.inputs.iter().find(|i| i.node_id == node_id)
                                    {
                                        let edit_hwnd = inp.hwnd;
                                        unsafe {
                                            SetFocus(edit_hwnd);
                                        }
                                    }
                                }
                                _ => unsafe {
                                    SetFocus(hwnd);
                                },
                            }
                        }
                    } else {
                        unsafe {
                            SetFocus(hwnd);
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
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            let hdc = wparam as isize;
            let palette = ThemePalette::get(is_current_dark());
            unsafe {
                SetTextColor(hdc, palette.text_input);
                SetBkColor(hdc, palette.bg_input);
                if let Ok(mut g) = state_arc.try_lock() {
                    if g.edit_bg_brush != 0 && g.edit_bg_color != palette.bg_input {
                        DeleteObject(g.edit_bg_brush);
                        g.edit_bg_brush = 0;
                    }
                    if g.edit_bg_brush == 0 {
                        g.edit_bg_brush = CreateSolidBrush(palette.bg_input);
                        g.edit_bg_color = palette.bg_input;
                    }
                    return g.edit_bg_brush;
                }
                return get_cached_edit_brush(palette.bg_input);
            }
        }
        WM_COMMAND => {
            let notify = (wparam >> 16) as u16;
            let edit_hwnd = lparam;
            if notify == 0x0300
            /* EN_CHANGE */
            {
                let mut pending: Option<(Arc<dyn Fn(M) + Send + Sync>, M)> = None;
                if let Ok(mut guard) = state_arc.try_lock() {
                    let on_msg = guard.on_msg.clone();
                    for input in &mut guard.inputs {
                        if input.hwnd == edit_hwnd {
                            let len = unsafe { GetWindowTextLengthW(edit_hwnd) };
                            let mut buf = vec![0u16; (len + 1) as usize];
                            unsafe {
                                GetWindowTextW(edit_hwnd, buf.as_mut_ptr(), buf.len() as i32)
                            };
                            let text = String::from_utf16_lossy(&buf[..len as usize]);
                            if text != input.last_val {
                                input.last_val = text.clone();
                                if let (Some(cb), Some(h)) = (&input.on_change, on_msg) {
                                    pending = Some((h, cb(text)));
                                }
                            }
                            break;
                        }
                    }
                }
                if let Some((h, m)) = pending {
                    h(m);
                }
                return 0;
            } else if notify == 0x0100
            /* EN_SETFOCUS */
            {
                if let Ok(mut guard) = state_arc.try_lock() {
                    for input in &guard.inputs {
                        if input.hwnd == edit_hwnd {
                            guard.focused_node_id = Some(input.node_id);
                            break;
                        }
                    }
                }
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
                return 0;
            } else if notify == 0x0200
            /* EN_KILLFOCUS */
            {
                if let Ok(mut guard) = state_arc.try_lock() {
                    if let Some(focused) = guard.focused_node_id {
                        for input in &guard.inputs {
                            if input.hwnd == edit_hwnd && input.node_id == focused {
                                guard.focused_node_id = None;
                                break;
                            }
                        }
                    }
                }
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
                return 0;
            }
            0
        }
        msg if msg == WM_APP_FOCUS_NEXT => {
            let is_shift = wparam != 0;
            let current_hwnd = lparam;
            let mut next_target = 0isize;
            if let Ok(guard) = state_arc.try_lock() {
                if !guard.inputs.is_empty() {
                    let cur_idx = guard.inputs.iter().position(|inp| inp.hwnd == current_hwnd);
                    if let Some(idx) = cur_idx {
                        let target_idx = if is_shift {
                            if idx == 0 {
                                guard.inputs.len() - 1
                            } else {
                                idx - 1
                            }
                        } else {
                            (idx + 1) % guard.inputs.len()
                        };
                        next_target = guard.inputs[target_idx].hwnd;
                    } else {
                        next_target = guard.inputs[0].hwnd;
                    }
                }
            }
            if next_target != 0 {
                unsafe {
                    SetFocus(next_target);
                }
            }
            0
        }
        WM_KEYDOWN => {
            if wparam == 0x1B
            /* VK_ESCAPE */
            {
                let mut closed = false;
                if let Ok(mut guard) = state_arc.try_lock() {
                    if guard.open_select.is_some() {
                        guard.open_select = None;
                        guard.update_input_scroll();
                        closed = true;
                    }
                }
                if closed {
                    unsafe {
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                    return 0;
                }
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
