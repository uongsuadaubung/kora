//! Tab 2: System controls (Khởi động cùng Windows, Giao diện, Ngôn ngữ, Nhật ký lỗi, Cập nhật)
//! Redesigned with modern Fluent cards, matching mockup_profiles layout.

use crate::language::LanguageStrings;
use crate::ui::colors::ThemePalette;
use crate::ui::win32::{
    create_app_font, to_wide, DeleteObject, InvalidateRect, SetBkColor, SetTextColor,
    SetWindowTextW, ShowWindow, UpdateWindow, FW_BOLD, SW_HIDE, SW_SHOW,
};
use crate::ui::components::*;
use crate::ui::tokens::id::*;
use crate::ui::tokens::layout::*;
use crate::ui::tokens::typography::*;

pub struct TabSystem {
    pub lbl_header_title: Label,
    pub lbl_header_sub: Label,
    pub label_sys_title: Label,
    pub check_autostart: CheckBox,
    pub check_show_dialog: CheckBox,
    pub check_debug_log: CheckBox,
    pub btn_open_log: PushButton,
    pub btn_check_update: PushButton,
    pub label_theme_title: Label,
    pub label_theme: Label,
    pub combo_theme: ComboBox,
    pub label_lang: Label,
    pub combo_lang: ComboBox,
    pub hfont_title: isize,
}

impl TabSystem {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let hfont_title = create_app_font(FONT_SIZE_TITLE, FW_BOLD);

        // --- Header Section ---
        let lbl_header_title = Label::create_with_id(
            parent,
            IDC_SYS_HEADER_TITLE,
            strings.sys_header_title,
            HEADER_X,
            HEADER_TITLE_Y,
            HEADER_TITLE_WIDTH,
            HEADER_TITLE_HEIGHT,
            hfont_title,
        )?;

        let lbl_header_sub = Label::create_with_id(
            parent,
            IDC_SYS_HEADER_SUB,
            strings.sys_header_sub,
            HEADER_X,
            HEADER_SUB_Y,
            HEADER_SUB_WIDTH,
            HEADER_SUB_HEIGHT,
            hfont_normal,
        )?;

        // --- Card 1: Windows System ---
        let label_sys_title = Label::create_with_id(
            parent,
            IDC_LABEL_SYS_TITLE,
            strings.sys_title,
            266,
            102,
            400,
            20,
            hfont_bold,
        )?;

        let check_autostart = CheckBox::create(
            parent,
            IDC_CHECK_AUTOSTART,
            strings.check_autostart,
            266,
            130,
            520,
            24,
            hfont_normal,
        )?;

        let check_show_dialog = CheckBox::create(
            parent,
            IDC_CHECK_SHOW_DIALOG,
            strings.check_show_dialog,
            266,
            160,
            520,
            24,
            hfont_normal,
        )?;

        let check_debug_log = CheckBox::create(
            parent,
            IDC_CHECK_DEBUG_LOG,
            strings.check_debug_log,
            266,
            190,
            260,
            24,
            hfont_normal,
        )?;

        let btn_open_log = PushButton::create_with_variant(
            parent,
            IDC_BTN_OPEN_LOG,
            strings.btn_open_log,
            534,
            189,
            100,
            26,
            hfont_normal,
            ButtonVariant::Secondary,
            None,
        )?;

        let btn_check_update = PushButton::create_with_variant(
            parent,
            IDC_BTN_CHECK_UPDATE,
            strings.btn_check_update,
            642,
            189,
            144,
            26,
            hfont_normal,
            ButtonVariant::Secondary,
            None,
        )?;

        // --- Card 2: Theme & Language ---
        let label_theme_title = Label::create_with_id(
            parent,
            IDC_LABEL_THEME_TITLE,
            strings.label_theme_title,
            266,
            264,
            400,
            20,
            hfont_bold,
        )?;

        let label_theme = Label::create_with_id(
            parent,
            IDC_LABEL_THEME,
            strings.label_theme,
            266,
            292,
            245,
            18,
            hfont_normal,
        )?;

        let combo_theme = ComboBox::create_with_items(
            parent,
            IDC_COMBO_THEME,
            266,
            314,
            250,
            160,
            hfont_normal,
            &[strings.theme_auto, strings.theme_light, strings.theme_dark],
            0,
        )?;

        let label_lang = Label::create_with_id(
            parent,
            IDC_LABEL_LANG,
            strings.label_language,
            536,
            292,
            250,
            18,
            hfont_normal,
        )?;

        let combo_lang = ComboBox::create_with_items(
            parent,
            IDC_COMBO_LANG,
            536,
            314,
            250,
            160,
            hfont_normal,
            &["Tiếng Việt", "English"],
            0,
        )?;

