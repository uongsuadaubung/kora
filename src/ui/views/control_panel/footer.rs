//! Footer controls ([ Thoát Kora ] on left, [ Đóng ] on right panel)
//! Powered by Declarative Single-HWND UI Engine (`Surface<FooterMsg>`).

use crate::language::LanguageStrings;
use crate::ui::win32::PostQuitMessage;
use crate::ui_framework::core::*;

#[derive(Clone, Debug)]
pub enum FooterMsg {
    Exit,
    Close,
}

pub struct FooterControls {
    pub surface: Surface<FooterMsg>,
}

impl FooterControls {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let surface = Surface::create(parent, 244, 542, 562, 30)?;
        surface.set_content_width(562);

        let strings = crate::language::current();
        surface.render(Self::view(strings));

        surface.set_on_msg(|msg| match msg {
            FooterMsg::Exit => {
                println!("[UI] Người dùng nhấn Tắt Kora -> Thoát Kora");
                crate::engine::config_store::flush_config_debounced();
                unsafe {
                    PostQuitMessage(0);
                }
            }
            FooterMsg::Close => {
                crate::ui::hide_control_panel();
            }
        });

        Some(Self { surface })
    }

    pub fn view(strings: &LanguageStrings) -> Element<FooterMsg> {
        row(vec![
            button(strings.btn_exit, Variant::Danger, FooterMsg::Exit).w(114),
            spacer(),
            button(strings.btn_close, Variant::Primary, FooterMsg::Close).w(112),
        ])
        .items_center()
        .w_full()
    }

    #[inline]
    pub fn on_command(&self, _hwnd: isize, _ctl_id: u32, _notify_code: u32) -> bool {
        false
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.surface.render(Self::view(strings));
    }
}
