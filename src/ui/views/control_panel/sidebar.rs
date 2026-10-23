//! Sidebar Component for Profile-First Workspace Hub
//! Contains Profile list with add button and System Settings / About quick links.
//! Migrated to Declarative UI Surface (`Surface<SidebarMsg>`).

use super::ViewMode;
use crate::engine::profile::ProfileRef;
use crate::language::LanguageStrings;
use crate::ui::tokens::layout::*;
use crate::ui::tokens::radii::*;
use crate::ui::win32::MEASUREITEMSTRUCT;
use crate::ui_framework::core::*;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct SidebarProfileItem {
    pub id: String,
    pub icon: String,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug)]
pub enum SidebarMsg {
    AddProfile,
    SelectProfile(String),
    SystemSettings,
    About,
}

pub struct SidebarControls {
    pub surface: Surface<SidebarMsg>,
    pub items: Arc<RwLock<Vec<SidebarProfileItem>>>,
    pub current_view_mode: Arc<RwLock<ViewMode>>,
    pub selected_id: Arc<RwLock<String>>,
}

impl SidebarControls {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let surface = Surface::create(parent, SIDEBAR_X, 12, SIDEBAR_WIDTH, 566)?;
        surface.set_bg(BgStyle::Sidebar);
        surface.set_content_width(SIDEBAR_WIDTH);

        let items = Arc::new(RwLock::new(Vec::new()));
        let current_view_mode = Arc::new(RwLock::new(ViewMode::Profile(0)));
        let selected_id = Arc::new(RwLock::new(crate::ui::GLOBAL_PROFILE_ID.to_string()));

        surface.set_on_msg(move |msg| match msg {
            SidebarMsg::AddProfile => {
                crate::ui::views::dialogs::show_add_profile_dialog(parent);
            }
            SidebarMsg::SelectProfile(id) => {
                if let Some(c) = crate::ui::controls() {
                    Self::switch_to_profile(c, parent, &id);
                }
            }
            SidebarMsg::SystemSettings => {
                if let Some(c) = crate::ui::controls() {
                    c.switch_global_view(parent, ViewMode::SystemSettings);
                }
            }
            SidebarMsg::About => {
                if let Some(c) = crate::ui::controls() {
                    c.switch_global_view(parent, ViewMode::About);
                }
            }
        });

        let sidebar = Self {
            surface,
            items,
            current_view_mode,
            selected_id,
        };

