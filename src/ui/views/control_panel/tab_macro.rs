//! Tab 1: Macro controls (Gõ tắt)
//! 3-Card Modern Layout: Master Options (Card 1), Action Bar (Card 2), ActionTable Component (Card 3)

use crate::language::LanguageStrings;
use crate::ui::prelude::*;
use crate::ui_framework::win32::{
    HWND_TOP, InvalidateRect, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SetWindowPos,
};

pub struct TabMacro {
    layout: CardLayout,
    pub parent_hwnd: isize,
    pub check_use_macro: CheckBox<Inheritable>,
    pub check_macro_in_english: CheckBox<Inheritable>,
    pub btn_add_macro: PushButton,
    pub list_macro: ActionTable,
}
impl TabMacro {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);

        // --- Card 1: Master Options ---
        let (check_use_macro, check_macro_in_english) = ly.card_with_pad(12, 12, |card| {
            let check_use_macro = card.inheritable_checkbox(|s| s.check_use_macro, false)?;
            let check_macro_in_english = card.inheritable_checkbox(|s| s.check_macro_in_english, false)?;
            check_macro_in_english.set_align_right(true);

            card.horizontal(24, 0, |row| {
                row.align(CrossAlign::Center);
                row.add_fixed(220, Some(check_use_macro.hwnd()));
                row.add_spacer();
                row.add_stretch(1, Some(check_macro_in_english.hwnd()));
            });

            Some((check_use_macro, check_macro_in_english))
        })?;

        // --- Card 2: Action Bar (Only "+ Thêm" button) ---
        let btn_add_macro = ly.card_with_pad(10, 10, |card| {
            let btn = card.button(
                |s| s.btn_add_macro,
                true,
                ButtonVariant::Primary,
            )?;
            card.horizontal(28, 0, |row| {
                row.add_fixed(104, Some(btn.hwnd()));
            });
            Some(btn)
        })?;

        // --- Card 3: 5-Column ActionTable Component ---
        let card3_top = ly.y();
        let list_macro = ly.card_until(0, layout::CARD_BOTTOM_Y, |card| {
            let table_inset = borders::WIDTH_FOCUS;
            let table_x = layout::CARD_X + table_inset;
            let table_y = card3_top + table_inset;
            let table_width = layout::CARD_WIDTH - table_inset * 2;
            let table_height = (layout::CARD_BOTTOM_Y - card3_top) - table_inset * 2;

            Some(card.track_table_i18n(
                ActionTable::create_styled(
                    parent,
                    0,
                    table_x,
                    table_y,
                    table_width,
                    table_height,
                    hfont_normal,
                    &[
                        (strings.col_macro_key, 90, ColumnStyle::KeyBadge),
                        (strings.col_macro_val, 150, ColumnStyle::Default),
                        (strings.col_macro_type, 75, ColumnStyle::CenterSecondary),
                        (strings.col_origin, 115, ColumnStyle::Default),
                        (strings.col_action, 124, ColumnStyle::Action),
                    ],
                )?,
                |s| {
                    vec![
                        s.col_macro_key,
                        s.col_macro_val,
                        s.col_macro_type,
                        s.col_origin,
                        s.col_action,
                    ]
                },
            ))
        })?;

        check_use_macro.add_dependents(&[&check_macro_in_english, &btn_add_macro]);

        let tab = Self {
            layout: ly,
            parent_hwnd: parent,
            check_use_macro,
            check_macro_in_english,
            btn_add_macro,
            list_macro,
        };
        Some(tab)
    }

    pub fn paint(&self, hdc: isize, right_bound: i32) {
        self.layout.paint_cards(hdc, layout::CARD_X, right_bound);
    }

    pub fn trigger_edit(parent: isize, data: crate::ui::views::dialogs::MacroEditData) {
        crate::ui::views::dialogs::show_macro_dialog(parent, Some(data));
    }

    pub fn toggle_macro_mute(key: String, mute: bool) {
        crate::ui::controller::set_macro_muted(key, mute);
    }

    pub fn trigger_delete_by_key(parent: isize, key_to_del: String) {
        if key_to_del.is_empty() {
            return;
        }

        let strings = crate::language::current();
        let message = format!("{} \"{}\"?", strings.confirm_delete_macro_msg, key_to_del);
        crate::ui::views::dialogs::show_delete_confirm(parent, message, move || {
            crate::ui::controller::delete_macro(&key_to_del);
        });
    }

    pub fn update_visibility(&self, visible: bool) {
        self.layout.set_visible(visible);
        if visible {
            unsafe {
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
                InvalidateRect(self.list_macro.hwnd(), std::ptr::null(), 0);
            }
        }
    }

    pub fn load_profile(
        &self,
        child: Option<&crate::engine::profile::AppProfile<crate::engine::profile::Child>>,
        global_config: &crate::engine::EngineConfig,
    ) {
        let is_root = child.is_none();
        self.check_use_macro.set_inherited(
            is_root,
            child.and_then(|c| c.use_macro),
            global_config.use_macro,
        );
        self.check_macro_in_english.set_inherited(
            is_root,
            child.and_then(|c| c.use_macro_in_english_mode),
            global_config.use_macro_in_english_mode,
        );
    }

    /// Populates Macro ListView with resolved Live Inheritance items
    pub fn populate_macro_items(&self, items: &[crate::engine::profile::ResolvedMacroItem]) {
        self.list_macro.clear();
        let strings = crate::language::current();
        let parent = self.parent_hwnd;

        let rows: Vec<TableRow> = items
            .iter()
            .map(|entry| {
                let type_str = match entry.macro_type {
                    crate::engine::macro_table::MacroType::Normal => strings.macro_type_normal,
                    crate::engine::macro_table::MacroType::StartConsonant => {
                        strings.macro_type_start
                    }
                    crate::engine::macro_table::MacroType::EndConsonant => strings.macro_type_end,
                };
                let origin_str = entry.origin.localized_badge_label(strings);
                let k_mute = entry.key.clone();
                let k_del = entry.key.clone();
                let edit_data = crate::ui::views::dialogs::MacroEditData {
                    original_key: entry.key.clone(),
                    key: entry.key.clone(),
                    value: entry.value.clone(),
                    macro_type: entry.macro_type,
                };

                let actions = crate::ui::controller::build_rule_actions(
                    entry.origin,
                    entry.is_active,
                    Some(move || Self::trigger_edit(parent, edit_data.clone())),
                    move |mute| Self::toggle_macro_mute(k_mute.clone(), mute),
                    move || Self::trigger_delete_by_key(parent, k_del.clone()),
                );

                TableRow::new()
                    .key(&entry.key)
                    .text(&entry.value)
                    .secondary(type_str)
                    .text(origin_str)
                    .actions(actions)
            })
            .collect();

        self.list_macro.set_rows(rows);
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);

        // Rows and tri-state notes contain localized labels: rebuild them (lock released first).
        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (
                pm.get_profile(&pm.selected_id).cloned(),
                e.config().clone(),
                crate::ui::controller::selected_macro_items(e),
            )
        });
        if let Some((child, cfg, items)) = snap {
            self.load_profile(child.as_ref(), &cfg);
            self.populate_macro_items(&items);
        }
    }

    pub fn on_command(
        &self,
        _controls: &super::ControlPanelControls,
        ctl_id: u32,
        notify_code: u32,
    ) -> bool {
        if self.btn_add_macro.is_clicked(ctl_id, notify_code) {
            crate::ui::views::dialogs::show_macro_dialog(self.parent_hwnd, None);
            return true;
        }
        let is_use_toggled = self.check_use_macro.is_toggled(ctl_id, notify_code);
        let is_en_toggled = self.check_macro_in_english.is_toggled(ctl_id, notify_code);
        if is_use_toggled || is_en_toggled {
            crate::ui::controller::edit_selected(|mut target| {
                target.sync_checkbox(
                    &self.check_use_macro,
                    |c| &mut c.use_macro,
                    |p| &mut p.use_macro,
                );
                target.sync_checkbox(
                    &self.check_macro_in_english,
                    |c| &mut c.use_macro_in_english_mode,
                    |p| &mut p.use_macro_in_english_mode,
                );
            });
            return true;
        }
        false
    }
}
