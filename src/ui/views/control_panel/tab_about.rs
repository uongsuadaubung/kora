//! Tab 3: About controls (Thông tin tác giả, phiên bản, liên hệ)
//!
//! Powered by the Declarative Single-HWND UI Engine (`Surface<AboutMsg>`).

use crate::language::LanguageStrings;
use crate::ui::colors::ThemePalette;
use crate::ui::views::control_panel::components::page_header;
use crate::ui_framework::core::*;
use crate::ui_framework::tokens::layout;
use crate::ui_framework::win32::{SW_HIDE, ShowWindow};
use std::cell::Cell;

const HERO_CARD_X: i32 = 246;
const HERO_START_Y: i32 = 64;

#[derive(Clone, Debug)]
pub enum AboutMsg {
    CheckUpdate,
    OpenGithub,
    ContactAuthor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum UpdateBtnState {
    Idle,
    Checking,
    Updating,
}

pub struct TabAbout {
    surface: Surface<AboutMsg>,
    btn_state: Cell<UpdateBtnState>,
}

impl TabAbout {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let viewport_w = layout::WINDOW_CLIENT_WIDTH - HERO_CARD_X - 8;
        let viewport_h = layout::CARD_BOTTOM_Y - HERO_START_Y;

        let surface = Surface::create(parent, HERO_CARD_X, HERO_START_Y, viewport_w, viewport_h)?;
        unsafe {
            ShowWindow(surface.hwnd(), SW_HIDE);
        }

        surface.set_on_msg(move |msg| {
            Self::handle_message(parent, msg);
        });

        let tab = Self {
            surface,
            btn_state: Cell::new(UpdateBtnState::Idle),
        };
        tab.refresh();
        Some(tab)
    }

    pub fn viewport_hwnd(&self) -> isize {
        self.surface.hwnd()
    }

    fn refresh(&self) {
        let strings = crate::language::current();
        let tree = Self::view(self.btn_state.get(), strings);
        self.surface.render(tree);
    }

    pub fn paint(&self, _hdc: isize, _right_bound: i32, _palette: &ThemePalette) {
        self.surface.invalidate();
    }

    pub fn update_visibility(&self, visible: bool) {
        self.surface.set_visible(visible);
    }

    pub fn on_command(&self, _hwnd: isize, _ctl_id: u32, _notify_code: u32) -> bool {
        false
    }

    pub fn update_language(&self, _strings: &LanguageStrings) {
        self.refresh();
    }

    pub fn set_updating_state(&self) {
        self.btn_state.set(UpdateBtnState::Updating);
        self.refresh();
    }

    /// UI-thread completion of the background update check (see WM_APP_UPDATE_CHECKED).
    pub fn on_update_checked(&self) {
        self.btn_state.set(UpdateBtnState::Idle);
        self.refresh();
    }

    fn view(btn_state: UpdateBtnState, strings: &LanguageStrings) -> Element<AboutMsg> {
        let header = page_header(strings.about_header_title, strings.about_header_sub);

        let (btn_update_txt, is_busy) = match btn_state {
            UpdateBtnState::Idle => (strings.btn_check_update, false),
            UpdateBtnState::Checking => (strings.btn_checking_update, true),
            UpdateBtnState::Updating => (strings.btn_updating, true),
        };

        let hero_card = card(vec![
            row(vec![
                spacer(),
                badge("KORA", Variant::Primary),
                spacer(),
            ])
            .pt(2),
            text(strings.about_title).bold().size(Size::Lg).text_center(),
            text(strings.about_engine_desc).muted().text_center(),
            divider(),
            row(vec![
                spacer(),
                text(strings.about_author).muted(),
                button("✉️ Email", Variant::Ghost, AboutMsg::ContactAuthor).size(Size::Sm),
                spacer(),
            ])
            .items_center()
            .gap(2),
            row(vec![
                spacer(),
                button(btn_update_txt, Variant::Primary, AboutMsg::CheckUpdate)
                    .enabled(!is_busy)
                    .w(164),
                button(
                    strings.about_github_btn,
                    Variant::Secondary,
                    AboutMsg::OpenGithub,
                )
                .w(164),
                spacer(),
            ])
            .items_center()
            .gap(3)
            .pt(2)
            .pb(2),
        ])
        .p(5)
        .gap(3);

        col(vec![header, hero_card]).gap(3).pb(3)
    }

    fn handle_message(parent_hwnd: isize, msg: AboutMsg) {
        match msg {
            AboutMsg::CheckUpdate => {
                if let Some(c) = crate::ui::controls() {
                    c.tab_about.btn_state.set(UpdateBtnState::Checking);
                    c.tab_about.refresh();
                }
                std::thread::spawn(move || {
                    let status = Box::new(crate::updater::check_for_updates());
                    let ptr = Box::into_raw(status);
                    let posted = unsafe {
                        crate::ui::win32::PostMessageW(
                            parent_hwnd,
                            crate::ui::WM_APP_UPDATE_CHECKED,
                            0,
                            ptr as isize,
                        )
                    };
                    if posted == 0 {
                        drop(unsafe { Box::from_raw(ptr) });
                    }
                });
            }
            AboutMsg::OpenGithub => {
                crate::ui::wnd_proc::tray::open_url("https://github.com/uongsuadaubung/kora");
            }
            AboutMsg::ContactAuthor => {
                crate::ui::wnd_proc::tray::open_url("mailto:manhkien13041997@gmail.com");
            }
        }
    }
}
