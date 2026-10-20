//! Sub-panel: Key Disabling (Khóa phím)
//! Allows blocking specific hardware keys from reaching the system or apps.

use crate::engine::profile::ResolvedDisableItem;
use crate::ui::prelude::*;
use crate::ui::views::control_panel::tab_keys::RuleSubPanel;
use std::ops::Deref;

#[repr(transparent)]
pub struct DisablePanel(pub RuleSubPanel);

impl Deref for DisablePanel {
    type Target = RuleSubPanel;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DisablePanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        RuleSubPanel::create(
            parent,
            hfont_normal,
            hfont_bold,
            |s| s.disable_table_title,
            |s| s.btn_add_disable,
            &[
                (strings.col_disable_key, 125, ColumnStyle::DangerKeyBadge),
                (strings.col_status, 165, ColumnStyle::Default),
                (strings.col_origin, 135, ColumnStyle::Default),
                (strings.col_action, 105, ColumnStyle::Action),
            ],
            |s| vec![s.col_disable_key, s.col_status, s.col_origin, s.col_action],
        )
        .map(Self)
    }

    /// Populates Disable ActionTable with resolved Live Inheritance items
    pub fn populate(&self, items: &[ResolvedDisableItem], parent_hwnd: isize) {
        self.list.clear();
        let strings = crate::language::current();

        let rows: Vec<TableRow> = items
            .iter()
            .map(|item| {
                let vk = item.vk;
                let key_name = item.key_name.clone();
                let status = if item.is_active {
                    strings.status_blocked
                } else {
                    strings.status_unblocked
                };
                let key_label = format!("🚫 {}", item.key_name);

                let actions = crate::ui::controller::build_rule_actions(
                    item.origin,
                    item.is_active,
                    None::<fn()>,
                    move |mute| Self::toggle_mute(vk, mute),
                    move || Self::trigger_delete(parent_hwnd, vk, &key_name),
                );

                TableRow::new()
                    .danger_key(&key_label)
                    .text(status)
                    .text(item.origin.localized_badge_label(strings))
                    .actions(actions)
            })
            .collect();

        self.list.set_rows(rows);
    }

    pub fn toggle_mute(vk: u32, mute: bool) {
        crate::ui::controller::set_disable_muted(vk, mute);
    }

    pub fn trigger_delete(parent: isize, vk: u32, key_name: &str) {
        let message = crate::language::current()
            .confirm_delete_disable_msg
            .replace("{key}", key_name);
        crate::ui::views::dialogs::show_delete_confirm(parent, message, move || {
            crate::ui::controller::delete_disable(vk);
        });
    }
}
