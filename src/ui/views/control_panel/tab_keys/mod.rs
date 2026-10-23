//! Tab 2: Keyboard Management (Quản lý phím)
//! Sub-tab architecture coordinating the Keyboard Trifecta & Overview:
//! - Sub-tab 0: Key Overview & Conflict Diagnostics (Tổng quan & Chẩn đoán)
//! - Sub-tab 1: Key Remap (Đổi phím)
//! - Sub-tab 2: Key Disable (Khóa phím)
//! - Sub-tab 3: Key Macro (Phím macro)

pub mod panel_disable;
pub mod panel_macro;
pub mod panel_overview;
pub mod panel_remap;

pub use panel_disable::DisablePanel;
pub use panel_macro::MacroPanel;
pub use panel_overview::OverviewPanel;
pub use panel_remap::RemapPanel;

use crate::engine::profile::{ResolvedDisableItem, ResolvedKeyMacroItem, ResolvedRemapItem};
use crate::language::LanguageStrings;
use crate::ui::components::TabBar;
use crate::ui::tokens::layout;
use crate::ui::tokens::radii;
use crate::ui::tokens::spacing;
use crate::ui::views::control_panel::components::inheritable_switch_row;
use crate::ui::win32::{
    HWND_TOP, InvalidateRect, SW_HIDE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    SetWindowPos, ShowWindow,
};
use crate::ui_framework::components::action_table::{ActionTable, ColumnStyle};
use crate::ui_framework::core::*;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

type TextFn = fn(&LanguageStrings) -> &'static str;

/// Sub-tab bar geometry
const SUB_TAB_Y: i32 = layout::CARD_START_Y; // 112
const SUB_TAB_WIDTH: i32 = layout::CARD_WIDTH;
const SUB_TAB_HEIGHT: i32 = layout::TAB_BAR_HEIGHT;

/// Top of the shared card below the sub-tab bar (painted in paint.rs)
pub const SUB_CARD_Y: i32 = SUB_TAB_Y + SUB_TAB_HEIGHT + spacing::SPACE_10; // 152

// Shared sub-panel geometry (all sub-tabs use identical slots)
pub const BTN_WIDTH: i32 = 104;

pub const TABLE_X: i32 = layout::CARD_CONTENT_X; // 260
pub const TABLE_WIDTH: i32 = layout::CARD_CONTENT_WIDTH; // 530
pub const TABLE_Y: i32 = SUB_CARD_Y + 44; // 196
pub const TABLE_HEIGHT: i32 = layout::CARD_BOTTOM_Y - spacing::SPACE_12 - TABLE_Y; // 324

#[derive(Clone, Debug)]
pub enum RuleMsg {
    Toggle(Option<bool>),
    Add,
}

#[derive(Default)]
pub struct RuleSubPanelState {
    pub is_root: bool,
    pub override_val: Option<bool>,
    pub global_val: bool,
}

/// Shared scaffold for a rule table sub-panel (`Surface<RuleMsg>` + `ActionTable`).
pub struct RuleSubPanel {
    pub surface: Surface<RuleMsg>,
    pub list: ActionTable,
    pub state: Arc<RwLock<RuleSubPanelState>>,
    pub title_fn: TextFn,
    pub add_btn_fn: TextFn,
    hwnd_slice: [isize; 2],
}

