//! Tab 1: Macro controls (Gõ tắt)
//! Powered by the Declarative Single-HWND UI Engine (`Surface<MacroMsg>`) + `ActionTable`.

use crate::language::LanguageStrings;
use crate::ui::tokens::layout;
use crate::ui::views::control_panel::components::inheritable_switch_row;
use crate::ui::win32::{
    HWND_TOP, InvalidateRect, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SetWindowPos,
};
use crate::ui_framework::components::action_table::{ActionTable, ColumnStyle, TableRow};
use crate::ui_framework::core::*;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub enum MacroMsg {
    ToggleUseMacro(Option<bool>),
    ToggleEnglishMode(Option<bool>),
    AddMacro,
}

#[derive(Default)]
struct MacroViewState {
    is_root: bool,
    use_macro_override: Option<bool>,
    global_use_macro: bool,
    use_macro_en_override: Option<bool>,
    global_use_macro_en: bool,
}

pub struct TabMacro {
    pub parent_hwnd: isize,
    surface: Surface<MacroMsg>,
    pub list_macro: ActionTable,
    state: Arc<RwLock<MacroViewState>>,
}

impl TabMacro {
    pub fn create(parent: isize, hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let surface_x = layout::CARD_X;
        let surface_y = layout::CARD_START_Y;
        let surface_w = layout::CARD_WIDTH;
        let surface_h = 100;

        let surface = Surface::create(parent, surface_x, surface_y, surface_w, surface_h)?;
        surface.set_content_width(surface_w);

        let table_y = surface_y + surface_h + 8;
        let table_h = layout::CARD_BOTTOM_Y - table_y;

        let list_macro = ActionTable::create_styled(
            parent,
            0,
            surface_x,
            table_y,
            surface_w,
            table_h,
            hfont_normal,
            &[
                (strings.col_macro_key, 90, ColumnStyle::KeyBadge),
                (strings.col_macro_val, 150, ColumnStyle::Default),
                (strings.col_macro_type, 75, ColumnStyle::CenterSecondary),
                (strings.col_origin, 115, ColumnStyle::Default),
                (strings.col_action, 124, ColumnStyle::Action),
            ],
        )?;

        let state = Arc::new(RwLock::new(MacroViewState::default()));

        let state_clone = state.clone();
        let list_macro_hwnd = list_macro.hwnd();
        surface.set_on_msg(move |msg| match msg {
            MacroMsg::AddMacro => {
                crate::ui::views::dialogs::show_macro_dialog(parent, None);
            }
            MacroMsg::ToggleUseMacro(new_val) => {
                crate::ui::controller::edit_selected(|target| match target {
                    crate::ui::controller::EditTarget::Global(cfg) => {
                        cfg.use_macro = new_val.unwrap_or(true);
                    }
                    crate::ui::controller::EditTarget::Child(p) => {
                        p.use_macro = new_val;
                    }
                });
                if let Ok(mut st) = state_clone.write() {
                    if st.is_root {
                        st.global_use_macro = new_val.unwrap_or(true);
                    } else {
                        st.use_macro_override = new_val;
                    }
                    let enabled = st.use_macro_override.unwrap_or(st.global_use_macro);
                    unsafe {
                        crate::ui::win32::EnableWindow(list_macro_hwnd, enabled as i32);
                        InvalidateRect(list_macro_hwnd, null_mut(), 0);
                    }
                }
                if let Some(c) = crate::ui::controls() {
                    c.tab_macro.render();
                }
            }
            MacroMsg::ToggleEnglishMode(new_val) => {
                crate::ui::controller::edit_selected(|target| match target {
                    crate::ui::controller::EditTarget::Global(cfg) => {
                        cfg.use_macro_in_english_mode = new_val.unwrap_or(false);
                    }
                    crate::ui::controller::EditTarget::Child(p) => {
                        p.use_macro_in_english_mode = new_val;
                    }
                });
                if let Ok(mut st) = state_clone.write() {
                    if st.is_root {
                        st.global_use_macro_en = new_val.unwrap_or(false);
                    } else {
                        st.use_macro_en_override = new_val;
                    }
                }
                if let Some(c) = crate::ui::controls() {
                    c.tab_macro.render();
                }
            }
        });

        let tab = Self {
            parent_hwnd: parent,
            surface,
            list_macro,
            state,
        };

        tab.render();
        Some(tab)
    }

    pub fn render(&self) {
        let strings = crate::language::current();
        if let Ok(st) = self.state.read() {
            self.surface.render(Self::view(&st, strings));
        }
    }

    fn view(state: &MacroViewState, strings: &LanguageStrings) -> Element<MacroMsg> {
        let is_root = state.is_root;
        let use_macro_override = state.use_macro_override;
        let global_use_macro = state.global_use_macro;
        let en_override = state.use_macro_en_override;
        let global_en = state.global_use_macro_en;

        let effective_use_macro = use_macro_override.unwrap_or(global_use_macro);

        let sw_use_macro = inheritable_switch_row(
            strings.check_use_macro,
            None,
            is_root,
            use_macro_override,
            global_use_macro,
            strings,
            MacroMsg::ToggleUseMacro,
        );

        let sw_en = inheritable_switch_row(
            strings.check_macro_in_english,
            None,
            is_root,
            en_override,
            global_en,
            strings,
            MacroMsg::ToggleEnglishMode,
        );

        let add_btn = button(strings.btn_add_macro, Variant::Primary, MacroMsg::AddMacro)
            .disabled(!effective_use_macro)
            .w(104);

        col(vec![
            card(vec![row(vec![
                sw_use_macro.flex_1(),
                sw_en.flex_1(),
            ])
            .gap(16)
            .items_center()]),
            row(vec![add_btn]),
        ])
        .gap(8)
        .w_full()
    }

    pub fn paint(&self, _hdc: isize, _right_bound: i32) {}

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
        self.surface.set_visible(visible);
        self.list_macro.set_visible(visible);
        if visible {
            unsafe {
                SetWindowPos(
                    self.list_macro.hwnd(),
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW,
                );
                InvalidateRect(self.list_macro.hwnd(), null_mut(), 0);
            }
        }
    }

    pub fn load_profile(
        &self,
        child: Option<&crate::engine::profile::AppProfile<crate::engine::profile::Child>>,
        global_config: &crate::engine::EngineConfig,
    ) {
        let is_root = child.is_none();
        if let Ok(mut st) = self.state.write() {
            st.is_root = is_root;
            st.use_macro_override = child.and_then(|c| c.use_macro);
            st.global_use_macro = global_config.use_macro;
            st.use_macro_en_override = child.and_then(|c| c.use_macro_in_english_mode);
            st.global_use_macro_en = global_config.use_macro_in_english_mode;
        }
        let effective_enabled = child
            .and_then(|c| c.use_macro)
            .unwrap_or(global_config.use_macro);
        self.list_macro.set_enabled(effective_enabled);
        self.render();
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
        self.list_macro.set_column_titles(&[
            strings.col_macro_key,
            strings.col_macro_val,
            strings.col_macro_type,
            strings.col_origin,
            strings.col_action,
        ]);

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
        } else {
            self.render();
        }
    }

    #[inline]
    pub fn on_command(
        &self,
        _controls: &super::ControlPanelControls,
        _ctl_id: u32,
        _notify_code: u32,
    ) -> bool {
        false
    }
}
