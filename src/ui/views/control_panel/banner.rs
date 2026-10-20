//! Profile Header Banner Component
//! Shows current profile icon, name, active status, and delete action.

use crate::engine::profile::AppProfile;
use crate::ui::components::{ButtonVariant, Label, LabelTone, PushButton, SurfaceRole};
use crate::ui::theme::cached_font;
use crate::ui::tokens::layout::*;
use crate::ui::tokens::typography::*;
use crate::ui::win32::{EnableWindow, FW_BOLD, SW_HIDE, SW_SHOW, ShowWindow, set_hwnds_visible};
use std::sync::atomic::{AtomicBool, Ordering};

const DELETE_BTN_X: i32 = 690;

pub struct ProfileBanner {
    pub lbl_name: Label,
    pub lbl_status: Label,
    pub btn_delete: PushButton,
    pub is_root: AtomicBool,
}

impl ProfileBanner {
    pub fn create(parent: isize, hfont_normal: isize) -> Option<Self> {
        let strings = crate::language::current();
        let hfont_title = cached_font(FONT_SIZE_TITLE, FW_BOLD);

        let initial_title = format!("🌐 {}", strings.banner_root_title);
        let lbl_name = Label::create(
            parent,
            &initial_title,
            HEADER_X,
            HEADER_TITLE_Y,
            DELETE_BTN_X - HEADER_X - 8, // end before the delete button so it doesn't cover it
            HEADER_TITLE_HEIGHT,
            hfont_title,
        )?
        .on_surface(SurfaceRole::Window);

        let lbl_status = Label::create(
            parent,
            strings.banner_root_desc,
            HEADER_X,
            HEADER_SUB_Y,
            HEADER_SUB_WIDTH,
            HEADER_SUB_HEIGHT,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary)
        .on_surface(SurfaceRole::Window);

        let btn_delete = PushButton::create_with_variant(
            parent,
            0,
            strings.banner_btn_delete,
            DELETE_BTN_X,
            HEADER_TITLE_Y,
            114,
            28,
            hfont_normal,
            ButtonVariant::Danger,
            None,
        )?;

        // Hide delete button initially because root profile is active on startup
        unsafe {
            ShowWindow(btn_delete.hwnd(), SW_HIDE);
        }

        Some(Self {
            lbl_name,
            lbl_status,
            btn_delete,
            is_root: AtomicBool::new(true),
        })
    }

    /// Updates banner display for the selected profile (`None` = Global Root, `Some(child)` = Child profile).
    pub fn update_profile(&self, child: Option<&AppProfile>) {
        let strings = crate::language::current();
        self.is_root.store(child.is_none(), Ordering::SeqCst);

        match child {
            None => {
                self.lbl_name
                    .set_text(&format!("🌐 {}", strings.banner_root_title));
                self.lbl_status.set_text(strings.banner_root_desc);
                unsafe {
                    ShowWindow(self.btn_delete.hwnd(), SW_HIDE);
                }
            }
            Some(profile) => {
                self.lbl_name
                    .set_text(&format!("{} {}", profile.icon, profile.name));
                let proc_str = if profile.process_names.is_empty() {
                    strings.sidebar_unassigned.to_string()
                } else {
                    profile.process_names.join(", ")
                };
                self.lbl_status.set_text(&format!(
                    "{}{}",
                    strings.banner_status_inactive_prefix, proc_str
                ));
                unsafe {
                    ShowWindow(self.btn_delete.hwnd(), SW_SHOW);
                    EnableWindow(self.btn_delete.hwnd(), 1);
                }
            }
        }
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        if let Some(child) = crate::ui::controller::with_engine(|engine| {
            let sel_id = &engine.profile_manager.selected_id;
            engine.profile_manager.get_profile(sel_id).cloned()
        }) {
            self.update_profile(child.as_ref());
        }
        self.btn_delete.set_text(strings.banner_btn_delete);
    }

    pub fn handle_delete_profile(&self, hwnd: isize) {
        let name = crate::ui::controller::with_engine(|engine| {
            let sel_id = &engine.profile_manager.selected_id;
            (sel_id != crate::ui::GLOBAL_PROFILE_ID)
                .then(|| {
                    engine
                        .profile_manager
                        .get_profile(sel_id)
                        .map(|p| p.name.clone())
                })
                .flatten()
        })
        .flatten();
        let Some(name) = name else { return };

        let strings = crate::language::current();
        let message = strings.confirm_delete_profile_msg.replace("{name}", &name);
        crate::ui::views::dialogs::show_delete_confirm(hwnd, message, move || {
            let deleted = crate::ui::controller::with_engine(|engine| {
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id != crate::ui::GLOBAL_PROFILE_ID {
                    engine.profile_manager.remove_profile(&sel_id);
                    engine.profile_manager.selected_id = crate::ui::GLOBAL_PROFILE_ID.to_string();
                    crate::ui::controller::save(engine);
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);
            if deleted && let Some(c) = crate::ui::controls() {
                super::SidebarControls::switch_to_profile(c, hwnd, crate::ui::GLOBAL_PROFILE_ID);
            }
        });
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        if self.btn_delete.is_clicked(ctl_id, notify_code) {
            self.handle_delete_profile(hwnd);
            true
        } else {
            false
        }
    }

    pub fn set_visible(&self, visible: bool) {
        let is_root = self.is_root.load(Ordering::SeqCst);
        set_hwnds_visible(&[self.lbl_name.hwnd(), self.lbl_status.hwnd()], visible);
        unsafe {
            if visible && !is_root {
                ShowWindow(self.btn_delete.hwnd(), SW_SHOW);
                EnableWindow(self.btn_delete.hwnd(), 1);
            } else {
                ShowWindow(self.btn_delete.hwnd(), SW_HIDE);
            }
        }
    }
}
