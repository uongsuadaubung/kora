//! UI Framework Theme Engine, Font Cache & GDI Brush Cache
//!
//! Zero business-logic dependencies: manages Dark/Light state, cached GDI fonts,
//! cached surface brushes, and recursive child-window dark mode styling.

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::tokens::typography;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

static PANEL_BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static TAB_CARD_BRUSH: AtomicIsize = AtomicIsize::new(0);
static INPUT_BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static SIDEBAR_BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static CURRENT_IS_DARK: AtomicBool = AtomicBool::new(false);
static IS_APPLYING_THEME: AtomicBool = AtomicBool::new(false);

pub fn is_current_dark() -> bool {
    CURRENT_IS_DARK.load(Ordering::Relaxed)
}

/// Process-lifetime font cache keyed by (point size, weight, dpi).
pub fn cached_font_for_dpi(point_size: i32, weight: i32, dpi: i32) -> isize {
    static CACHE: Mutex<Vec<(i32, i32, i32, isize)>> = Mutex::new(Vec::new());
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(&(_, _, _, f)) = cache
        .iter()
        .find(|&&(s, w, d, _)| s == point_size && w == weight && d == dpi)
    {
        return f;
    }
    let f = create_app_font_for_dpi(point_size, weight, dpi);
    cache.push((point_size, weight, dpi, f));
    f
}

/// Process-lifetime font cache keyed by (point size, weight). Only a handful of sizes are used,
/// so fonts are created once and never freed: no per-paint CreateFontW/DeleteObject churn and
/// no per-dialog leaks.
pub fn cached_font(point_size: i32, weight: i32) -> isize {
    cached_font_for_dpi(point_size, weight, 96)
}

pub fn get_app_font_normal() -> isize {
    cached_font(typography::FONT_SIZE_BODY, FW_NORMAL)
}

pub fn get_app_font_bold() -> isize {
    cached_font(typography::FONT_SIZE_BODY, FW_SEMIBOLD)
}

pub fn get_panel_bg_brush() -> isize {
    PANEL_BG_BRUSH.load(Ordering::Relaxed)
}

pub fn get_tab_card_brush() -> isize {
    TAB_CARD_BRUSH.load(Ordering::Relaxed)
}

pub fn get_input_bg_brush() -> isize {
    let br = INPUT_BG_BRUSH.load(Ordering::Relaxed);
    if br != 0 {
        return br;
    }
    let palette = ThemePalette::get(is_current_dark());
    let new_br = unsafe { CreateSolidBrush(palette.bg_input) };
    if INPUT_BG_BRUSH
        .compare_exchange(0, new_br, Ordering::SeqCst, Ordering::Relaxed)
        .is_ok()
    {
        new_br
    } else {
        unsafe { DeleteObject(new_br) };
        INPUT_BG_BRUSH.load(Ordering::Relaxed)
    }
}

pub fn get_sidebar_bg_brush() -> isize {
    SIDEBAR_BG_BRUSH.load(Ordering::Relaxed)
}

/// Recreates cached solid GDI brushes for the given theme mode
pub fn recreate_theme_brushes(is_dark: bool) {
    CURRENT_IS_DARK.store(is_dark, Ordering::Relaxed);
    let palette = ThemePalette::get(is_dark);

    let old_panel = PANEL_BG_BRUSH.swap(
        unsafe { CreateSolidBrush(palette.bg_window) },
        Ordering::Relaxed,
    );
    if old_panel != 0 {
        unsafe { DeleteObject(old_panel) };
    }

    let old_sidebar = SIDEBAR_BG_BRUSH.swap(
        unsafe { CreateSolidBrush(palette.bg_sidebar) },
        Ordering::Relaxed,
    );
    if old_sidebar != 0 {
        unsafe { DeleteObject(old_sidebar) };
    }

    let old_tab = TAB_CARD_BRUSH.swap(
        unsafe { CreateSolidBrush(palette.bg_card) },
        Ordering::Relaxed,
    );
    if old_tab != 0 {
        unsafe { DeleteObject(old_tab) };
    }

    let old_input = INPUT_BG_BRUSH.swap(
        unsafe { CreateSolidBrush(palette.bg_input) },
        Ordering::Relaxed,
    );
    if old_input != 0 {
        unsafe { DeleteObject(old_input) };
    }
}

