//! Tab 2: Key Remap and Key Disabling (Đổi phím & Khóa phím)
//! Sub-tab layout: a segmented bar switches between the Remap and Disable panels,
//! each panel = toggle + add button + full-height ActionTable inside one shared card.

use crate::engine::profile::{ResolvedDisableItem, ResolvedRemapItem};
use crate::ui::components::action_table::{ActionTable, ColumnStyle, TableAction, TableRow};
use crate::ui::components::{ButtonVariant, CheckBox, PushButton, TabBar};
use crate::ui::tokens::{id::*, layout, spacing};
use crate::ui::win32::{
    HWND_TOP, InvalidateRect, SetWindowPos, ShowWindow, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    SW_HIDE, SW_SHOW,
};

/// Sub-tab bar geometry
const SUB_TAB_Y: i32 = layout::CARD_START_Y; // 112
const SUB_TAB_W_PER_ITEM: i32 = 130;
const SUB_TAB_HEIGHT: i32 = 30;
/// Top of the shared card below the sub-tab bar (painted in paint.rs)
pub const SUB_CARD_Y: i32 = SUB_TAB_Y + SUB_TAB_HEIGHT + spacing::SPACE_10; // 152

pub struct TabRemapDisable {
    pub parent_hwnd: isize,
    pub sub_tabs: TabBar,
    pub check_remap_enabled: CheckBox,
    pub btn_add_remap: PushButton,
    pub list_remap: ActionTable,
    pub check_disable_enabled: CheckBox,
    pub btn_add_disable: PushButton,
    pub list_disable: ActionTable,
}

impl TabRemapDisable {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let sub_titles = [strings.keys_sub_remap, strings.keys_sub_disable];
        let sub_tabs = TabBar::create(
            parent,
            IDC_TAB_KEYS_SUB,
            layout::CARD_X,
            SUB_TAB_Y,
            SUB_TAB_W_PER_ITEM * sub_titles.len() as i32,
            SUB_TAB_HEIGHT,
            hfont_normal,
            hfont_bold,
            &sub_titles,
        )?;

        // Shared panel geometry (both sub-tabs use the same slots)
        const TITLE_X: i32 = layout::CARD_CONTENT_X; // 260
        const TITLE_Y: i32 = SUB_CARD_Y + spacing::SPACE_12; // 164
        const TITLE_WIDTH: i32 = 380;
        const TITLE_HEIGHT: i32 = 24;

        const BTN_WIDTH: i32 = 104;
        const BTN_HEIGHT: i32 = 28;
        const BTN_X: i32 = layout::CARD_X + layout::CARD_WIDTH - spacing::SPACE_16 - BTN_WIDTH; // 686
        const BTN_Y: i32 = SUB_CARD_Y + spacing::SPACE_8 + 1; // 161

        const TABLE_X: i32 = layout::CARD_CONTENT_X; // 260
        const TABLE_WIDTH: i32 = layout::CARD_CONTENT_WIDTH; // 530
        const TABLE_Y: i32 = SUB_CARD_Y + 44; // 196
        const TABLE_HEIGHT: i32 = layout::CARD_BOTTOM_Y - spacing::SPACE_12 - TABLE_Y; // 324

        // --- Sub-tab 0: Key Remapping ---
        let check_remap_enabled = CheckBox::create(
            parent,
            IDC_CHECK_REMAP_ENABLED,
            strings.remap_table_title,
            TITLE_X,
            TITLE_Y,
            TITLE_WIDTH,
            TITLE_HEIGHT,
            hfont_bold,
        )?;
        check_remap_enabled.set_checked(true);

