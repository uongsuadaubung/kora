//! Tab 0: Typing & Input Method Configuration (Bộ gõ)
//!
//! Powered by the Declarative Single-HWND UI Engine (`Surface<TypingMsg>`),
//! composing React-style functional components and Bootstrap-style layout utilities.

use crate::engine::EngineConfig;
use crate::engine::config::InputMethod;
use crate::engine::profile::{
    AppProfile, CharsetMode, Child, KeyInjectionStrategy, LanguageMode, MethodMode,
};
use crate::language::LanguageStrings;
use crate::ui::colors::ThemePalette;
use crate::ui::views::control_panel::components::{
    form_group, inheritable_switch_row, section_header, target_rule_card,
};
use crate::ui_framework::core::*;
use crate::ui_framework::tokens::layout;
use crate::vietnamese::Charset;

#[derive(Clone, Debug)]
pub enum TypingMsg {
    SelectMode(usize),
    SelectMethod(usize),
    SelectCharset(usize),
    ToggleSwitchKey(Option<bool>),
    OpenSwitchHotkeyDialog,
    ToggleRestoreWrong(Option<bool>),
    ToggleAutoUpper(Option<bool>),
    ToggleTargetCapture(bool),
    OpenCaptureHotkeyDialog,
    ChangeAppStrategy(usize),
    ChangeRuleStrategy(String, usize),
    DeleteRule(String),
}

fn simple_telex_keep_variant(current: InputMethod) -> InputMethod {
    match current {
        InputMethod::SimpleTelex2 => InputMethod::SimpleTelex2,
        _ => InputMethod::SimpleTelex1,
    }
}

fn decode_input_method(idx: usize, current: InputMethod) -> Option<InputMethod> {
    match idx {
        0 => Some(InputMethod::Telex),
        1 => Some(InputMethod::Vni),
        2 => Some(simple_telex_keep_variant(current)),
        _ => None,
    }
}

fn index_to_strategy(idx: usize) -> KeyInjectionStrategy {
    match idx {
        1 => KeyInjectionStrategy::ClipboardPaste,
        _ => KeyInjectionStrategy::AutocompleteGuard,
    }
}

fn app_strategy_to_index(strategy: Option<KeyInjectionStrategy>) -> usize {
    match strategy {
        None => 0,
        Some(KeyInjectionStrategy::AutocompleteGuard) => 1,
        Some(KeyInjectionStrategy::ClipboardPaste) => 2,
    }
}

fn index_to_app_strategy(idx: usize) -> Option<KeyInjectionStrategy> {
    match idx {
        1 => Some(KeyInjectionStrategy::AutocompleteGuard),
        2 => Some(KeyInjectionStrategy::ClipboardPaste),
        _ => None,
    }
}

fn app_strategy_description(
    strategy: Option<KeyInjectionStrategy>,
    strings: &LanguageStrings,
) -> &'static str {
    match strategy {
        None => strings.typing_app_strategy_desc_inherit,
        Some(KeyInjectionStrategy::AutocompleteGuard) => strings.strategy_desc_autocomplete_guard,
        Some(KeyInjectionStrategy::ClipboardPaste) => strings.strategy_desc_clipboard_paste,
    }
}

pub struct TabTyping {
    surface: Surface<TypingMsg>,
}

impl TabTyping {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let viewport_x = layout::CARD_X;
        let viewport_y = layout::CARD_START_Y;
        let viewport_w = layout::CARD_WIDTH + 8;
        let viewport_h = layout::CARD_BOTTOM_Y - layout::CARD_START_Y;

        let surface = Surface::create(parent, viewport_x, viewport_y, viewport_w, viewport_h)?;
        let surface_for_cb = surface.clone();
        surface.set_on_msg(move |msg| {
            Self::handle_message(parent, &surface_for_cb, msg);
        });

