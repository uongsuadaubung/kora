//! Tab 2: Mechanical Keyboard Typing Sound (Bàn phím cơ & Âm thanh)
//!
//! Powered by the Declarative Single-HWND UI Engine (`Surface<SoundMsg>`).

use crate::engine::EngineConfig;
use crate::engine::profile::{AppProfile, Child};
use crate::language::LanguageStrings;
use crate::ui::views::control_panel::components::{inheritable_switch_row, section_header};
use crate::ui_framework::core::*;
use crate::ui_framework::tokens::layout;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum SoundMsg {
    ToggleSoundEnabled(Option<bool>),
    SelectSwitchProfile(usize),
    TestSound,
    ChangeVolume(i32, bool),
    DownloadSoundpack,
}

struct SoundViewState {
    is_root: bool,
    sound_override: Option<bool>,
    global_sound_enabled: bool,
    volume: u8,
    selected_profile: String,
    switch_profiles: Vec<String>,
    is_downloading: bool,
    custom_note: Option<String>,
}

pub struct TabSound {
    surface: Surface<SoundMsg>,
    state: Rc<RefCell<SoundViewState>>,
    pub sound_available: Cell<bool>,
}

impl TabSound {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let viewport_x = layout::CARD_X;
        let viewport_y = layout::CARD_START_Y;
        let viewport_w = layout::CARD_WIDTH + 8;
        let viewport_h = layout::CARD_BOTTOM_Y - layout::CARD_START_Y;

        let surface = Surface::create(parent, viewport_x, viewport_y, viewport_w, viewport_h)?;
        let switch_profiles = crate::engine::config_store::list_switch_profiles();
        let sound_available = !switch_profiles.is_empty();

        let state = Rc::new(RefCell::new(SoundViewState {
            is_root: true,
            sound_override: None,
            global_sound_enabled: false,
            volume: 50,
            selected_profile: switch_profiles.first().cloned().unwrap_or_default(),
            switch_profiles,
            is_downloading: false,
            custom_note: None,
        }));

        let surface_cb = surface.clone();
        surface.set_on_msg(move |msg| {
            Self::handle_message(parent, &surface_cb, msg);
        });

