//! Main Control Panel View for Kora Multi-Profile Workspace Hub
//! 2-Column Layout (Sidebar 230px + Right Workspace Panel 590px)

pub mod banner;
pub mod footer;
pub mod paint;
pub mod sidebar;
pub mod tab_about;
pub mod tab_macro;
pub mod tab_remap;
pub mod tab_sound;
pub mod tab_system;
pub mod tab_typing;

pub use banner::*;
pub use footer::*;
pub use paint::*;
pub use sidebar::*;
pub use tab_about::*;
pub use tab_macro::*;
pub use tab_remap::*;
pub use tab_sound::*;
pub use tab_system::*;
pub use tab_typing::*;

use crate::engine::EngineConfig;
use crate::engine::profile::AppProfile;
use crate::language::Language;
use crate::ui::components::*;
use crate::ui::win32::{InvalidateRect, MEASUREITEMSTRUCT};
pub use crate::ui::tokens::id::*;
pub use crate::ui::tokens::layout::*;
use std::ptr::null_mut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Profile(usize),
    SystemSettings,
    About,
}

pub struct ControlPanelControls {
    pub sidebar: SidebarControls,
    pub banner: ProfileBanner,
    pub tab_bar: TabBar,
    pub tab_typing: TabTyping,
    pub tab_remap: TabRemapDisable,
    pub tab_sound: TabSound,
    pub tab_macro: TabMacro,
    pub tab_system: TabSystem,
    pub tab_about: TabAbout,
    pub footer: FooterControls,
    pub current_view_mode: std::sync::RwLock<ViewMode>,
}

impl ControlPanelControls {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let sidebar = SidebarControls::create(parent, hfont_normal, hfont_bold)?;
        let banner = ProfileBanner::create(parent, hfont_normal, hfont_bold)?;

        let strings = crate::language::current();
        let tab_bar = TabBar::create(
            parent,
            IDC_TAB_MAIN,
            TAB_BAR_X,
            TAB_BAR_Y,
            TAB_BAR_WIDTH,
            TAB_BAR_HEIGHT,
            hfont_normal,
            hfont_bold,
            &strings.tab_titles(),
        )?;

        let tab_typing = TabTyping::create(parent, hfont_normal, hfont_bold)?;
        let tab_remap = TabRemapDisable::create(parent, hfont_normal, hfont_bold)?;
        let tab_sound = TabSound::create(parent, hfont_normal, hfont_bold)?;
        let tab_macro = TabMacro::create(parent, hfont_normal, hfont_bold)?;
        let tab_system = TabSystem::create(parent, hfont_normal, hfont_bold)?;
        let tab_about = TabAbout::create(parent, hfont_normal, hfont_bold)?;
        let footer = FooterControls::create(parent, hfont_normal, hfont_bold)?;

