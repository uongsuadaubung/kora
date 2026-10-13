//! Modal Dialog Base Scaffold & Shared Visual Components
//!
//! Cung cấp các tiện ích dùng chung cho các hộp thoại Modal trong Kora:
//! - Khởi tạo cửa sổ căn giữa và quản lý trạng thái modal (`create_modal_dialog`, `cleanup_modal_dialog`).
//! - Tạo font kích thước lớn cho ô phím bấm (`create_display_font`).
//! - Renderer chuẩn hóa vẽ khung phím tương tác (`draw_key_box`).

use crate::ui::colors::ThemePalette;
use crate::ui::tokens::{borders, radii};
use crate::ui::win32::*;
use std::ptr::null_mut;

/// Tạo font Segoe UI 20px Bold dùng riêng cho các ô hiển thị phím bấm lớn
pub fn create_display_font() -> isize {
    unsafe {
        CreateFontW(
            20,
            0,
            0,
            0,
            700,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            to_wide("Segoe UI").as_ptr(),
        )
    }
}

/// Khởi tạo và hiển thị một cửa sổ hộp thoại Modal chuẩn Win32 căn giữa cửa sổ cha
pub fn create_modal_dialog(
    parent: isize,
    title: &str,
    client_width: i32,
    client_height: i32,
    class_name: &str,
    wnd_proc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    registered: &std::sync::Once,
) -> isize {
    register_class_once(registered, class_name, wnd_proc, 0);

    unsafe {
        let class_name_w = to_wide(class_name);
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: client_width,
            bottom: client_height,
        };
        let style = 0x00080000 | 0x10000000 | 0x00C00000 | 0x02000000; // WS_SYSMENU | WS_VISIBLE | WS_CAPTION | WS_CLIPCHILDREN
        let ex_style = 0x00010000 | 0x00000001; // WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME
        AdjustWindowRect(&mut rc, style, 0);

        let width = rc.right - rc.left;
        let height = rc.bottom - rc.top;

        let mut parent_rc: RECT = std::mem::zeroed();
        GetWindowRect(parent, &mut parent_rc);
        let x = parent_rc.left + ((parent_rc.right - parent_rc.left) - width) / 2;
        let y = parent_rc.top + ((parent_rc.bottom - parent_rc.top) - height) / 2;

        let title_w = to_wide(title);

        EnableWindow(parent, 0);

        let hwnd = CreateWindowExW(
            ex_style,
            class_name_w.as_ptr(),
            title_w.as_ptr(),
            style,
            x,
            y,
            width,
            height,
            parent,
            0,
            0,
            null_mut(),
        );

        if hwnd != 0 {
            apply_modern_window_styling(hwnd, crate::ui::theme::is_current_dark());
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        } else {
            EnableWindow(parent, 1);
        }

        hwnd
    }
}

/// Dọn dẹp tài nguyên và hoàn trả quyền điều khiển về cửa sổ cha khi hộp thoại đóng
pub fn cleanup_modal_dialog(parent: isize, font: isize) {
    unsafe {
        if font != 0 {
            DeleteObject(font);
        }
        if parent != 0 {
            EnableWindow(parent, 1);
            SetForegroundWindow(parent);
        }
    }
}

/// Vẽ ô hiển thị phím bấm tương tác chuẩn hóa (KeyBox) dùng chung cho các Dialogs
pub fn draw_key_box(
    dis: &DRAWITEMSTRUCT,
    is_dark: bool,
    is_listening: bool,
    text: &str,
    is_placeholder: bool,
    is_danger: bool,
    font: isize,
) {
    let palette = ThemePalette::get(is_dark);
    unsafe {
        // 1. Xóa nền góc ngoài để bo tròn mượt mà, không bị vệt trắng
        let br_bg = CreateSolidBrush(palette.bg_window);
        FillRect(dis.hdc, &dis.rc_item, br_bg);
        DeleteObject(br_bg);

        // 2. Màu nền và viền hộp phím
        let (bg_color, border_color) = if is_danger {
            let bg = if is_listening { palette.danger_bg } else { palette.bg_input };
            (bg, palette.danger_border)
        } else {
            let bg = if is_listening {
                if is_dark { 0x003A2810 } else { 0x00F8EFEA }
            } else {
                palette.bg_input
            };
            (bg, palette.accent)
        };

        let brush = CreateSolidBrush(bg_color);
        let pen = CreatePen(PS_SOLID, borders::WIDTH_FOCUS, border_color);
        let prev_brush = SelectObject(dis.hdc, brush);
        let prev_pen = SelectObject(dis.hdc, pen);

        RoundRect(
            dis.hdc,
            dis.rc_item.left,
            dis.rc_item.top,
            dis.rc_item.right,
            dis.rc_item.bottom,
            radii::RADIUS_MD,
            radii::RADIUS_MD,
        );

        SelectObject(dis.hdc, prev_brush);
        SelectObject(dis.hdc, prev_pen);
        DeleteObject(brush);
        DeleteObject(pen);

        // 3. Hiển thị chữ
        SetBkMode(dis.hdc, TRANSPARENT);
        let text_color = if is_placeholder {
            palette.text_secondary
        } else if is_danger {
            palette.danger_border
        } else {
            palette.accent
        };
        SetTextColor(dis.hdc, text_color);

        let prev_font = if font != 0 {
            SelectObject(dis.hdc, font)
        } else {
            0
        };

        let wide_text = to_wide(text);
        let mut rc = dis.rc_item;
        DrawTextW(
            dis.hdc,
            wide_text.as_ptr(),
            -1,
            &mut rc,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        if prev_font != 0 {
            SelectObject(dis.hdc, prev_font);
        }
    }
}
