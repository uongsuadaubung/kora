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
use crate::ui::components::TabBar;
use crate::ui::tokens::{id::*, layout, spacing};
use crate::ui::win32::{
    HWND_TOP, InvalidateRect, SendMessageW, SetWindowPos, ShowWindow, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, SW_HIDE, SW_SHOW, WM_COMMAND,
};

/// Sub-tab bar geometry
const SUB_TAB_Y: i32 = layout::CARD_START_Y; // 112
const SUB_TAB_W_PER_ITEM: i32 = 125;
const SUB_TAB_HEIGHT: i32 = 30;

/// Top of the shared card below the sub-tab bar (painted in paint.rs)
pub const SUB_CARD_Y: i32 = SUB_TAB_Y + SUB_TAB_HEIGHT + spacing::SPACE_10; // 152

// Shared sub-panel geometry (all sub-tabs use identical slots)
pub const TITLE_X: i32 = layout::CARD_CONTENT_X; // 260
pub const TITLE_Y: i32 = SUB_CARD_Y + spacing::SPACE_12; // 164
pub const TITLE_WIDTH: i32 = 380;
pub const TITLE_HEIGHT: i32 = 24;

pub const BTN_WIDTH: i32 = 104;
pub const BTN_HEIGHT: i32 = 28;
pub const BTN_X: i32 = layout::CARD_X + layout::CARD_WIDTH - spacing::SPACE_16 - BTN_WIDTH; // 686
pub const BTN_Y: i32 = SUB_CARD_Y + spacing::SPACE_8 + 1; // 161

