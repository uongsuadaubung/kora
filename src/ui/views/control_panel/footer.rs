//! Footer controls ([ Thoát Kora ] on left, [ Đóng ] on right panel)

use crate::language::LanguageStrings;
use crate::ui::prelude::*;
use crate::ui::win32::PostQuitMessage;

pub struct FooterControls {
    layout: CardLayout,
    pub btn_exit: PushButton,
    pub btn_close: PushButton,
}

impl FooterControls {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);

        let btn_exit = ly.track_i18n(
            PushButton::create_with_variant(
                parent,
                0,
                strings.btn_exit,
                244,
                542,
                114,
                30,
                hfont_normal,
                ButtonVariant::Danger,
                None,
            )?,
            |s| s.btn_exit,
        );

        let btn_close = ly.track_i18n(
            PushButton::create_with_variant(
                parent,
                0,
                strings.btn_close,
                694,
                542,
                112,
                30,
                hfont_bold,
                ButtonVariant::Primary,
                None,
            )?,
            |s| s.btn_close,
        );

        Some(Self {
            layout: ly,
            btn_exit,
            btn_close,
        })
    }

    pub fn on_command(&self, _hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        if self.btn_exit.is_clicked(ctl_id, notify_code) {
            println!("[UI] Người dùng nhấn Tắt Kora -> Thoát Kora");
            crate::engine::config_store::flush_config_debounced();
            unsafe {
                PostQuitMessage(0);
            }
            return true;
        }
        if self.btn_close.is_clicked(ctl_id, notify_code) {
            crate::ui::hide_control_panel();
            return true;
        }
        false
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);
    }
}
