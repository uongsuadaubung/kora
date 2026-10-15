//! Sub-panel: Key Remapping (Đổi phím)
//! Allows mapping one hardware key to another key.

use crate::engine::profile::ResolvedRemapItem;
use crate::ui::views::control_panel::tab_keys::RuleSubPanel;
use crate::ui::prelude::*;
use std::ops::Deref;

#[repr(transparent)]
pub struct RemapPanel(pub RuleSubPanel);

impl Deref for RemapPanel {
    type Target = RuleSubPanel;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RemapPanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        RuleSubPanel::create(
            parent,
            hfont_normal,
            hfont_bold,
            |s| s.remap_table_title,
            |s| s.btn_add_remap,
            &[
                (strings.col_remap_from, 145, ColumnStyle::KeyBadgeWithArrow),
                (strings.col_remap_to, 135, ColumnStyle::KeyBadge),
                (strings.col_origin, 125, ColumnStyle::Default),
                (strings.col_action, 125, ColumnStyle::Action),
            ],
            |s| vec![s.col_remap_from, s.col_remap_to, s.col_origin, s.col_action],
        )
        .map(Self)
    }

    /// Populates Remap ActionTable with resolved Live Inheritance items
    pub fn populate(&self, items: &[ResolvedRemapItem], parent_hwnd: isize) {
        self.list.clear();
        let strings = crate::language::current();

        let rows: Vec<TableRow> = items
            .iter()
            .map(|item| {
                let from_vk = item.from_vk;
                let to_vk = item.to_vk;
                let from_name = item.from_name.clone();
                let to_name = item.to_name.clone();

                let actions = crate::ui::controller::build_rule_actions(
                    item.origin,
                    item.is_active,
                    Some(move || {
                        crate::ui::views::dialogs::show_remap_dialog_with_keys(
                            parent_hwnd,
                            Some(from_vk),
                            Some(to_vk),
                        );
                    }),
                    move |mute| Self::toggle_mute(from_vk, mute),
                    move || Self::trigger_delete(parent_hwnd, from_vk, &from_name, &to_name),
                );

                TableRow::new()
                    .key_with_arrow(&item.from_name)
                    .key(&item.to_name)
                    .text(item.origin.localized_badge_label(strings))
                    .actions(actions)
            })
            .collect();

        self.list.set_rows(rows);
    }

    pub fn toggle_mute(from_vk: u32, mute: bool) {
        crate::ui::controller::set_remap_muted(from_vk, mute);
    }

    pub fn trigger_delete(parent: isize, from_vk: u32, from_name: &str, to_name: &str) {
        let message = crate::language::current()
            .confirm_delete_remap_msg
            .replace("{from}", from_name)
            .replace("{to}", to_name);
        crate::ui::views::dialogs::show_delete_confirm(parent, message, move || {
            crate::ui::controller::delete_remap(from_vk);
        });
    }
}

