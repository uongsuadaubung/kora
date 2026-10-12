//! Footer controls ([ Thoát Kora ] on left, [ Đóng ] on right panel)

use crate::language::LanguageStrings;
use crate::ui::components::*;
use crate::ui::tokens::id::*;
use crate::ui::win32::*;

pub struct FooterControls {
    pub btn_exit: PushButton,
    pub btn_close: PushButton,
}

impl FooterControls {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let btn_exit = PushButton::create_with_variant(
            parent,
            IDC_BTN_EXIT,
            strings.btn_exit,
            244,
            542,
            114,
            30,
            hfont_normal,
            ButtonVariant::Danger,
            None,
        )?;

        let btn_close = PushButton::create_with_variant(
            parent,
            IDC_BTN_CLOSE,
            strings.btn_close,
            694,
            542,
            112,
            30,
            hfont_bold,
            ButtonVariant::Primary,
            None,
        )?;

        Some(Self {
            btn_exit,
            btn_close,
        })
    }

    pub fn on_command(&self, _hwnd: isize, ctl_id: u32, _controls: &super::ControlPanelControls) -> bool {
        match ctl_id {
            IDC_BTN_EXIT => {
                println!("[UI] Người dùng nhấn Tắt Kora -> Thoát Kora");
                crate::engine::config_store::flush_config_debounced();
                unsafe {
                    PostQuitMessage(0);
                }
                true
            }
            IDC_BTN_CLOSE => {
                crate::ui::hide_control_panel();
                true
            }
            _ => false,
        }
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.btn_exit.set_text(strings.btn_exit);
        self.btn_close.set_text(strings.btn_close);
    }
}