        Some(Self {
            sidebar,
            banner,
            tab_bar,
            tab_typing,
            tab_remap,
            tab_sound,
            tab_macro,
            tab_system,
            tab_about,
            footer,
            current_view_mode: std::sync::RwLock::new(ViewMode::Profile(0)),
        })
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.current_view_mode
            .read()
            .map(|g| *g)
            .unwrap_or(ViewMode::Profile(0))
    }

    /// Dynamically switches visibility between Profile Subtabs, System Settings, and About
    pub fn update_view_mode(&self, mode: ViewMode) {
        if let Ok(mut lock) = self.current_view_mode.write() {
            *lock = mode;
        }
        self.sidebar.set_view_mode(mode);
        match mode {
            ViewMode::Profile(subtab) => {
                self.banner.set_visible(true);
                self.tab_bar.set_visible(true);
                self.tab_typing.update_visibility(subtab == 0);
                self.tab_remap.update_visibility(subtab == 1);
                self.tab_sound.update_visibility(subtab == 2);
                self.tab_macro.update_visibility(subtab == 3);
                self.tab_system.update_visibility(false);
                self.tab_about.update_visibility(false);
            }
            ViewMode::SystemSettings => {
                self.banner.set_visible(false);
                self.tab_bar.set_visible(false);
                self.tab_typing.update_visibility(false);
                self.tab_remap.update_visibility(false);
                self.tab_sound.update_visibility(false);
                self.tab_macro.update_visibility(false);
                self.tab_system.update_visibility(true);
                self.tab_about.update_visibility(false);
            }
            ViewMode::About => {
                self.banner.set_visible(false);
                self.tab_bar.set_visible(false);
                self.tab_typing.update_visibility(false);
                self.tab_remap.update_visibility(false);
                self.tab_sound.update_visibility(false);
                self.tab_macro.update_visibility(false);
                self.tab_system.update_visibility(false);
                self.tab_about.update_visibility(true);
            }
        }
        unsafe {
            InvalidateRect(self.sidebar.btn_system_settings.hwnd(), null_mut(), 0);
            InvalidateRect(self.sidebar.btn_about.hwnd(), null_mut(), 0);
        }
    }

    pub fn update_tab_visibility(&self, cur_tab: usize) {
        self.update_view_mode(ViewMode::Profile(cur_tab));
    }

    pub fn populate_macros(&self, macros: &[crate::engine::macro_table::MacroEntry]) {
        self.tab_macro.populate_macros(macros);
    }

    pub fn populate_macro_items(&self, items: &[crate::engine::profile::ResolvedMacroItem]) {
        self.tab_macro.populate_macro_items(items);
    }

    pub fn update_sound_controls_state(&self) {
        self.tab_sound.update_sound_controls_state();
    }

    pub fn update_macro_checkboxes_state(&self) {
        self.tab_macro.update_macro_checkboxes_state();
    }


    /// Populates current selected profile state into the UI
    pub fn load_profile_data(
        &self,
        profile: &AppProfile,
        is_active: bool,
        global_config: &EngineConfig,
        all_profiles: &[&AppProfile],
        active_id: &str,
    ) {
        self.sidebar
            .populate_profiles(all_profiles, &profile.id, active_id);
        self.banner.update_profile(profile, is_active);
        self.tab_typing.load_profile(profile, global_config);
        self.tab_remap.load_profile(profile, global_config);
        self.tab_sound.load_profile(profile, global_config);
        self.tab_macro.load_profile(profile, global_config);
        self.tab_system.load_config(global_config);
    }

    pub fn load_config(&self, config: &EngineConfig) {
        self.tab_typing.load_config(config);
        self.tab_remap.load_config(config);
        self.tab_sound.load_config(config);
        self.tab_macro.load_config(config);
        self.tab_system.load_config(config);
    }

    pub fn read_config(&self, config: &mut EngineConfig) {
        self.tab_typing.read_config(config);
        self.tab_remap.read_config(config);
        self.tab_sound.read_config(config);
        self.tab_macro.read_config(config);
        self.tab_system.read_config(config);
    }

    pub fn read_profile(&self, profile: &mut AppProfile) {
        self.tab_typing.read_profile(profile);
        self.tab_remap.read_profile(profile);
        self.tab_sound.read_profile(profile, false);
    }

    pub fn update_language(&self, lang: Language) {
        crate::language::set_current_language(lang);
        let strings = crate::language::get_strings(lang);
        self.sidebar.update_language(strings);
        self.banner.update_language(strings);
        self.tab_typing.update_language(strings);
        self.tab_remap.update_language(strings);
        self.tab_sound.update_language(strings);
        self.tab_macro.update_language(strings);
        self.tab_system.update_language(lang, strings);
        self.tab_about.update_language(strings);
        self.footer.update_language(strings);
        self.tab_bar.set_tabs(&strings.tab_titles());

        unsafe {
            InvalidateRect(self.tab_bar.hwnd(), null_mut(), 0);
            InvalidateRect(self.sidebar.list_profiles.hwnd(), null_mut(), 0);
            InvalidateRect(self.sidebar.btn_system_settings.hwnd(), null_mut(), 0);
            InvalidateRect(self.sidebar.btn_about.hwnd(), null_mut(), 0);
            InvalidateRect(self.banner.btn_delete.hwnd(), null_mut(), 0);
        }
    }

    /// Master dispatcher for WM_CTLCOLORSTATIC across all components
    pub fn on_ctlcolor_static(&self, hdc: isize, ctrl_id: u32, is_dark: bool) -> Option<isize> {
        if let Some(br) = self.sidebar.on_ctlcolor_static(hdc, ctrl_id, is_dark) {
            return Some(br);
        }
        if let Some(br) = self.banner.on_ctlcolor_static(hdc, ctrl_id, is_dark) {
            return Some(br);
        }
        if let Some(br) = self.tab_typing.on_ctlcolor_static(hdc, ctrl_id, is_dark) {
            return Some(br);
        }
        if let Some(br) = self.tab_system.on_ctlcolor_static(hdc, ctrl_id, is_dark) {
            return Some(br);
        }
        if let Some(br) = self.tab_about.on_ctlcolor_static(hdc, ctrl_id, is_dark) {
            return Some(br);
        }
        None
    }

    /// Master dispatcher for WM_COMMAND across all components
    pub fn on_command(&self, hwnd: isize, ctrl_id: u32, notify_code: u32) -> bool {
        if ctrl_id == IDC_TAB_MAIN {
            let cur_tab = self.tab_bar.get_cur_sel();
            self.update_view_mode(ViewMode::Profile(cur_tab));
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
            return true;
        }
        if ctrl_id == IDC_BTN_SIDEBAR_SETTINGS {
            self.sidebar.list_profiles.set_selected_index(-1);
            self.update_view_mode(ViewMode::SystemSettings);
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
                InvalidateRect(self.sidebar.list_profiles.hwnd(), null_mut(), 0);
            }
            return true;
        }
        if ctrl_id == IDC_BTN_SIDEBAR_ABOUT {
            self.sidebar.list_profiles.set_selected_index(-1);
            self.update_view_mode(ViewMode::About);
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
                InvalidateRect(self.sidebar.list_profiles.hwnd(), null_mut(), 0);
            }
            return true;
        }
        if self.footer.on_command(hwnd, ctrl_id, self) {
            return true;
        }
        if self.banner.on_command(hwnd, ctrl_id, self) {
            return true;
        }
        if self.sidebar.on_command(hwnd, ctrl_id, notify_code, self) {
            return true;
        }
        if self.tab_typing.on_command(hwnd, ctrl_id, notify_code) {
            return true;
        }
        if self.tab_remap.on_command(hwnd, ctrl_id, notify_code) {
            return true;
        }
        if self.tab_sound.on_command(hwnd, ctrl_id, notify_code) {
            return true;
        }
        if self.tab_macro.on_command(self, ctrl_id, notify_code) {
            return true;
        }
        if self.tab_system.on_command(hwnd, ctrl_id, notify_code, self) {
            return true;
        }
        if self.tab_about.on_command(hwnd, ctrl_id) {
            return true;
        }
        false
    }

    /// Master dispatcher for WM_HSCROLL across all components
    pub fn on_hscroll(&self, slider_hwnd: isize) -> bool {
        self.tab_sound.on_hscroll(slider_hwnd)
    }

    /// Master dispatcher for WM_MEASUREITEM across all components
    pub fn on_measure_item(&self, mis: &mut MEASUREITEMSTRUCT) -> bool {
        self.sidebar.on_measure_item(mis)
    }
}
