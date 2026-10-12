//! Modern Windows 10/11 Window Styling, Theming & Window Registration
//!
//! Quản lý giao diện hiện đại:
//! - Bo góc chuẩn Windows 11 (`DWMWA_WINDOW_CORNER_PREFERENCE`).
//! - Chế độ nền tối Dark Mode cho tiêu đề và menu (`DWMWA_USE_IMMERSIVE_DARK_MODE`, `SetPreferredAppMode`).
//! - Căn giữa cửa sổ màn hình và đăng ký lớp cửa sổ singleton.

use super::ffi::*;
use super::types::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

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

/// Registers a window class once per process (subsequent calls are no-ops).
/// `registered` must be a dedicated `static` per class.
pub fn register_class_once(
    registered: &std::sync::Once,
    class_name: &str,
    wnd_proc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    style: u32,
) {
    registered.call_once(|| unsafe {
        let name = to_wide(class_name);
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = style;
        wc.lpfn_wnd_proc = Some(wnd_proc);
        wc.h_instance = 0;
        wc.h_cursor = LoadCursorW(0, IDC_ARROW);
        wc.lpsz_class_name = name.as_ptr();
        RegisterClassExW(&wc);
    });
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

