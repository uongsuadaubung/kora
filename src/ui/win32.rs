//! Native Win32 FFI Bindings, GDI Types, and Platform Helper Functions

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// Explicit type aliases for Win32 opaque handles
pub type HWND = isize;
pub type HDC = isize;
pub type HFONT = isize;
pub type HBRUSH = isize;
pub type HICON = isize;
pub type HCURSOR = isize;
pub type HMENU = isize;
pub type HHOOK = isize;
pub type HINSTANCE = isize;
pub type HMODULE = isize;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: Option<unsafe extern "system" fn(isize, u32, usize, isize) -> isize>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: isize,
    pub h_icon: isize,
    pub h_cursor: isize,
    pub h_br_background: isize,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: isize,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub struct NMHDR {
    pub hwnd_from: isize,
    pub id_from: usize,
    pub code: u32,
}

#[repr(C)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub struct NMCUSTOMDRAW {
    pub hdr: NMHDR,
    pub dw_draw_stage: u32,
    pub hdc: isize,
    pub rc: RECT,
    pub dw_item_spec: usize,
    pub u_item_state: u32,
    pub l_item_lparam: isize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct NMLVCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub clr_text: u32,
    pub clr_text_bk: u32,
    pub i_sub_item: i32,
    pub dw_item_type: u32,
    pub clr_face: u32,
    pub i_icon_effect: i32,
    pub i_icon_phase: i32,
    pub i_part_id: i32,
    pub i_state_id: i32,
    pub rc_text: RECT,
    pub u_align: u32,
}

pub const GWLP_WNDPROC: i32 = -4;
pub const GWL_STYLE: i32 = -16;

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
    pub fn CallWindowProcW(
        lp_prev_wnd_func: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
        h_wnd: isize,
        msg: u32,
        w_param: usize,
        l_param: isize,
    ) -> isize;
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
    pub fn SetScrollInfo(hwnd: isize, n_bar: i32, lpsi: *const SCROLLINFO, redraw: i32) -> i32;
    pub fn GetScrollInfo(hwnd: isize, n_bar: i32, lpsi: *mut SCROLLINFO) -> i32;
    pub fn SetCapture(h_wnd: isize) -> isize;
    pub fn ReleaseCapture() -> i32;
}

#[repr(C)]
pub struct SCROLLINFO {
    pub cb_size: u32,
    pub f_mask: u32,
    pub n_min: i32,
    pub n_max: i32,
    pub n_page: u32,
    pub n_pos: i32,
    pub n_track_pos: i32,
}
pub const SB_HORZ: i32 = 0;
pub const SB_VERT: i32 = 1;
pub const SB_CTL: i32 = 2;
pub const SB_BOTH: i32 = 3;

pub const SIF_RANGE: u32 = 0x0001;
pub const SIF_PAGE: u32 = 0x0002;
pub const SIF_POS: u32 = 0x0004;
pub const SIF_DISABLENOSCROLL: u32 = 0x0008;
pub const SIF_TRACKPOS: u32 = 0x0010;
pub const SIF_ALL: u32 = SIF_RANGE | SIF_PAGE | SIF_POS | SIF_TRACKPOS;

pub const SB_LINEUP: u32 = 0;
pub const SB_LINEDOWN: u32 = 1;
pub const SB_PAGEUP: u32 = 2;
pub const SB_PAGEDOWN: u32 = 3;
pub const SB_THUMBPOSITION: u32 = 4;
pub const SB_THUMBTRACK: u32 = 5;