        let tab = Self { surface };
        tab.refresh_from_engine();
        Some(tab)
    }

    pub fn viewport_hwnd(&self) -> isize {
        self.surface.hwnd()
    }

    pub fn refresh_from_engine(&self) {
        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((child, cfg)) = snap {
            self.load_profile(child.as_ref(), &cfg);
        } else {
            let def_cfg = EngineConfig::default();
            self.load_profile(None, &def_cfg);
        }
    }

    pub fn load_profile(&self, child: Option<&AppProfile<Child>>, global_config: &EngineConfig) {
        let strings = crate::language::current();
        let tree = Self::view(child, global_config, strings);
        self.surface.render(tree);
    }

    pub fn update_language(&self, _strings: &LanguageStrings) {
        self.refresh_from_engine();
    }

    pub fn paint(&self, _hwnd: isize, _hdc: isize, _right_bound: i32, _palette: &ThemePalette) {
        self.surface.invalidate();
    }

    pub fn on_command(&self, _hwnd: isize, _ctrl_id: u32, _notify_code: u32) -> bool {
        false
    }

    pub fn update_visibility(&self, visible: bool) {
        self.surface.set_visible(visible);
    }

    /// Pure React-style view function: `UI = f(child, global_config, strings)`
    fn view(
        child: Option<&AppProfile<Child>>,
        global_config: &EngineConfig,
        strings: &LanguageStrings,
    ) -> Element<TypingMsg> {
        let is_root = child.is_none();

        // --- Card 1: Thiết lập Bộ gõ tiếng Việt ---
        let mode_options = if is_root {
            vec![
                strings.mode_vietnamese.to_string(),
                strings.mode_english.to_string(),
            ]
        } else {
            vec![
                strings.typing_inherit_global.to_string(),
                strings.mode_vietnamese.to_string(),
                strings.mode_english.to_string(),
            ]
        };
        let mode_idx = match child.map(|c| c.mode) {
            None => usize::from(!global_config.enabled),
            Some(LanguageMode::Inherit) => 0,
            Some(LanguageMode::Vietnamese) => 1,
            Some(LanguageMode::English) => 2,
        };

        let method_options = if is_root {
            vec![
                strings.method_telex.to_string(),
                strings.method_vni.to_string(),
                strings.method_simple_telex.to_string(),
            ]
        } else {
            vec![
                strings.typing_inherit_global.to_string(),
                strings.method_telex.to_string(),
                strings.method_vni.to_string(),
                strings.method_simple_telex.to_string(),
            ]
        };
        let method_idx_of = |m: InputMethod| match m {
            InputMethod::Telex => 0,
            InputMethod::Vni => 1,
            InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2 => 2,
        };
        let method_idx = match child.map(|c| c.method) {
            None => method_idx_of(global_config.method),
            Some(MethodMode::Inherit) => 0,
            Some(MethodMode::Specific(m)) => 1 + method_idx_of(m),
        };

        let charset_options = if is_root {
            vec![
                strings.charset_unicode.to_string(),
                strings.charset_tcvn3.to_string(),
            ]
        } else {
            vec![
                strings.typing_inherit_global.to_string(),
                strings.charset_unicode.to_string(),
                strings.charset_tcvn3.to_string(),
            ]
        };
        let charset_idx_of = |c: Charset| match c {
            Charset::Unicode => 0,
            Charset::Tcvn3 => 1,
        };
        let charset_idx = match child.map(|c| c.charset) {
            None => charset_idx_of(global_config.charset),
            Some(CharsetMode::Inherit) => 0,
            Some(CharsetMode::Specific(c)) => 1 + charset_idx_of(c),
        };

        let card1 = card(elements![
            section_header(strings.typing_group_main),
            row(elements![
                form_group(
                    strings.typing_lbl_mode,
                    select(mode_options, mode_idx, TypingMsg::SelectMode).w_full(),
                )
                .flex_1(),
                form_group(
                    strings.typing_lbl_method,
                    select(method_options, method_idx, TypingMsg::SelectMethod).w_full(),
                )
                .flex_1(),
                form_group(
                    strings.typing_lbl_charset,
                    select(charset_options, charset_idx, TypingMsg::SelectCharset).w_full(),
                )
                .flex_1(),
            ])
            .gap(3),
        ]);

        // --- Card 2: Phím chuyển đổi E ⇄ V ---
        let switch_override = child.and_then(|c| c.switch_key_enabled);
        let switch_effective = switch_override.unwrap_or(global_config.switch_key_enabled);
        let switch_badge_text = match child {
            None => global_config.switch_key.display_text(),
            Some(c) => match &c.switch_key {
                Some(k) => k.display_text(),
                None => format!(
                    "{} ({})",
                    strings.typing_inherit_prefix,
                    global_config.switch_key.display_text()
                ),
            },
        };

        let card2 = card(elements![
            inheritable_switch_row(
                strings.typing_group_switch,
                None,
                is_root,
                switch_override,
                global_config.switch_key_enabled,
                strings,
                TypingMsg::ToggleSwitchKey,
            ),
            row(elements![
                label(strings.typing_lbl_current_key).muted().flex_1(),
                row(elements![
                    badge(
                        switch_badge_text,
                        if switch_effective {
                            Variant::Primary
                        } else {
                            Variant::Secondary
                        },
                    ),
                    button(
                        strings.typing_btn_change_key,
                        Variant::Primary,
                        TypingMsg::OpenSwitchHotkeyDialog,
                    )
                    .size(Size::Sm)
                    .enabled(switch_effective),
                ])
                .items_center()
                .gap(2),
            ])
            .items_center()
            .justify_between()
            .p(2)
            .bg_window()
            .border()
            .rounded(6),
        ]);

        // --- Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả ---
        let card3 = card(elements![
            section_header(strings.typing_group_features),
            inheritable_switch_row(
                strings.check_restore_wrong,
                None,
                is_root,
                child.and_then(|c| c.restore_on_wrong_spelling),
                global_config.restore_on_wrong_spelling,
                strings,
                TypingMsg::ToggleRestoreWrong,
            ),
            divider(),
            inheritable_switch_row(
                strings.check_auto_upper,
                None,
                is_root,
                child.and_then(|c| c.auto_uppercase_first_char),
                global_config.auto_uppercase_first_char,
                strings,
                TypingMsg::ToggleAutoUpper,
            ),
        ]);

        // --- Card 4: Root Capture Hotkey vs Child App Strategy + Target Rules ---
        let card4 = if let Some(c) = child {
            let app_strat_items = vec![
                strings.typing_app_strategy_inherit.to_string(),
                strings.strategy_autocomplete_guard.to_string(),
                strings.strategy_clipboard_paste.to_string(),
            ];
            let sel_app_idx = app_strategy_to_index(c.app_strategy);
            let sub_text = strings
                .typing_app_strategy_sub_format
                .replace("{app}", &c.name);
            let app_strat_desc = app_strategy_description(c.app_strategy, strings);
            let count_text = strings
                .typing_target_count_format
                .replace("{count}", &c.target_rules.len().to_string());

            let rules_block = if c.target_rules.is_empty() {
                let empty_text = match c.app_strategy {
                    None => strings
                        .typing_target_empty_inherit_format
                        .replace("{app}", &c.name),
                    Some(KeyInjectionStrategy::AutocompleteGuard) => strings
                        .typing_target_empty_guard_format
                        .replace("{app}", &c.name),
                    Some(KeyInjectionStrategy::ClipboardPaste) => strings
                        .typing_target_empty_paste_format
                        .replace("{app}", &c.name),
                };
                col(elements![text(empty_text).muted().size(Size::Sm)])
                    .p(3)
                    .bg_window()
                    .border()
                    .rounded_card()
            } else {
                let cards: Vec<Element<TypingMsg>> = c
                    .target_rules
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let tid1 = r.target_id.clone();
                        let tid2 = r.target_id.clone();
                        target_rule_card(
                            i,
                            r,
                            strings,
                            move |idx| TypingMsg::ChangeRuleStrategy(tid1.clone(), idx),
                            TypingMsg::DeleteRule(tid2),
                        )
                    })
                    .collect();
                col(cards).gap(2)
            };

            card(elements![
                section_header(strings.typing_group_target),
                text(strings.typing_target_desc).muted().size(Size::Sm),
                col(elements![
                    row(elements![
                        text(strings.typing_app_strategy_label).bold().w(200),
                        select(app_strat_items, sel_app_idx, TypingMsg::ChangeAppStrategy).flex_1(),
                    ])
                    .items_center()
                    .gap(2),
                    text(sub_text).muted().size(Size::Sm),
                    divider(),
                    text(app_strat_desc).muted().size(Size::Sm),
                ])
                .p(3)
                .gap(2)
                .bg_window()
                .border()
                .rounded_card(),
                divider(),
                row(elements![
                    text(strings.typing_exceptions_title).bold().accent().flex_1(),
                    badge(count_text, Variant::Primary),
                ])
                .items_center()
                .justify_between(),
                text(strings.typing_exceptions_desc).muted().size(Size::Sm),
                rules_block,
            ])
        } else {
            let cap_enabled = global_config.target_capture_enabled;
            let cap_hotkey_txt = global_config.target_capture_hotkey.display_text();

            card(elements![
                row(elements![
                    checkbox("", cap_enabled, TypingMsg::ToggleTargetCapture),
                    text(strings.typing_group_target_hotkey)
                        .bold()
                        .accent(),
                ])
                .items_center()
                .gap(2),
                col(elements![
                    row(elements![
                        text(strings.typing_lbl_current_key).bold().flex_1(),
                        row(elements![
                            badge(
                                cap_hotkey_txt,
                                if cap_enabled {
                                    Variant::Primary
                                } else {
                                    Variant::Secondary
                                },
                            ),
                            button(
                                strings.typing_btn_change_key,
                                Variant::Primary,
                                TypingMsg::OpenCaptureHotkeyDialog,
                            )
                            .size(Size::Sm)
                            .enabled(cap_enabled),
                        ])
                        .items_center()
                        .gap(2),
                    ])
                    .items_center()
                    .justify_between(),
                    col(elements![
                        text(strings.typing_target_step_guide)
                            .muted()
                            .size(Size::Sm),
                    ])
                    .p(2)
                    .bg_card()
                    .border()
                    .rounded(6),
                ])
                .p(3)
                .gap(2)
                .bg_window()
                .border()
                .rounded_card(),
            ])
        };

        col(elements![card1, card2, card3, card4]).gap(3).pb(3)
    }

    fn handle_message(parent_hwnd: isize, surface: &Surface<TypingMsg>, msg: TypingMsg) {
        use crate::ui::controller::{self, EditTarget};

        match msg {
            TypingMsg::SelectMode(sel) => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => cfg.enabled = sel == 0,
                    EditTarget::Child(prof) => {
                        prof.mode = match sel {
                            1 => LanguageMode::Vietnamese,
                            2 => LanguageMode::English,
                            _ => LanguageMode::Inherit,
                        };
                    }
                });
                if let Some(is_viet) = controller::with_engine(|e| e.effective_config().enabled) {
                    crate::ui::update_tray_icon(is_viet);
                }
            }
            TypingMsg::SelectMethod(sel) => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => {
                        if let Some(m) = decode_input_method(sel, cfg.method) {
                            cfg.method = m;
                        }
                    }
                    EditTarget::Child(prof) => {
                        let current = match prof.method {
                            MethodMode::Specific(m) => m,
                            MethodMode::Inherit => InputMethod::SimpleTelex1,
                        };
                        prof.method = sel
                            .checked_sub(1)
                            .and_then(|idx| decode_input_method(idx, current))
                            .map_or(MethodMode::Inherit, MethodMode::Specific);
                    }
                });
            }
            TypingMsg::SelectCharset(sel) => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => {
                        cfg.charset = match sel {
                            1 => Charset::Tcvn3,
                            _ => Charset::Unicode,
                        };
                    }
                    EditTarget::Child(prof) => {
                        prof.charset = match sel {
                            1 => CharsetMode::Specific(Charset::Unicode),
                            2 => CharsetMode::Specific(Charset::Tcvn3),
                            _ => CharsetMode::Inherit,
                        };
                    }
                });
            }
            TypingMsg::ToggleSwitchKey(next_opt) => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => {
                        cfg.switch_key_enabled = next_opt.unwrap_or(!cfg.switch_key_enabled);
                    }
                    EditTarget::Child(prof) => {
                        prof.switch_key_enabled = next_opt;
                    }
                });
                controller::refresh_keys_overview();
            }
            TypingMsg::OpenSwitchHotkeyDialog => {
                crate::ui::views::dialogs::show_hotkey_dialog(parent_hwnd);
                return;
            }
            TypingMsg::ToggleRestoreWrong(next_opt) => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => {
                        cfg.restore_on_wrong_spelling =
                            next_opt.unwrap_or(!cfg.restore_on_wrong_spelling);
                    }
                    EditTarget::Child(prof) => {
                        prof.restore_on_wrong_spelling = next_opt;
                    }
                });
            }
            TypingMsg::ToggleAutoUpper(next_opt) => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => {
                        cfg.auto_uppercase_first_char =
                            next_opt.unwrap_or(!cfg.auto_uppercase_first_char);
                    }
                    EditTarget::Child(prof) => {
                        prof.auto_uppercase_first_char = next_opt;
                    }
                });
            }
            TypingMsg::ToggleTargetCapture(enabled) => {
                controller::save_target_capture_enabled(enabled);
            }
            TypingMsg::OpenCaptureHotkeyDialog => {
                crate::ui::views::dialogs::show_target_capture_hotkey_dialog(parent_hwnd);
                return;
            }
            TypingMsg::ChangeAppStrategy(sel_idx) => {
                let strat = index_to_app_strategy(sel_idx);
                controller::update_app_key_injection_strategy(strat);
                return;
            }
            TypingMsg::ChangeRuleStrategy(tid, sel_idx) => {
                let strat = index_to_strategy(sel_idx);
                controller::update_target_rule_strategy(&tid, strat);
                return;
            }
            TypingMsg::DeleteRule(tid) => {
                controller::delete_target_rule(&tid);
                return;
            }
        }

        // Re-render Surface with updated state
        let snap = controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((child, cfg)) = snap {
            let strings = crate::language::current();
            surface.render(Self::view(child.as_ref(), &cfg, strings));
        }
    }
}
