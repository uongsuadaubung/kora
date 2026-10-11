//! Tab 1: Macro controls (Gõ tắt)
//! 3-Card Modern Layout: Master Options (Card 1), Action Bar (Card 2), ActionTable Component (Card 3)

use crate::language::LanguageStrings;
use crate::ui::components::action_table::ActionTable;
use crate::ui::win32::{HWND_TOP, InvalidateRect, SetWindowPos, ShowWindow, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW};
use crate::ui::components::*;
use crate::ui::tokens::{borders, id::*, layout, spacing};


pub struct TabMacro {
    pub parent_hwnd: isize,
    pub check_use_macro: CheckBox,
    pub check_macro_in_english: CheckBox,
    pub btn_add_macro: PushButton,
    pub list_macro: ActionTable,
}
impl TabMacro {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        // --- Card 1: Master Options ---
        const CARD1_Y: i32 = layout::CARD_START_Y;
        const CARD1_HEIGHT: i32 = 48;
        const CHECKBOX_HEIGHT: i32 = 24;
        const CHECKBOX_Y: i32 = CARD1_Y + (CARD1_HEIGHT - CHECKBOX_HEIGHT) / 2;
        const CHECKBOX_1_X: i32 = layout::CARD_CONTENT_X;
        const CHECKBOX_1_WIDTH: i32 = 220;
        const CHECKBOX_GAP: i32 = spacing::SPACE_10;
        const CHECKBOX_2_X: i32 = CHECKBOX_1_X + CHECKBOX_1_WIDTH + CHECKBOX_GAP;
        const CHECKBOX_2_WIDTH: i32 = layout::CARD_CONTENT_WIDTH - CHECKBOX_1_WIDTH - CHECKBOX_GAP;

        let check_use_macro = CheckBox::create(
            parent,
            IDC_CHECK_USE_MACRO,
            strings.check_use_macro,
            CHECKBOX_1_X,
            CHECKBOX_Y,
            CHECKBOX_1_WIDTH,
            CHECKBOX_HEIGHT,
            hfont_normal,
        )?;
        let check_macro_in_english = CheckBox::create(
            parent,
            IDC_CHECK_MACRO_EN,
            strings.check_macro_in_english,
            CHECKBOX_2_X,
            CHECKBOX_Y,
            CHECKBOX_2_WIDTH,
            CHECKBOX_HEIGHT,
            hfont_normal,
        )?;
        check_macro_in_english.set_align_right(true);

        // --- Card 2: Action Bar (Only "+ Thêm" button) ---
        const CARD2_Y: i32 = CARD1_Y + CARD1_HEIGHT + layout::CARD_GAP;
        const CARD2_HEIGHT: i32 = 48;
        const BTN_ADD_HEIGHT: i32 = 28;
        const BTN_ADD_Y: i32 = CARD2_Y + (CARD2_HEIGHT - BTN_ADD_HEIGHT) / 2;
        const BTN_ADD_X: i32 = layout::CARD_CONTENT_X;
        const BTN_ADD_WIDTH: i32 = 104;

        let btn_add_macro = PushButton::create_with_variant(
            parent,
            IDC_BTN_ADD_MACRO,
            strings.btn_add_macro,
            BTN_ADD_X,
            BTN_ADD_Y,
            BTN_ADD_WIDTH,
            BTN_ADD_HEIGHT,
            hfont_bold,
            ButtonVariant::Primary,
            None,
        )?;

        // --- Card 3: 5-Column ActionTable Component ---
        // Header always includes: VIẾT TẮT | NỘI DUNG THAY THẾ | LOẠI | NGUỒN GỐC | HÀNH ĐỘNG
        const CARD3_Y: i32 = CARD2_Y + CARD2_HEIGHT + layout::CARD_GAP;
        const TABLE_INSET: i32 = borders::WIDTH_FOCUS;
        const TABLE_X: i32 = layout::CARD_X + TABLE_INSET;
        const TABLE_Y: i32 = CARD3_Y + TABLE_INSET;
        const TABLE_WIDTH: i32 = layout::CARD_WIDTH - TABLE_INSET * 2;
        const TABLE_HEIGHT: i32 = (layout::CARD_BOTTOM_Y - CARD3_Y) - TABLE_INSET * 2;

        let list_macro = ActionTable::create_styled(
            parent,
            IDC_LIST_MACRO,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            &[
                (strings.col_macro_key, 90, ColumnStyle::KeyBadge),
                (strings.col_macro_val, 160, ColumnStyle::Default),
                (strings.col_macro_type, 75, ColumnStyle::CenterSecondary),
                (strings.col_origin, 125, ColumnStyle::Default),
                (strings.col_action, 104, ColumnStyle::Action),
            ],
        )?;

