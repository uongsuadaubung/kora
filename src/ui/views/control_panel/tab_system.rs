//! Tab 2: System controls (Khởi động cùng Windows, Giao diện, Ngôn ngữ, Nhật ký lỗi, Cập nhật)
//! Uses CardLayout (page_header, section_title, checkbox_row, auto-HWND, auto-i18n).

use crate::language::LanguageStrings;
use crate::ui::prelude::*;
use crate::ui::views::dialogs::show_danger_confirm;
use crate::ui_framework::win32::{RedrawWindow, SetWindowTextW, RDW_ALLCHILDREN, RDW_INVALIDATE};

const SYS_CARD_X: i32 = 246;
const SYS_CONTENT_X: i32 = 266;
const SYS_CONTENT_W: i32 = 520;

pub struct TabSystem {
    layout: CardLayout,
    pub check_autostart: CheckBox,
    pub check_show_dialog: CheckBox,
    pub check_debug_log: CheckBox,
    pub btn_open_log: PushButton,
    pub combo_theme: ComboBox,
    pub combo_lang: ComboBox,
    pub btn_restore_defaults: PushButton,
}

impl TabSystem {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold)
            .at(86)
            .with_bounds(SYS_CONTENT_X, SYS_CONTENT_W);

        // --- Header Section ---
        ly.page_header(|s| s.sys_header_title, |s| s.sys_header_sub)?;

        // --- Card 1: Windows System ---
        let (check_autostart, check_show_dialog, check_debug_log, btn_open_log) =
            ly.card_with_pad(16, 18, |card| {
                card.section_title(|s| s.sys_title, 8)?;
                let check_autostart =
                    card.checkbox_row(|s| s.check_autostart, 24, 6, false, false)?;
                let check_show_dialog =
                    card.checkbox_row(|s| s.check_show_dialog, 24, 6, false, false)?;

                let ((x_dbg, w_dbg), (x_btn_log, w_btn_log)) = card.split_right(144, 16);
                let y_chk3 = card.row(24, 0);
                let check_debug_log =
                    card.checkbox_at(|s| s.check_debug_log, x_dbg, y_chk3, w_dbg, 24, false)?;

                let btn_open_log = card.button_at(
                    |s| s.btn_open_log,
                    x_btn_log,
                    y_chk3 - 1,
                    w_btn_log,
                    26,
                    false,
                    ButtonVariant::Secondary,
                )?;
                Some((check_autostart, check_show_dialog, check_debug_log, btn_open_log))
            })?;

        // --- Card 2: Theme & Language ---
        ly.skip(4);
        let (combo_theme, combo_lang) = ly.card_with_pad(16, 26, |card| {
            card.section_title(|s| s.label_theme_title, 8)?;
            let ((_, combo_theme), (_, combo_lang)) = card.two_col_combos(
                20,
                0,
                false,
                (
                    |s| s.label_theme,
                    |s| vec![s.theme_auto, s.theme_light, s.theme_dark],
                ),
                (|s| s.label_language, |_s| vec!["Tiếng Việt", "English"]),
            )?;
            Some((combo_theme, combo_lang))
        })?;

        // --- Card 3: Restore Defaults ---
        ly.skip(4);
        let btn_restore_defaults = ly.card_with_pad(16, 12, |card| {
            let ((x_def, w_def), (x_btn_def, w_btn_def)) = card.split_right(144, 16);
            let y_def_title = card.row(20, 4);
            card.label_at(
                |s| s.sys_defaults_title,
                x_def,
                y_def_title,
                w_def,
                20,
                true,
            )?;

            let y_def_desc = card.row(36, 0);
            card.label_at(
                |s| s.sys_defaults_desc,
                x_def,
                y_def_desc,
                w_def,
                36,
                false,
            )?
            .with_tone(LabelTone::Secondary);

            card.button_at(
                |s| s.btn_restore_defaults,
                x_btn_def,
                y_def_title + 12,
                w_btn_def,
                32,
                false,
                ButtonVariant::Secondary,
            )
        })?;

        Some(Self {
            layout: ly,
            check_autostart,
            check_show_dialog,
            check_debug_log,
            btn_open_log,
            combo_theme,
            combo_lang,
            btn_restore_defaults,
        })
    }

    pub fn paint(&self, hdc: isize, right_bound: i32) {
        self.layout.paint_cards(hdc, SYS_CARD_X, right_bound);
    }

    pub fn update_visibility(&self, visible: bool) {
        self.layout.set_visible(visible);
    }

    pub fn load_config(&self, config: &crate::engine::EngineConfig) {
        self.check_autostart
            .set_checked(crate::platform::is_windows_autostart_enabled());
        self.check_show_dialog
            .set_checked(config.show_dialog_on_startup);
        self.check_debug_log.set_checked(config.debug);

        self.combo_theme.set_selected(config.theme.into());
        self.combo_lang.set_selected(config.language.into());
    }

    pub fn read_config(&self, config: &mut crate::engine::EngineConfig) {
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
        self.layout.update_language(strings);
        self.combo_lang.set_selected(lang.into());
    }

    pub fn on_command(
        &self,
        hwnd: isize,
        ctl_id: u32,
        notify_code: u32,
        controls: &super::ControlPanelControls,
    ) -> bool {
        if self.btn_restore_defaults.is_clicked(ctl_id, notify_code) {
            let strings = crate::language::current();
            let hwnd_panel = hwnd;
            show_danger_confirm(
                hwnd_panel,
                strings.confirm_defaults_title,
                strings.confirm_defaults_msg,
                move || {
                let def_config = crate::engine::EngineConfig::default();
                crate::ui::controller::with_engine(|engine| {
                    *engine.config_mut() = def_config.clone();
                    engine.profile_manager = crate::engine::profile::ProfileManager::new();
                    engine.macro_table = crate::engine::macro_table::MacroTable::new();
                    let _ = crate::engine::config_store::save_config_all(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                    crate::ui::controller::reconfigure_sound(engine);
                });

                if let Some(controls) = crate::ui::controls() {
                    controls.refresh_from_engine();
                    controls.update_language(def_config.language);
                    let is_dark = crate::ui::theme::resolve_is_dark(def_config.theme);
                    crate::ui::theme::apply_ui_theme(hwnd_panel, controls, is_dark);
                }
                unsafe {
                    let strings = crate::language::get_strings(def_config.language);
                    SetWindowTextW(hwnd_panel, to_wide(strings.window_title).as_ptr());
                    RedrawWindow(
                        hwnd_panel,
                        std::ptr::null(),
                        0,
                        RDW_INVALIDATE | RDW_ALLCHILDREN,
                    );
                }
            });
            return true;
        }

        if self.btn_open_log.is_clicked(ctl_id, notify_code) {
            let log_file = {
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
                    guard
                        .as_ref()
                        .and_then(|e| {
                            e.config()
                                .debug_file_path
                                .as_deref()
                                .map(|s| s.to_string())
                        })
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
            return true;
        }

        if self.check_autostart.is_toggled(ctl_id, notify_code) {
            crate::platform::set_windows_autostart(self.check_autostart.is_checked());
            return true;
        }

        let is_theme_changed = self.combo_theme.is_changed(ctl_id, notify_code);
        let is_lang_changed = self.combo_lang.is_changed(ctl_id, notify_code);
        let is_chk_changed = self.check_show_dialog.is_toggled(ctl_id, notify_code)
            || self.check_debug_log.is_toggled(ctl_id, notify_code);

        if is_theme_changed || is_lang_changed || is_chk_changed {
            let snapshot = crate::platform::win32::ENGINE_INSTANCE
                .lock()
                .ok()
                .and_then(|mut guard| {
                    guard.as_mut().map(|engine| {
                        self.read_config(engine.config_mut());
                        crate::engine::config_store::save_config_all_debounced(
                            engine.config(),
                            &engine.macro_table,
                            &engine.profile_manager,
                        );
                        (engine.config().theme, engine.config().language)
                    })
                });

            if let Some((theme, lang)) = snapshot {
                if is_theme_changed {
                    let is_dark = crate::ui::theme::resolve_is_dark(theme);
                    crate::ui::theme::apply_ui_theme(hwnd, controls, is_dark);
                    if let Some(ui) = crate::ui::ui() {
                        ui.tray.borrow_mut().refresh_icon();
                    }
                } else if is_lang_changed {
                    controls.update_language(lang);
                    if let Some(ui) = crate::ui::ui() {
                        ui.tray.borrow_mut().refresh_icon();
                    }
                    unsafe {
                        let strings = crate::language::get_strings(lang);
                        SetWindowTextW(hwnd, to_wide(strings.window_title).as_ptr());
                        RedrawWindow(
                            hwnd,
                            std::ptr::null(),
                            0,
                            RDW_INVALIDATE | RDW_ALLCHILDREN,
                        );
                    }
                }
            }
            return true;
        }

        false
    }
}