        let tab = Self {
            surface,
            state,
            sound_available: Cell::new(sound_available),
        };
        tab.refresh();
        Some(tab)
    }

    pub fn viewport_hwnd(&self) -> isize {
        self.surface.hwnd()
    }

    fn refresh(&self) {
        let strings = crate::language::current();
        let st = self.state.borrow();
        let tree = Self::view(&st, self.sound_available.get(), strings);
        drop(st);
        self.surface.render(tree);
    }

    pub fn load_profile(&self, child: Option<&AppProfile<Child>>, global_config: &EngineConfig) {
        let vol = child
            .and_then(|c| c.sound_volume)
            .unwrap_or(global_config.sound_volume);
        let prof_name = child
            .and_then(|c| c.sound_profile.as_deref())
            .unwrap_or(&global_config.sound_profile)
            .to_string();

        {
            let mut st = self.state.borrow_mut();
            st.is_root = child.is_none();
            st.sound_override = child.and_then(|c| c.sound_enabled);
            st.global_sound_enabled = global_config.sound_enabled;
            st.volume = vol;
            st.selected_profile = prof_name;
        }
        self.refresh();
    }

    pub fn update_language(&self, _strings: &LanguageStrings) {
        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((child, cfg)) = snap {
            self.load_profile(child.as_ref(), &cfg);
        } else {
            self.refresh();
        }
    }

    pub fn on_command(&self, _hwnd: isize, _ctl_id: u32, _notify_code: u32) -> bool {
        false
    }

    pub fn on_hscroll(&self, _wparam: usize, _slider_hwnd: isize) -> bool {
        false
    }

    pub fn on_soundpack_installed(
        &self,
        status: &crate::platform::win32::sound::installer::SoundpackInstallStatus,
    ) {
        if let crate::platform::win32::sound::installer::SoundpackInstallStatus::Success {
            ..
        } = status
        {
            let profiles = crate::engine::config_store::list_switch_profiles();
            self.sound_available.set(!profiles.is_empty());

            if let Some(first_name) = profiles.first().cloned() {
                let name_arc = std::sync::Arc::from(first_name.as_str());
                crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().sound_profile = name_arc;
                    crate::ui::controller::reconfigure_sound(engine);
                });
                let mut st = self.state.borrow_mut();
                st.selected_profile = first_name;
                st.switch_profiles = profiles;
            }
        }
        {
            let mut st = self.state.borrow_mut();
            st.is_downloading = false;
            st.custom_note = None;
        }
        self.refresh();
    }

    pub fn paint(&self, _hdc: isize, _right_bound: i32) {
        self.surface.invalidate();
    }

    pub fn update_visibility(&self, visible: bool) {
        self.surface.set_visible(visible);
    }

    fn view(
        st: &SoundViewState,
        sound_available: bool,
        strings: &LanguageStrings,
    ) -> Element<SoundMsg> {
        let effective_enabled = st.sound_override.unwrap_or(st.global_sound_enabled);
        let controls_enabled = sound_available && effective_enabled;

        let switch_options = if st.switch_profiles.is_empty() {
            vec![strings.sound_profile_default.to_string()]
        } else {
            st.switch_profiles.clone()
        };
        let sel_idx = st
            .switch_profiles
            .iter()
            .position(|p| p.eq_ignore_ascii_case(&st.selected_profile))
            .unwrap_or(0);

        let card1 = card(elements![
            inheritable_switch_row(
                strings.sound_title,
                Some(strings.check_sound_enabled.to_string()),
                st.is_root,
                st.sound_override,
                st.global_sound_enabled,
                strings,
                SoundMsg::ToggleSoundEnabled,
            ),
            divider(),
            col(elements![
                text(strings.label_switch_type).bold(),
                row(elements![
                    select(switch_options, sel_idx, SoundMsg::SelectSwitchProfile)
                        .enabled(controls_enabled)
                        .flex_1(),
                    button(
                        strings.btn_test_sound,
                        Variant::Secondary,
                        SoundMsg::TestSound,
                    )
                    .enabled(controls_enabled),
                ])
                .items_center()
                .gap(3),
            ])
            .gap(1),
            col(elements![
                row(elements![
                    text(strings.label_volume).bold().flex_1(),
                    text(format!("{}%", st.volume)).bold().accent(),
                ])
                .items_center()
                .justify_between(),
                slider(0, 100, st.volume as i32, SoundMsg::ChangeVolume).enabled(controls_enabled),
            ])
            .gap(1),
        ]);

        let note_txt = st.custom_note.clone().unwrap_or_else(|| {
            if sound_available {
                strings.soundpack_note_installed.to_string()
            } else {
                strings.soundpack_note_missing.to_string()
            }
        });
        let btn_txt = if st.is_downloading {
            strings.btn_downloading_soundpack
        } else if sound_available {
            strings.btn_update_soundpack
        } else {
            strings.btn_download_soundpack
        };

        let card2 = card(elements![
            section_header(strings.soundpack_card_title),
            text(note_txt).muted().size(Size::Sm),
            row(elements![
                button(
                    btn_txt,
                    if sound_available {
                        Variant::Secondary
                    } else {
                        Variant::Primary
                    },
                    SoundMsg::DownloadSoundpack,
                )
                .enabled(!st.is_downloading),
                spacer(),
            ])
            .pt(1),
        ]);

        col(elements![card1, card2]).gap(3).pb(3)
    }

    fn handle_message(parent_hwnd: isize, _surface: &Surface<SoundMsg>, msg: SoundMsg) {
        use crate::ui::controller::{self, EditTarget};

        let Some(c) = crate::ui::controls() else {
            return;
        };
        let tab = &c.tab_sound;

        match msg {
            SoundMsg::ToggleSoundEnabled(next_opt) => {
                controller::edit_selected_then(
                    |target| match target {
                        EditTarget::Global(cfg) => {
                            cfg.sound_enabled = next_opt.unwrap_or(!cfg.sound_enabled);
                        }
                        EditTarget::Child(prof) => {
                            prof.sound_enabled = next_opt;
                        }
                    },
                    |engine| controller::reconfigure_sound(engine),
                );
                if let Some((child, cfg)) = controller::with_engine(|e| {
                    let pm = &e.profile_manager;
                    (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
                }) {
                    tab.load_profile(child.as_ref(), &cfg);
                }
            }
            SoundMsg::SelectSwitchProfile(idx) => {
                let chosen = tab
                    .state
                    .borrow()
                    .switch_profiles
                    .get(idx)
                    .cloned();
                if let Some(name) = chosen {
                    tab.state.borrow_mut().selected_profile = name.clone();
                    controller::edit_selected_then(
                        |target| match target {
                            EditTarget::Global(cfg) => {
                                cfg.sound_profile = std::sync::Arc::from(name.as_str());
                            }
                            EditTarget::Child(prof) => {
                                prof.sound_profile = Some(std::sync::Arc::from(name.as_str()));
                            }
                        },
                        |engine| controller::reconfigure_sound(engine),
                    );
                    crate::platform::win32::sound::play_test_sound();
                    tab.refresh();
                }
            }
            SoundMsg::TestSound => {
                crate::platform::win32::sound::play_test_sound();
            }
            SoundMsg::ChangeVolume(pos, committed) => {
                let vol = pos.clamp(0, 100) as u8;
                tab.state.borrow_mut().volume = vol;
                if committed {
                    controller::edit_selected_then(
                        |target| match target {
                            EditTarget::Global(cfg) => cfg.sound_volume = vol,
                            EditTarget::Child(child) => child.sound_volume = Some(vol),
                        },
                        |engine| controller::reconfigure_sound(engine),
                    );
                    tab.refresh();
                }
            }
            SoundMsg::DownloadSoundpack => {
                {
                    let mut st = tab.state.borrow_mut();
                    st.is_downloading = true;
                    st.custom_note =
                        Some("Đang kết nối GitHub & tải 13 switch... ⏳".to_string());
                }
                tab.refresh();

                std::thread::spawn(move || {
                    let status = Box::new(
                        crate::platform::win32::sound::installer::download_and_install_soundpack(),
                    );
                    let ptr = Box::into_raw(status);
                    let posted = unsafe {
                        crate::ui::win32::PostMessageW(
                            parent_hwnd,
                            crate::ui::WM_APP_SOUNDPACK_INSTALLED,
                            0,
                            ptr as isize,
                        )
                    };
                    if posted == 0 {
                        drop(unsafe { Box::from_raw(ptr) });
                    }
                });
            }
        }
    }
}