        let tab = Self {
            parent_hwnd: parent,
            check_use_macro,
            check_macro_in_english,
            btn_add_macro,
            list_macro,
        };
        tab.update_macro_checkboxes_state();
        Some(tab)
    }

    pub fn trigger_edit_by_row(parent: isize, lv_hwnd: isize, idx: i32) {
        let key = ActionTable::get_text(lv_hwnd, idx, 0);
        let val = ActionTable::get_text(lv_hwnd, idx, 1);
        let typ_str = ActionTable::get_text(lv_hwnd, idx, 2);
        if key.is_empty() {
            return;
        }
        let strings = crate::language::current();
        let mtype = if typ_str == strings.macro_type_start {
            crate::engine::macro_table::MacroType::StartConsonant
        } else if typ_str == strings.macro_type_end {
            crate::engine::macro_table::MacroType::EndConsonant
        } else {
            crate::engine::macro_table::MacroType::Normal
        };

        let data = crate::ui::views::dialogs::MacroEditData {
            original_key: key.clone(),
            key,
            value: val,
            macro_type: mtype,
        };
        crate::ui::views::dialogs::show_macro_dialog(parent, Some(data));
    }

    pub fn trigger_delete_by_row(parent: isize, lv_hwnd: isize, idx: i32) {
        let key_to_del = ActionTable::get_text(lv_hwnd, idx, 0);
        if key_to_del.is_empty() {
            return;
        }

        let strings = crate::language::current();
        let message = format!("{} \"{}\"?", strings.confirm_delete_macro_msg, key_to_del);
        let options = crate::ui::views::dialogs::ConfirmDialogOptions::new(
            strings.confirm_delete_title,
            message,
        )
        .danger(true)
        .yes_text(strings.btn_yes)
        .no_text(strings.btn_no);

        crate::ui::views::dialogs::show_confirm_dialog(parent, options, move || {
            if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id == "global" {
                    engine.macro_table.remove(&key_to_del);
                } else if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                    let is_global = engine.macro_table.get(&key_to_del).is_some();
                    if is_global {
                        child.app_macros.retain(|m| m.key != key_to_del);
                        child.overridden_macros.remove(&key_to_del);
                        if !child.muted_macros.contains(&key_to_del) {
                            child.muted_macros.push(key_to_del.clone());
                        }
                    } else {
                        child.app_macros.retain(|m| m.key != key_to_del);
                    }
                }

                crate::engine::config_store::save_config_all_debounced(
                    engine.config(),
                    &engine.macro_table,
                    &engine.profile_manager,
                );

                if let Ok(ui_guard) = crate::ui::UI_MANAGER.try_lock()
                    && let Some(ref ui) = *ui_guard
                {
                    let items = engine
                        .profile_manager
                        .resolve_macro_items_for_ui(&sel_id, &engine.macro_table.get_sorted_entries());
                    ui.controls.tab_macro.populate_macro_items(&items);
                }
            }
        });
    }

    /// Dynamically enables or disables child macro checkbox based on whether `check_use_macro` is checked
    pub fn update_macro_checkboxes_state(&self) {
        let is_enabled = self.check_use_macro.is_checked();
        self.check_macro_in_english.set_enabled(is_enabled);
        self.btn_add_macro.set_enabled(is_enabled);
    }


    pub fn update_visibility(&self, visible: bool) {
        unsafe {
            let show_cmd = if visible { SW_SHOW } else { SW_HIDE };
            ShowWindow(self.check_use_macro.hwnd(), show_cmd);
            ShowWindow(self.check_macro_in_english.hwnd(), show_cmd);
            ShowWindow(self.btn_add_macro.hwnd(), show_cmd);
            ShowWindow(self.list_macro.hwnd(), show_cmd);

            if visible {
                SetWindowPos(
                    self.btn_add_macro.hwnd(),
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW,
                );
                SetWindowPos(
                    self.list_macro.hwnd(),
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW,
                );
                InvalidateRect(self.list_macro.hwnd(), std::ptr::null(), 1);
            }
        }
    }

    pub fn load_config(&self, config: &crate::engine::EngineConfig) {
        self.check_use_macro.set_checked(config.use_macro);
        self.check_macro_in_english
            .set_checked(config.use_macro_in_english_mode);
        self.update_macro_checkboxes_state();
    }

    pub fn load_profile(&self, profile: &crate::engine::profile::AppProfile, global_config: &crate::engine::EngineConfig) {
        if profile.is_root {
            self.load_config(global_config);
            return;
        }
        self.check_use_macro
            .set_tri_state(profile.use_macro, global_config.use_macro);
        self.check_macro_in_english.set_tri_state(
            profile.use_macro_in_english_mode,
            global_config.use_macro_in_english_mode,
        );
        self.update_macro_checkboxes_state();
    }

    pub fn read_config(&self, config: &mut crate::engine::EngineConfig) {
        config.use_macro = self.check_use_macro.is_checked();
        config.use_macro_in_english_mode = self.check_macro_in_english.is_checked();
    }

    pub fn read_profile(&self, profile: &mut crate::engine::profile::AppProfile) {
        profile.use_macro = self.check_use_macro.tri_state();
        profile.use_macro_in_english_mode = self.check_macro_in_english.tri_state();
    }

    pub fn populate_macros(&self, macros: &[crate::engine::macro_table::MacroEntry]) {
        self.list_macro.clear();
        let strings = crate::language::current();
        let parent = self.parent_hwnd;
        let lv_hwnd = self.list_macro.hwnd();

        let rows: Vec<TableRow> = macros
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let idx = idx as i32;
                let type_str = match entry.macro_type {
                    crate::engine::macro_table::MacroType::Normal => strings.macro_type_normal,
                    crate::engine::macro_table::MacroType::StartConsonant => strings.macro_type_start,
                    crate::engine::macro_table::MacroType::EndConsonant => strings.macro_type_end,
                };

                TableRow::new()
                    .text(&entry.key)
                    .text(&entry.value)
                    .secondary(type_str)
                    .text("")
                    .actions(vec![
                        TableAction::edit(strings.action_edit, move || {
                            Self::trigger_edit_by_row(parent, lv_hwnd, idx);
                        }),
                        TableAction::delete(strings.action_delete, move || {
                            Self::trigger_delete_by_row(parent, lv_hwnd, idx);
                        }),
                    ])
            })
            .collect();

        self.list_macro.set_rows(rows);
    }

    /// Populates Macro ListView with resolved Live Inheritance items
    pub fn populate_macro_items(&self, items: &[crate::engine::profile::ResolvedMacroItem]) {
        self.list_macro.clear();
        let strings = crate::language::current();
        let parent = self.parent_hwnd;
        let lv_hwnd = self.list_macro.hwnd();

        let rows: Vec<TableRow> = items
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let idx = idx as i32;
                let type_str = match entry.macro_type {
                    crate::engine::macro_table::MacroType::Normal => strings.macro_type_normal,
                    crate::engine::macro_table::MacroType::StartConsonant => strings.macro_type_start,
                    crate::engine::macro_table::MacroType::EndConsonant => strings.macro_type_end,
                };
                let origin_str = entry.origin.localized_badge_label(strings);

                TableRow::new()
                    .key(&entry.key)
                    .text(&entry.value)
                    .secondary(type_str)
                    .text(origin_str)
                    .actions(vec![
                        TableAction::edit(strings.action_edit, move || {
                            Self::trigger_edit_by_row(parent, lv_hwnd, idx);
                        }),
                        TableAction::delete(strings.action_delete, move || {
                            Self::trigger_delete_by_row(parent, lv_hwnd, idx);
                        }),
                    ])
            })
            .collect();

        self.list_macro.set_rows(rows);
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.check_use_macro.set_text(strings.check_use_macro);
        self.check_macro_in_english
            .set_text(strings.check_macro_in_english);
        self.btn_add_macro.set_text(strings.btn_add_macro);

        self.list_macro.set_column_title(0, strings.col_macro_key);
        self.list_macro.set_column_title(1, strings.col_macro_val);
        self.list_macro.set_column_title(2, strings.col_macro_type);
        self.list_macro.set_column_title(3, strings.col_origin);
        self.list_macro.set_column_title(4, strings.col_action);
        self.list_macro.refresh_header();

        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let sel_id = engine.profile_manager.selected_id.clone();
            let macros = engine
                .profile_manager
                .resolve_macro_items_for_ui(&sel_id, &engine.macro_table.get_sorted_entries());
            self.populate_macro_items(&macros);
        }
    }


    pub fn on_command(
        &self,
        _controls: &super::ControlPanelControls,
        ctl_id: u32,
        notify_code: u32,
    ) -> bool {
        match ctl_id {
            IDC_BTN_ADD_MACRO => {
                crate::ui::views::dialogs::show_macro_dialog(self.parent_hwnd, None);
                true
            }
            IDC_CHECK_USE_MACRO | IDC_CHECK_MACRO_EN => {
                if notify_code == 0 {
                    if ctl_id == IDC_CHECK_USE_MACRO {
                        self.update_macro_checkboxes_state();
                    }
                    if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        let sel_id = engine.profile_manager.selected_id.clone();
                        if sel_id == "global" {
                            self.read_config(engine.config_mut());
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
            _ => false,
        }
    }

    pub fn handle_edit_macro_at(&self, idx: i32) {
        Self::trigger_edit_by_row(self.parent_hwnd, self.list_macro.hwnd(), idx);
    }

    pub fn handle_del_macro_at(&self, idx: i32) {
        Self::trigger_delete_by_row(self.parent_hwnd, self.list_macro.hwnd(), idx);
    }
}
