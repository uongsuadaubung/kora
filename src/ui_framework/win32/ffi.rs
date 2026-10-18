//! Win32 Native FFI Bindings for UI Subsystems
//!
//! Khai báo các hàm liên kết động từ các thư viện Windows:
//! - `user32.dll`: Quản lý cửa sổ, con trỏ, vẽ cơ bản, sự kiện chuột/phím, menu.
//! - `gdi32.dll`: Đồ họa GDI, DC, Pen, Brush, Font, RoundRect, Bitmap, BitBlt.
//! - `comctl32.dll`: Subclassing (SetWindowSubclass) và khởi tạo Common Controls v6.
//! - `kernel32.dll`: Module handle & Proc address.
//! - `uxtheme.dll`: Quản lý Visual Styles và Dark Mode.
//! - `dwmapi.dll`: Desktop Window Manager (Bo tròn góc Windows 11, Dark Title Bar).

use super::types::*;

#[link(name = "user32")]
#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn DefWindowProcW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    pub fn RegisterClassExW(lp_wcx: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: isize,
        h_menu: isize,
        h_instance: isize,
        lp_param: *mut std::ffi::c_void,
    ) -> isize;
    pub fn DestroyWindow(h_wnd: isize) -> i32;
    pub fn ShowWindow(h_wnd: isize, n_cmd_show: i32) -> i32;
    pub fn IsWindowVisible(h_wnd: isize) -> i32;
    pub fn EnableWindow(h_wnd: isize, b_enable: i32) -> i32;
    pub fn IsWindowEnabled(h_wnd: isize) -> i32;
    pub fn SetForegroundWindow(h_wnd: isize) -> i32;
    pub fn GetFocus() -> isize;
    pub fn SetFocus(h_wnd: isize) -> isize;
    pub fn LoadCursorW(h_instance: isize, lp_cursor_name: usize) -> isize;
    pub fn SetCursor(h_cursor: isize) -> isize;
    pub fn GetSystemMetrics(n_index: i32) -> i32;
    pub fn CreateFontW(
        c_height: i32,
        c_width: i32,
        c_escapement: i32,
        c_orientation: i32,
        c_weight: i32,
        b_italic: u32,
        b_underline: u32,
        b_strike_out: u32,
        i_char_set: u32,
        i_out_precision: u32,
        i_clip_precision: u32,
        i_quality: u32,
        i_pitch_and_family: u32,
        psz_face_name: *const u16,
    ) -> isize;
    pub fn DeleteObject(ho: isize) -> i32;
    pub fn AdjustWindowRect(lp_rect: *mut RECT, dw_style: u32, b_menu: i32) -> i32;
    pub fn SetWindowPos(
        h_wnd: isize,
        h_wnd_insert_after: isize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> i32;
    pub fn SetBkMode(hdc: isize, mode: i32) -> i32;
    pub fn SetTextColor(hdc: isize, color: u32) -> u32;
    pub fn CreateSolidBrush(cr_color: u32) -> isize;
    pub fn SendMessageW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    pub fn InvalidateRect(h_wnd: isize, lp_rect: *const RECT, b_erase: i32) -> i32;
    pub fn UpdateWindow(h_wnd: isize) -> i32;
    pub fn SetBkColor(hdc: isize, color: u32) -> u32;
    pub fn GetWindowRect(h_wnd: isize, lp_rect: *mut RECT) -> i32;
    pub fn GetClientRect(h_wnd: isize, lp_rect: *mut RECT) -> i32;
    pub fn FillRect(hdc: isize, lprc: *const RECT, hbr: isize) -> i32;
    pub fn FrameRect(hdc: isize, lprc: *const RECT, hbr: isize) -> i32;
    pub fn ScreenToClient(h_wnd: isize, lp_point: *mut POINT) -> i32;
    pub fn GetDC(h_wnd: isize) -> isize;
    pub fn ReleaseDC(h_wnd: isize, hdc: isize) -> i32;
    pub fn SetWindowLongPtrW(h_wnd: isize, n_index: i32, dw_new_long: isize) -> isize;
    pub fn GetWindowLongPtrW(h_wnd: isize, n_index: i32) -> isize;
    pub fn GetClassNameW(h_wnd: isize, lp_class_name: *mut u16, n_max_count: i32) -> i32;
    pub fn CreatePen(i_style: i32, c_width: i32, color: u32) -> isize;
    pub fn SelectObject(hdc: isize, h: isize) -> isize;
    pub fn RoundRect(
        hdc: isize,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> i32;
    pub fn Ellipse(hdc: isize, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    pub fn MoveToEx(hdc: isize, x: i32, y: i32, lppt: *mut POINT) -> i32;
    pub fn LineTo(hdc: isize, x: i32, y: i32) -> i32;
    pub fn GetWindowTextW(h_wnd: isize, lp_string: *mut u16, n_max_count: i32) -> i32;
    pub fn GetWindowTextLengthW(h_wnd: isize) -> i32;
    pub fn DrawTextW(
        hdc: isize,
        lpch_text: *const u16,
        cch_text: i32,
        lprc: *mut RECT,
        format: u32,
    ) -> i32;
    pub fn GetTextExtentPoint32W(
        hdc: isize,
        lp_string: *const u16,
        c: i32,
        psizl: *mut SIZE,
    ) -> i32;
    pub fn BeginPaint(h_wnd: isize, lp_paint: *mut PAINTSTRUCT) -> isize;
    pub fn EndPaint(h_wnd: isize, lp_paint: *const PAINTSTRUCT) -> i32;
    pub fn RedrawWindow(
        h_wnd: isize,
        lprc_update: *const RECT,
        hrgn_update: isize,
        flags: u32,
    ) -> i32;
    pub fn SetWindowTextW(h_wnd: isize, lp_string: *const u16) -> i32;
    pub fn GetWindowLongW(h_wnd: isize, n_index: i32) -> i32;
    pub fn GetDlgCtrlID(h_wnd: isize) -> i32;
    pub fn GetDlgItem(h_dummy: isize, n_id_dlg_item: i32) -> isize;
    pub fn PostMessageW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> i32;
    pub fn EnumChildWindows(
        h_wnd_parent: isize,
        lp_enum_func: Option<unsafe extern "system" fn(isize, isize) -> i32>,
        l_param: isize,
    ) -> i32;
    pub fn GetWindow(h_wnd: isize, u_cmd: u32) -> isize;
    pub fn GetCursorPos(lp_point: *mut POINT) -> i32;
    pub fn WindowFromPoint(point: POINT) -> isize;
    pub fn TrackMouseEvent(lp_event_track: *mut TRACKMOUSEEVENT) -> i32;
    pub fn PostQuitMessage(n_exit_code: i32);
    pub fn CreateCompatibleDC(hdc: isize) -> isize;
    pub fn CreateCompatibleBitmap(hdc: isize, cx: i32, cy: i32) -> isize;
    pub fn DeleteDC(hdc: isize) -> i32;
    pub fn BitBlt(
        hdc_dest: isize,
        x_dest: i32,
        y_dest: i32,
        w: i32,
        h: i32,
        hdc_src: isize,
        x_src: i32,
        y_src: i32,
        rop: u32,
    ) -> i32;
    pub fn SetCapture(h_wnd: isize) -> isize;
    pub fn ReleaseCapture() -> i32;
    pub fn SetTimer(
        h_wnd: isize,
        n_id_event: usize,
        u_elapse: u32,
        lp_timer_func: Option<unsafe extern "system" fn(isize, u32, usize, u32)>,
    ) -> usize;
    pub fn KillTimer(h_wnd: isize, u_id_event: usize) -> i32;
    pub fn IsDialogMessageW(h_dlg: isize, lp_msg: *const std::ffi::c_void) -> i32;
    pub fn GetActiveWindow() -> isize;
    pub fn GetParent(h_wnd: isize) -> isize;
    pub fn SetThreadDpiAwarenessContext(dpi_context: isize) -> isize;
}

#[link(name = "comctl32")]
unsafe extern "system" {
    pub fn SetWindowSubclass(
        h_wnd: isize,
        pfn_subclass: unsafe extern "system" fn(isize, u32, usize, isize, usize, usize) -> isize,
        u_id_subclass: usize,
        dw_ref_data: usize,
    ) -> i32;
    pub fn DefSubclassProc(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    pub fn RemoveWindowSubclass(
        h_wnd: isize,
        pfn_subclass: unsafe extern "system" fn(isize, u32, usize, isize, usize, usize) -> isize,
        u_id_subclass: usize,
    ) -> i32;
    pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lp_module_name: *const u16) -> isize;
    pub fn GetProcAddress(h_module: isize, lp_proc_name: *const u8) -> *const ();
}

#[link(name = "uxtheme")]
unsafe extern "system" {
    pub fn SetWindowTheme(
        hwnd: isize,
        psz_sub_app_name: *const u16,
        psz_sub_id_list: *const u16,
    ) -> i32;
}

#[link(name = "dwmapi")]
unsafe extern "system" {
    pub fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const std::ffi::c_void,
        cb_attribute: u32,
    ) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn GetStockObject(i: i32) -> isize;
    pub fn SetDCBrushColor(hdc: isize, color: u32) -> u32;
    pub fn SetDCPenColor(hdc: isize, color: u32) -> u32;
}

