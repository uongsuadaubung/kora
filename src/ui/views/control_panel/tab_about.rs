//! Tab 3: About controls (Thông tin tác giả, phiên bản, liên hệ)
//! Uses CardLayout (page_header, auto-HWND, auto-i18n) and semantic component event matchers.

use crate::language::LanguageStrings;
use crate::ui::prelude::*;
use crate::ui_framework::tokens::typography::*;
use crate::ui_framework::win32::{
    CreatePen, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject, DrawTextW,
    FW_BOLD, PS_SOLID, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};

const HERO_CARD_X: i32 = 246;
const HERO_CARD_W: i32 = 560;
const LOGO_TOP: i32 = 114;
const LOGO_SIZE: i32 = 48;

pub struct TabAbout {
    layout: CardLayout,
    pub label_about_author: Label<Link>,
    pub btn_check_update: PushButton,
    pub btn_github: PushButton,
}

impl TabAbout {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let hfont_title = cached_font(FONT_SIZE_TITLE, FW_BOLD);
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold)
            .at(86)
            .with_bounds(HERO_CARD_X, HERO_CARD_W);

        // --- Header Section ---
        ly.page_header(|s| s.about_header_title, |s| s.about_header_sub)?;

        // --- Hero Card ---
        let (label_about_author, btn_check_update, btn_github) =
            ly.card_with_pad(88, 38, |card| {
                // Reserve space at top for painted K logo (LOGO_TOP..LOGO_TOP+LOGO_SIZE)
                let lbl_title = card.track_i18n(
                    Label::new_centered(parent, strings.about_title, hfont_title)?,
                    |s| s.about_title,
                );
                card.horizontal(26, 4, |row| {
                    row.add_stretch(1, Some(lbl_title.hwnd()));
                });

                let label_about_author = card.track_i18n(
                    Label::new_centered(parent, strings.about_author, hfont_normal)?
                        .as_link()
                        .with_tone(LabelTone::Secondary),
                    |s| s.about_author,
                );
                card.horizontal(20, 4, |row| {
                    row.add_stretch(1, Some(label_about_author.hwnd()));
                });

                let lbl_engine = card.track_i18n(
                    Label::new_centered(parent, strings.about_engine_desc, hfont_normal)?
                        .with_tone(LabelTone::Secondary),
                    |s| s.about_engine_desc,
                );
                card.horizontal(20, 18, |row| {
                    row.add_stretch(1, Some(lbl_engine.hwnd()));
                });

                let btn_check_update = card.button(
                    |s| s.btn_check_update,
                    true,
                    ButtonVariant::Primary,
                )?;
                let btn_github = card.button(
                    |s| s.about_github_btn,
                    true,
                    ButtonVariant::Secondary,
                )?;

                card.horizontal(32, 0, |row| {
                    row.align(CrossAlign::Center);
                    row.add_spacer();
                    row.add_fixed(156, Some(btn_check_update.hwnd()));
                    row.add_fixed(16, None);
                    row.add_fixed(156, Some(btn_github.hwnd()));
                    row.add_spacer();
                });
                Some((label_about_author, btn_check_update, btn_github))
            })?;

        Some(Self {
            layout: ly,
            label_about_author,
            btn_check_update,
            btn_github,
        })
    }

    pub fn paint(&self, hdc: isize, right_bound: i32, palette: &ThemePalette) {
        self.layout.paint_cards(hdc, HERO_CARD_X, right_bound);

        unsafe {
            let logo_x = HERO_CARD_X + (right_bound - HERO_CARD_X - LOGO_SIZE) / 2;
            let logo_br = CreateSolidBrush(palette.accent);
            let logo_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.accent);
            let old_b = SelectObject(hdc, logo_br);
            let old_p = SelectObject(hdc, logo_pen);
            RoundRect(
                hdc,
                logo_x,
                LOGO_TOP,
                logo_x + LOGO_SIZE,
                LOGO_TOP + LOGO_SIZE,
                radii::RADIUS_LG,
                radii::RADIUS_LG,
            );
            SelectObject(hdc, old_b);
            SelectObject(hdc, old_p);
            DeleteObject(logo_br);
            DeleteObject(logo_pen);

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, palette.text_white);
            let logo_font = cached_font(FONT_SIZE_HERO, FW_BOLD);
            let old_f = SelectObject(hdc, logo_font);
            let mut text_rc = RECT {
                left: logo_x,
                top: LOGO_TOP,
                right: logo_x + LOGO_SIZE,
                bottom: LOGO_TOP + LOGO_SIZE,
            };
            DrawTextW(
                hdc,
                to_wide("K").as_ptr(),
                -1,
                &mut text_rc,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER,
            );
            SelectObject(hdc, old_f);
        }
    }

    pub fn update_visibility(&self, visible: bool) {
        self.layout.set_visible(visible);
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        if self.btn_check_update.is_clicked(ctl_id, notify_code) {
            let hwnd_panel = hwnd;
            let strings = crate::language::current();
            self.btn_check_update.set_text(strings.btn_checking_update);
            self.btn_check_update.set_enabled(false);

            std::thread::spawn(move || {
                let status = Box::new(crate::updater::check_for_updates());
                let ptr = Box::into_raw(status);
                let posted = unsafe {
                    crate::ui::win32::PostMessageW(
                        hwnd_panel,
                        crate::ui::WM_APP_UPDATE_CHECKED,
                        0,
                        ptr as isize,
                    )
                };
                if posted == 0 {
                    drop(unsafe { Box::from_raw(ptr) });
                }
            });
            return true;
        }
        if self.btn_github.is_clicked(ctl_id, notify_code) {
            crate::ui::wnd_proc::tray::open_url("https://github.com/uongsuadaubung/kora");
            return true;
        }
        if self.label_about_author.is_clicked(ctl_id, notify_code) {
            crate::ui::wnd_proc::tray::open_url("mailto:manhkien13041997@gmail.com");
            return true;
        }
        false
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);
    }

    /// UI-thread completion of the background update check (see WM_APP_UPDATE_CHECKED).
    pub fn on_update_checked(&self) {
        self.btn_check_update
            .set_text(crate::language::current().btn_check_update);
        self.btn_check_update.set_enabled(true);
    }
}