        Some(Self {
            lbl_header_title,
            lbl_header_sub,
            label_sys_title,
            check_autostart,
            check_show_dialog,
            check_debug_log,
            btn_open_log,
            btn_check_update,
            label_theme_title,
            label_theme,
            combo_theme,
            label_lang,
            combo_lang,
            hfont_title,
        })
    }

    pub fn update_visibility(&self, visible: bool) {
        unsafe {
            let show_cmd = if visible { SW_SHOW } else { SW_HIDE };
            ShowWindow(self.lbl_header_title.hwnd(), show_cmd);
            ShowWindow(self.lbl_header_sub.hwnd(), show_cmd);
            ShowWindow(self.label_sys_title.hwnd(), show_cmd);
            ShowWindow(self.check_autostart.hwnd(), show_cmd);
            ShowWindow(self.check_show_dialog.hwnd(), show_cmd);
            ShowWindow(self.check_debug_log.hwnd(), show_cmd);
            ShowWindow(self.btn_open_log.hwnd(), show_cmd);
            ShowWindow(self.btn_check_update.hwnd(), show_cmd);
            ShowWindow(self.label_theme_title.hwnd(), show_cmd);
            ShowWindow(self.label_theme.hwnd(), show_cmd);
            ShowWindow(self.combo_theme.hwnd(), show_cmd);
            ShowWindow(self.label_lang.hwnd(), show_cmd);
            ShowWindow(self.combo_lang.hwnd(), show_cmd);
        }
    }

    pub fn load_config(&self, config: &crate::engine::EngineConfig) {
        // Luôn đọc trực tiếp từ Registry làm nguồn chuẩn xác nhất
        self.check_autostart
            .set_checked(crate::platform::is_windows_autostart_enabled());
        self.check_show_dialog
            .set_checked(config.show_dialog_on_startup);
        self.check_debug_log.set_checked(config.debug);

        self.combo_theme.set_selected(config.theme.into());
        self.combo_lang.set_selected(config.language.into());
    }

    pub fn read_config(&self, config: &mut crate::engine::EngineConfig) {
        let new_autostart = self.check_autostart.is_checked();
        if crate::platform::is_windows_autostart_enabled() != new_autostart {
            crate::platform::set_windows_autostart(new_autostart);
            println!(
                "[Kora] >> Khởi động cùng Windows: {}",
                if new_autostart { "BẬT" } else { "TẮT" }
            );
        }

        config.show_dialog_on_startup = self.check_show_dialog.is_checked();

        let new_debug = self.check_debug_log.is_checked();
        if config.debug != new_debug {
            config.debug = new_debug;
            println!(
                "[Kora] >> Ghi nhật ký lỗi (Debug Log): {}",
                if new_debug { "BẬT" } else { "TẮT" }
            );
        }

        if let Some(theme_sel) = self.combo_theme.get_selected() {
            let new_theme = crate::engine::config::UiTheme::from(theme_sel);
            if config.theme != new_theme {
                config.theme = new_theme;
                println!("[Kora] >> Đổi giao diện sang: {:?}", new_theme);
            }
        }

        if let Some(lang_sel) = self.combo_lang.get_selected() {
            let new_lang = crate::language::Language::from(lang_sel);
            if config.language != new_lang {
                config.language = new_lang;
                println!("[Kora] >> Đổi ngôn ngữ giao diện sang: {:?}", new_lang);
            }
        }
    }

    pub fn update_language(&self, lang: crate::language::Language, strings: &LanguageStrings) {
        self.lbl_header_title.set_text(strings.sys_header_title);
        self.lbl_header_sub.set_text(strings.sys_header_sub);
        self.label_sys_title.set_text(strings.sys_title);
        self.check_autostart.set_text(strings.check_autostart);
        self.check_show_dialog.set_text(strings.check_show_dialog);
        self.check_debug_log.set_text(strings.check_debug_log);
        self.btn_open_log.set_text(strings.btn_open_log);
        self.btn_check_update.set_text(strings.btn_check_update);
        self.label_theme_title.set_text(strings.label_theme_title);
        self.label_theme.set_text(strings.label_theme);

        let cur_theme = self.combo_theme.get_selected().unwrap_or(0);
        self.combo_theme.reset_items(
            &[strings.theme_auto, strings.theme_light, strings.theme_dark],
            Some(cur_theme),
        );

        self.label_lang.set_text(strings.label_language);
        self.combo_lang.set_selected(lang.into());
    }


    pub fn on_ctlcolor_static(&self, hdc: isize, ctrl_id: u32, is_dark: bool) -> Option<isize> {
        let palette = ThemePalette::get(is_dark);
        match ctrl_id {
            IDC_SYS_HEADER_TITLE => {
                unsafe {
                    SetBkColor(hdc, palette.bg_window);
                    SetTextColor(hdc, palette.text_primary);
                }
                let br = crate::ui::theme::get_panel_bg_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            IDC_SYS_HEADER_SUB => {
                unsafe {
                    SetBkColor(hdc, palette.bg_window);
                    SetTextColor(hdc, palette.text_secondary);
                }
                let br = crate::ui::theme::get_panel_bg_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            IDC_LABEL_SYS_TITLE | IDC_LABEL_THEME_TITLE => {
                unsafe {
                    SetBkColor(hdc, palette.bg_card);
                    SetTextColor(hdc, palette.text_primary);
                }
                let br = crate::ui::theme::get_tab_card_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            IDC_LABEL_THEME | IDC_LABEL_LANG => {
                unsafe {
                    SetBkColor(hdc, palette.bg_card);
                    SetTextColor(hdc, palette.text_secondary);
                }
                let br = crate::ui::theme::get_tab_card_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            _ => {}
        }
        None
    }

    pub fn on_command(
        &self,
        hwnd: isize,
        ctl_id: u32,
        notify_code: u32,
        controls: &super::ControlPanelControls,
    ) -> bool {
        match ctl_id {
            IDC_BTN_OPEN_LOG => {
                let log_file = {
                    if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
                        guard
                            .as_ref()
                            .and_then(|e| e.config().debug_file_path.clone())
                            .unwrap_or_else(|| "kora_debug.log".to_string())
                    } else {
                        "kora_debug.log".to_string()
                    }
                };
                if !std::path::Path::new(&log_file).exists() {
                    let _ = std::fs::File::create(&log_file);
                }
                let _ = std::process::Command::new("notepad.exe")
                    .arg(&log_file)
                    .spawn();
                true
            }
            IDC_BTN_CHECK_UPDATE => {
                let hwnd_panel = hwnd;
                let strings = crate::language::current();
                self.btn_check_update.set_text(strings.btn_checking_update);
                self.btn_check_update.set_enabled(false);

                std::thread::spawn(move || {
                    let status = crate::updater::check_for_updates();

                    if let Ok(ui_guard) = crate::ui::UI_MANAGER.try_lock()
                        && let Some(ref ui) = *ui_guard
                    {
                        let s = crate::language::current();
                        ui.controls
                            .tab_system
                            .btn_check_update
                            .set_text(s.btn_check_update);
                        ui.controls.tab_system.btn_check_update.set_enabled(true);
                    }

                    crate::updater::show_update_result_dialog(hwnd_panel, &status);
                });
                true
            }
            IDC_COMBO_THEME | IDC_COMBO_LANG | IDC_CHECK_AUTOSTART | IDC_CHECK_SHOW_DIALOG | IDC_CHECK_DEBUG_LOG => {
                let is_combo = (ctl_id == IDC_COMBO_THEME || ctl_id == IDC_COMBO_LANG) && notify_code == 1;
                let is_check = (ctl_id == IDC_CHECK_AUTOSTART || ctl_id == IDC_CHECK_SHOW_DIALOG || ctl_id == IDC_CHECK_DEBUG_LOG) && notify_code == 0;

                if is_combo || is_check {
                    let mut theme_opt = None;
                    let mut lang_opt = None;

                    if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        self.read_config(engine.config_mut());
                        crate::engine::config_store::save_config_all_debounced(
                            engine.config(),
                            &engine.macro_table,
                            &engine.profile_manager,
                        );
                        theme_opt = Some(engine.config().theme);
                        lang_opt = Some(engine.config().language);
                    }

                    if ctl_id == IDC_COMBO_THEME {
                        if let Some(theme) = theme_opt {
                            let is_dark = crate::ui::theme::resolve_is_dark(theme);
                            crate::ui::theme::apply_ui_theme(hwnd, controls, is_dark);
                        }
                    } else if ctl_id == IDC_COMBO_LANG {
                        if let Some(lang) = lang_opt {
                            crate::language::set_current_language(lang);
                            controls.update_language(lang);
                            unsafe {
                                let strings = crate::language::get_strings(lang);
                                SetWindowTextW(hwnd, to_wide(strings.window_title).as_ptr());
                                InvalidateRect(controls.tab_bar.hwnd(), std::ptr::null(), 1);
                                UpdateWindow(controls.tab_bar.hwnd());
                            }
                            let is_dark = crate::ui::theme::resolve_is_dark(theme_opt.unwrap_or(crate::engine::config::UiTheme::Auto));
                            crate::ui::theme::apply_ui_theme(hwnd, controls, is_dark);
                        }
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl Drop for TabSystem {
    fn drop(&mut self) {
        unsafe {
            if self.hfont_title != 0 {
                DeleteObject(self.hfont_title);
            }
        }
    }
}
