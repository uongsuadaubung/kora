//! Tab 2: System controls (Khởi động cùng Windows, Giao diện, Ngôn ngữ, Nhật ký lỗi, Cập nhật)
//!
//! Powered by the Declarative Single-HWND UI Engine (`Surface<SystemMsg>`).

use crate::engine::EngineConfig;
use crate::engine::config::UiTheme;
use crate::language::{Language, LanguageStrings};
use crate::ui::views::control_panel::components::{form_group, page_header, section_header};
use crate::ui::views::dialogs::show_danger_confirm;
use crate::ui_framework::core::*;
use crate::ui_framework::tokens::layout;
use crate::ui_framework::win32::{
    RDW_ALLCHILDREN, RDW_INVALIDATE, RedrawWindow, SW_HIDE, SetWindowTextW, ShowWindow, to_wide,
};

const SYS_CARD_X: i32 = layout::CARD_X;
const SYS_START_Y: i32 = layout::HEADER_TITLE_Y;

#[derive(Clone, Debug)]
pub enum SystemMsg {
    ToggleAutostart(bool),
    ToggleShowDialog(bool),
    ToggleDebugLog(bool),
    OpenDebugLog,
    SelectTheme(usize),
    SelectLanguage(usize),
    RestoreDefaults,
}

pub struct TabSystem {
    surface: Surface<SystemMsg>,
}

impl TabSystem {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let viewport_w = layout::CARD_WIDTH;
        let viewport_h = layout::CARD_BOTTOM_Y - SYS_START_Y;

        let surface = Surface::create(parent, SYS_CARD_X, SYS_START_Y, viewport_w, viewport_h)?;
        unsafe {
            ShowWindow(surface.hwnd(), SW_HIDE);
        }

        let surface_cb = surface.clone();
        surface.set_on_msg(move |msg| {
            Self::handle_message(parent, &surface_cb, msg);
        });

