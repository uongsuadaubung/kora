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
use crate::ui::views::control_panel::tab_keys::{
    TABLE_HEIGHT, TABLE_WIDTH, TABLE_X, TABLE_Y, TITLE_HEIGHT, TITLE_X, TITLE_Y,
};
use crate::ui::prelude::*;

pub struct OverviewPanel {
    layout: CardLayout,
    pub lbl_status: Label,
    pub list: ActionTable,
}

impl OverviewPanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);

        ly.track_i18n(
            Label::create(
                parent,
                strings.overview_title,
                TITLE_X,
                TITLE_Y,
                240,
                TITLE_HEIGHT,
                hfont_bold,
            )?,
            |s| s.overview_title,
        );

        let lbl_status = ly.track(Label::create_right(
            parent,
            strings.overview_healthy,
            TITLE_X + 245,
            TITLE_Y + 2,
            TABLE_WIDTH - 245,
            TITLE_HEIGHT,
            hfont_normal,
        )?);

        let list = ly.track_table_i18n(
            ActionTable::create_styled(
                parent,
                0,
                TABLE_X,
                TABLE_Y,
                TABLE_WIDTH,
                TABLE_HEIGHT,
                hfont_normal,
                &[
                    (strings.col_overview_key, 115, ColumnStyle::KeyBadge),
                    (strings.col_overview_feature, 90, ColumnStyle::Default),
                    (strings.col_overview_action, 125, ColumnStyle::Default),
                    (strings.col_overview_origin, 85, ColumnStyle::Default),
                    (strings.col_overview_status, 115, ColumnStyle::Default),
                ],
            )?,
            |s| {
                vec![
                    s.col_overview_key,
                    s.col_overview_feature,
                    s.col_overview_action,
                    s.col_overview_origin,
                    s.col_overview_status,
                ]
            },
        );

        Some(Self {
            layout: ly,
            lbl_status,
            list,
        })
    }

    /// Returns the window handles belonging to this panel
    pub fn hwnds(&self) -> &[isize] {
        self.layout.hwnds()
    }

    /// Updates localized strings for headers and column titles
    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.layout.update_language(strings);
    }

    /// Populates the consolidated ActionTable and updates the diagnostic header
    #[allow(clippy::too_many_arguments)]
    pub fn populate(
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

            let row = TableRow::new()
                .key(switch_key.display_text())
                .text(strings.feature_switch_key)
                .text(strings.action_toggle_vietnamese)
                .text(get_origin_label(switch_key_origin))
                .cell(TableCell::Badge(status_badge));
            rows.push(row);
        }

        // 2B. Key Remappings
        for r in remaps {
            let from_vk = r.from_vk;
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
                .cell(TableCell::Badge(status_badge));
            rows.push(row);
        }

        // 2C. Disabled Keys
        for d in disabled {
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
                .cell(TableCell::Badge(status_badge));
            rows.push(row);
        }

        // 2D. Key Macros
        for m in key_macros {
            let trigger_vk = m.trigger_vk;
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
                .cell(TableCell::Badge(status_badge));
            rows.push(row);
        }

        self.list.set_rows(rows);
        diag
    }
}
