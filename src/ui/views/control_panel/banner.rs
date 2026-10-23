//! Profile Header Banner Component
//! Shows current profile icon, name, active status, and delete action.
//! Powered by Declarative Single-HWND UI Engine (`Surface<BannerMsg>`).

use crate::engine::profile::AppProfile;
use crate::language::LanguageStrings;
use crate::ui::tokens::layout::*;
use crate::ui_framework::core::*;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub enum BannerMsg {
    DeleteProfile,
}

pub struct ProfileBanner {
    pub surface: Surface<BannerMsg>,
    current_data: Arc<RwLock<(String, String, bool)>>,
}

impl ProfileBanner {
    pub fn create(parent: isize, _hfont_normal: isize) -> Option<Self> {
        let surface =
            Surface::create(parent, HEADER_X, HEADER_TITLE_Y - 2, HEADER_TITLE_WIDTH, 52)?;
        surface.set_content_width(HEADER_TITLE_WIDTH);

        let strings = crate::language::current();
        let initial_title = format!("🌐 {}", strings.banner_root_title);
        let initial_subtitle = strings.banner_root_desc.to_string();

        let current_data = Arc::new(RwLock::new((
            initial_title.clone(),
            initial_subtitle.clone(),
            false,
        )));
        surface.render(Self::view(
            &initial_title,
            &initial_subtitle,
            false,
            strings,
        ));

        surface.set_on_msg(move |msg| match msg {
            BannerMsg::DeleteProfile => {
                Self::handle_delete_profile(parent);
            }
        });

        Some(Self {
            surface,
            current_data,
        })
    }

    pub fn view(
        title: &str,
        subtitle: &str,
        is_child: bool,
        strings: &LanguageStrings,
    ) -> Element<BannerMsg> {
        row(elements![
            col(elements![
                text(title).bold().size(Size::Lg),
                text(subtitle).muted().size(Size::Sm),
            ])
            .gap(1)
            .flex_1(),
            if is_child {
                button(
                    strings.banner_btn_delete,
                    Variant::Danger,
                    BannerMsg::DeleteProfile,
                )
                .w(114)
                .into()
            } else {
                empty()
            },
        ])
        .items_center()
        .w_full()
    }

    /// Updates banner display for the selected profile (`None` = Global Root, `Some(child)` = Child profile).
    pub fn update_profile(&self, child: Option<&AppProfile>) {
        let strings = crate::language::current();
        let (title, subtitle, is_child) = match child {
            None => (
                format!("🌐 {}", strings.banner_root_title),
                strings.banner_root_desc.to_string(),
                false,
            ),
            Some(profile) => {
                let proc_str = if profile.process_names.is_empty() {
                    strings.sidebar_unassigned.to_string()
                } else {
                    profile.process_names.join(", ")
                };
                (
                    format!("{} {}", profile.icon, profile.name),
                    format!("{}{}", strings.banner_status_inactive_prefix, proc_str),
                    true,
                )
            }
        };

        if let Ok(mut d) = self.current_data.write() {
            *d = (title.clone(), subtitle.clone(), is_child);
        }
        self.surface
            .render(Self::view(&title, &subtitle, is_child, strings));
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        if let Ok(d) = self.current_data.read() {
            self.surface.render(Self::view(&d.0, &d.1, d.2, strings));
        }
    }

    pub fn handle_delete_profile(hwnd: isize) {
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

    #[inline]
    pub fn on_command(&self, _hwnd: isize, _ctl_id: u32, _notify_code: u32) -> bool {
        false
    }

    #[inline]
    pub fn set_visible(&self, visible: bool) {
        self.surface.set_visible(visible);
    }
}
