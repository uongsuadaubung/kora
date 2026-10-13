//! Sub-panel: Key Macro (Phím Macro - Hardware Key Automation)
//! Allows mapping a physical trigger key to an automation sequence (press/down/up/delay/text).

use crate::engine::profile::{ResolvedKeyMacroItem, RuleOrigin};
use crate::ui::components::action_table::{
    ActionTable, ActionVariant, ColumnStyle, TableAction, TableRow,
};
use crate::ui::components::{ButtonVariant, CheckBox, PushButton};
use crate::ui::tokens::id::*;
use crate::ui::views::control_panel::tab_keys::{
    BTN_HEIGHT, BTN_WIDTH, BTN_X, BTN_Y, TABLE_HEIGHT, TABLE_WIDTH, TABLE_X, TABLE_Y, TITLE_HEIGHT,
    TITLE_WIDTH, TITLE_X, TITLE_Y,
};

pub struct MacroPanel {
    pub check_enabled: CheckBox,
    pub btn_add: PushButton,
    pub list: ActionTable,
}

impl MacroPanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let check_enabled = CheckBox::create(
            parent,
            IDC_CHECK_KEY_MACRO_ENABLED,
            strings.key_macro_table_title,
            TITLE_X,
            TITLE_Y,
            TITLE_WIDTH,
            TITLE_HEIGHT,
            hfont_bold,
        )?;
        check_enabled.set_checked(false);

        let btn_add = PushButton::create_with_variant(
            parent,
            IDC_BTN_ADD_KEY_MACRO,
            strings.btn_add_key_macro,
            BTN_X,
            BTN_Y,
            BTN_WIDTH,
            BTN_HEIGHT,
            hfont_bold,
            ButtonVariant::Primary,
            None,
        )?;

        let list = ActionTable::create_styled(
            parent,
            IDC_LIST_KEY_MACRO,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            &[
                (strings.col_key_macro_trigger, 120, ColumnStyle::KeyBadge),
                (strings.col_key_macro_name, 165, ColumnStyle::Default),
                (strings.col_origin, 120, ColumnStyle::Default),
                (strings.col_action, 125, ColumnStyle::Action),
            ],
        )?;

        let panel = Self {
            check_enabled,
            btn_add,
            list,
        };
        panel.update_controls_state();
        Some(panel)
    }

    /// Dynamically enables or disables the Macro table and Add button based on `check_enabled`
    pub fn update_controls_state(&self) {
        let is_enabled = self.check_enabled.is_checked();
        self.btn_add.set_enabled(is_enabled);
        self.list.set_enabled(is_enabled);
    }

    /// Returns the window handles belonging to this panel
    pub fn hwnds(&self) -> [isize; 3] {
        [
            self.check_enabled.hwnd(),
            self.btn_add.hwnd(),
            self.list.hwnd(),
        ]
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

                let mut actions = Vec::new();

                match item.origin {
                    RuleOrigin::MutedGlobal => {
                        actions.push(
                            TableAction::new(strings.action_unmute, ActionVariant::Primary)
                                .with_callback(move || {
                                    Self::toggle_mute(trigger_vk, false);
                                }),
                        );
                    }
                    RuleOrigin::InheritedGlobal | RuleOrigin::OverriddenGlobal => {
                        actions.push(TableAction::edit(strings.action_edit, move || {
                            crate::ui::views::dialogs::show_key_macro_dialog_with_key(
                                parent_hwnd,
                                Some(trigger_vk),
                            );
                        }));
                        actions.push(TableAction::secondary(strings.action_mute, move || {
                            Self::toggle_mute(trigger_vk, true);
                        }));
                    }
                    RuleOrigin::Root | RuleOrigin::AppSpecific => {
                        actions.push(TableAction::edit(strings.action_edit, move || {
                            crate::ui::views::dialogs::show_key_macro_dialog_with_key(
                                parent_hwnd,
                                Some(trigger_vk),
                            );
                        }));
                        if item.is_active {
                            actions.push(TableAction::secondary(strings.action_mute, move || {
                                Self::toggle_mute(trigger_vk, true);
                            }));
                        } else {
                            actions.push(
                                TableAction::new(strings.action_unmute, ActionVariant::Primary)
                                    .with_callback(move || {
                                        Self::toggle_mute(trigger_vk, false);
                                    }),
                            );
                        }
                        actions.push(TableAction::delete(strings.action_delete, move || {
                            Self::trigger_delete(parent_hwnd, trigger_vk, &trigger_name, &name);
                        }));
                    }
                }

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
        let changed = crate::ui::controller::with_engine(|engine| {
            let pm = &mut engine.profile_manager;
            let sel_id = pm.selected_id.clone();
            if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                let target_profile = &mut pm.global_profile;
                if mute {
                    target_profile.overridden_key_macros.remove(&trigger_vk);
                    if !target_profile.muted_key_macros.contains(&trigger_vk) {
                        target_profile.muted_key_macros.push(trigger_vk);
                    }
                } else {
                    target_profile.muted_key_macros.retain(|&v| v != trigger_vk);
                }
            } else {
                let is_root_muted = pm.global_profile.muted_key_macros.contains(&trigger_vk);
                let global_macro = pm
                    .global_profile
                    .app_key_macros
                    .iter()
                    .find(|m| m.trigger_vk == trigger_vk)
                    .cloned();
                let child = match pm.get_profile_mut(&sel_id) {
                    Some(c) => c,
                    None => return false,
                };
                if mute {
                    child.overridden_key_macros.remove(&trigger_vk);
                    child.app_key_macros.retain(|m| m.trigger_vk != trigger_vk);
                    if !child.muted_key_macros.contains(&trigger_vk) {
                        child.muted_key_macros.push(trigger_vk);
                    }
                } else {
                    child.muted_key_macros.retain(|&v| v != trigger_vk);
                    if is_root_muted
                        && !child.app_key_macros.iter().any(|m| m.trigger_vk == trigger_vk)
                        && let Some(m) = global_macro
                    {
                        child.app_key_macros.push(m);
                    }
                }
            }
            crate::ui::controller::save(engine);
            true
        });
        if changed == Some(true) {
            crate::ui::controller::refresh_key_macro_list();
        }
    }

    pub fn trigger_delete(parent: isize, trigger_vk: u32, trigger_name: &str, name: &str) {
        let strings = crate::language::current();
        let message = strings
            .confirm_delete_key_macro_msg
            .replace("{key}", trigger_name)
            .replace("{name}", name);

        let options = crate::ui::views::dialogs::ConfirmDialogOptions::new(
            strings.confirm_delete_title,
            message,
        )
        .danger(true)
        .yes_text(strings.btn_yes)
        .no_text(strings.btn_no);

        crate::ui::views::dialogs::show_confirm_dialog(parent, options, move || {
            let changed = crate::ui::controller::with_engine(|engine| {
                let pm = &mut engine.profile_manager;
                let sel_id = pm.selected_id.clone();
                let changed = if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                    pm.global_profile
                        .app_key_macros
                        .retain(|r| r.trigger_vk != trigger_vk);
                    true
                } else if let Some(child) = pm.get_profile_mut(&sel_id) {
                    child
                        .app_key_macros
                        .retain(|r| r.trigger_vk != trigger_vk);
                    true
                } else {
                    false
                };
                if changed {
                    crate::ui::controller::save(engine);
                }
                changed
            });
            if changed == Some(true) {
                crate::ui::controller::refresh_key_macro_list();
            }
        });
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.check_enabled.set_text(strings.key_macro_table_title);
        self.btn_add.set_text(strings.btn_add_key_macro);

        self.list.set_column_title(0, strings.col_key_macro_trigger);
        self.list.set_column_title(1, strings.col_key_macro_name);
        self.list.set_column_title(2, strings.col_origin);
        self.list.set_column_title(3, strings.col_action);
        self.list.refresh_header();
    }
}

