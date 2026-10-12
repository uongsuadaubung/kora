//! Sub-panel: Key Disabling (Khóa phím)
//! Allows blocking specific hardware keys from reaching the system or apps.

use crate::engine::profile::{ResolvedDisableItem, RuleOrigin};
use crate::ui::components::action_table::{
    ActionTable, ActionVariant, ColumnStyle, TableAction, TableRow,
};
use crate::ui::components::{ButtonVariant, CheckBox, PushButton};
use crate::ui::tokens::id::*;
use crate::ui::views::control_panel::tab_keys::{
    BTN_HEIGHT, BTN_WIDTH, BTN_X, BTN_Y, TABLE_HEIGHT, TABLE_WIDTH, TABLE_X, TABLE_Y, TITLE_HEIGHT,
    TITLE_WIDTH, TITLE_X, TITLE_Y,
};

pub struct DisablePanel {
    pub check_enabled: CheckBox,
    pub btn_add: PushButton,
    pub list: ActionTable,
}

impl DisablePanel {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let check_enabled = CheckBox::create(
            parent,
            IDC_CHECK_DISABLE_ENABLED,
            strings.disable_table_title,
            TITLE_X,
            TITLE_Y,
            TITLE_WIDTH,
            TITLE_HEIGHT,
            hfont_bold,
        )?;
        check_enabled.set_checked(false);

        let btn_add = PushButton::create_with_variant(
            parent,
            IDC_BTN_ADD_DISABLE,
            strings.btn_add_disable,
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
            IDC_LIST_DISABLE,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            &[
                (strings.col_disable_key, 125, ColumnStyle::DangerKeyBadge),
                (strings.col_status, 165, ColumnStyle::Default),
                (strings.col_origin, 135, ColumnStyle::Default),
                (strings.col_action, 105, ColumnStyle::Action),
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

    /// Dynamically enables or disables the Disable table and Add button based on `check_enabled`
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

                let mut actions = Vec::new();

                match item.origin {
                    RuleOrigin::MutedGlobal => {
                        actions.push(
                            TableAction::new(strings.action_unmute, ActionVariant::Primary)
                                .with_callback(move || {
                                    Self::toggle_mute(vk, false);
                                }),
                        );
                    }
                    RuleOrigin::InheritedGlobal | RuleOrigin::OverriddenGlobal => {
                        actions.push(TableAction::delete(strings.action_mute, move || {
                            Self::toggle_mute(vk, true);
                        }));
                    }
                    RuleOrigin::Root | RuleOrigin::AppSpecific => {
                        actions.push(TableAction::delete(strings.action_delete, move || {
                            Self::trigger_delete(parent_hwnd, vk, &key_name);
                        }));
                    }
                }

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
        let changed = crate::ui::controller::with_engine(|engine| {
            let pm = &mut engine.profile_manager;
            let sel_id = pm.selected_id.clone();
            if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                return false;
            }
            if let Some(child) = pm.get_profile_mut(&sel_id) {
                if mute {
                    if !child.unmuted_disabled.contains(&vk) {
                        child.unmuted_disabled.push(vk);
                    }
                } else {
                    child.unmuted_disabled.retain(|&v| v != vk);
                }
                crate::ui::controller::save(engine);
                true
            } else {
                false
            }
        });
        if changed == Some(true) {
            crate::ui::controller::refresh_disable_list();
        }
    }

    pub fn trigger_delete(parent: isize, vk: u32, key_name: &str) {
        let strings = crate::language::current();
        let message = strings
            .confirm_delete_disable_msg
            .replace("{key}", key_name);

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
                    pm.global_profile.app_disabled.retain(|&v| v != vk);
                    true
                } else if let Some(child) = pm.get_profile_mut(&sel_id) {
                    child.app_disabled.retain(|&v| v != vk);
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
                crate::ui::controller::refresh_disable_list();
            }
        });
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.check_enabled.set_text(strings.disable_table_title);
        self.btn_add.set_text(strings.btn_add_disable);

        self.list.set_column_title(0, strings.col_disable_key);
        self.list.set_column_title(1, strings.col_status);
        self.list.set_column_title(2, strings.col_origin);
        self.list.set_column_title(3, strings.col_action);
        self.list.refresh_header();
    }
}

