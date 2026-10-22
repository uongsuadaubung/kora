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
use crate::ui::prelude::*;
use crate::ui::win32::{
    HWND_TOP, InvalidateRect, SW_HIDE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    SetWindowPos, ShowWindow,
};

/// Sub-tab bar geometry
const SUB_TAB_Y: i32 = layout::CARD_START_Y; // 112
const SUB_TAB_W_PER_ITEM: i32 = 125;
const SUB_TAB_HEIGHT: i32 = 30;

/// Top of the shared card below the sub-tab bar (painted in paint.rs)
pub const SUB_CARD_Y: i32 = SUB_TAB_Y + SUB_TAB_HEIGHT + spacing::SPACE_10; // 152

// Shared sub-panel geometry (all sub-tabs use identical slots)
pub const TITLE_Y: i32 = SUB_CARD_Y + spacing::SPACE_12; // 164
pub const TITLE_HEIGHT: i32 = 24;

pub const BTN_WIDTH: i32 = 104;
pub const BTN_HEIGHT: i32 = 28;

pub const TABLE_X: i32 = layout::CARD_CONTENT_X; // 260
pub const TABLE_WIDTH: i32 = layout::CARD_CONTENT_WIDTH; // 530
pub const TABLE_Y: i32 = SUB_CARD_Y + 44; // 196
pub const TABLE_HEIGHT: i32 = layout::CARD_BOTTOM_Y - spacing::SPACE_12 - TABLE_Y; // 324

/// Shared scaffold for a rule table sub-panel (`CheckBox<Inheritable>` + `+ Add` `PushButton` + `ActionTable`).
pub struct RuleSubPanel {
    layout: CardLayout,
    pub check_enabled: CheckBox<Inheritable>,
    pub btn_add: PushButton,
    pub list: ActionTable,
}

impl RuleSubPanel {
    pub fn create(
        parent: isize,
        hfont_normal: isize,
        hfont_bold: isize,
        title_fn: TextFn,
        add_btn_fn: TextFn,
        cols: &[(&str, i32, ColumnStyle)],
        cols_i18n_fn: TextListFn,
    ) -> Option<Self> {
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);
        let check_enabled = ly.inheritable_checkbox(title_fn, true)?;
        let btn_add = ly.button(add_btn_fn, true, ButtonVariant::Primary)?;

        // Auto layout top row: Checkbox on left, stretch, Add button on right
        let mut top_row = HStack::at(TABLE_X, SUB_CARD_Y + spacing::SPACE_8, TABLE_WIDTH, BTN_HEIGHT);
        top_row.align(CrossAlign::Center);
        top_row.add_stretch(1, Some(check_enabled.hwnd()));
        top_row.add_fixed(BTN_WIDTH, Some(btn_add.hwnd()));
        top_row.apply();
        let list = ly.track_table_i18n(
            ActionTable::create_styled(
                parent,
                0,
                TABLE_X,
                TABLE_Y,
                TABLE_WIDTH,
                TABLE_HEIGHT,
                hfont_normal,
                cols,
            )?,
            cols_i18n_fn,
        );
        check_enabled.add_dependents(&[&btn_add, &list]);

        let panel = Self {
            layout: ly,
            check_enabled,
            btn_add,
            list,
        };
        Some(panel)
    }

    pub fn hwnds(&self) -> &[isize] {
        self.layout.hwnds()
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);
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
            SUB_TAB_W_PER_ITEM * sub_titles.len() as i32,
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
        CardLayout::draw_cards(
            hdc,
            layout::CARD_X,
            right_bound,
            &[(SUB_CARD_Y, layout::CARD_BOTTOM_Y)],
        );
    }

    pub fn load_profile(
        &self,
        child: Option<&crate::engine::profile::AppProfile<crate::engine::profile::Child>>,
        global_config: &crate::engine::EngineConfig,
    ) {
        let is_root = child.is_none();
        self.remap.check_enabled.set_inherited(
            is_root,
            child.and_then(|c| c.remap_enabled),
            global_config.remap_enabled,
        );

        self.disable.check_enabled.set_inherited(
            is_root,
            child.and_then(|c| c.disable_enabled),
            global_config.disable_enabled,
        );

        self.key_macro.check_enabled.set_inherited(
            is_root,
            child.and_then(|c| c.key_macro_enabled),
            global_config.key_macro_enabled,
        );
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        if self.sub_tabs.is_changed(ctl_id, notify_code) {
            self.update_visibility(true);
            return true;
        }

        let is_remap_toggle = self.remap.check_enabled.is_toggled(ctl_id, notify_code);
        let is_disable_toggle = self.disable.check_enabled.is_toggled(ctl_id, notify_code);
        let is_macro_toggle = self.key_macro.check_enabled.is_toggled(ctl_id, notify_code);

        if is_remap_toggle || is_disable_toggle || is_macro_toggle {
            let remap_on = self.remap.check_enabled.is_checked();
            let disable_on = self.disable.check_enabled.is_checked();
            let macro_on = self.key_macro.check_enabled.is_checked();
            crate::ui::controller::edit_selected_then(
                |mut target| {
                    target.sync_checkbox(
                        &self.remap.check_enabled,
                        |c| &mut c.remap_enabled,
                        |p| &mut p.remap_enabled,
                    );
                    target.sync_checkbox(
                        &self.disable.check_enabled,
                        |c| &mut c.disable_enabled,
                        |p| &mut p.disable_enabled,
                    );
                    target.sync_checkbox(
                        &self.key_macro.check_enabled,
                        |c| &mut c.key_macro_enabled,
                        |p| &mut p.key_macro_enabled,
                    );
                },
                |engine| {
                    let pm = &mut engine.profile_manager;
                    if pm.selected_id == crate::ui::GLOBAL_PROFILE_ID {
                        pm.global_profile.remap_enabled = remap_on;
                        pm.global_profile.disable_enabled = disable_on;
                        pm.global_profile.key_macro_enabled = macro_on;
                    }
                },
            );
            return true;
        }

        if self.remap.btn_add.is_clicked(ctl_id, notify_code) {
            crate::ui::views::dialogs::show_remap_dialog(hwnd);
            return true;
        }
        if self.disable.btn_add.is_clicked(ctl_id, notify_code) {
            crate::ui::views::dialogs::show_disable_dialog(hwnd);
            return true;
        }
        if self.key_macro.btn_add.is_clicked(ctl_id, notify_code) {
            crate::ui::views::dialogs::show_key_macro_dialog(hwnd);
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

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
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
                InvalidateRect(*h, std::ptr::null(), 0);
            }
        }
    }
}
