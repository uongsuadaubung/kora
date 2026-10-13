//! Sub-panel: Key Remapping (Đổi phím)
//! Allows mapping one hardware key to another key.

use crate::engine::profile::{ResolvedRemapItem, RuleOrigin};
use crate::ui::components::action_table::{
    ActionTable, ActionVariant, ColumnStyle, TableAction, TableRow,
};
use crate::ui::components::{ButtonVariant, CheckBox, PushButton};
use crate::ui::tokens::id::*;
use crate::ui::views::control_panel::tab_keys::{
    BTN_HEIGHT, BTN_WIDTH, BTN_X, BTN_Y, TABLE_HEIGHT, TABLE_WIDTH, TABLE_X, TABLE_Y, TITLE_HEIGHT,
    TITLE_WIDTH, TITLE_X, TITLE_Y,
};

pub struct RemapPanel {
    pub check_enabled: CheckBox,
    pub btn_add: PushButton,
    pub list: ActionTable,
}

impl RemapPanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let check_enabled = CheckBox::create(
            parent,
            IDC_CHECK_REMAP_ENABLED,
            strings.remap_table_title,
            TITLE_X,
            TITLE_Y,
            TITLE_WIDTH,
            TITLE_HEIGHT,
            hfont_bold,
        )?;
        check_enabled.set_checked(false);

        let btn_add = PushButton::create_with_variant(
            parent,
            IDC_BTN_ADD_REMAP,
            strings.btn_add_remap,
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
            IDC_LIST_REMAP,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            &[
                (strings.col_remap_from, 145, ColumnStyle::KeyBadgeWithArrow),
                (strings.col_remap_to, 135, ColumnStyle::KeyBadge),
                (strings.col_origin, 125, ColumnStyle::Default),
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

    /// Dynamically enables or disables the Remap table and Add button based on `check_enabled`
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

                let mut actions = Vec::new();

                match item.origin {
                    RuleOrigin::MutedGlobal => {
                        actions.push(
                            TableAction::new(strings.action_unmute, ActionVariant::Primary)
                                .with_callback(move || {
                                    Self::toggle_mute(from_vk, false);
                                }),
                        );
                    }
                    RuleOrigin::InheritedGlobal | RuleOrigin::OverriddenGlobal => {
                        actions.push(TableAction::edit(strings.action_edit, move || {
                            crate::ui::views::dialogs::show_remap_dialog_with_keys(
                                parent_hwnd,
                                Some(from_vk),
                                Some(to_vk),
                            );
                        }));
                        actions.push(TableAction::secondary(strings.action_mute, move || {
                            Self::toggle_mute(from_vk, true);
                        }));
                    }
                    RuleOrigin::Root | RuleOrigin::AppSpecific => {
                        actions.push(TableAction::edit(strings.action_edit, move || {
                            crate::ui::views::dialogs::show_remap_dialog_with_keys(
                                parent_hwnd,
                                Some(from_vk),
                                Some(to_vk),
                            );
                        }));
                        if item.is_active {
                            actions.push(TableAction::secondary(strings.action_mute, move || {
                                Self::toggle_mute(from_vk, true);
                            }));
                        } else {
                            actions.push(
                                TableAction::new(strings.action_unmute, ActionVariant::Primary)
                                    .with_callback(move || {
                                        Self::toggle_mute(from_vk, false);
                                    }),
                            );
                        }
                        actions.push(TableAction::delete(strings.action_delete, move || {
                            Self::trigger_delete(parent_hwnd, from_vk, &from_name, &to_name);
                        }));
                    }
                }

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
        let changed = crate::ui::controller::with_engine(|engine| {
            let pm = &mut engine.profile_manager;
            let sel_id = pm.selected_id.clone();
            if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                let target_profile = &mut pm.global_profile;
                if mute {
                    target_profile.overridden_remaps.remove(&from_vk);
                    if !target_profile.muted_remaps.contains(&from_vk) {
                        target_profile.muted_remaps.push(from_vk);
                    }
                } else {
                    target_profile.muted_remaps.retain(|&v| v != from_vk);
                }
            } else {
                let is_root_muted = pm.global_profile.muted_remaps.contains(&from_vk);
                let global_rule = pm
                    .global_profile
                    .app_remaps
                    .iter()
                    .find(|r| r.from_vk == from_vk)
                    .cloned();
                let child = match pm.get_profile_mut(&sel_id) {
                    Some(c) => c,
                    None => return false,
                };
                if mute {
                    child.overridden_remaps.remove(&from_vk);
                    child.app_remaps.retain(|r| r.from_vk != from_vk);
                    if !child.muted_remaps.contains(&from_vk) {
                        child.muted_remaps.push(from_vk);
                    }
                } else {
                    child.muted_remaps.retain(|&v| v != from_vk);
                    if is_root_muted
                        && !child.app_remaps.iter().any(|r| r.from_vk == from_vk)
                        && let Some(rule) = global_rule
                    {
                        child.app_remaps.push(rule);
                    }
                }
            }
            crate::ui::controller::save(engine);
            true
        });
        if changed == Some(true) {
            crate::ui::controller::refresh_remap_list();
        }
    }

    pub fn trigger_delete(parent: isize, from_vk: u32, from_name: &str, to_name: &str) {
        let strings = crate::language::current();
        let message = strings
            .confirm_delete_remap_msg
            .replace("{from}", from_name)
            .replace("{to}", to_name);

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
                    pm.global_profile.app_remaps.retain(|r| r.from_vk != from_vk);
                    true
                } else if let Some(child) = pm.get_profile_mut(&sel_id) {
                    child.app_remaps.retain(|r| r.from_vk != from_vk);
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
                crate::ui::controller::refresh_remap_list();
            }
        });
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.check_enabled.set_text(strings.remap_table_title);
        self.btn_add.set_text(strings.btn_add_remap);

        self.list.set_column_title(0, strings.col_remap_from);
        self.list.set_column_title(1, strings.col_remap_to);
        self.list.set_column_title(2, strings.col_origin);
        self.list.set_column_title(3, strings.col_action);
        self.list.refresh_header();
    }
}

