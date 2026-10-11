//! System Tray Handler & Popup Context Menu for Kora

use crate::engine::{EngineConfig, config::InputMethod};
use super::icon::TrayIcon;
use super::loader::{get_small_icon_size, load_icon_from_memory, safe_destroy_icon};
use crate::ui::components::PopupMenu;

pub const WM_TRAY_MESSAGE: u32 = 0x8001; // WM_USER + 1
pub const TRAY_ICON_ID: u32 = 1001;

// Menu Command IDs
pub const IDM_TOGGLE_VIET: u32 = 2001;
pub const IDM_TELEX: u32 = 2002;
pub const IDM_VNI: u32 = 2003;
pub const IDM_SIMPLE_TELEX: u32 = 2004;

pub const IDM_UNICODE: u32 = 2010;
pub const IDM_TCVN3: u32 = 2011;
pub const IDM_VNI_WIN: u32 = 2012;
pub const IDM_UNICODE_COMPOUND: u32 = 2013;

pub const IDM_SPELLING: u32 = 2020;
pub const IDM_USE_MACRO: u32 = 2021;

pub const IDM_CONTROL_PANEL: u32 = 2030;
pub const IDM_ABOUT: u32 = 2031;
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
    pub fn new(hwnd: isize) -> Self {
        let (w, h) = get_small_icon_size();
        let hicon_viet = load_icon_from_memory(VIET_BYTES, w, h).unwrap_or(0);
        let hicon_eng = load_icon_from_memory(ENG_BYTES, w, h).unwrap_or(0);

        let mut handler = Self {
            tray: TrayIcon::new(hwnd, TRAY_ICON_ID, WM_TRAY_MESSAGE),
            hicon_viet,
            hicon_eng,
            is_vietnamese: true,
        };

        handler.refresh_icon();
        handler.tray.show();
        handler
    }

    pub fn is_vietnamese(&self) -> bool {
        self.is_vietnamese
    }

    /// Dynamically updates the tray icon depending on Vietnamese mode (V vs E)
    pub fn refresh_icon(&mut self) {
        let hicon = if self.is_vietnamese {
            self.hicon_viet
        } else {
            self.hicon_eng
        };

        if hicon != 0 {
            self.tray.set_icon(hicon);
        }
        let strings = crate::language::current();
        let tip = if self.is_vietnamese {
            strings.tray_tooltip_vi
        } else {
            strings.tray_tooltip_en
        };
        self.tray.set_tooltip(tip);
    }

    pub fn set_vietnamese_mode(&mut self, enabled: bool) {
        self.is_vietnamese = enabled;
        self.refresh_icon();
    }

    pub fn toggle_mode(&mut self) -> bool {
        let next = !self.is_vietnamese;
        self.set_vietnamese_mode(next);
        next
    }

    pub fn show_context_menu(&self, hwnd: isize, x: i32, y: i32, config: &EngineConfig) -> u32 {
        let mut menu = match PopupMenu::new() {
            Some(m) => m,
            None => return 0,
        };

        let strings = crate::language::current();

        // 1. Toggle V/E
        menu.add_checked_item(IDM_TOGGLE_VIET, strings.tray_toggle_vi, self.is_vietnamese);
        menu.add_separator();

        // 2. Input Method
        let is_telex = matches!(config.method, InputMethod::Telex);
        let is_vni = matches!(config.method, InputMethod::Vni);
        let is_simple = matches!(
            config.method,
            InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2
        );

        menu.add_checked_item(IDM_TELEX, strings.tray_method_telex, is_telex);
        menu.add_checked_item(IDM_VNI, strings.tray_method_vni, is_vni);
        menu.add_checked_item(
            IDM_SIMPLE_TELEX,
            strings.tray_method_simple_telex,
            is_simple,
        );
        menu.add_separator();

        // 3. Control Panel & Exit
        menu.add_item(IDM_CONTROL_PANEL, strings.tray_control_panel);
        menu.add_separator();
        menu.add_item(IDM_EXIT, strings.tray_exit);

        menu.track(hwnd, x, y)
    }
}

impl Drop for TrayHandler {
    fn drop(&mut self) {
        self.tray.hide();
        safe_destroy_icon(self.hicon_viet);
        safe_destroy_icon(self.hicon_eng);
    }
}