pub const TABLE_X: i32 = layout::CARD_CONTENT_X; // 260
pub const TABLE_WIDTH: i32 = layout::CARD_CONTENT_WIDTH; // 530
pub const TABLE_Y: i32 = SUB_CARD_Y + 44; // 196
pub const TABLE_HEIGHT: i32 = layout::CARD_BOTTOM_Y - spacing::SPACE_12 - TABLE_Y; // 324

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
            IDC_TAB_KEYS_SUB,
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

    pub fn load_config(&self, config: &crate::engine::EngineConfig) {
        self.remap.check_enabled.set_checked(config.remap_enabled);
        self.remap.update_controls_state();

        self.disable
            .check_enabled
            .set_checked(config.disable_enabled);
        self.disable.update_controls_state();

        self.key_macro
            .check_enabled
            .set_checked(config.key_macro_enabled);
        self.key_macro.update_controls_state();
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
        self.remap
            .check_enabled
            .set_tri_state(profile.remap_enabled, global_config.remap_enabled);
        self.remap.update_controls_state();

        self.disable
            .check_enabled
            .set_tri_state(profile.disable_enabled, global_config.disable_enabled);
        self.disable.update_controls_state();

        self.key_macro
            .check_enabled
            .set_tri_state(profile.key_macro_enabled, global_config.key_macro_enabled);
        self.key_macro.update_controls_state();
    }

    pub fn read_config(&self, config: &mut crate::engine::EngineConfig) {
        config.remap_enabled = self.remap.check_enabled.is_checked();
        config.disable_enabled = self.disable.check_enabled.is_checked();
        config.key_macro_enabled = self.key_macro.check_enabled.is_checked();
    }

    pub fn read_profile(&self, profile: &mut crate::engine::profile::AppProfile) {
        profile.remap_enabled = self.remap.check_enabled.tri_state();
        profile.disable_enabled = self.disable.check_enabled.tri_state();
        profile.key_macro_enabled = self.key_macro.check_enabled.tri_state();
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        use crate::ui::controller::EditTarget;
        match ctl_id {
            IDC_TAB_KEYS_SUB => {
                self.update_visibility(true);
                true
            }
            IDC_CHECK_REMAP_ENABLED | IDC_CHECK_DISABLE_ENABLED | IDC_CHECK_KEY_MACRO_ENABLED => {
                if notify_code != 0 {
                    return false;
                }
                match ctl_id {
                    IDC_CHECK_REMAP_ENABLED => self.remap.update_controls_state(),
                    IDC_CHECK_DISABLE_ENABLED => self.disable.update_controls_state(),
                    IDC_CHECK_KEY_MACRO_ENABLED => self.key_macro.update_controls_state(),
                    _ => {}
                }
                let remap_on = self.remap.check_enabled.is_checked();
                let disable_on = self.disable.check_enabled.is_checked();
                let macro_on = self.key_macro.check_enabled.is_checked();
                crate::ui::controller::edit_selected_then(
                    |target| match target {
                        EditTarget::Global(cfg) => self.read_config(cfg),
                        EditTarget::Child(child) => self.read_profile(child),
                    },
                    |engine| {
                        // Root profile mirrors the global toggle in its profile record too.
                        let pm = &mut engine.profile_manager;
                        if pm.selected_id == crate::ui::GLOBAL_PROFILE_ID {
                            match ctl_id {
                                IDC_CHECK_REMAP_ENABLED => pm.global_profile.remap_enabled = Some(remap_on),
                                IDC_CHECK_DISABLE_ENABLED => pm.global_profile.disable_enabled = Some(disable_on),
                                IDC_CHECK_KEY_MACRO_ENABLED => pm.global_profile.key_macro_enabled = Some(macro_on),
                                _ => {}
                            }
                        }
                    },
                );
                true
            }
            IDC_BTN_ADD_REMAP => {
                crate::ui::views::dialogs::show_remap_dialog(hwnd);
                true
            }
            IDC_BTN_ADD_DISABLE => {
                crate::ui::views::dialogs::show_disable_dialog(hwnd);
                true
            }
            IDC_BTN_ADD_KEY_MACRO => {
                crate::ui::views::dialogs::show_key_macro_dialog(hwnd);
                true
            }
            _ => false,
        }
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
                } else if pm.get_profile(sel_id).map(|p| p.switch_key.is_some()).unwrap_or(false) {
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

        let sub_tabs = self.sub_tabs.clone();
        let parent_hwnd = self.parent_hwnd;
        let on_nav = move |target_sub: usize| {
            if target_sub == 0 {
                crate::ui::views::dialogs::show_hotkey_dialog(parent_hwnd);
            } else {
                sub_tabs.set_cur_sel(target_sub);
                unsafe {
                    SendMessageW(
                        parent_hwnd,
                        WM_COMMAND,
                        (IDC_TAB_KEYS_SUB & 0xFFFF) as usize,
                        sub_tabs.hwnd(),
                    );
                }
            }
        };

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
            on_nav,
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

        // Rows contain localized labels: rebuild them (engine lock released first).
        let items = crate::ui::controller::with_engine(|e| {
            (
                crate::ui::controller::selected_remap_items(e),
                crate::ui::controller::selected_disable_items(e),
                crate::ui::controller::selected_key_macro_items(e),
            )
        });
        if let Some((remaps, disabled, macros)) = items {
            self.populate_remaps(&remaps);
            self.populate_disabled(&disabled);
            self.populate_key_macros(&macros);
            self.refresh_overview(&remaps, &disabled, &macros);
        }
    }

    pub fn update_visibility(&self, visible: bool) {
        let sub = self.sub_tabs.get_cur_sel();
        let panels = [
            (self.overview.hwnds().to_vec(), visible && sub == 0),
            (self.remap.hwnds().to_vec(), visible && sub == 1),
            (self.disable.hwnds().to_vec(), visible && sub == 2),
            (self.key_macro.hwnds().to_vec(), visible && sub == 3),
        ];
        unsafe {
            ShowWindow(
                self.sub_tabs.hwnd(),
                if visible { SW_SHOW } else { SW_HIDE },
            );
            // Hide outgoing panels first so they never overlap the incoming one
            for h in panels.iter().filter(|(_, on)| !on).flat_map(|(hs, _)| hs) {
                ShowWindow(*h, SW_HIDE);
            }
            for h in panels.iter().filter(|(_, on)| *on).flat_map(|(hs, _)| hs) {
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

