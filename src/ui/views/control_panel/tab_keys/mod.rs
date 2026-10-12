//! Tab 2: Keyboard Management (Quản lý phím)
//! Sub-tab architecture coordinating the Keyboard Trifecta:
//! - Sub-tab 0: Key Remap (Đổi phím)
//! - Sub-tab 1: Key Disable (Khóa phím)
//! - Sub-tab 2 (extensible): Key Macro (Phím macro)

pub mod panel_disable;
pub mod panel_remap;

pub use panel_disable::DisablePanel;
pub use panel_remap::RemapPanel;

use crate::engine::profile::{ResolvedDisableItem, ResolvedRemapItem};
use crate::ui::components::TabBar;
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
    pub remap: RemapPanel,
    pub disable: DisablePanel,
}

impl TabKeys {
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

        let remap = RemapPanel::create(parent, hfont_normal, hfont_bold)?;
        let disable = DisablePanel::create(parent, hfont_normal, hfont_bold)?;

        let tab = Self {
            parent_hwnd: parent,
            sub_tabs,
            remap,
            disable,
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
    }

    pub fn read_config(&self, config: &mut crate::engine::EngineConfig) {
        config.remap_enabled = self.remap.check_enabled.is_checked();
        config.disable_enabled = self.disable.check_enabled.is_checked();
    }

    pub fn read_profile(&self, profile: &mut crate::engine::profile::AppProfile) {
        profile.remap_enabled = self.remap.check_enabled.tri_state();
        profile.disable_enabled = self.disable.check_enabled.tri_state();
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        use crate::ui::controller::EditTarget;
        match ctl_id {
            IDC_TAB_KEYS_SUB => {
                self.update_visibility(true);
                true
            }
            IDC_CHECK_REMAP_ENABLED | IDC_CHECK_DISABLE_ENABLED => {
                if notify_code != 0 {
                    return false;
                }
                let is_remap = ctl_id == IDC_CHECK_REMAP_ENABLED;
                if is_remap {
                    self.remap.update_controls_state();
                } else {
                    self.disable.update_controls_state();
                }
                let remap_on = self.remap.check_enabled.is_checked();
                let disable_on = self.disable.check_enabled.is_checked();
                crate::ui::controller::edit_selected_then(
                    |target| match target {
                        EditTarget::Global(cfg) => self.read_config(cfg),
                        EditTarget::Child(child) => self.read_profile(child),
                    },
                    |engine| {
                        // Root profile mirrors the global toggle in its profile record too.
                        let pm = &mut engine.profile_manager;
                        if pm.selected_id == crate::ui::GLOBAL_PROFILE_ID {
                            if is_remap {
                                pm.global_profile.remap_enabled = Some(remap_on);
                            } else {
                                pm.global_profile.disable_enabled = Some(disable_on);
                            }
                        }
                    },
                );
                true
            }
            // The dialogs are modeless and refresh the lists themselves on save.
            IDC_BTN_ADD_REMAP => {
                crate::ui::views::dialogs::show_remap_dialog(hwnd);
                true
            }
            IDC_BTN_ADD_DISABLE => {
                crate::ui::views::dialogs::show_disable_dialog(hwnd);
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

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.sub_tabs
            .set_tabs(&[strings.keys_sub_remap, strings.keys_sub_disable]);
        self.remap.update_language(strings);
        self.disable.update_language(strings);

        // Rows contain localized labels: rebuild them (engine lock released first).
        let items = crate::ui::controller::with_engine(|e| {
            (
                crate::ui::controller::selected_remap_items(e),
                crate::ui::controller::selected_disable_items(e),
            )
        });
        if let Some((remaps, disabled)) = items {
            self.populate_remaps(&remaps);
            self.populate_disabled(&disabled);
        }
    }

    pub fn update_visibility(&self, visible: bool) {
        let sub = self.sub_tabs.get_cur_sel();
        let panels = [
            (self.remap.hwnds(), visible && sub == 0),
            (self.disable.hwnds(), visible && sub == 1),
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
                InvalidateRect(*h, std::ptr::null(), 1);
            }
        }
    }
}

