//! Control Colors (`WM_CTLCOLOR*`) and Cursor (`WM_SETCURSOR`) Framework Handlers
//!
//! Tự động giải mã metadata bit-packed trên `GWLP_USERDATA` của `Label` và tên lớp Win32
//! để cấp màu chữ, brush nền và con trỏ chuột `IDC_HAND` / `IDC_IBEAM` cho cả cửa sổ chính lẫn các hộp thoại.

use super::ffi::*;
use super::types::*;
use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::components::{Label, LabelTone, SurfaceRole};
use crate::ui_framework::theme::{
    get_input_bg_brush, get_panel_bg_brush, get_sidebar_bg_brush, get_tab_card_brush,
    is_current_dark,
};

pub fn handle_ctlcolor_edit_listbox(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    let hdc = wparam as isize;
    let hwnd_ctrl = lparam;
    unsafe {
        let palette = ThemePalette::get(is_current_dark());
        let mut class_buf = [0u16; 32];
        let len = GetClassNameW(hwnd_ctrl, class_buf.as_mut_ptr(), 32);
        let class_slice = if len > 0 {
            &class_buf[..len as usize]
        } else {
            &[]
        };

        if utf16_str_eq_ignore_case(class_slice, "listbox") {
            SetTextColor(hdc, palette.text_primary);
            SetBkColor(hdc, palette.bg_sidebar);
            let sb_br = get_sidebar_bg_brush();
            if sb_br != 0 {
                return sb_br;
            }
        } else {
            SetTextColor(hdc, palette.text_input);
            SetBkColor(hdc, palette.bg_input);
            let input_br = get_input_bg_brush();
            if input_br != 0 {
                return input_br;
            }
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub fn handle_ctlcolor_static(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    let hdc = wparam as isize;
    let hwnd_ctrl = lparam;
    unsafe {
        let palette = ThemePalette::get(is_current_dark());
        let mut class_buf = [0u16; 32];
        let len = GetClassNameW(hwnd_ctrl, class_buf.as_mut_ptr(), 32);
        let class_slice = if len > 0 {
            &class_buf[..len as usize]
        } else {
            &[]
        };

        if utf16_str_eq_ignore_case(class_slice, "combobox")
            || utf16_str_eq_ignore_case(class_slice, "edit")
        {
            SetTextColor(hdc, palette.text_input);
            SetBkColor(hdc, palette.bg_input);
            let input_br = get_input_bg_brush();
            if input_br != 0 {
                return input_br;
            }
        } else {
            if utf16_str_eq_ignore_case(class_slice, "msctls_trackbar32") {
                SetBkMode(hdc, OPAQUE);
            } else {
                SetBkMode(hdc, TRANSPARENT);
            }

            let (tone, surface, _) = Label::style_of(hwnd_ctrl).unwrap_or((
                LabelTone::Primary,
                SurfaceRole::Card,
                false,
            ));

            let text_color = match tone {
                LabelTone::Primary => palette.text_primary,
                LabelTone::Secondary => palette.text_secondary,
                LabelTone::Accent => palette.accent,
                LabelTone::Link => palette.text_link,
            };
            let (bg_color, brush) = match surface {
                SurfaceRole::Card => (palette.bg_card, get_tab_card_brush()),
                SurfaceRole::Window => (palette.bg_window, get_panel_bg_brush()),
                SurfaceRole::Sidebar => (palette.bg_sidebar, get_sidebar_bg_brush()),
                SurfaceRole::Input => (palette.bg_input, get_input_bg_brush()),
            };

            SetTextColor(hdc, text_color);
            SetBkColor(hdc, bg_color);
            if brush != 0 {
                return brush;
            }
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub fn handle_setcursor(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    let child_hwnd = wparam as isize;
    if Label::is_link_hwnd(child_hwnd) {
        unsafe {
            SetCursor(LoadCursorW(0, IDC_HAND));
        }
        1
    } else {
        let mut class_buf = [0u16; 16];
        let len = unsafe { GetClassNameW(child_hwnd, class_buf.as_mut_ptr(), 16) };
        let class_slice = if len > 0 {
            &class_buf[..len as usize]
        } else {
            &[]
        };
        if utf16_str_eq_ignore_case(class_slice, "Edit") {
            unsafe {
                SetCursor(LoadCursorW(0, IDC_IBEAM));
            }
            1
        } else if utf16_str_eq_ignore_case(class_slice, "Button") {
            let is_enabled = unsafe { IsWindowEnabled(child_hwnd) } != 0;
            if is_enabled {
                unsafe {
                    SetCursor(LoadCursorW(0, IDC_HAND));
                }
                1
            } else {
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        } else {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }
}

fn utf16_str_eq_ignore_case(slice: &[u16], ascii_str: &str) -> bool {
    if slice.len() != ascii_str.len() {
        return false;
    }
    slice
        .iter()
        .zip(ascii_str.bytes())
        .all(|(&u, b)| (u as u8).eq_ignore_ascii_case(&b))
}