        let btn_add_remap = PushButton::create_with_variant(
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

        let list_remap = ActionTable::create_styled(
            parent,
            IDC_LIST_REMAP,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            &[
                (strings.col_remap_from, 150, ColumnStyle::KeyBadgeWithArrow),
                (strings.col_remap_to, 140, ColumnStyle::KeyBadge),
                (strings.col_origin, 140, ColumnStyle::Default),
                (strings.col_action, 100, ColumnStyle::Action),
            ],
        )?;

        // --- Sub-tab 1: Key Disabling ---
        let check_disable_enabled = CheckBox::create(
            parent,
            IDC_CHECK_DISABLE_ENABLED,
            strings.disable_table_title,
            TITLE_X,
            TITLE_Y,
            TITLE_WIDTH,
            TITLE_HEIGHT,
            hfont_bold,
        )?;
        check_disable_enabled.set_checked(true);

        let btn_add_disable = PushButton::create_with_variant(
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

        let list_disable = ActionTable::create_styled(
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

        let tab = Self {
            parent_hwnd: parent,
            sub_tabs,
            check_remap_enabled,
            btn_add_remap,
            list_remap,
            check_disable_enabled,
            btn_add_disable,
            list_disable,
        };
        tab.update_remap_controls_state();
        tab.update_disable_controls_state();
        Some(tab)
    }

    /// Dynamically enables or disables the Remap table and Add button based on `check_remap_enabled`
    pub fn update_remap_controls_state(&self) {
        let is_enabled = self.check_remap_enabled.is_checked();
        self.btn_add_remap.set_enabled(is_enabled);
        self.list_remap.set_enabled(is_enabled);
    }

    /// Dynamically enables or disables the Disable table and Add button based on `check_disable_enabled`
    pub fn update_disable_controls_state(&self) {
        let is_enabled = self.check_disable_enabled.is_checked();
        self.btn_add_disable.set_enabled(is_enabled);
        self.list_disable.set_enabled(is_enabled);
    }

    pub fn load_config(&self, config: &crate::engine::EngineConfig) {
        self.check_remap_enabled.set_checked(config.remap_enabled);
        self.update_remap_controls_state();
        self.check_disable_enabled.set_checked(config.disable_enabled);
        self.update_disable_controls_state();
    }

    pub fn load_profile(
        &self,
        profile: &crate::engine::profile::AppProfile,
        global_config: &crate::engine::EngineConfig,
    ) {
        if profile.is_root {
            self.load_config(global_config);
            return;
        }
        self.check_remap_enabled
            .set_tri_state(profile.remap_enabled, global_config.remap_enabled);
        self.update_remap_controls_state();

        self.check_disable_enabled
            .set_tri_state(profile.disable_enabled, global_config.disable_enabled);
        self.update_disable_controls_state();
    }

    pub fn read_config(&self, config: &mut crate::engine::EngineConfig) {
        config.remap_enabled = self.check_remap_enabled.is_checked();
        config.disable_enabled = self.check_disable_enabled.is_checked();
    }

    pub fn read_profile(&self, profile: &mut crate::engine::profile::AppProfile) {
        profile.remap_enabled = self.check_remap_enabled.tri_state();
        profile.disable_enabled = self.check_disable_enabled.tri_state();
    }

    pub fn trigger_delete_remap_by_row(parent: isize, row_idx: i32) {
        let item_info = if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let sel_id = engine.profile_manager.selected_id.clone();
            let remap_items = engine.profile_manager.resolve_remap_items_for_ui(&sel_id);
            remap_items
                .get(row_idx as usize)
                .map(|item| (item.from_vk, item.from_name.clone(), item.to_name.clone()))
        } else {
            None
        };

        let Some((from_vk, from_name, to_name)) = item_info else {
            return;
        };

        let strings = crate::language::current();
        let message = strings
            .confirm_delete_remap_msg
            .replace("{from}", &from_name)
            .replace("{to}", &to_name);

        let options = crate::ui::views::dialogs::ConfirmDialogOptions::new(
            strings.confirm_delete_title,
            message,
        )
        .danger(true)
        .yes_text(strings.btn_yes)
        .no_text(strings.btn_no);

        crate::ui::views::dialogs::show_confirm_dialog(parent, options, move || {
            let mut changed = false;
            if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id == "global" {
                    engine
                        .profile_manager
                        .global_profile
                        .app_remaps
                        .retain(|r| r.from_vk != from_vk);
                    changed = true;
                } else {
                    let is_global = engine
                        .profile_manager
                        .global_profile
                        .app_remaps
                        .iter()
                        .any(|r| r.from_vk == from_vk);
                    if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                        if is_global {
                            child.overridden_remaps.remove(&from_vk);
                            if !child.muted_remaps.contains(&from_vk) {
                                child.muted_remaps.push(from_vk);
                            }
                        } else {
                            child.app_remaps.retain(|r| r.from_vk != from_vk);
                        }
                        changed = true;
                    }
                }
                if changed {
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                }
            }
            if changed {
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                    && let Some(ref engine) = *guard
                    && let Ok(ui_guard) = crate::ui::UI_MANAGER.try_lock()
                    && let Some(ref ui) = *ui_guard
                {
                    let sel_id = engine.profile_manager.selected_id.clone();
                    let remap_items = engine.profile_manager.resolve_remap_items_for_ui(&sel_id);
                    ui.controls.tab_remap.populate_remaps(&remap_items);
                }
            }
        });
    }

    pub fn trigger_delete_disable_by_row(parent: isize, row_idx: i32) {
        let item_info = if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let sel_id = engine.profile_manager.selected_id.clone();
            let disable_items = engine.profile_manager.resolve_disable_items_for_ui(&sel_id);
            disable_items
                .get(row_idx as usize)
                .map(|item| (item.vk, item.key_name.clone()))
        } else {
            None
        };

        let Some((vk, key_name)) = item_info else {
            return;
        };

        let strings = crate::language::current();
        let message = strings
            .confirm_delete_disable_msg
            .replace("{key}", &key_name);

        let options = crate::ui::views::dialogs::ConfirmDialogOptions::new(
            strings.confirm_delete_title,
            message,
        )
        .danger(true)
        .yes_text(strings.btn_yes)
        .no_text(strings.btn_no);

        crate::ui::views::dialogs::show_confirm_dialog(parent, options, move || {
            let mut changed = false;
            if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id == "global" {
                    engine
                        .profile_manager
                        .global_profile
                        .app_disabled
                        .retain(|&v| v != vk);
                    changed = true;
                } else {
                    let is_global = engine
                        .profile_manager
                        .global_profile
                        .app_disabled
                        .contains(&vk);
                    if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                        if is_global {
                            if !child.unmuted_disabled.contains(&vk) {
                                child.unmuted_disabled.push(vk);
                            }
                        } else {
                            child.app_disabled.retain(|&v| v != vk);
                        }
                        changed = true;
                    }
                }
                if changed {
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                }
            }
            if changed {
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                    && let Some(ref engine) = *guard
                    && let Ok(ui_guard) = crate::ui::UI_MANAGER.try_lock()
                    && let Some(ref ui) = *ui_guard
                {
                    let sel_id = engine.profile_manager.selected_id.clone();
                    let disable_items = engine.profile_manager.resolve_disable_items_for_ui(&sel_id);
                    ui.controls.tab_remap.populate_disabled(&disable_items);
                }
            }
        });
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        match ctl_id {
            IDC_TAB_KEYS_SUB => {
                self.update_visibility(true);
                true
            }
            IDC_CHECK_REMAP_ENABLED => {
                if notify_code == 0 {
                    self.update_remap_controls_state();
                    if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        let sel_id = engine.profile_manager.selected_id.clone();
                        if sel_id == "global" {
                            self.read_config(engine.config_mut());
                            engine.profile_manager.global_profile.remap_enabled =
                                Some(self.check_remap_enabled.is_checked());
                        } else if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                            self.read_profile(child);
                        }
                        crate::engine::config_store::save_config_all_debounced(
                            engine.config(),
                            &engine.macro_table,
                            &engine.profile_manager,
                        );
                    }
                    true
                } else {
                    false
                }
            }
            IDC_CHECK_DISABLE_ENABLED => {
                if notify_code == 0 {
                    self.update_disable_controls_state();
                    if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        let sel_id = engine.profile_manager.selected_id.clone();
                        if sel_id == "global" {
                            self.read_config(engine.config_mut());
                            engine.profile_manager.global_profile.disable_enabled =
                                Some(self.check_disable_enabled.is_checked());
                        } else if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                            self.read_profile(child);
                        }
                        crate::engine::config_store::save_config_all_debounced(
                            engine.config(),
                            &engine.macro_table,
                            &engine.profile_manager,
                        );
                    }
                    true
                } else {
                    false
                }
            }
            IDC_BTN_ADD_REMAP => {
                crate::ui::views::dialogs::show_remap_dialog(hwnd);
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                    && let Some(ref engine) = *guard
                {
                    let sel_id = engine.profile_manager.selected_id.clone();
                    let remap_items = engine.profile_manager.resolve_remap_items_for_ui(&sel_id);
                    self.populate_remaps(&remap_items);
                }
                true
            }
            IDC_BTN_ADD_DISABLE => {
                crate::ui::views::dialogs::show_disable_dialog(hwnd);
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                    && let Some(ref engine) = *guard
                {
                    let sel_id = engine.profile_manager.selected_id.clone();
                    let disable_items = engine.profile_manager.resolve_disable_items_for_ui(&sel_id);
                    self.populate_disabled(&disable_items);
                }
                true
            }
            _ => false,
        }
    }

    /// Populates Remap ActionTable with resolved Live Inheritance items
    pub fn populate_remaps(&self, items: &[ResolvedRemapItem]) {
        self.list_remap.clear();
        let strings = crate::language::current();
        let parent = self.parent_hwnd;

        let rows: Vec<TableRow> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let idx = idx as i32;
                let from_vk = item.from_vk;
                let to_vk = item.to_vk;

                let actions = vec![
                    TableAction::edit(strings.action_edit, move || {
                        crate::ui::views::dialogs::show_remap_dialog_with_keys(
                            parent,
                            Some(from_vk),
                            Some(to_vk),
                        );
                    }),
                    TableAction::delete(strings.action_delete, move || {
                        Self::trigger_delete_remap_by_row(parent, idx);
                    }),
                ];

                TableRow::new()
                    .key_with_arrow(&item.from_name)
                    .key(&item.to_name)
                    .text(item.origin.localized_badge_label(strings))
                    .actions(actions)
            })
            .collect();

        self.list_remap.set_rows(rows);
    }

    /// Populates Disable ActionTable with resolved Live Inheritance items
    pub fn populate_disabled(&self, items: &[ResolvedDisableItem]) {
        self.list_disable.clear();
        let strings = crate::language::current();
        let parent = self.parent_hwnd;

        let rows: Vec<TableRow> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let idx = idx as i32;
                let status = if item.is_active {
                    strings.status_blocked
                } else {
                    strings.status_unblocked
                };
                let key_label = format!("🚫 {}", item.key_name);

                let actions = vec![
                    TableAction::delete(strings.action_delete, move || {
                        Self::trigger_delete_disable_by_row(parent, idx);
                    }),
                ];

                TableRow::new()
                    .danger_key(&key_label)
                    .text(status)
                    .text(item.origin.localized_badge_label(strings))
                    .actions(actions)
            })
            .collect();

        self.list_disable.set_rows(rows);
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.sub_tabs.set_tabs(&[strings.keys_sub_remap, strings.keys_sub_disable]);
        self.check_remap_enabled.set_text(strings.remap_table_title);
        self.btn_add_remap.set_text(strings.btn_add_remap);
        self.check_disable_enabled.set_text(strings.disable_table_title);
        self.btn_add_disable.set_text(strings.btn_add_disable);

        self.list_remap.set_column_title(0, strings.col_remap_from);
        self.list_remap.set_column_title(1, strings.col_remap_to);
        self.list_remap.set_column_title(2, strings.col_origin);
        self.list_remap.set_column_title(3, strings.col_action);
        self.list_remap.refresh_header();

        self.list_disable.set_column_title(0, strings.col_disable_key);
        self.list_disable.set_column_title(1, strings.col_status);
        self.list_disable.set_column_title(2, strings.col_origin);
        self.list_disable.set_column_title(3, strings.col_action);
        self.list_disable.refresh_header();

        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let sel_id = engine.profile_manager.selected_id.clone();
            let remap_items = engine.profile_manager.resolve_remap_items_for_ui(&sel_id);
            self.populate_remaps(&remap_items);
            let disable_items = engine.profile_manager.resolve_disable_items_for_ui(&sel_id);
            self.populate_disabled(&disable_items);
        }
    }

    pub fn update_visibility(&self, visible: bool) {
        let sub = self.sub_tabs.get_cur_sel();
        let panels = [
            ([self.check_remap_enabled.hwnd(), self.btn_add_remap.hwnd(), self.list_remap.hwnd()], visible && sub == 0),
            ([self.check_disable_enabled.hwnd(), self.btn_add_disable.hwnd(), self.list_disable.hwnd()], visible && sub == 1),
        ];
        unsafe {
            ShowWindow(self.sub_tabs.hwnd(), if visible { SW_SHOW } else { SW_HIDE });
            // Hide outgoing panels first so they never overlap the incoming one
            for h in panels.iter().filter(|(_, on)| !on).flat_map(|(hs, _)| hs) {
                ShowWindow(*h, SW_HIDE);
            }
            for h in panels.iter().filter(|(_, on)| *on).flat_map(|(hs, _)| hs) {
                SetWindowPos(*h, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW);
                InvalidateRect(*h, std::ptr::null(), 1);
            }
        }
    }
}
