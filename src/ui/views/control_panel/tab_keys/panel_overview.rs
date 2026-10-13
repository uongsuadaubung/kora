//! Sub-panel: Key Customization Overview & Conflict Diagnostics (Tổng quan & Chẩn đoán phím)
//!
//! Provides a consolidated single-pane dashboard of all keyboard interventions
//! (Remaps, Disables, Key Macros, and Mode Switch Hotkey), highlighting cross-feature
//! execution conflicts and allowing 1-click navigation to the respective editor sub-tab.

use crate::engine::config::hotkey::vk_to_name;
use crate::engine::config::Hotkey;
use crate::engine::profile::{
    check_profile_conflicts, ConflictFeature, KeyDiagnostics, ResolvedDisableItem,
    ResolvedKeyMacroItem, ResolvedRemapItem, RuleOrigin,
};
use crate::ui::components::action_table::{
    ActionTable, ActionVariant, BadgeConfig, ColumnStyle, TableAction, TableCell, TableRow,
};
use crate::ui::components::Label;
use crate::ui::tokens::id::*;
use crate::ui::views::control_panel::tab_keys::{
    TABLE_HEIGHT, TABLE_WIDTH, TABLE_X, TABLE_Y, TITLE_HEIGHT, TITLE_X, TITLE_Y,
};
use std::sync::Arc;

pub struct OverviewPanel {
    pub lbl_title: Label,
    pub lbl_status: Label,
    pub list: ActionTable,
}