impl RuleSubPanel {
    pub fn create(
        parent: isize,
        hfont_normal: isize,
        _hfont_bold: isize,
        title_fn: TextFn,
        add_btn_fn: TextFn,
        cols: &[(&str, i32, ColumnStyle)],
        on_add: impl Fn(isize) + Send + Sync + 'static,
        on_toggle: impl Fn(&mut crate::ui::controller::EditTarget, Option<bool>) + Send + Sync + 'static,
    ) -> Option<Self> {
        let surface_x = TABLE_X;
        let surface_y = SUB_CARD_Y + 8;
        let surface_w = TABLE_WIDTH;
        let surface_h = 32;

        let surface = Surface::create(parent, surface_x, surface_y, surface_w, surface_h)?;
        surface.set_bg(BgStyle::Card);
        surface.set_content_width(surface_w);

        let list = ActionTable::create_styled(
            parent,
            0,
            TABLE_X,
            TABLE_Y,
            TABLE_WIDTH,
            TABLE_HEIGHT,
            hfont_normal,
            cols,
        )?;

        let state = Arc::new(RwLock::new(RuleSubPanelState::default()));

        let state_clone = state.clone();
        let list_hwnd = list.hwnd();
        let on_add = Arc::new(on_add);
        let on_toggle = Arc::new(on_toggle);

        let on_add_clone = on_add.clone();
        let on_toggle_clone = on_toggle.clone();

        surface.set_on_msg(move |msg| match msg {
            RuleMsg::Add => {
                on_add_clone(parent);
            }
            RuleMsg::Toggle(new_val) => {
                crate::ui::controller::edit_selected(|mut target| {
                    on_toggle_clone(&mut target, new_val);
                });
                if let Ok(mut st) = state_clone.write() {
                    if st.is_root {
                        st.global_val = new_val.unwrap_or(true);
                    } else {
                        st.override_val = new_val;
                    }
                    let effective = st.override_val.unwrap_or(st.global_val);
                    unsafe {
                        crate::ui::win32::EnableWindow(list_hwnd, effective as i32);
                        InvalidateRect(list_hwnd, null_mut(), 0);
                    }
                }
                if let Some(c) = crate::ui::controls() {
                    c.tab_keys.render_active_subpanel();
                }
            }
        });

        let hwnd_slice = [surface.hwnd(), list.hwnd()];

        let panel = Self {
            surface,
            list,
            state,
            title_fn,
            add_btn_fn,
            hwnd_slice,
        };

        panel.render();
        Some(panel)
    }

    pub fn render(&self) {
        let strings = crate::language::current();
        if let Ok(st) = self.state.read() {
            self.surface.render(Self::view(&st, self.title_fn, self.add_btn_fn, strings));
        }
    }

    fn view(
        state: &RuleSubPanelState,
        title_fn: TextFn,
        add_btn_fn: TextFn,
        strings: &LanguageStrings,
    ) -> Element<RuleMsg> {
        let is_root = state.is_root;
        let override_val = state.override_val;
        let global_val = state.global_val;
        let effective = override_val.unwrap_or(global_val);

        let sw = inheritable_switch_row(
            title_fn(strings),
            None,
            is_root,
            override_val,
            global_val,
            strings,
            RuleMsg::Toggle,
        );

        let btn = button(add_btn_fn(strings), Variant::Primary, RuleMsg::Add)
            .disabled(!effective)
            .w(BTN_WIDTH);

        row(vec![
            sw.flex_1(),
            btn,
        ])
        .items_center()
        .w_full()
    }

    pub fn set_inherited(&self, is_root: bool, override_val: Option<bool>, global_val: bool) {
        if let Ok(mut st) = self.state.write() {
            st.is_root = is_root;
            st.override_val = override_val;
            st.global_val = global_val;
        }
        let effective = override_val.unwrap_or(global_val);
        self.list.set_enabled(effective);
        self.render();
    }

    pub fn hwnds(&self) -> &[isize] {
        &self.hwnd_slice
    }

    pub fn update_language(&self, _strings: &LanguageStrings) {
        self.render();
    }
}

/// TabKeys: Master coordinator for the Keyboard Management tab
pub struct TabKeys {
    pub parent_hwnd: isize,
    pub sub_tabs: TabBar,
    pub overview: OverviewPanel,
    pub remap: RemapPanel,
    pub disable: DisablePanel,
    pub key_macro: MacroPanel,
}

