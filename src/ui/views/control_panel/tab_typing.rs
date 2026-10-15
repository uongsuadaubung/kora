//! Tab 0: Typing & Input Method Configuration (Bộ gõ)
//! Uses CardLayout (1-line row builders, auto-HWND & auto-i18n), Label::as_pill_badge,
//! and semantic component event matchers.

use crate::engine::EngineConfig;
use crate::engine::config::InputMethod;
use crate::engine::profile::{AppProfile, Child, LanguageMode, MethodMode};
use crate::language::LanguageStrings;
use crate::ui::prelude::*;

/// The single "Simple Telex" combo entry maps to both SimpleTelex1/2: keep whichever is stored.
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

pub struct TabTyping {
    layout: CardLayout,
    // Card 1: Thiết lập Bộ gõ tiếng Việt
    pub combo_mode: ComboBox,
    pub combo_method: ComboBox,

    // Card 2: Phím chuyển đổi E ⇄ V (Checkbox title toggles switch shortcut)
    pub check_switch_key: CheckBox<Inheritable>,
    pub lbl_switch_badge: Label<PillBadge>,
    pub btn_switch_key: PushButton,

    // Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả
    pub check_restore_wrong: CheckBox<Inheritable>,
    pub check_auto_upper: CheckBox<Inheritable>,
}

