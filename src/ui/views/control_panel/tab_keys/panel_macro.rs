//! Sub-panel: Key Macro (Phím Macro - Hardware Key Automation)
//! Allows mapping a physical trigger key to an automation sequence (press/down/up/delay/text).

use crate::engine::profile::ResolvedKeyMacroItem;
use crate::ui::views::control_panel::tab_keys::RuleSubPanel;
use crate::ui_framework::components::action_table::{ColumnStyle, TableRow};
use std::ops::Deref;

#[repr(transparent)]
pub struct MacroPanel(pub RuleSubPanel);

impl Deref for MacroPanel {
    type Target = RuleSubPanel;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MacroPanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        RuleSubPanel::create(
            parent,
            hfont_normal,
            hfont_bold,
            |s| s.key_macro_table_title,
            |s| s.btn_add_key_macro,
            &[
                (strings.col_key_macro_trigger, 120, ColumnStyle::KeyBadge),
                (strings.col_key_macro_name, 165, ColumnStyle::Default),
                (strings.col_origin, 120, ColumnStyle::Default),
                (strings.col_action, 125, ColumnStyle::Action),
            ],
            move |hwnd| crate::ui::views::dialogs::show_key_macro_dialog(hwnd),
            |target, new_val| match target {
                crate::ui::controller::EditTarget::Global(cfg) => {
                    cfg.key_macro_enabled = new_val.unwrap_or(true);
                }
                crate::ui::controller::EditTarget::Child(p) => {
                    p.key_macro_enabled = new_val;
                }
            },
        )
        .map(Self)
    }

    /// Populates Key Macro ActionTable with resolved Live Inheritance items
    pub fn populate(&self, items: &[ResolvedKeyMacroItem], parent_hwnd: isize) {
        self.list.clear();
        let strings = crate::language::current();

        let rows: Vec<TableRow> = items
            .iter()
            .map(|item| {
                let trigger_vk = item.trigger_vk;
                let trigger_name = item.trigger_name.clone();
                let name = item.name.clone();

                let actions = crate::ui::controller::build_rule_actions(
                    item.origin,
                    item.is_active,
                    Some(move || {
                        crate::ui::views::dialogs::show_key_macro_dialog_with_key(
                            parent_hwnd,
                            Some(trigger_vk),
                        );
                    }),
                    move |mute| Self::toggle_mute(trigger_vk, mute),
                    move || Self::trigger_delete(parent_hwnd, trigger_vk, &trigger_name, &name),
                );

                TableRow::new()
                    .key(&item.trigger_name)
                    .text(&item.name)
                    .text(item.origin.localized_badge_label(strings))
                    .actions(actions)
            })
            .collect();

        self.list.set_rows(rows);
    }

    pub fn toggle_mute(trigger_vk: u32, mute: bool) {
        crate::ui::controller::set_key_macro_muted(trigger_vk, mute);
    }

    pub fn trigger_delete(parent: isize, trigger_vk: u32, trigger_name: &str, name: &str) {
        let message = crate::language::current()
            .confirm_delete_key_macro_msg
            .replace("{key}", trigger_name)
            .replace("{name}", name);
        crate::ui::views::dialogs::show_delete_confirm(parent, message, move || {
            crate::ui::controller::delete_key_macro(trigger_vk);
        });
    }
}