impl TabKeys {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        let sub_titles = [
            strings.keys_sub_overview,
            strings.keys_sub_remap,
            strings.keys_sub_disable,
            strings.keys_sub_macro,
        ];
        let sub_tabs = TabBar::create(
            parent,
            0,
            layout::CARD_X,
            SUB_TAB_Y,
            SUB_TAB_WIDTH,
            SUB_TAB_HEIGHT,
            hfont_normal,
            hfont_bold,
            &sub_titles,
        )?;

        let overview = OverviewPanel::create(parent, hfont_normal, hfont_bold)?;
        let remap = RemapPanel::create(parent, hfont_normal, hfont_bold)?;
        let disable = DisablePanel::create(parent, hfont_normal, hfont_bold)?;
        let key_macro = MacroPanel::create(parent, hfont_normal, hfont_bold)?;

        let tab = Self {
            parent_hwnd: parent,
            sub_tabs,
            overview,
            remap,
            disable,
            key_macro,
        };
        Some(tab)
    }

    pub fn paint(&self, hdc: isize, right_bound: i32) {
        if self.sub_tabs.get_cur_sel() == 0 {
            self.overview.invalidate();
            return;
        }
        let is_dark = crate::ui::theme::is_current_dark();
        let palette = crate::ui::colors::ThemePalette::get(is_dark);
        let rc = crate::ui::win32::RECT {
            left: layout::CARD_X,
            top: SUB_CARD_Y,
            right: right_bound,
            bottom: layout::CARD_BOTTOM_Y,
        };
        crate::ui::win32::round_rect(
            hdc,
            &rc,
            radii::RADIUS_MD,
            palette.bg_card,
            Some(palette.border),
        );
    }

    pub fn render_active_subpanel(&self) {
        match self.sub_tabs.get_cur_sel() {
            1 => self.remap.render(),
            2 => self.disable.render(),
            3 => self.key_macro.render(),
            _ => {}
        }
    }

    pub fn load_profile(
        &self,
        child: Option<&crate::engine::profile::AppProfile<crate::engine::profile::Child>>,
        global_config: &crate::engine::EngineConfig,
    ) {
        let is_root = child.is_none();
        self.remap.set_inherited(
            is_root,
            child.and_then(|c| c.remap_enabled),
            global_config.remap_enabled,
        );

        self.disable.set_inherited(
            is_root,
            child.and_then(|c| c.disable_enabled),
            global_config.disable_enabled,
        );

        self.key_macro.set_inherited(
            is_root,
            child.and_then(|c| c.key_macro_enabled),
            global_config.key_macro_enabled,
        );
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        if self.sub_tabs.is_changed(ctl_id, notify_code) {
            self.update_visibility(true);
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
            return true;
        }
        false
    }

    /// Populates Remap ActionTable with resolved Live Inheritance items
    pub fn populate_remaps(&self, items: &[ResolvedRemapItem]) {
        self.remap.populate(items, self.parent_hwnd);
    }

    /// Populates Disable ActionTable with resolved Live Inheritance items
    pub fn populate_disabled(&self, items: &[ResolvedDisableItem]) {
        self.disable.populate(items, self.parent_hwnd);
    }

    /// Populates Key Macro ActionTable with resolved Live Inheritance items
    pub fn populate_key_macros(&self, items: &[ResolvedKeyMacroItem]) {
        self.key_macro.populate(items, self.parent_hwnd);
    }

    pub fn refresh_overview(
        &self,
        remaps: &[ResolvedRemapItem],
        disabled: &[ResolvedDisableItem],
        key_macros: &[ResolvedKeyMacroItem],
    ) {
        let (remap_on, disable_on, macro_on, switch_key, switch_key_enabled, switch_key_origin) =
            crate::ui::controller::with_engine(|e| {
                let pm = &e.profile_manager;
                let sel_id = &pm.selected_id;
                let eff = pm.resolve_effective_config(sel_id, e.config());
                let origin = if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                    crate::engine::profile::RuleOrigin::Root
                } else if pm
                    .get_profile(sel_id)
                    .map(|p| p.switch_key.is_some())
                    .unwrap_or(false)
                {
                    crate::engine::profile::RuleOrigin::AppSpecific
                } else {
                    crate::engine::profile::RuleOrigin::InheritedGlobal
                };
                (
                    eff.remap_enabled,
                    eff.disable_enabled,
                    eff.key_macro_enabled,
                    eff.switch_key,
                    eff.switch_key_enabled,
                    origin,
                )
            })
            .unwrap_or((
                false,
                false,
                false,
                crate::engine::config::Hotkey::default(),
                false,
                crate::engine::profile::RuleOrigin::Root,
            ));

        let diag = self.overview.populate(
            remaps,
            remap_on,
            disabled,
            disable_on,
            key_macros,
            macro_on,
            switch_key,
            switch_key_enabled,
            switch_key_origin,
        );

        let strings = crate::language::current();
        let overview_title = if diag.is_healthy() {
            strings.keys_sub_overview.to_string()
        } else {
            format!("{} ⚠️", strings.keys_sub_overview)
        };

        self.sub_tabs.set_tabs(&[
            overview_title.as_str(),
            strings.keys_sub_remap,
            strings.keys_sub_disable,
            strings.keys_sub_macro,
        ]);
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.remap.list.set_column_titles(&[
            strings.col_remap_from,
            strings.col_remap_to,
            strings.col_origin,
            strings.col_action,
        ]);
        self.disable.list.set_column_titles(&[
            strings.col_disable_key,
            strings.col_status,
            strings.col_origin,
            strings.col_action,
        ]);
        self.key_macro.list.set_column_titles(&[
            strings.col_key_macro_trigger,
            strings.col_key_macro_name,
            strings.col_origin,
            strings.col_action,
        ]);
        self.sub_tabs.set_tabs(&[
            strings.keys_sub_overview,
            strings.keys_sub_remap,
            strings.keys_sub_disable,
            strings.keys_sub_macro,
        ]);
        self.overview.update_language(strings);
        self.remap.update_language(strings);
        self.disable.update_language(strings);
        self.key_macro.update_language(strings);

        // Rows and tri-state notes contain localized labels: rebuild them (engine lock released first).
        let items = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (
                pm.get_profile(&pm.selected_id).cloned(),
                e.config().clone(),
                crate::ui::controller::selected_remap_items(e),
                crate::ui::controller::selected_disable_items(e),
                crate::ui::controller::selected_key_macro_items(e),
            )
        });
        if let Some((child, cfg, remaps, disabled, macros)) = items {
            self.load_profile(child.as_ref(), &cfg);
            self.populate_remaps(&remaps);
            self.populate_disabled(&disabled);
            self.populate_key_macros(&macros);
            self.refresh_overview(&remaps, &disabled, &macros);
        }
    }

    pub fn update_visibility(&self, visible: bool) {
        let sub = self.sub_tabs.get_cur_sel();
        let panels = [
            (self.overview.hwnds(), visible && sub == 0),
            (self.remap.hwnds(), visible && sub == 1),
            (self.disable.hwnds(), visible && sub == 2),
            (self.key_macro.hwnds(), visible && sub == 3),
        ];
        unsafe {
            ShowWindow(
                self.sub_tabs.hwnd(),
                if visible { SW_SHOW } else { SW_HIDE },
            );
            // Hide outgoing panels first so they never overlap the incoming one
            for h in panels.iter().filter(|(_, on)| !on).flat_map(|(hs, _)| *hs) {
                ShowWindow(*h, SW_HIDE);
            }
            for h in panels.iter().filter(|(_, on)| *on).flat_map(|(hs, _)| *hs) {
                SetWindowPos(
                    *h,
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_SHOWWINDOW,
                );
                InvalidateRect(*h, null_mut(), 0);
            }
        }
    }
}