impl OverviewPanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let lbl_title = Label::create_with_id(
            parent,
            IDC_LABEL_OVERVIEW_TITLE,
            strings.overview_title,
            TITLE_X,
            TITLE_Y,
            240,
            TITLE_HEIGHT,
            hfont_bold,
        )?;

        let lbl_status = Label::create_right_with_id(
            parent,
            IDC_LABEL_OVERVIEW_STATUS,
            strings.overview_healthy,
            TITLE_X + 245,
            TITLE_Y + 2,
            TABLE_WIDTH - 245,
            TITLE_HEIGHT,
            hfont_normal,
        )?;

        let list = ActionTable::create_styled(
            parent,
            IDC_LIST_OVERVIEW,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            &[
                (strings.col_overview_key, 105, ColumnStyle::KeyBadge),
                (strings.col_overview_feature, 75, ColumnStyle::Default),
                (strings.col_overview_action, 110, ColumnStyle::Default),
                (strings.col_overview_origin, 75, ColumnStyle::Default),
                (strings.col_overview_status, 110, ColumnStyle::Default),
                (strings.col_action, 55, ColumnStyle::Action),
            ],
        )?;

        Some(Self {
            lbl_title,
            lbl_status,
            list,
        })
    }

    /// Returns the window handles belonging to this panel
    pub fn hwnds(&self) -> [isize; 3] {
        [
            self.lbl_title.hwnd(),
            self.lbl_status.hwnd(),
            self.list.hwnd(),
        ]
    }

    /// Updates localized strings for headers and column titles
    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.lbl_title.set_text(strings.overview_title);
        self.list.set_columns(&[
            (strings.col_overview_key, 105, ColumnStyle::KeyBadge),
            (strings.col_overview_feature, 75, ColumnStyle::Default),
            (strings.col_overview_action, 110, ColumnStyle::Default),
            (strings.col_overview_origin, 75, ColumnStyle::Default),
            (strings.col_overview_status, 110, ColumnStyle::Default),
            (strings.col_action, 55, ColumnStyle::Action),
        ]);
    }

    /// Populates the consolidated ActionTable and updates the diagnostic header
    #[allow(clippy::too_many_arguments)]
    pub fn populate<F: Fn(usize) + Send + Sync + 'static>(
        &self,
        remaps: &[ResolvedRemapItem],
        remap_enabled: bool,
        disabled: &[ResolvedDisableItem],
        disable_enabled: bool,
        key_macros: &[ResolvedKeyMacroItem],
        key_macro_enabled: bool,
        switch_key: Hotkey,
        switch_key_enabled: bool,
        switch_key_origin: RuleOrigin,
        on_navigate: F,
    ) -> KeyDiagnostics {
        let strings = crate::language::current();
        let diag = check_profile_conflicts(
            remaps,
            remap_enabled,
            disabled,
            disable_enabled,
            key_macros,
            key_macro_enabled,
            switch_key,
            switch_key_enabled,
        );

        // 1. Update Diagnostics Header
        if diag.is_healthy() {
            self.lbl_status.set_text(strings.overview_healthy);
        } else {
            let conflict_count = diag.conflicts.len();
            let text = strings
                .overview_conflicts_format
                .replace("{count}", &conflict_count.to_string());
            self.lbl_status.set_text(&text);
        }

        // 2. Build Consolidated Master Table Rows
        let nav = Arc::new(on_navigate);
        let mut rows = Vec::new();
        let get_origin_label = |origin: RuleOrigin| -> &'static str {
            match origin {
                RuleOrigin::Root => strings.origin_root,
                _ => origin.localized_badge_label(strings),
            }
        };

        // 2A. Mode Switch Hotkey
        if switch_key_enabled {
            let switch_conflict = diag
                .conflicts
                .iter()
                .find(|c| c.shadowed == ConflictFeature::SwitchKey);
            let status_badge = if let Some(c) = switch_conflict {
                BadgeConfig::warning(
                    strings
                        .status_shadowed_by
                        .replace("{feature}", c.winner.localized_name(strings)),
                )
            } else {
                BadgeConfig::success(strings.status_active)
            };

            let nav_clone = nav.clone();
            let row = TableRow::new()
                .key(switch_key.display_text())
                .text(strings.feature_switch_key)
                .text(strings.action_toggle_vietnamese)
                .text(get_origin_label(switch_key_origin))
                .cell(TableCell::Badge(status_badge))
                .cell(TableCell::Actions(vec![TableAction::new(
                    strings.action_jump_to_feature,
                    ActionVariant::Primary,
                )
                .with_callback(move || nav_clone(0))]));
            rows.push(row);
        }

        // 2B. Key Remappings
        for r in remaps {
            let from_vk = r.from_vk;
            let nav_clone = nav.clone();
            let is_shadowed = diag.is_shadowed(from_vk, ConflictFeature::Remap);

            let status_badge = if let Some(c) = is_shadowed {
                BadgeConfig::warning(
                    strings
                        .status_shadowed_by
                        .replace("{feature}", c.winner.localized_name(strings)),
                )
            } else if r.is_active && remap_enabled {
                BadgeConfig::success(strings.status_active)
            } else {
                BadgeConfig::secondary(strings.status_disabled)
            };

            let row = TableRow::new()
                .key(&r.from_name)
                .text(strings.feature_remap)
                .text(format!("➜ {}", r.to_name))
                .text(get_origin_label(r.origin))
                .cell(TableCell::Badge(status_badge))
                .cell(TableCell::Actions(vec![TableAction::new(
                    strings.action_jump_to_feature,
                    ActionVariant::Primary,
                )
                .with_callback(move || nav_clone(1))]));
            rows.push(row);
        }

        // 2C. Disabled Keys
        for d in disabled {
            let nav_clone = nav.clone();
            let status_badge = if d.is_active && disable_enabled {
                BadgeConfig::success(strings.status_active)
            } else {
                BadgeConfig::secondary(strings.status_disabled)
            };

            let row = TableRow::new()
                .danger_key(&d.key_name)
                .text(strings.feature_disable)
                .text(strings.status_blocked)
                .text(get_origin_label(d.origin))
                .cell(TableCell::Badge(status_badge))
                .cell(TableCell::Actions(vec![TableAction::new(
                    strings.action_jump_to_feature,
                    ActionVariant::Primary,
                )
                .with_callback(move || nav_clone(2))]));
            rows.push(row);
        }

        // 2D. Key Macros
        for m in key_macros {
            let trigger_vk = m.trigger_vk;
            let nav_clone = nav.clone();
            let is_shadowed = diag.is_shadowed(trigger_vk, ConflictFeature::KeyMacro);

            let status_badge = if let Some(c) = is_shadowed {
                BadgeConfig::warning(
                    strings
                        .status_shadowed_by
                        .replace("{feature}", c.winner.localized_name(strings)),
                )
            } else if m.is_active && key_macro_enabled {
                BadgeConfig::success(strings.status_active)
            } else {
                BadgeConfig::secondary(strings.status_disabled)
            };

            let trigger_display = if !m.trigger_name.is_empty() {
                m.trigger_name.clone()
            } else {
                let name = vk_to_name(m.trigger_vk);
                if !name.is_empty() {
                    name.to_string()
                } else {
                    format!("VK {:#04X}", m.trigger_vk)
                }
            };

            let row = TableRow::new()
                .key(trigger_display)
                .text(strings.feature_key_macro)
                .text(&m.name)
                .text(get_origin_label(m.origin))
                .cell(TableCell::Badge(status_badge))
                .cell(TableCell::Actions(vec![TableAction::new(
                    strings.action_jump_to_feature,
                    ActionVariant::Primary,
                )
                .with_callback(move || nav_clone(3))]));
            rows.push(row);
        }

        self.list.set_rows(rows);
        diag
    }
}