pub const SRCCOPY: u32 = 0x00CC0020;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_VSCROLL: u32 = 0x0115;
pub const WM_HSCROLL: u32 = 0x0114;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const CS_VREDRAW: u32 = 0x0001;
pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_DBLCLKS: u32 = 0x0008;

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
pub struct TRACKMOUSEEVENT {
    pub cb_size: u32,
    pub dw_flags: u32,
    pub hwnd_track: isize,
    pub dw_hover_time: u32,
}
pub const TME_LEAVE: u32 = 0x00000002;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct DRAWITEMSTRUCT {
    pub ctl_type: u32,
    pub ctl_id: u32,
    pub item_id: u32,
    pub item_action: u32,
    pub item_state: u32,
    pub hwnd_item: isize,
    pub hdc: isize,
    pub rc_item: RECT,
    pub item_data: usize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct MEASUREITEMSTRUCT {
    pub ctl_type: u32,
    pub ctl_id: u32,
    pub item_id: u32,
    pub item_width: u32,
    pub item_height: u32,
    pub item_data: usize,
}

pub const GWLP_USERDATA: i32 = -21;

// Win32 Standard Message Reflection (OCM_*) Constants
pub const OCM_BASE: u32 = 0x2000;
pub const OCM_NOTIFY: u32 = OCM_BASE + 0x004E; // WM_NOTIFY
pub const OCM_COMMAND: u32 = OCM_BASE + 0x0111; // WM_COMMAND
pub const OCM_DRAWITEM: u32 = OCM_BASE + 0x002B; // WM_DRAWITEM
pub const OCM_MEASUREITEM: u32 = OCM_BASE + 0x002C; // WM_MEASUREITEM
pub const OCM_CTLCOLORSTATIC: u32 = OCM_BASE + 0x0138;
pub const OCM_CTLCOLOREDIT: u32 = OCM_BASE + 0x0133;
pub const OCM_CTLCOLORLISTBOX: u32 = OCM_BASE + 0x0134;

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
}

pub const RDW_INVALIDATE: u32 = 0x0001;
pub const RDW_ERASE: u32 = 0x0004;
pub const RDW_ALLCHILDREN: u32 = 0x0080;
pub const RDW_UPDATENOW: u32 = 0x0100;
pub const RDW_FRAME: u32 = 0x0400;

pub const SWP_FRAMECHANGED: u32 = 0x0020;
pub const SWP_NOZORDER: u32 = 0x0004;

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: isize,
    pub f_erase: i32,
    pub rc_paint: RECT,
    pub f_restore: i32,
    pub f_inc_update: i32,
    pub rgb_reserved: [u8; 32],
}

pub const DT_LEFT: u32 = 0x00000000;
pub const DT_CENTER: u32 = 0x00000001;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_WORDBREAK: u32 = 0x00000010;
pub const DT_SINGLELINE: u32 = 0x00000020;
pub const DT_NOPREFIX: u32 = 0x00000800;
pub const PS_SOLID: i32 = 0;
pub const PS_DASH: i32 = 1;
pub const PS_NULL: i32 = 5;

pub const IDC_ARROW: usize = 32512;
pub const IDC_IBEAM: usize = 32513;
pub const IDC_HAND: usize = 32649;

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
    pub fn OpenThemeData(hwnd: isize, psz_class_list: *const u16) -> isize;
    pub fn CloseThemeData(h_theme: isize) -> i32;
    pub fn DrawThemeBackground(
        h_theme: isize,
        hdc: isize,
        i_part_id: i32,
        i_state_id: i32,
        p_rect: *const RECT,
        p_clip_rect: *const RECT,
    ) -> i32;
}

pub const BP_CHECKBOX: i32 = 3;
pub const CBS_UNCHECKEDNORMAL: i32 = 1;
pub const CBS_UNCHECKEDHOT: i32 = 2;
pub const CBS_UNCHECKEDPRESSED: i32 = 3;
pub const CBS_UNCHECKEDDISABLED: i32 = 4;
pub const CBS_CHECKEDNORMAL: i32 = 5;
pub const CBS_CHECKEDHOT: i32 = 6;
pub const CBS_CHECKEDPRESSED: i32 = 7;
pub const CBS_CHECKEDDISABLED: i32 = 8;
pub const WS_DISABLED: u32 = 0x08000000;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_GETFONT: u32 = 0x0031;

/// Sets native Windows uxtheme preferred application mode (Windows 10 1903+ / Windows 11)
pub fn set_preferred_app_mode(is_dark: bool) {
    unsafe {
        let uxtheme = GetModuleHandleW(to_wide("uxtheme.dll").as_ptr());
        if uxtheme != 0 {
            // Ordinal 135: SetPreferredAppMode (0 = Default, 1 = AllowDark, 2 = ForceDark, 3 = ForceLight)
            let set_mode: Option<unsafe extern "system" fn(i32) -> i32> =
                std::mem::transmute(GetProcAddress(uxtheme, 135 as *const u8));
            if let Some(f) = set_mode {
                let mode = if is_dark {
                    2 /* ForceDark */
                } else {
                    3 /* ForceLight */
                };
                f(mode);
            }
            // Ordinal 136: FlushMenuThemes
            let flush: Option<unsafe extern "system" fn()> =
                std::mem::transmute(GetProcAddress(uxtheme, 136 as *const u8));
            if let Some(f) = flush {
                f();
            }
        }
    }
}