/// Win32 Child window enumeration callback: automatically themes Buttons, Checkboxes,
/// ComboBoxes, TextBoxes, Trackbars, and Labels without requiring manual handle tracking.
unsafe extern "system" fn enum_child_theme_proc(child: isize, lparam: isize) -> i32 {
    let is_dark = lparam != 0;
    allow_window_dark_mode(child, is_dark);

    unsafe {
        let mut class_buf = [0u16; 32];
        let len = GetClassNameW(child, class_buf.as_mut_ptr(), 32);
        let class_name = String::from_utf16_lossy(&class_buf[..len.max(0) as usize]);

        let theme_str = if is_dark {
            "DarkMode_Explorer"
        } else {
            "Explorer"
        };
        let theme_w = to_wide(theme_str);

        if class_name.eq_ignore_ascii_case("Button") {
            let style = GetWindowLongW(child, -16 /* GWL_STYLE */) as u32;
            let is_owner_draw = (style & 0x0000000F) == 0x0000000B; // BS_OWNERDRAW
            if is_owner_draw {
                let empty = to_wide(" ");
                SetWindowTheme(child, empty.as_ptr(), empty.as_ptr());
            } else {
                SetWindowTheme(child, theme_w.as_ptr(), null_mut());
            }
            InvalidateRect(child, null_mut(), 1);
        } else if class_name.eq_ignore_ascii_case("ComboBox")
            || class_name.eq_ignore_ascii_case("Edit")
        {
            let cfd_theme_str = if is_dark { "CFD" } else { "" };
            let theme_cfd_w = to_wide(cfd_theme_str);
            let res = SetWindowTheme(child, theme_cfd_w.as_ptr(), null_mut());
            if is_dark && res != 0 {
                SetWindowTheme(child, to_wide("DarkMode_CFD").as_ptr(), null_mut());
            }
            SetWindowPos(
                child,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        } else if class_name.eq_ignore_ascii_case("Static") {
            InvalidateRect(child, null_mut(), 1);
        } else if class_name.eq_ignore_ascii_case("msctls_trackbar32") {
            let res = SetWindowTheme(child, theme_w.as_ptr(), null_mut());
            if is_dark && res != 0 {
                SetWindowTheme(child, to_wide("DarkMode_CFD").as_ptr(), null_mut());
            }
            SetWindowPos(
                child,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
            InvalidateRect(child, null_mut(), 1);
            UpdateWindow(child);
        } else {
            SendMessageW(child, OCM_THEMECHANGED, is_dark as usize, 0);
            InvalidateRect(child, null_mut(), 0);
            UpdateWindow(child);
        }
    }

    1
}

/// Applies Light or Dark theme styling to any top-level window and all of its child controls.
/// Returns `true` if the theme was applied, or `false` if a theme application was already in progress.
pub fn apply_window_theme(hwnd: isize, is_dark: bool) -> bool {
    if IS_APPLYING_THEME
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        return false;
    }

    recreate_theme_brushes(is_dark);
    apply_modern_window_styling(hwnd, is_dark);
    set_preferred_app_mode(is_dark);
    allow_window_dark_mode(hwnd, is_dark);

    unsafe {
        EnumChildWindows(hwnd, Some(enum_child_theme_proc), is_dark as isize);
        RedrawWindow(
            hwnd,
            null_mut(),
            0,
            RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_FRAME,
        );
    }

    IS_APPLYING_THEME.store(false, Ordering::SeqCst);
    true
}
