//! Tab 3: About controls (Thông tin tác giả, phiên bản, liên hệ)
//! Redesigned with modern Fluent Hero Card matching mockup_profiles layout.

use crate::language::LanguageStrings;
use crate::ui::colors::ThemePalette;
use crate::ui::win32::{
    create_app_font, DeleteObject, SetBkColor, SetTextColor, ShowWindow, FW_BOLD,
    SW_HIDE, SW_SHOW,
};
use crate::ui::components::*;
use crate::ui::tokens::id::*;
use crate::ui::tokens::layout::*;
use crate::ui::tokens::typography::*;

pub struct TabAbout {
    pub lbl_header_title: Label,
    pub lbl_header_sub: Label,
    pub label_about_title: Label,
    pub label_about_author: Label,
    pub label_about_engine: Label,
    pub btn_github: PushButton,
    pub hfont_title: isize,
}

impl TabAbout {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let hfont_title = create_app_font(FONT_SIZE_TITLE, FW_BOLD);

        // --- Header Section ---
        let lbl_header_title = Label::create_with_id(
            parent,
            IDC_ABOUT_HEADER_TITLE,
            strings.about_header_title,
            HEADER_X,
            HEADER_TITLE_Y,
            HEADER_TITLE_WIDTH,
            HEADER_TITLE_HEIGHT,
            hfont_title,
        )?;

        let lbl_header_sub = Label::create_with_id(
            parent,
            IDC_ABOUT_HEADER_SUB,
            strings.about_header_sub,
            HEADER_X,
            HEADER_SUB_Y,
            HEADER_SUB_WIDTH,
            HEADER_SUB_HEIGHT,
            hfont_normal,
        )?;

        // --- Inside Hero Card ---
        let label_about_title = Label::create_centered(
            parent,
            IDC_ABOUT_APP_TITLE,
            strings.about_title,
            246,
            174,
            560,
            26,
            hfont_title,
        )?;

        let label_about_author = Label::create_centered(
            parent,
            IDC_ABOUT_APP_AUTHOR,
            strings.about_author,
            246,
            204,
            560,
            20,
            hfont_normal,
        )?;

        let label_about_engine = Label::create_centered(
            parent,
            IDC_ABOUT_APP_ENGINE,
            strings.about_engine_desc,
            246,
            228,
            560,
            20,
            hfont_normal,
        )?;

        let btn_github = PushButton::create_with_variant(
            parent,
            IDC_BTN_GITHUB,
            strings.about_github_btn,
            446,
            266,
            160,
            32,
            hfont_bold,
            ButtonVariant::Primary,
            None,
        )?;

        Some(Self {
            lbl_header_title,
            lbl_header_sub,
            label_about_title,
            label_about_author,
            label_about_engine,
            btn_github,
            hfont_title,
        })
    }

    pub fn update_visibility(&self, visible: bool) {
        unsafe {
            let show_cmd = if visible { SW_SHOW } else { SW_HIDE };
            ShowWindow(self.lbl_header_title.hwnd(), show_cmd);
            ShowWindow(self.lbl_header_sub.hwnd(), show_cmd);
            ShowWindow(self.label_about_title.hwnd(), show_cmd);
            ShowWindow(self.label_about_author.hwnd(), show_cmd);
            ShowWindow(self.label_about_engine.hwnd(), show_cmd);
            ShowWindow(self.btn_github.hwnd(), show_cmd);
        }
    }

    pub fn on_command(&self, _hwnd: isize, ctl_id: u32) -> bool {
        match ctl_id {
            IDC_BTN_GITHUB => {
                crate::ui::wnd_proc::tray::open_url("https://github.com/uongsuadaubung/kora");
                true
            }
            IDC_ABOUT_APP_AUTHOR => {
                crate::ui::wnd_proc::tray::open_url("mailto:manhkien13041997@gmail.com");
                true
            }
            _ => false,
        }
    }

    pub fn on_ctlcolor_static(&self, hdc: isize, ctrl_id: u32, is_dark: bool) -> Option<isize> {
        let palette = ThemePalette::get(is_dark);
        match ctrl_id {
            IDC_ABOUT_HEADER_TITLE => {
                unsafe {
                    SetBkColor(hdc, palette.bg_window);
                    SetTextColor(hdc, palette.text_primary);
                }
                let br = crate::ui::theme::get_panel_bg_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            IDC_ABOUT_HEADER_SUB => {
                unsafe {
                    SetBkColor(hdc, palette.bg_window);
                    SetTextColor(hdc, palette.text_secondary);
                }
                let br = crate::ui::theme::get_panel_bg_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            IDC_ABOUT_APP_TITLE => {
                unsafe {
                    SetBkColor(hdc, palette.bg_card);
                    SetTextColor(hdc, palette.text_primary);
                }
                let br = crate::ui::theme::get_tab_card_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            IDC_ABOUT_APP_AUTHOR | IDC_ABOUT_APP_ENGINE => {
                unsafe {
                    SetBkColor(hdc, palette.bg_card);
                    SetTextColor(hdc, palette.text_secondary);
                }
                let br = crate::ui::theme::get_tab_card_brush();
                if br != 0 {
                    return Some(br);
                }
            }
            _ => {}
        }
        None
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.lbl_header_title.set_text(strings.about_header_title);
        self.lbl_header_sub.set_text(strings.about_header_sub);
        self.label_about_title.set_text(strings.about_title);
        self.label_about_author.set_text(strings.about_author);
        self.label_about_engine.set_text(strings.about_engine_desc);
        self.btn_github.set_text(strings.about_github_btn);
    }
}

impl Drop for TabAbout {
    fn drop(&mut self) {
        unsafe {
            if self.hfont_title != 0 {
                DeleteObject(self.hfont_title);
            }
        }
    }
}
