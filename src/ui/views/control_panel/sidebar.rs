//! Sidebar Component for Profile-First Workspace Hub
//! Contains Profile list with add button and System Settings / About quick links.

use super::ViewMode;
use crate::engine::profile::ProfileRef;
use crate::platform::win32::ENGINE_INSTANCE;
use crate::ui::colors::ThemePalette;
use crate::ui::components::listbox::ListBox;
use crate::ui::components::{ButtonVariant, Label, PushButton, SurfaceRole};
use crate::ui::tokens::layout::*;
use crate::ui::tokens::radii::*;
use crate::ui::tokens::typography::*;
use crate::ui_framework::layout::{CrossAlign, HStack};
use crate::ui::win32::{
    DRAWITEMSTRUCT, DT_CENTER, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DrawTextW, FW_BOLD,
    FW_NORMAL, InvalidateRect, MEASUREITEMSTRUCT, RECT, SelectObject, SetBkMode, SetTextColor,
    TRANSPARENT, fill_rect_color, round_rect, to_wide,
};
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

const DT_END_ELLIPSIS: u32 = 0x00004000;

#[derive(Clone, Debug)]
pub struct SidebarProfileItem {
    pub id: String,
    pub icon: String,
    pub name: String,
    pub desc: String,
}

pub struct SidebarControls {
    pub lbl_title: Label,
    pub btn_add_profile: PushButton,
    pub list_profiles: ListBox,
    pub btn_system_settings: PushButton,
    pub btn_about: PushButton,
    pub items: Arc<RwLock<Vec<SidebarProfileItem>>>,
    pub current_view_mode: Arc<RwLock<ViewMode>>,
}

impl SidebarControls {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let hfont_title = crate::ui::theme::cached_font(
            crate::ui::tokens::typography::FONT_SIZE_SIDEBAR_TITLE,
            FW_BOLD,
        );
        let lbl_title = Label::new(parent, strings.sidebar_profiles_title, hfont_title)?
            .on_surface(SurfaceRole::Sidebar);

        let btn_add_profile = PushButton::new(parent, "+", hfont_bold, ButtonVariant::Primary)?;
        btn_add_profile.set_corner_radius(4);

        let mut header = HStack::at(12, 14, 204, 24);
        header.align(CrossAlign::Center);
        header.add_stretch(1, Some(lbl_title.hwnd()));
        header.add_fixed(22, Some(btn_add_profile.hwnd()));
        header.apply();

        let list_profiles = ListBox::create_owner_draw(
            parent,
            0,
            SIDEBAR_X,
            SIDEBAR_LIST_Y,
            SIDEBAR_WIDTH,
            SIDEBAR_LIST_HEIGHT,
            hfont_normal,
        )?;
        list_profiles.set_item_height(SIDEBAR_ITEM_HEIGHT);

        let btn_system_settings = PushButton::create_owner_draw(
            parent,
            0,
            strings.sidebar_btn_settings,
            SIDEBAR_X,
            SIDEBAR_BTN_SETTINGS_Y,
            SIDEBAR_WIDTH,
            SIDEBAR_BTN_HEIGHT,
            hfont_normal,
        )?;

        let btn_about = PushButton::create_owner_draw(
            parent,
            0,
            strings.sidebar_btn_about,
            SIDEBAR_X,
            SIDEBAR_BTN_ABOUT_Y,
            SIDEBAR_WIDTH,
            SIDEBAR_BTN_HEIGHT,
            hfont_normal,
        )?;

        let hf_sub = crate::ui::theme::cached_font(FONT_SIZE_SUBTITLE, FW_NORMAL);

        let items = Arc::new(RwLock::new(Vec::new()));
        let current_view_mode = Arc::new(RwLock::new(ViewMode::Profile(0)));

        // Setup message reflection drawers
        let items_clone = items.clone();
        let hf_bold = hfont_bold;
        list_profiles.set_item_drawer(move |dis, is_dark| {
            if dis.item_id != u32::MAX {
                let is_selected = (dis.item_state & 0x0001/* ODS_SELECTED */) != 0;
                Self::draw_profile_item_ext(
                    &items_clone,
                    hf_bold,
                    hf_sub,
                    dis.hdc,
                    &dis.rc_item,
                    dis.item_id,
                    is_selected,
                    is_dark,
                );
            }
            true
        });

        let mode_settings = current_view_mode.clone();
        btn_system_settings.set_custom_drawer(move |dis, is_dark| {
            let is_active = if let Ok(m) = mode_settings.read() {
                matches!(*m, ViewMode::SystemSettings)
            } else {
                false
            };
            Self::draw_footer_button(dis, is_dark, is_active, true);
            true
        });

