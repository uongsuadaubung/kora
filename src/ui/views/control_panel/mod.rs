//! Main Control Panel View for Kora Multi-Profile Workspace Hub
//! 2-Column Layout (Sidebar 230px + Right Workspace Panel 590px)

pub mod banner;
pub mod footer;
pub mod paint;
pub mod sidebar;
pub mod tab_about;
pub mod tab_keys;
pub mod tab_macro;
pub mod tab_sound;
pub mod tab_system;
pub mod tab_typing;

pub use banner::*;
pub use footer::*;
pub use paint::*;
pub use sidebar::*;
pub use tab_about::*;
pub use tab_keys::*;
pub use tab_macro::*;
pub use tab_sound::*;
pub use tab_system::*;
pub use tab_typing::*;

use crate::engine::EngineConfig;
use crate::engine::profile::{AppProfile, Child, ProfileRef, Root};
use crate::language::Language;
use crate::ui::components::*;
pub use crate::ui::tokens::layout::*;
use crate::ui::win32::{InvalidateRect, MEASUREITEMSTRUCT};
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
    pub tab_keys: TabKeys,
    pub tab_sound: TabSound,
    pub tab_macro: TabMacro,
    pub tab_system: TabSystem,
    pub tab_about: TabAbout,
    pub footer: FooterControls,
}

/// Engine data needed to populate the panel, copied out under one short lock.
struct EngineSnapshot {
    config: EngineConfig,
    global_profile: AppProfile<Root>,
    selected_child: Option<AppProfile<Child>>,
    children: Vec<AppProfile<Child>>,
    selected_id: String,
    remaps: Vec<crate::engine::profile::ResolvedRemapItem>,
    disabled: Vec<crate::engine::profile::ResolvedDisableItem>,
    key_macros: Vec<crate::engine::profile::ResolvedKeyMacroItem>,
    macros: Vec<crate::engine::profile::ResolvedMacroItem>,
}

impl ControlPanelControls {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let sidebar = SidebarControls::create(parent, hfont_normal, hfont_bold)?;
        let banner = ProfileBanner::create(parent, hfont_normal)?;

        let strings = crate::language::current();
        let tab_bar = TabBar::create(
            parent,
            0,
            TAB_BAR_X,
            TAB_BAR_Y,
            TAB_BAR_WIDTH,
            TAB_BAR_HEIGHT,
            hfont_normal,
            hfont_bold,
            &strings.tab_titles(),
        )?;

