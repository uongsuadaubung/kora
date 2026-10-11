//! Footer controls ([ Kết thúc ] on sidebar, [ Mặc định ] - [ Đóng ] on right panel)

use crate::language::LanguageStrings;
use crate::ui::components::*;
use crate::ui::tokens::id::*;
use crate::ui::win32::*;

pub struct FooterControls {
    pub btn_exit: PushButton,
    pub btn_defaults: PushButton,
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

        let btn_defaults = PushButton::create_with_variant(
            parent,
            IDC_BTN_DEFAULTS,
            strings.btn_defaults,
            580,
            542,
            104,
            30,
            hfont_normal,
            ButtonVariant::Secondary,
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
            btn_defaults,
            btn_close,
        })
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, controls: &super::ControlPanelControls) -> bool {
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
                unsafe {
                    ShowWindow(hwnd, SW_HIDE);
                }
                crate::platform::win32::trim_working_set();
                true
            }
            IDC_BTN_DEFAULTS => {
                self.handle_defaults_button(controls, hwnd);
                true
            }
            _ => false,
        }
    }

    pub fn handle_defaults_button(&self, controls: &super::ControlPanelControls, hwnd: isize) {
        let def_config = crate::engine::EngineConfig::default();
        if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref mut engine) = *guard
        {
            *engine.config_mut() = def_config.clone();
            let _ = crate::engine::config_store::save_config_all(
                engine.config(),
                &engine.macro_table,
                &engine.profile_manager,
            );
        }
        crate::platform::win32::sound::reconfigure_sound(
            def_config.sound_enabled,
            &def_config.sound_profile,
            def_config.sound_volume,
        );
        crate::language::set_current_language(def_config.language);
        controls.load_config(&def_config);
        controls.update_language(def_config.language);
        unsafe {
            let strings = crate::language::get_strings(def_config.language);
            SetWindowTextW(hwnd, to_wide(strings.window_title).as_ptr());
            InvalidateRect(controls.tab_bar.hwnd(), std::ptr::null(), 1);
            UpdateWindow(controls.tab_bar.hwnd());
        }
        let is_dark = crate::ui::theme::resolve_is_dark(def_config.theme);
        crate::ui::theme::apply_ui_theme(hwnd, controls, is_dark);
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.btn_exit.set_text(strings.btn_exit);
        self.btn_defaults.set_text(strings.btn_defaults);
        self.btn_close.set_text(strings.btn_close);
    }
}
