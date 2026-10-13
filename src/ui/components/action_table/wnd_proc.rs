//! ActionTable Win32 Window Procedure & Event Handlers
//!
//! Tiếp nhận và xử lý toàn bộ các thông điệp sự kiện người dùng từ Windows:
//! - Khởi tạo & Dọn dẹp bộ nhớ Arc/raw pointer (`WM_CREATE`, `WM_DESTROY`).
//! - Thay đổi kích thước (`WM_SIZE`), Cuộn chuột (`WM_MOUSEWHEEL`).
//! - Di chuột & Đổi con trỏ bàn tay (`WM_MOUSEMOVE`, `WM_SETCURSOR`, `WM_MOUSELEAVE`).
//! - Tương tác click chuột (`WM_LBUTTONDOWN`, `WM_LBUTTONUP`, `WM_LBUTTONDBLCLK`).
//! - Vẽ lại giao diện (`WM_PAINT`).

use super::render::*;
use super::types::*;
use crate::ui::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_SIZE: u32 = 0x0005;
const WM_PAINT: u32 = 0x000F;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SETCURSOR: u32 = 0x0020;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_MOUSELEAVE: u32 = 0x02A3;

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn action_table_wndproc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<ActionTableState> };

    if msg == WM_CREATE {
        return 0;
    }

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
    let state_mutex = unsafe { &*state_ptr };
    // try_lock: some calls made while holding the lock (e.g. ReleaseCapture) re-enter this proc synchronously.
    let header_h = state_mutex.try_lock().map(|g| g.header_h).unwrap_or(HEADER_HEIGHT);

    if msg == WM_ERASEBKGND {
        return 1;
    }

    if msg == WM_SIZE {
        let h = ((lparam >> 16) & 0xFFFF) as i32;
        if let Ok(mut g) = state_mutex.lock() {
            let body_h = (h - header_h).max(0);
            let total_h = g.rows.len() as i32 * ROW_HEIGHT;
            let max_scroll = (total_h - body_h).max(0);
            if g.scroll_y > max_scroll {
                g.scroll_y = max_scroll;
            }
        }
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }
        return 0;
    }

    if msg == WM_MOUSEWHEEL {
        let delta = ((wparam >> 16) as i16) as i32;
        let mut client_rc = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rc); }
        let body_h = (client_rc.bottom - header_h).max(0);

        let changed = if let Ok(mut g) = state_mutex.lock() {
            let total_h = g.rows.len() as i32 * ROW_HEIGHT;
            let max_scroll = (total_h - body_h).max(0);
            let scroll_lines = (delta / 120) * (ROW_HEIGHT * 3);
            let old_scroll = g.scroll_y;
            g.scroll_y = (g.scroll_y - scroll_lines).clamp(0, max_scroll);
            old_scroll != g.scroll_y
        } else {
            false
        };

        if changed {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
        return 0;
    }

    if msg == WM_MOUSEMOVE {
        let x = (lparam as i16) as i32;
        let y = ((lparam >> 16) as i16) as i32;

        let mut client_rc = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rc); }
        let width = client_rc.right - client_rc.left;
        let height = client_rc.bottom - client_rc.top;
        let body_h = (height - header_h).max(0);

        let (need_redraw, is_hand_cursor, dirty_row1, dirty_row2, sb_changed, is_drag) = if let Ok(mut g) = state_mutex.lock() {
            if g.is_dragging_scroll {
                let total_h = g.rows.len() as i32 * ROW_HEIGHT;
                let max_scroll = (total_h - body_h).max(0);
                let changed = if max_scroll > 0 {
                    let track_top = header_h + 3;
                    let track_bottom = height - 3;
                    let track_h = (track_bottom - track_top).max(10);
                    let thumb_h = ((body_h as f32 / total_h as f32) * track_h as f32).clamp(18.0, track_h as f32) as i32;
                    let travel = (track_h - thumb_h).max(1);
                    let delta_y = y - g.scroll_drag_start_y;
                    let scroll_delta = ((delta_y as f32 / travel as f32) * max_scroll as f32) as i32;
                    let new_scroll = (g.scroll_drag_start_val + scroll_delta).clamp(0, max_scroll);
                    let changed = g.scroll_y != new_scroll;
                    g.scroll_y = new_scroll;
                    changed
                } else {
                    false
                };
                (changed, false, None, None, false, true)
            } else {
                let old_hot_row = g.hover_row;
                let old_action = g.hover_action;
                let old_hover_scroll = g.is_hover_scrollbar;

                let total_h = g.rows.len() as i32 * ROW_HEIGHT;
                let has_scroll = total_h > body_h;
                let hover_scrollbar = has_scroll && x >= width - 14 && y >= header_h;
                g.is_hover_scrollbar = hover_scrollbar;

                if !hover_scrollbar && y >= header_h && y < client_rc.bottom {
                    let row_idx = ((y - header_h + g.scroll_y) / ROW_HEIGHT) as usize;
                    if row_idx < g.rows.len() {
                        g.hover_row = Some(row_idx);
                        let mut action_hover = ActionHover::None;

                        let col_rects = compute_column_rects(&g.columns, width);
                        let row_y = header_h + (row_idx as i32 * ROW_HEIGHT) - g.scroll_y;

                        if let Some(row_cells) = g.rows.get(row_idx) {
                            for (col_idx, &(col_x, col_w)) in col_rects.iter().enumerate() {
                                if x >= col_x && x < col_x + col_w {
                                    if let Some(TableCell::Actions(actions)) = row_cells.get(col_idx) {
                                        let cell_rc = RECT {
                                            left: col_x,
                                            top: row_y,
                                            right: col_x + col_w,
                                            bottom: row_y + ROW_HEIGHT - 1,
                                        };
                                        let btn_rects = get_dynamic_action_button_rects(&cell_rc, actions);
                                        for (act_idx, btn_rc) in btn_rects.iter().enumerate() {
                                            if pt_in_rect(x, y, btn_rc) {
                                                action_hover = ActionHover::Action { row: row_idx, col: col_idx, action_idx: act_idx };
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        g.hover_action = action_hover;
                    } else {
                        g.hover_row = None;
                        g.hover_action = ActionHover::None;
                    }
                } else {
                    g.hover_row = None;
                    g.hover_action = ActionHover::None;
                }

                let row_changed = old_hot_row != g.hover_row || old_action != g.hover_action;
                let sb_changed = old_hover_scroll != g.is_hover_scrollbar;
                let hand = g.hover_action != ActionHover::None;
                (row_changed || sb_changed, hand, old_hot_row, g.hover_row, sb_changed, false)
            }
        } else {
            (false, false, None, None, false, false)
        };

        if need_redraw {
            unsafe {
                if is_drag {
                    InvalidateRect(hwnd, null_mut(), 0);
                } else {
                    let scroll_y = state_mutex.lock().map(|g| g.scroll_y).unwrap_or(0);
                    let invalidate_row = |r_opt: Option<usize>| {
                        if let Some(r) = r_opt {
                            let ry = header_h + (r as i32 * ROW_HEIGHT) - scroll_y;
                            if ry < height && ry + ROW_HEIGHT > header_h {
                                let r_rc = RECT {
                                    left: 0,
                                    top: ry.max(header_h),
                                    right: width,
                                    bottom: (ry + ROW_HEIGHT).min(height),
                                };
                                InvalidateRect(hwnd, &r_rc, 0);
                            }
                        }
                    };
                    invalidate_row(dirty_row1);
                    if dirty_row1 != dirty_row2 {
                        invalidate_row(dirty_row2);
                    }
                    if sb_changed {
                        let sb_rc = RECT {
                            left: width - 14,
                            top: header_h,
                            right: width,
                            bottom: height,
                        };
                        InvalidateRect(hwnd, &sb_rc, 0);
                    }
                }
            }
        }

        unsafe {
            if is_hand_cursor {
                SetCursor(LoadCursorW(0, IDC_HAND_ID));
            } else {
                SetCursor(LoadCursorW(0, IDC_ARROW_ID));
            }
            let mut tme = TRACKMOUSEEVENT {
                cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dw_flags: TME_LEAVE,
                hwnd_track: hwnd,
                dw_hover_time: 0,
            };
            TrackMouseEvent(&mut tme);
        }
        return 0;
    }

    if msg == WM_SETCURSOR
        && let Ok(g) = state_mutex.lock()
        && g.hover_action != ActionHover::None
    {
        unsafe {
            SetCursor(LoadCursorW(0, IDC_HAND_ID));
        }
        return 1;
    }

    if msg == WM_MOUSELEAVE {
        let changed = if let Ok(mut g) = state_mutex.lock() {
            let was = g.hover_row.is_some() || g.hover_action != ActionHover::None || g.is_hover_scrollbar;
            g.hover_row = None;
            g.hover_action = ActionHover::None;
            g.is_hover_scrollbar = false;
            was
        } else {
            false
        };
        if changed {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
        return 0;
    }

    if msg == WM_LBUTTONDOWN {
        let x = (lparam as i16) as i32;
        let y = ((lparam >> 16) as i16) as i32;

        let mut client_rc = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rc); }
        let width = client_rc.right - client_rc.left;
        let height = client_rc.bottom - client_rc.top;
        let body_h = (height - header_h).max(0);

        let (action, hot_row, cb_sel, clicked_scroll) = if let Ok(mut g) = state_mutex.lock() {
            let total_rows = g.rows.len();
            let total_h = total_rows as i32 * ROW_HEIGHT;
            let has_scroll = total_h > body_h;

            if has_scroll && x >= width - 14 && y >= header_h {
                if let Some((thumb_rc, max_scroll)) = compute_scroll_thumb_rect(width, height, header_h, total_rows, g.scroll_y, true) {
                    if pt_in_rect(x, y, &thumb_rc) {
                        g.is_dragging_scroll = true;
                        g.scroll_drag_start_y = y;
                        g.scroll_drag_start_val = g.scroll_y;
                        unsafe { SetCapture(hwnd); }
                    } else if y < thumb_rc.top {
                        g.scroll_y = (g.scroll_y - body_h).max(0);
                    } else if y > thumb_rc.bottom {
                        g.scroll_y = (g.scroll_y + body_h).min(max_scroll);
                    }
                }
                (ActionHover::None, None, None, true)
            } else {
                let mut act = g.hover_action;
                let mut hr = g.hover_row;
                if act == ActionHover::None && y >= header_h && y < client_rc.bottom {
                    let row_idx = ((y - header_h + g.scroll_y) / ROW_HEIGHT) as usize;
                    if row_idx < g.rows.len() {
                        hr = Some(row_idx);
                        let col_rects = compute_column_rects(&g.columns, width);
                        let row_y = header_h + (row_idx as i32 * ROW_HEIGHT) - g.scroll_y;
                        if let Some(row_cells) = g.rows.get(row_idx) {
                            for (col_idx, &(col_x, col_w)) in col_rects.iter().enumerate() {
                                if x >= col_x && x < col_x + col_w {
                                    if let Some(TableCell::Actions(actions)) = row_cells.get(col_idx) {
                                        let cell_rc = RECT {
                                            left: col_x,
                                            top: row_y,
                                            right: col_x + col_w,
                                            bottom: row_y + ROW_HEIGHT - 1,
                                        };
                                        let btn_rects = get_dynamic_action_button_rects(&cell_rc, actions);
                                        for (act_idx, btn_rc) in btn_rects.iter().enumerate() {
                                            if pt_in_rect(x, y, btn_rc) {
                                                act = ActionHover::Action { row: row_idx, col: col_idx, action_idx: act_idx };
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                if hr.is_some() {
                    g.selected_row = hr;
                }
                (act, hr, g.on_select.clone(), false)
            }
        } else {
            (ActionHover::None, None, None, false)
        };

        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }

        if clicked_scroll {
            return 0;
        }

        match action {
            ActionHover::Action { row, col, action_idx } => {
                let cb = if let Ok(g) = state_mutex.lock() {
                    if let Some(row_cells) = g.rows.get(row)
                        && let Some(TableCell::Actions(actions)) = row_cells.get(col)
                        && let Some(act) = actions.get(action_idx)
                    {
                        act.on_click.clone()
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(f) = cb {
                    f();
                }
                return 0;
            }
            ActionHover::None => {
                if let (Some(row), Some(cb)) = (hot_row, cb_sel) {
                    cb(row as i32);
                }
            }
        }
        return 0;
    }

    if msg == WM_LBUTTONUP {
        let changed = if let Ok(mut g) = state_mutex.lock() {
            if g.is_dragging_scroll {
                g.is_dragging_scroll = false;
                unsafe { ReleaseCapture(); }
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            unsafe { InvalidateRect(hwnd, null_mut(), 0); }
        }
        return 0;
    }

    if msg == WM_LBUTTONDBLCLK {
        let cb = if let Ok(g) = state_mutex.lock() {
            if matches!(g.hover_action, ActionHover::Action { .. }) {
                None
            } else if let Some(row_idx) = g.hover_row {
                if let Some(row_cells) = g.rows.get(row_idx) {
                    row_cells.iter().find_map(|cell| {
                        if let TableCell::Actions(actions) = cell {
                            actions.iter().find_map(|act| {
                                if act.variant == ActionVariant::Primary {
                                    act.on_click.clone()
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        if let Some(f) = cb {
            f();
        }
        return 0;
    }

    if msg == WM_PAINT {
        let mut ps = PAINTSTRUCT {
            hdc: 0,
            f_erase: 0,
            rc_paint: RECT::default(),
            f_restore: 0,
            f_inc_update: 0,
            rgb_reserved: [0; 32],
        };
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        if hdc != 0 {
            if let Ok(g) = state_mutex.lock() {
                render_table(hwnd, hdc, &g);
            }
            unsafe { EndPaint(hwnd, &ps) };
        }
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