        let tab_typing = TabTyping::create(parent, hfont_normal, hfont_bold)?;
        let tab_keys = TabKeys::create(parent, hfont_normal, hfont_bold)?;
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
            tab_keys,
            tab_sound,
            tab_macro,
            tab_system,
            tab_about,
            footer,
        })
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.sidebar.get_view_mode()
    }

    /// Dynamically switches visibility between Profile Subtabs, System Settings, and About
    pub fn update_view_mode(&self, mode: ViewMode) {
        self.sidebar.set_view_mode(mode);
        let subtab = match mode {
            ViewMode::Profile(t) => Some(t),
            _ => None,
        };
        let is_profile = subtab.is_some();
        self.banner.set_visible(is_profile);
        self.tab_bar.set_visible(is_profile);
        self.tab_typing.update_visibility(subtab == Some(0));
        self.tab_keys.update_visibility(subtab == Some(1));
        self.tab_sound.update_visibility(subtab == Some(2));
        self.tab_macro.update_visibility(subtab == Some(3));
        self.tab_system
            .update_visibility(mode == ViewMode::SystemSettings);
        self.tab_about.update_visibility(mode == ViewMode::About);
        unsafe {
            InvalidateRect(self.sidebar.btn_system_settings.hwnd(), null_mut(), 0);
            InvalidateRect(self.sidebar.btn_about.hwnd(), null_mut(), 0);
        }
    }

    pub fn populate_macro_items(&self, items: &[crate::engine::profile::ResolvedMacroItem]) {
        self.tab_macro.populate_macro_items(items);
    }

    /// Snapshots the engine state under a short lock, then loads it into every view
    /// (sidebar, banner, all tabs, lists). Does not change the view mode.
    /// Must be called without holding `ENGINE_INSTANCE`. Returns the global config.
    pub fn refresh_from_engine(&self) -> Option<EngineConfig> {
        let snap = {
            let guard = crate::platform::win32::ENGINE_INSTANCE.lock().ok()?;
            let engine = guard.as_ref()?;
            let pm = &engine.profile_manager;
            let sel_id = pm.selected_id.as_str();
            EngineSnapshot {
                config: engine.config().clone(),
                global_profile: pm.global_profile.clone(),
                selected_child: pm.get_profile(sel_id).cloned(),
                children: pm.profiles.clone(),
                selected_id: pm.selected_id.clone(),
                remaps: pm.resolve_remap_items_for_ui(sel_id),
                disabled: pm.resolve_disable_items_for_ui(sel_id),
                key_macros: pm.resolve_key_macro_items_for_ui(sel_id),
                macros: pm
                    .resolve_macro_items_for_ui(sel_id, &engine.macro_table.get_sorted_entries()),
            }
        };

        let mut all_refs = Vec::with_capacity(1 + snap.children.len());
        all_refs.push(ProfileRef::Root(&snap.global_profile));
        all_refs.extend(snap.children.iter().map(ProfileRef::Child));

        let (child, cfg) = (snap.selected_child.as_ref(), &snap.config);
        self.sidebar.populate_profiles(&all_refs, &snap.selected_id);
        self.banner.update_profile(child);
        self.tab_typing.load_profile(child, cfg);
        self.tab_keys.load_profile(child, cfg);
        self.tab_sound.load_profile(child, cfg);
        self.tab_macro.load_profile(child, cfg);
        self.tab_system.load_config(cfg);
        self.tab_keys.populate_remaps(&snap.remaps);
        self.tab_keys.populate_disabled(&snap.disabled);
        self.tab_keys.populate_key_macros(&snap.key_macros);
        self.tab_keys
            .refresh_overview(&snap.remaps, &snap.disabled, &snap.key_macros);
        self.populate_macro_items(&snap.macros);
        Some(snap.config)
    }

    pub fn update_language(&self, lang: Language) {
        crate::language::set_current_language(lang);
        let strings = crate::language::get_strings(lang);
        self.sidebar.update_language(strings);
        self.banner.update_language(strings);
        self.tab_typing.update_language(strings);
        self.tab_keys.update_language(strings);
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

    /// Delegates custom card and decoration painting to the currently active view/tab
    pub fn paint_active_view(
        &self,
        hwnd: isize,
        hdc: isize,
        right_bound: i32,
        palette: &crate::ui::colors::ThemePalette,
    ) {
        match self.get_view_mode() {
            ViewMode::Profile(0) => self.tab_typing.paint(hwnd, hdc, right_bound, palette),
            ViewMode::Profile(1) => self.tab_keys.paint(hdc, right_bound),
            ViewMode::Profile(2) => self.tab_sound.paint(hdc, right_bound),
            ViewMode::Profile(3) => self.tab_macro.paint(hdc, right_bound),
            ViewMode::SystemSettings => self.tab_system.paint(hdc, right_bound),
            ViewMode::About => self.tab_about.paint(hdc, right_bound, palette),
            _ => {}
        }
    }

    fn switch_global_view(&self, hwnd: isize, mode: ViewMode) -> bool {
        self.sidebar.list_profiles.set_selected_index(-1);
        self.update_view_mode(mode);
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
            InvalidateRect(self.sidebar.list_profiles.hwnd(), null_mut(), 0);
        }
        true
    }

    /// Master dispatcher for WM_COMMAND across all components
    pub fn on_command(&self, hwnd: isize, ctrl_id: u32, notify_code: u32) -> bool {
        if self.tab_bar.is_changed(ctrl_id, notify_code) {
            self.update_view_mode(ViewMode::Profile(self.tab_bar.get_cur_sel()));
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
            return true;
        }
        if self
            .sidebar
            .btn_system_settings
            .is_clicked(ctrl_id, notify_code)
        {
            return self.switch_global_view(hwnd, ViewMode::SystemSettings);
        }
        if self.sidebar.btn_about.is_clicked(ctrl_id, notify_code) {
            return self.switch_global_view(hwnd, ViewMode::About);
        }
        self.footer.on_command(hwnd, ctrl_id, notify_code)
            || self.banner.on_command(hwnd, ctrl_id, notify_code)
            || self.sidebar.on_command(hwnd, ctrl_id, notify_code, self)
            || self.tab_typing.on_command(hwnd, ctrl_id, notify_code)
            || self.tab_keys.on_command(hwnd, ctrl_id, notify_code)
            || self.tab_sound.on_command(hwnd, ctrl_id, notify_code)
            || self.tab_macro.on_command(self, ctrl_id, notify_code)
            || self.tab_system.on_command(hwnd, ctrl_id, notify_code, self)
            || self.tab_about.on_command(hwnd, ctrl_id, notify_code)
    }

    /// Master dispatcher for WM_HSCROLL across all components
    pub fn on_hscroll(&self, wparam: usize, slider_hwnd: isize) -> bool {
        self.tab_sound.on_hscroll(wparam, slider_hwnd)
    }

    /// Master dispatcher for WM_MEASUREITEM across all components
    pub fn on_measure_item(&self, mis: &mut MEASUREITEMSTRUCT) -> bool {
        self.sidebar.on_measure_item(mis)
    }
}
