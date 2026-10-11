//! Profile Header Banner Component
//! Shows current profile icon, name, active status, and delete action.

use crate::engine::profile::AppProfile;
use crate::ui::colors::ThemePalette;
use crate::ui::win32::{
    create_app_font, DeleteObject, EnableWindow, SetBkColor, SetTextColor,
    ShowWindow, FW_BOLD, SW_HIDE, SW_SHOW,
};
use crate::ui::components::{ButtonVariant, Label, PushButton};
use crate::ui::theme::get_panel_bg_brush;
use crate::ui::tokens::id::*;
use crate::ui::tokens::layout::*;
use crate::ui::tokens::typography::*;
use std::sync::atomic::{AtomicBool, Ordering};

const DELETE_BTN_X: i32 = 690;

pub struct ProfileBanner {
    pub lbl_name: Label,
    pub lbl_status: Label,
    pub btn_delete: PushButton,
    pub is_root: AtomicBool,
    pub hfont_title: isize,
}

impl ProfileBanner {
    pub fn create(parent: isize, hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let hfont_title = create_app_font(FONT_SIZE_TITLE, FW_BOLD);

        let initial_title = format!("🌐 {}", strings.banner_root_title);
        let lbl_name = Label::create_with_id(
            parent,
            IDC_BANNER_TITLE,
            &initial_title,
            HEADER_X,
            HEADER_TITLE_Y,
            DELETE_BTN_X - HEADER_X - 8, // end before the delete button so it doesn't cover it
            HEADER_TITLE_HEIGHT,
            hfont_title,
        )?;

        let lbl_status = Label::create_with_id(
            parent,
            IDC_BANNER_STATUS,
            strings.banner_root_desc,
            HEADER_X,
            HEADER_SUB_Y,
            HEADER_SUB_WIDTH,
            HEADER_SUB_HEIGHT,
            hfont_normal,
        )?;

        let btn_delete = PushButton::create_with_variant(
            parent,
            IDC_BTN_DEL_PROFILE,
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
            hfont_title,
        })
    }

    /// Updates banner display for the selected profile
    pub fn update_profile(&self, profile: &AppProfile, is_active: bool) {
        let strings = crate::language::current();
        self.is_root.store(profile.is_root, Ordering::SeqCst);

        let title = if profile.is_root {
            format!("{} {}", profile.icon, strings.banner_root_title)
        } else {
            format!("{} {}", profile.icon, profile.name)
        };
        self.lbl_name.set_text(&title);

        let status = if profile.is_root {
            strings.banner_root_desc.to_string()
        } else {
            let proc_str = if profile.process_names.is_empty() {
                strings.sidebar_unassigned.to_string()
            } else {
                profile.process_names.join(", ")
            };
            if is_active {
                format!("{}{}", strings.banner_status_active_prefix, proc_str)
            } else {
                format!("{}{}", strings.banner_status_inactive_prefix, proc_str)
            }
        };
        self.lbl_status.set_text(&status);

        unsafe {
            if profile.is_root {
                ShowWindow(self.btn_delete.hwnd(), SW_HIDE);
            } else {
                ShowWindow(self.btn_delete.hwnd(), SW_SHOW);
                EnableWindow(self.btn_delete.hwnd(), 1);
            }
        }
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let sel_id = engine.profile_manager.selected_id.clone();
            let act_id = engine.profile_manager.active_id.clone();
            if let Some(prof) = engine.profile_manager.get_profile(&sel_id) {
                let is_active = sel_id == act_id;
                self.update_profile(prof, is_active);
            }
        }
        self.btn_delete.set_text(strings.banner_btn_delete);
    }


    pub fn on_ctlcolor_static(&self, hdc: isize, ctrl_id: u32, is_dark: bool) -> Option<isize> {
        if ctrl_id == IDC_BANNER_TITLE || ctrl_id == IDC_BANNER_STATUS {
            let palette = ThemePalette::get(is_dark);
            unsafe {
                SetBkColor(hdc, palette.bg_window);
                SetTextColor(
                    hdc,
                    if ctrl_id == IDC_BANNER_STATUS {
                        palette.text_secondary
                    } else {
                        palette.text_primary
                    },
                );
            }
            let p_br = get_panel_bg_brush();
            if p_br != 0 {
                return Some(p_br);
            }
        }
        None
    }

    pub fn handle_delete_profile(&self, _controls: &super::ControlPanelControls, hwnd: isize) {
        let name = match crate::platform::win32::ENGINE_INSTANCE.lock() {
            Ok(guard) => guard.as_ref().and_then(|engine| {
                let sel_id = &engine.profile_manager.selected_id;
                (sel_id != "global")
                    .then(|| engine.profile_manager.get_profile(sel_id).map(|p| p.name.clone()))
                    .flatten()
            }),
            Err(_) => None,
        };
        let Some(name) = name else { return };

        let strings = crate::language::current();
        let options = crate::ui::views::dialogs::ConfirmDialogOptions::new(
            strings.confirm_delete_title,
            strings.confirm_delete_profile_msg.replace("{name}", &name),
        )
        .danger(true);

        crate::ui::views::dialogs::show_confirm_dialog(hwnd, options, move || {
            let mut deleted = false;
            if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id != "global" {
                    engine.profile_manager.remove_profile(&sel_id);
                    engine.profile_manager.selected_id = "global".to_string();
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                    deleted = true;
                }
            }
            if deleted {
                unsafe {
                    let ptr = crate::ui::win32::GetWindowLongPtrW(hwnd, crate::ui::win32::GWLP_USERDATA)
                        as *const super::ControlPanelControls;
                    if !ptr.is_null() {
                        super::SidebarControls::switch_to_profile(&*ptr, hwnd, "global");
                    }
                }
            }
        });
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, controls: &super::ControlPanelControls) -> bool {
        if ctl_id == IDC_BTN_DEL_PROFILE {
            self.handle_delete_profile(controls, hwnd);
            true
        } else {
            false
        }
    }

    pub fn set_visible(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        let is_root = self.is_root.load(Ordering::SeqCst);
        unsafe {
            ShowWindow(self.lbl_name.hwnd(), cmd);
            ShowWindow(self.lbl_status.hwnd(), cmd);
            if visible && !is_root {
                ShowWindow(self.btn_delete.hwnd(), SW_SHOW);
                EnableWindow(self.btn_delete.hwnd(), 1);
            } else {
                ShowWindow(self.btn_delete.hwnd(), SW_HIDE);
            }
        }
    }
}

impl Drop for ProfileBanner {
    fn drop(&mut self) {
        if self.hfont_title != 0 {
            unsafe {
                DeleteObject(self.hfont_title);
            }
        }
    }
}