        let tab = Self { surface };
        let def_cfg = EngineConfig::default();
        tab.load_config(&def_cfg);
        Some(tab)
    }

    pub fn viewport_hwnd(&self) -> isize {
        self.surface.hwnd()
    }

    pub fn paint(&self, _hdc: isize, _right_bound: i32) {
        self.surface.invalidate();
    }

    pub fn update_visibility(&self, visible: bool) {
        self.surface.set_visible(visible);
    }

    pub fn load_config(&self, config: &EngineConfig) {
        let strings = crate::language::current();
        let autostart = crate::platform::is_windows_autostart_enabled();
        let tree = Self::view(config, autostart, strings);
        self.surface.render(tree);
    }

    pub fn update_language(&self, _lang: Language, _strings: &LanguageStrings) {
        let cfg = crate::ui::controller::with_engine(|e| e.config().clone())
            .unwrap_or_default();
        self.load_config(&cfg);
    }

    pub fn on_command(
        &self,
        _hwnd: isize,
        _ctl_id: u32,
        _notify_code: u32,
        _controls: &super::ControlPanelControls,
    ) -> bool {
        false
    }

    fn view(
        config: &EngineConfig,
        autostart: bool,
        strings: &LanguageStrings,
    ) -> Element<SystemMsg> {
        let header = page_header(strings.sys_header_title, strings.sys_header_sub);

        let card1 = card(elements![
            section_header(strings.sys_title),
            switch(strings.check_autostart, autostart, SystemMsg::ToggleAutostart),
            divider(),
            switch(
                strings.check_show_dialog,
                config.show_dialog_on_startup,
                SystemMsg::ToggleShowDialog,
            ),
            divider(),
            row(elements![
                switch(
                    strings.check_debug_log,
                    config.debug,
                    SystemMsg::ToggleDebugLog,
                )
                .flex_1(),
                button(
                    strings.btn_open_log,
                    Variant::Secondary,
                    SystemMsg::OpenDebugLog,
                )
                .size(Size::Sm),
            ])
            .items_center()
            .justify_between()
            .gap(3),
        ]);

        let theme_idx: usize = config.theme.into();
        let lang_idx: usize = config.language.into();
        let card2 = card(elements![
            section_header(strings.label_theme_title),
            row(elements![
                form_group(
                    strings.label_theme,
                    select(
                        vec![
                            strings.theme_auto.to_string(),
                            strings.theme_light.to_string(),
                            strings.theme_dark.to_string(),
                        ],
                        theme_idx,
                        SystemMsg::SelectTheme,
                    )
                    .w_full(),
                )
                .flex_1(),
                form_group(
                    strings.label_language,
                    select(
                        vec!["Tiếng Việt".to_string(), "English".to_string()],
                        lang_idx,
                        SystemMsg::SelectLanguage,
                    )
                    .w_full(),
                )
                .flex_1(),
            ])
            .gap(3),
        ]);

        let card3 = card(elements![
            row(elements![
                col(elements![
                    text(strings.sys_defaults_title).bold(),
                    text(strings.sys_defaults_desc).muted().size(Size::Sm),
                ])
                .gap(1)
                .flex_1(),
                button(
                    strings.btn_restore_defaults,
                    Variant::Secondary,
                    SystemMsg::RestoreDefaults,
                ),
            ])
            .items_center()
            .justify_between()
            .gap(3),
        ]);

        col(elements![header, divider(), card1, card2, card3]).gap(3).pb(3)
    }

    fn handle_message(parent_hwnd: isize, surface: &Surface<SystemMsg>, msg: SystemMsg) {
        match msg {
            SystemMsg::ToggleAutostart(enabled) => {
                crate::platform::set_windows_autostart(enabled);
                let cfg = crate::ui::controller::with_engine(|e| e.config().clone())
                    .unwrap_or_default();
                let strings = crate::language::current();
                surface.render(Self::view(&cfg, enabled, strings));
            }
            SystemMsg::ToggleShowDialog(enabled) => {
                let cfg_opt = crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().show_dialog_on_startup = enabled;
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                    engine.config().clone()
                });
                if let Some(cfg) = cfg_opt {
                    let strings = crate::language::current();
                    surface.render(Self::view(
                        &cfg,
                        crate::platform::is_windows_autostart_enabled(),
                        strings,
                    ));
                }
            }
            SystemMsg::ToggleDebugLog(enabled) => {
                let cfg_opt = crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().debug = enabled;
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                    engine.config().clone()
                });
                if let Some(cfg) = cfg_opt {
                    let strings = crate::language::current();
                    surface.render(Self::view(
                        &cfg,
                        crate::platform::is_windows_autostart_enabled(),
                        strings,
                    ));
                }
            }
            SystemMsg::OpenDebugLog => {
                let log_file = crate::ui::controller::with_engine(|e| {
                    e.config()
                        .debug_file_path
                        .as_deref()
                        .map(|s| s.to_string())
                })
                .flatten()
                .unwrap_or_else(|| "kora_debug.log".to_string());

                if !std::path::Path::new(&log_file).exists() {
                    let _ = std::fs::File::create(&log_file);
                }
                let _ = std::process::Command::new("notepad.exe")
                    .arg(&log_file)
                    .spawn();
            }
            SystemMsg::SelectTheme(theme_sel) => {
                let new_theme = UiTheme::from(theme_sel);
                let cfg_opt = crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().theme = new_theme;
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                    engine.config().clone()
                });
                if let Some(cfg) = cfg_opt {
                    if let Some(controls) = crate::ui::controls() {
                        let is_dark = crate::ui::theme::resolve_is_dark(new_theme);
                        crate::ui::theme::apply_ui_theme(parent_hwnd, controls, is_dark);
                    }
                    if let Some(ui) = crate::ui::ui() {
                        ui.tray.borrow_mut().refresh_icon();
                    }
                    let strings = crate::language::current();
                    surface.render(Self::view(
                        &cfg,
                        crate::platform::is_windows_autostart_enabled(),
                        strings,
                    ));
                }
            }
            SystemMsg::SelectLanguage(lang_sel) => {
                let new_lang = Language::from(lang_sel);
                crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().language = new_lang;
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                });
                if let Some(controls) = crate::ui::controls() {
                    controls.update_language(new_lang);
                }
                if let Some(ui) = crate::ui::ui() {
                    ui.tray.borrow_mut().refresh_icon();
                }
                unsafe {
                    let strings = crate::language::get_strings(new_lang);
                    SetWindowTextW(parent_hwnd, to_wide(strings.window_title).as_ptr());
                    RedrawWindow(
                        parent_hwnd,
                        std::ptr::null(),
                        0,
                        RDW_INVALIDATE | RDW_ALLCHILDREN,
                    );
                }
            }
            SystemMsg::RestoreDefaults => {
                let strings = crate::language::current();
                show_danger_confirm(
                    parent_hwnd,
                    strings.confirm_defaults_title,
                    strings.confirm_defaults_msg,
                    move || {
                        let def_config = EngineConfig::default();
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
                            crate::ui::theme::apply_ui_theme(parent_hwnd, controls, is_dark);
                        }
                        unsafe {
                            let strings = crate::language::get_strings(def_config.language);
                            SetWindowTextW(parent_hwnd, to_wide(strings.window_title).as_ptr());
                            RedrawWindow(
                                parent_hwnd,
                                std::ptr::null(),
                                0,
                                RDW_INVALIDATE | RDW_ALLCHILDREN,
                            );
                        }
                    },
                );
            }
        }
    }
}
