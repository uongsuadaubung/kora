//! Kora UI Theme Engine & Dark Mode Integration

use crate::engine::config::UiTheme;
use crate::ui::colors::ThemePalette;
use crate::ui::win32::*;
use crate::ui::views::ControlPanelControls;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

static PANEL_BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static TAB_CARD_BRUSH: AtomicIsize = AtomicIsize::new(0);
static INPUT_BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static BORDER_BRUSH: AtomicIsize = AtomicIsize::new(0);
static SIDEBAR_BG_BRUSH: AtomicIsize = AtomicIsize::new(0);
static CURRENT_IS_DARK: AtomicBool = AtomicBool::new(false);
static IS_APPLYING_THEME: AtomicBool = AtomicBool::new(false);

pub fn is_current_dark() -> bool {
    CURRENT_IS_DARK.load(Ordering::Relaxed)
}

static APP_FONT_NORMAL: AtomicIsize = AtomicIsize::new(0);
static APP_FONT_BOLD: AtomicIsize = AtomicIsize::new(0);

pub fn set_app_fonts(normal: isize, bold: isize) {
    APP_FONT_NORMAL.store(normal, Ordering::Relaxed);
    APP_FONT_BOLD.store(bold, Ordering::Relaxed);
}

pub fn get_app_font_normal() -> isize {
    APP_FONT_NORMAL.load(Ordering::Relaxed)
}

pub fn get_app_font_bold() -> isize {
    APP_FONT_BOLD.load(Ordering::Relaxed)
}

pub fn get_panel_bg_brush() -> isize {
    PANEL_BG_BRUSH.load(Ordering::Relaxed)
}

pub fn get_tab_card_brush() -> isize {
    TAB_CARD_BRUSH.load(Ordering::Relaxed)
}

pub fn get_input_bg_brush() -> isize {
    INPUT_BG_BRUSH.load(Ordering::Relaxed)
}

pub fn get_sidebar_bg_brush() -> isize {
    SIDEBAR_BG_BRUSH.load(Ordering::Relaxed)
}

/// Resolves whether dark mode should be active given the theme setting
pub fn resolve_is_dark(theme: UiTheme) -> bool {
    match theme {
        UiTheme::Auto => crate::ui::tray::is_windows_dark_taskbar(),
        UiTheme::Light => false,
        UiTheme::Dark => true,
    }
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

    let old_border = BORDER_BRUSH.swap(
        unsafe { CreateSolidBrush(palette.border) },
        Ordering::Relaxed,
    );
    if old_border != 0 {
        unsafe { DeleteObject(old_border) };
    }
}

/// Win32 Child window enumeration callback: automatically themes Buttons, Checkboxes,
/// ComboBoxes, TextBoxes, and Labels without requiring manual handle tracking.
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
        }
    }

    1
}

/// Applies cohesive, modern Light or Dark theme styling to all Control Panel elements
pub fn apply_ui_theme(hwnd: isize, controls: &ControlPanelControls, is_dark: bool) {
    if IS_APPLYING_THEME
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    recreate_theme_brushes(is_dark);

    // Windows 11 title bar & rounded corners
    apply_modern_window_styling(hwnd, is_dark);

    // Windows 10/11 uxtheme native dark mode
    set_preferred_app_mode(is_dark);
    allow_window_dark_mode(hwnd, is_dark);

    // Automatically enumerate and theme all child controls (Buttons, CheckBoxes, Combos, Edits, Labels)
    unsafe {
        EnumChildWindows(hwnd, Some(enum_child_theme_proc), is_dark as isize);

        // Redraw TabBar with updated colors
        InvalidateRect(controls.tab_bar.hwnd(), null_mut(), 1);
        UpdateWindow(controls.tab_bar.hwnd());

        // Theme ListViews
        controls.tab_macro.list_macro.apply_theme(is_dark);
        controls.tab_remap.list_remap.apply_theme(is_dark);
        controls.tab_remap.list_disable.apply_theme(is_dark);

        // Recursively invalidate and redraw entire dialog and all child controls cleanly
        RedrawWindow(
            hwnd,
            null_mut(),
            0,
            RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_FRAME,
        );
    }

    IS_APPLYING_THEME.store(false, Ordering::SeqCst);
}