        sidebar.render();
        Some(sidebar)
    }

    pub fn render(&self) {
        let items = self.items.read().map(|l| l.clone()).unwrap_or_default();
        let sel_id = self.selected_id.read().map(|l| l.clone()).unwrap_or_default();
        let mode = self.current_view_mode.read().map(|l| *l).unwrap_or(ViewMode::Profile(0));
        let strings = crate::language::current();
        self.surface.render(Self::view(&items, &sel_id, mode, strings));
    }

    pub fn view(
        items: &[SidebarProfileItem],
        selected_id: &str,
        view_mode: ViewMode,
        strings: &LanguageStrings,
    ) -> Element<SidebarMsg> {
        let is_in_profile_mode = matches!(view_mode, ViewMode::Profile(_));
        let is_settings = matches!(view_mode, ViewMode::SystemSettings);
        let is_about = matches!(view_mode, ViewMode::About);

        // Header
        let header = row(vec![
            text(strings.sidebar_profiles_title)
                .bold()
                .size(Size::Sm)
                .flex_1(),
            button("+", Variant::Primary, SidebarMsg::AddProfile)
                .size(Size::Sm)
                .w(24)
                .h(24),
        ])
        .items_center()
        .w_full()
        .h(28);

        // Profile Items
        let mut profile_elements = Vec::with_capacity(items.len());
        for item in items {
            let is_selected = is_in_profile_mode && item.id == selected_id;
            let card = row(vec![
                text(&item.icon).bold().w(20).text_center(),
                col(vec![
                    text(&item.name)
                        .bold()
                        .tone(if is_selected { TextTone::Accent } else { TextTone::Primary }),
                    text(&item.desc)
                        .muted()
                        .size(Size::Sm),
                ])
                .gap(1)
                .flex_1(),
            ])
            .gap(2)
            .padding(Edges::xy(8, 6))
            .rounded(RADIUS_MD)
            .w_full()
            .items_center()
            .bg(if is_selected { BgStyle::TabActive } else { BgStyle::None })
            .border_style(if is_selected { BorderStyle::Normal } else { BorderStyle::None })
            .on_click(SidebarMsg::SelectProfile(item.id.clone()));

            profile_elements.push(card);
        }

        // Footer buttons
        let btn_settings = row(vec![
            text(strings.sidebar_btn_settings)
                .tone(if is_settings { TextTone::Primary } else { TextTone::Secondary }),
        ])
        .items_center()
        .padding(Edges::xy(8, 4))
        .rounded(RADIUS_MD)
        .w_full()
        .h(28)
        .bg(if is_settings { BgStyle::TabActive } else { BgStyle::None })
        .border_style(if is_settings { BorderStyle::Normal } else { BorderStyle::None })
        .on_click(SidebarMsg::SystemSettings);

        let btn_about = row(vec![
            text(strings.sidebar_btn_about)
                .tone(if is_about { TextTone::Primary } else { TextTone::Secondary }),
        ])
        .items_center()
        .padding(Edges::xy(8, 4))
        .rounded(RADIUS_MD)
        .w_full()
        .h(28)
        .bg(if is_about { BgStyle::TabActive } else { BgStyle::None })
        .border_style(if is_about { BorderStyle::Normal } else { BorderStyle::None })
        .on_click(SidebarMsg::About);

        col(vec![
            header,
            col(profile_elements).gap(2).w_full(),
            spacer().flex_1(),
            divider(),
            col(vec![btn_settings, btn_about]).gap(1).w_full(),
        ])
        .gap(2)
        .w_full()
        .h(566)
    }

    /// Populates profile items into the sidebar
    pub fn populate_profiles(&self, profiles: &[ProfileRef<'_>], selected_id: &str) {
        let strings = crate::language::current();
        let mut items = Vec::new();
        for &p in profiles {
            let (icon, name, desc) = match p {
                ProfileRef::Root(root) => (
                    &root.icon,
                    strings.banner_root_title.to_string(),
                    strings.sidebar_root_desc.to_string(),
                ),
                ProfileRef::Child(child) => (
                    &child.icon,
                    child.name.clone(),
                    if child.process_names.is_empty() {
                        strings.sidebar_unassigned.to_string()
                    } else {
                        child.process_names.join(", ")
                    },
                ),
            };
            items.push(SidebarProfileItem {
                id: p.id().to_string(),
                icon: if icon.is_empty() {
                    "⌨️".to_string()
                } else {
                    icon.clone()
                },
                name,
                desc,
            });
        }

        if let Ok(mut lock) = self.selected_id.write() {
            *lock = selected_id.to_string();
        }
        if let Ok(mut lock) = self.items.write() {
            *lock = items;
        }

        self.render();
    }

    pub fn update_language(&self, _strings: &crate::language::LanguageStrings) {
        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let profiles = engine.profile_manager.all_profiles();
            let sel_id = engine.profile_manager.selected_id.clone();
            self.populate_profiles(&profiles, &sel_id);
        } else {
            self.render();
        }
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.current_view_mode
            .read()
            .map(|m| *m)
            .unwrap_or(ViewMode::Profile(0))
    }

    pub fn set_view_mode(&self, mode: ViewMode) {
        if let Ok(mut lock) = self.current_view_mode.write() {
            *lock = mode;
        }
        self.render();
    }

    pub fn switch_to_profile(controls: &super::ControlPanelControls, hwnd: isize, id: &str) {
        if crate::ui::controller::with_engine(|e| e.profile_manager.selected_id = id.to_string())
            .is_none()
        {
            return;
        }
        let cur_tab = controls.tab_bar.get_cur_sel();
        controls.update_view_mode(ViewMode::Profile(cur_tab));
        controls.refresh_from_engine();

        unsafe {
            crate::ui::win32::InvalidateRect(hwnd, null_mut(), 0);
        }
    }

    #[inline]
    pub fn on_command(
        &self,
        _hwnd: isize,
        _ctrl_id: u32,
        _notify_code: u32,
        _controls: &super::ControlPanelControls,
    ) -> bool {
        false
    }

    #[inline]
    pub fn on_measure_item(&self, _mis: &mut MEASUREITEMSTRUCT) -> bool {
        false
    }
}