impl TabTyping {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);

        // --- Card 1: Thiết lập Bộ gõ tiếng Việt ---
        let (combo_mode, combo_method) = ly.card_with_pad(12, 20, |card| {
            card.section_title(|s| s.typing_group_main, 4)?;
            let ((_, combo_mode), (_, combo_method)) = card.two_col_combos(
                24,
                0,
                true,
                (|s| s.typing_lbl_mode, |s| vec![s.mode_vietnamese, s.mode_english]),
                (
                    |s| s.typing_lbl_method,
                    |s| vec![s.method_telex, s.method_vni, s.method_simple_telex],
                ),
            )?;
            Some((combo_mode, combo_method))
        })?;

        // --- Card 2: Phím chuyển đổi E ⇄ V ---
        let (check_switch_key, lbl_switch_badge, btn_switch_key) =
            ly.card_with_pad(6, 16, |card| {
                let check_switch_key =
                    card.inheritable_checkbox_row(|s| s.typing_group_switch, 24, 4, true)?;

                let y_sw_row = card.row(30, 0);
                card.label_at(
                    |s| s.typing_lbl_current_key,
                    card.x(),
                    y_sw_row + 6,
                    124,
                    20,
                    true,
                )?;

                let btn_w = 86;
                let btn_x = card.right() - btn_w - 4;
                let badge_x = card.x() + 130;
                let badge_max_w = btn_x - badge_x - 10;

                let lbl_switch_badge = card.track(
                    Label::create_centered(
                        parent,
                        "Ctrl + Shift",
                        badge_x,
                        y_sw_row,
                        120,
                        30,
                        hfont_bold,
                    )?
                    .as_pill_badge(120, badge_max_w),
                );

                let btn_switch_key = card.button_at(
                    |s| s.typing_btn_change_key,
                    btn_x,
                    y_sw_row,
                    btn_w,
                    30,
                    true,
                    ButtonVariant::Primary,
                )?;
                Some((check_switch_key, lbl_switch_badge, btn_switch_key))
            })?;

        // --- Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả ---
        let (check_restore_wrong, check_auto_upper) = ly.card_with_pad(8, 12, |card| {
            card.section_title(|s| s.typing_group_features, 6)?;
            let check_restore_wrong =
                card.inheritable_checkbox_row(|s| s.check_restore_wrong, 22, 4, false)?;
            let check_auto_upper =
                card.inheritable_checkbox_row(|s| s.check_auto_upper, 22, 0, false)?;
            Some((check_restore_wrong, check_auto_upper))
        })?;

        let tab = Self {
            layout: ly,
            combo_mode,
            combo_method,
            check_switch_key,
            lbl_switch_badge,
            btn_switch_key,
            check_restore_wrong,
            check_auto_upper,
        };
        tab.update_switch_controls_state();
        Some(tab)
    }

    /// Dynamically enables or disables switch-key controls based on `check_switch_key`
    pub fn update_switch_controls_state(&self) {
        let is_enabled = self.check_switch_key.is_checked();
        self.btn_switch_key.set_enabled(is_enabled);
        self.lbl_switch_badge.set_enabled(is_enabled);
    }

    /// Loads profile settings (`None` = Root/Global, `Some(child)` = Child profile with Inherit options)
    pub fn load_profile(&self, child: Option<&AppProfile<Child>>, global_config: &EngineConfig) {
        let strings = crate::language::current();
        let is_root = child.is_none();
        let offset = usize::from(is_root);

        let all_modes = [
            strings.typing_inherit_global,
            strings.mode_vietnamese,
            strings.mode_english,
        ];
        let mode_idx = match child.map(|c| c.mode) {
            None => usize::from(!global_config.enabled),
            Some(LanguageMode::Inherit) => 0,
            Some(LanguageMode::Vietnamese) => 1,
            Some(LanguageMode::English) => 2,
        };
        self.combo_mode
            .reset_items(&all_modes[offset..], Some(mode_idx));

        let all_methods = [
            strings.typing_inherit_global,
            strings.method_telex,
            strings.method_vni,
            strings.method_simple_telex,
        ];
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
        self.combo_method
            .reset_items(&all_methods[offset..], Some(method_idx));

        self.check_switch_key.set_inherited(
            is_root,
            child.and_then(|c| c.switch_key_enabled),
            global_config.switch_key_enabled,
        );
        self.update_switch_controls_state();

        let badge_text = match child {
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
        self.lbl_switch_badge.set_text(&badge_text);
        self.btn_switch_key.set_text(strings.typing_btn_change_key);

        self.check_restore_wrong.set_inherited(
            is_root,
            child.and_then(|c| c.restore_on_wrong_spelling),
            global_config.restore_on_wrong_spelling,
        );
        self.check_auto_upper.set_inherited(
            is_root,
            child.and_then(|c| c.auto_uppercase_first_char),
            global_config.auto_uppercase_first_char,
        );
    }

    /// Saves controls back into the active target (`Global` config or `Child` profile).
    fn sync_to_target(&self, mut target: crate::ui::controller::EditTarget<'_>) {
        use crate::ui::controller::EditTarget;
        match &mut target {
            EditTarget::Global(config) => {
                if let Some(sel) = self.combo_mode.get_selected() {
                    config.enabled = sel == 0;
                }
                if let Some(m) = self
                    .combo_method
                    .get_selected()
                    .and_then(|sel| decode_input_method(sel, config.method))
                {
                    config.method = m;
                }
            }
            EditTarget::Child(profile) => {
                if let Some(sel) = self.combo_mode.get_selected() {
                    profile.mode = match sel {
                        1 => LanguageMode::Vietnamese,
                        2 => LanguageMode::English,
                        _ => LanguageMode::Inherit,
                    };
                }
                if let Some(sel) = self.combo_method.get_selected() {
                    let current = match profile.method {
                        MethodMode::Specific(m) => m,
                        MethodMode::Inherit => InputMethod::SimpleTelex1,
                    };
                    profile.method = sel
                        .checked_sub(1)
                        .and_then(|idx| decode_input_method(idx, current))
                        .map_or(MethodMode::Inherit, MethodMode::Specific);
                }
            }
        }
        target.sync_checkbox(
            &self.check_switch_key,
            |c| &mut c.switch_key_enabled,
            |p| &mut p.switch_key_enabled,
        );
        target.sync_checkbox(
            &self.check_restore_wrong,
            |c| &mut c.restore_on_wrong_spelling,
            |p| &mut p.restore_on_wrong_spelling,
        );
        target.sync_checkbox(
            &self.check_auto_upper,
            |c| &mut c.auto_uppercase_first_char,
            |p| &mut p.auto_uppercase_first_char,
        );
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);

        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((child, cfg)) = snap {
            self.load_profile(child.as_ref(), &cfg);
        }
    }

    pub fn paint(&self, _hwnd: isize, hdc: isize, right_bound: i32, _palette: &ThemePalette) {
        self.layout.paint_cards(hdc, layout::CARD_X, right_bound);
    }

    pub fn on_command(&self, hwnd: isize, ctrl_id: u32, notify_code: u32) -> bool {
        if self.btn_switch_key.is_clicked(ctrl_id, notify_code) {
            if self.btn_switch_key.is_enabled() {
                crate::ui::views::dialogs::show_hotkey_dialog(hwnd);
            }
            return true;
        }

        let is_mode_changed = self.combo_mode.is_changed(ctrl_id, notify_code);
        let is_method_changed = self.combo_method.is_changed(ctrl_id, notify_code);
        let is_sw_toggled = self.check_switch_key.is_toggled(ctrl_id, notify_code);
        let is_feat_toggled = self.check_restore_wrong.is_toggled(ctrl_id, notify_code)
            || self.check_auto_upper.is_toggled(ctrl_id, notify_code);

        if is_mode_changed || is_method_changed || is_sw_toggled || is_feat_toggled {
            if is_sw_toggled {
                self.update_switch_controls_state();
            }
            crate::ui::controller::edit_selected(|target| self.sync_to_target(target));
            if is_mode_changed
                && let Some(is_viet) =
                    crate::ui::controller::with_engine(|e| e.effective_config().enabled)
            {
                crate::ui::update_tray_icon(is_viet);
            }
            if is_sw_toggled {
                crate::ui::controller::refresh_keys_overview();
            }
            return true;
        }

        false
    }

    pub fn update_visibility(&self, visible: bool) {
        self.layout.set_visible(visible);
    }
}