        let mode_about = current_view_mode.clone();
        btn_about.set_custom_drawer(move |dis, is_dark| {
            let is_active = if let Ok(m) = mode_about.read() {
                matches!(*m, ViewMode::About)
            } else {
                false
            };
            Self::draw_footer_button(dis, is_dark, is_active, false);
            true
        });

        Some(Self {
            lbl_title,
            btn_add_profile,
            list_profiles,
            btn_system_settings,
            btn_about,
            items,
            current_view_mode,
        })
    }

    /// Populates profile items into the sidebar ListBox
    pub fn populate_profiles(&self, profiles: &[ProfileRef<'_>], selected_id: &str) {
        let strings = crate::language::current();
        let mut items = Vec::new();
        let mut sel_idx = 0;
        for (i, &p) in profiles.iter().enumerate() {
            if p.id() == selected_id {
                sel_idx = i as i32;
            }
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

        let needs_rebuild = if let Ok(lock) = self.items.read() {
            if lock.len() != items.len() {
                true
            } else {
                lock.iter()
                    .zip(items.iter())
                    .any(|(old, new)| old.id != new.id || old.name != new.name)
            }
        } else {
            true
        };

        if needs_rebuild {
            self.list_profiles.reset_content();
            for item in &items {
                self.list_profiles.add_item(&item.name);
            }
        }
        if !items.is_empty() {
            let in_profile_mode = if let Ok(m) = self.current_view_mode.read() {
                matches!(*m, ViewMode::Profile(_))
            } else {
                true
            };
            if in_profile_mode {
                let cur = self.list_profiles.get_selected_index().unwrap_or(-1);
                if cur != sel_idx {
                    self.list_profiles.set_selected_index(sel_idx);
                }
            } else {
                self.list_profiles.set_selected_index(-1);
            }
        }

        if let Ok(mut lock) = self.items.write() {
            *lock = items;
        }

        unsafe {
            InvalidateRect(self.list_profiles.hwnd(), null_mut(), 0);
        }
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.lbl_title.set_text(strings.sidebar_profiles_title);
        self.btn_system_settings
            .set_text(strings.sidebar_btn_settings);
        self.btn_about.set_text(strings.sidebar_btn_about);
        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let profiles = engine.profile_manager.all_profiles();
            let sel_id = engine.profile_manager.selected_id.clone();
            self.populate_profiles(&profiles, &sel_id);
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
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_profile_item_ext(
        items: &RwLock<Vec<SidebarProfileItem>>,
        hfont_bold: isize,
        hfont_sub: isize,
        hdc: isize,
        rc_item: &RECT,
        item_id: u32,
        is_selected: bool,
        is_dark: bool,
    ) {
        unsafe {
            let palette = ThemePalette::get(is_dark);

            // 1. Fill entire item rectangle with sidebar background
            fill_rect_color(hdc, rc_item, palette.bg_sidebar);

            let item = {
                if let Ok(guard) = items.read() {
                    if (item_id as usize) < guard.len() {
                        guard[item_id as usize].clone()
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            let card_rc = RECT {
                left: rc_item.left + 4,
                top: rc_item.top + 2,
                right: rc_item.right - 4,
                bottom: rc_item.bottom - 2,
            };

            // 2. Draw card background (rounded rect if selected)
            if is_selected {
                round_rect(
                    hdc,
                    &card_rc,
                    RADIUS_MD,
                    palette.tab_active_bg,
                    Some(palette.border),
                );
            }

            SetBkMode(hdc, TRANSPARENT);

            // 3. Draw Icon (16px equivalent centered horizontally in 24px box)
            let mut icon_rc = RECT {
                left: card_rc.left + 6,
                top: card_rc.top + 6,
                right: card_rc.left + 30,
                bottom: card_rc.top + 34,
            };
            let name_color = if is_selected {
                palette.accent
            } else {
                palette.text_primary
            };
            SetTextColor(hdc, name_color);
            SelectObject(hdc, hfont_bold);
            DrawTextW(
                hdc,
                to_wide(&item.icon).as_ptr(),
                -1,
                &mut icon_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );

            // 4. Draw Profile Name
            let mut name_rc = RECT {
                left: card_rc.left + 32,
                top: card_rc.top + 6,
                right: card_rc.right - 8,
                bottom: card_rc.top + 24,
            };
            SelectObject(hdc, hfont_bold);
            DrawTextW(
                hdc,
                to_wide(&item.name).as_ptr(),
                -1,
                &mut name_rc,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            );

            // 5. Draw Subtitle / Process
            let mut desc_rc = RECT {
                left: card_rc.left + 32,
                top: card_rc.top + 25,
                right: card_rc.right - 8,
                bottom: card_rc.top + 41,
            };
            SetTextColor(hdc, palette.text_secondary);
            SelectObject(hdc, hfont_sub);
            DrawTextW(
                hdc,
                to_wide(&item.desc).as_ptr(),
                -1,
                &mut desc_rc,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
            );
        }
    }

    pub fn draw_footer_button(
        dis: &DRAWITEMSTRUCT,
        is_dark: bool,
        is_active: bool,
        is_settings: bool,
    ) {
        unsafe {
            let palette = ThemePalette::get(is_dark);
            let strings = crate::language::current();
            let label = if is_settings {
                strings.sidebar_btn_settings
            } else {
                strings.sidebar_btn_about
            };

            // 1. Completely erase full item bounds with sidebar background
            fill_rect_color(dis.hdc, &dis.rc_item, palette.bg_sidebar);

            let card_rc = RECT {
                left: dis.rc_item.left + 2,
                top: dis.rc_item.top + 1,
                right: dis.rc_item.right - 2,
                bottom: dis.rc_item.bottom - 1,
            };

            let is_pressed = (dis.item_state & 0x0001/* ODS_SELECTED */) != 0;
            let is_hover = PushButton::is_mouse_hover(dis.hwnd_item);

            // 2. Draw card background (rounded rect if active, pressed, or hovered)
            if is_active || is_pressed {
                round_rect(
                    dis.hdc,
                    &card_rc,
                    RADIUS_MD,
                    palette.tab_active_bg,
                    Some(palette.border),
                );
            } else if is_hover {
                round_rect(
                    dis.hdc,
                    &card_rc,
                    RADIUS_MD,
                    palette.tab_hover_bg,
                    Some(palette.border),
                );
            }

            // 3. Draw text and icon
            SetBkMode(dis.hdc, TRANSPARENT);
            SetTextColor(
                dis.hdc,
                if is_active || is_hover || is_pressed {
                    palette.text_primary
                } else {
                    palette.text_secondary
                },
            );

            let mut text_rc = RECT {
                left: card_rc.left + 10,
                top: card_rc.top,
                right: card_rc.right - 6,
                bottom: card_rc.bottom,
            };
            DrawTextW(
                dis.hdc,
                to_wide(label).as_ptr(),
                -1,
                &mut text_rc,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
            );
        }
    }

    pub fn on_measure_item(&self, mis: &mut MEASUREITEMSTRUCT) -> bool {
        if mis.ctl_id == self.list_profiles.id() {
            mis.item_height = 48;
            return true;
        }
        false
    }

    pub fn handle_profile_selection(&self, controls: &super::ControlPanelControls, hwnd: isize) {
        let sel_idx = self.list_profiles.get_selected_index();
        let current_view_mode = controls.get_view_mode();

        let target_idx = match sel_idx {
            Some(idx) if idx >= 0 => idx as usize,
            _ => return,
        };

        let (new_id, is_already_selected) = {
            if let Ok(guard) = ENGINE_INSTANCE.lock()
                && let Some(ref engine) = *guard
            {
                let profiles = engine.profile_manager.all_profiles();
                if let Some(p) = profiles.get(target_idx) {
                    let is_already = p.id() == engine.profile_manager.selected_id;
                    (Some(p.id().to_string()), is_already)
                } else {
                    (None, false)
                }
            } else {
                (None, false)
            }
        };

        // If already on this profile AND already in Profile view mode, skip to avoid flicker
        if is_already_selected && matches!(current_view_mode, ViewMode::Profile(_)) {
            return;
        }

        if let Some(id) = new_id {
            Self::switch_to_profile(controls, hwnd, &id);
        }
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
            InvalidateRect(hwnd, null_mut(), 0);
            InvalidateRect(controls.sidebar.list_profiles.hwnd(), null_mut(), 0);
        }
    }

    pub fn on_command(
        &self,
        hwnd: isize,
        ctrl_id: u32,
        notify_code: u32,
        controls: &super::ControlPanelControls,
    ) -> bool {
        if self.list_profiles.is_sel_changed(ctrl_id, notify_code) {
            self.handle_profile_selection(controls, hwnd);
            return true;
        }
        if self.btn_add_profile.is_clicked(ctrl_id, notify_code) {
            // Modeless dialog: it refreshes the panel itself after creating the profile
            crate::ui::views::dialogs::show_add_profile_dialog(hwnd);
            return true;
        }
        false
    }
}