/// Allows dark mode for a specific HWND (Windows 10 1809+ / Windows 11)
pub fn allow_window_dark_mode(hwnd: isize, is_dark: bool) {
    unsafe {
        let uxtheme = GetModuleHandleW(to_wide("uxtheme.dll").as_ptr());
        if uxtheme != 0 {
            // Ordinal 133: AllowDarkModeForWindow (hwnd, bool)
            let allow_dark: Option<unsafe extern "system" fn(isize, bool) -> bool> =
                std::mem::transmute(GetProcAddress(uxtheme, 133 as *const u8));
            if let Some(f) = allow_dark {
                f(hwnd, is_dark);
            }
        }
    }
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

#[link(name = "comctl32")]
unsafe extern "system" {
    pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> i32;
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dw_size: u32,
    pub dw_icc: u32,
}

pub const ICC_STANDARD_CLASSES: u32 = 0x00004000;
pub const ICC_TAB_CLASSES: u32 = 0x00000008;

pub const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
pub const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
pub const DWMWCP_ROUND: u32 = 2; // Windows 11 rounded corners

pub const TRANSPARENT: i32 = 1;
pub const OPAQUE: i32 = 2;

/// Applies Windows 11 native rounded corners and dark/light title bar
pub fn apply_modern_window_styling(hwnd: isize, is_dark: bool) {
    unsafe {
        let corner_pref: u32 = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_pref as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );

        let dark_mode: u32 = if is_dark { 1 } else { 0 };
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

pub const ICC_LISTVIEW_CLASSES: u32 = 0x00000001;

/// Ensures ComCtl32 v6 visual styles engine is fully initialized
pub fn init_common_controls() {
    let icce = INITCOMMONCONTROLSEX {
        dw_size: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dw_icc: ICC_STANDARD_CLASSES | ICC_TAB_CLASSES | ICC_LISTVIEW_CLASSES,
    };
    unsafe {
        InitCommonControlsEx(&icce);
    }
}

pub const WS_OVERLAPPED: u32 = 0x00000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_MINIMIZEBOX: u32 = 0x00020000;
pub const WS_CLIPCHILDREN: u32 = 0x02000000;
pub const WS_TABSTOP: u32 = 0x00010000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_EX_DLGMODALFRAME: u32 = 0x00000001;
pub const WS_EX_CONTROLPARENT: u32 = 0x00010000;
pub const WS_EX_APPWINDOW: u32 = 0x00040000;

pub const WM_SETICON: u32 = 0x0080;
pub const ICON_SMALL: usize = 0;
pub const ICON_BIG: usize = 1;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;

pub const HWND_TOP: isize = 0;
pub const HWND_BOTTOM: isize = 1;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;

pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

pub const COLOR_WINDOW: isize = 5;
pub const COLOR_BTNFACE: isize = 15;

pub const CLEARTYPE_QUALITY: u32 = 5;
pub const DEFAULT_CHARSET: u32 = 1;
pub const FW_NORMAL: i32 = 400;
pub const FW_SEMIBOLD: i32 = 600;
pub const FW_BOLD: i32 = 700;

pub fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Creates modern Segoe UI font matching Windows 10/11 standards
pub fn create_app_font(point_size: i32, weight: i32) -> isize {
    let font_name = to_wide("Segoe UI");
    // Standard 96 DPI: 1 pt ≈ 1.33 px -> height = - MulDiv(point_size, 96, 72)
    let height = -((point_size * 96 + 36) / 72);
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            0,
            0,
            CLEARTYPE_QUALITY,
            0,
            font_name.as_ptr(),
        )
    }
}

pub fn center_window(hwnd: isize, width: i32, height: i32) {
    unsafe {
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;
        SetWindowPos(
            hwnd,
            0,
            x,
            y,
            width,
            height,
            0x0004 /* SWP_NOZORDER */ | 0x0010, /* SWP_NOACTIVATE */
        );
    }
}
