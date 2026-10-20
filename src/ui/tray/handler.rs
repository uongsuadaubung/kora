//! System Tray Handler & Popup Context Menu for Kora

use super::icon::TrayIcon;
use super::loader::{get_small_icon_size, load_icon_from_memory, safe_destroy_icon};
use crate::engine::config::InputMethod;
use crate::ui::components::PopupMenu;
use crate::ui::tokens::spacing;

pub const WM_TRAY_MESSAGE: u32 = 0x8001; // WM_USER + 1
pub const TRAY_ICON_ID: u32 = 1001;

// Menu Command IDs
pub const IDM_TELEX: u32 = 2002;
pub const IDM_VNI: u32 = 2003;
pub const IDM_SIMPLE_TELEX: u32 = 2004;
pub const IDM_CONTROL_PANEL: u32 = 2030;
pub const IDM_EXIT: u32 = 2099;

static VIET_BYTES: &[u8] = include_bytes!("../../../assets/vi.ico");
static ENG_BYTES: &[u8] = include_bytes!("../../../assets/en.ico");

pub struct TrayHandler {
    tray: TrayIcon,
    hicon_viet: isize,
    hicon_eng: isize,
    is_vietnamese: bool,
}

impl TrayHandler {
    pub fn new(hwnd: isize, is_vietnamese: bool) -> Self {
        let (w, h) = get_small_icon_size();
        let hicon_viet = load_icon_from_memory(VIET_BYTES, w, h).unwrap_or(0);
        let hicon_eng = load_icon_from_memory(ENG_BYTES, w, h).unwrap_or(0);

        let mut handler = Self {
            tray: TrayIcon::new(hwnd, TRAY_ICON_ID, WM_TRAY_MESSAGE),
            hicon_viet,
            hicon_eng,
            is_vietnamese,
        };

        handler.refresh_icon();
        handler.tray.show();
        handler
    }

    /// Re-applies icon + localized tooltip for the current mode (one Explorer round-trip).
    pub fn refresh_icon(&mut self) {
        let strings = crate::language::current();
        let (hicon, tip) = if self.is_vietnamese {
            (self.hicon_viet, strings.tray_tooltip_vi)
        } else {
            (self.hicon_eng, strings.tray_tooltip_en)
        };
        self.tray.update(hicon, tip);
    }

    pub fn set_vietnamese_mode(&mut self, enabled: bool) {
        if self.is_vietnamese != enabled {
            self.is_vietnamese = enabled;
            self.refresh_icon();
        }
    }

    /// Explorer restarted: the notification area lost our icon.
    pub fn readd(&mut self) {
        self.tray.readd();
    }

    pub fn hide(&mut self) {
        self.tray.hide();
    }
}

/// Builds and tracks the tray context menu. Takes plain values so callers don't hold any
/// lock while TrackPopupMenu runs its modal loop (the keyboard hook fires inside it).
pub fn show_context_menu(hwnd: isize, x: i32, y: i32, method: InputMethod) -> u32 {
    let mut menu = match PopupMenu::new() {
        Some(m) => m,
        None => return 0,
    };

    let strings = crate::language::current();

    // 1. Input Method Submenu ("Kiểu gõ: Telex ▸")
    if let Some(mut method_menu) = PopupMenu::new() {
        method_menu.add_checked_item(
            IDM_TELEX,
            strings.method_telex,
            method == InputMethod::Telex,
        );
        method_menu.add_checked_item(IDM_VNI, strings.method_vni, method == InputMethod::Vni);
        method_menu.add_checked_item(
            IDM_SIMPLE_TELEX,
            strings.method_simple_telex,
            matches!(
                method,
                InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2
            ),
        );
        let current_method = match method {
            InputMethod::Telex => strings.method_telex,
            InputMethod::Vni => strings.method_vni,
            InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2 => strings.method_simple_telex,
        };
        let label = format!("{} {}", strings.label_method, current_method);
        menu.add_submenu(&label, method_menu);
    }
    menu.add_separator();

    // 2. Control Panel & Exit
    menu.add_item(IDM_CONTROL_PANEL, strings.tray_control_panel);
    menu.add_separator();
    menu.add_item(IDM_EXIT, strings.tray_exit);

    menu.track(hwnd, x, y - spacing::SPACE_16)
}

impl Drop for TrayHandler {
    fn drop(&mut self) {
        self.tray.hide();
        safe_destroy_icon(self.hicon_viet);
        safe_destroy_icon(self.hicon_eng);
    }
}
