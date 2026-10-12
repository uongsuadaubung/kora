//! Tab 2: Mechanical Keyboard Typing Sound (Bàn phím cơ & Âm thanh)

use crate::engine::EngineConfig;
use crate::engine::profile::AppProfile;
use crate::language::LanguageStrings;
use crate::ui::components::*;
use crate::ui::tokens::id::*;
use crate::ui::tokens::layout;
use crate::ui::win32::{CBN_SELCHANGE, SW_HIDE, SW_SHOW, ShowWindow};
use std::cell::{Cell, RefCell};

// Card 1: Thiết lập Âm thanh gõ phím cơ (layout::CARD_START_Y..SOUND_CARD1_BOTTOM)
pub const SOUND_CARD1_BOTTOM: i32 = 284;

// Card 2: Quản lý & Tải gói âm thanh switch (Cách Card 1 khoảng layout::CARD_GAP = 12px)
pub const SOUND_CARD2_TOP: i32 = SOUND_CARD1_BOTTOM + layout::CARD_GAP; // 296
pub const SOUND_CARD2_BOTTOM: i32 = SOUND_CARD2_TOP + 116; // 412

// Title row in Card 1: title label on the left, "enable" checkbox drawn flush right up to CHECK_RIGHT.
const TITLE_X: i32 = layout::CARD_CONTENT_X; // 260
const TITLE_Y: i32 = 126;
const TITLE_H: i32 = 20;
const CHECK_X: i32 = 500;
const CHECK_RIGHT: i32 = 790;
const CHECK_Y: i32 = 125;
const CHECK_H: i32 = 22;

pub struct TabSound {
    pub lbl_sound_title: Label,
    pub check_sound_enabled: CheckBox,
    pub lbl_switch_type: Label,
    pub combo_switch_type: ComboBox,
    pub btn_test_sound: PushButton,
    pub lbl_volume: Label,
    pub slider_volume: Slider,
    pub lbl_volume_val: Label,
    pub lbl_soundpack_title: Label,
    pub lbl_soundpack_note: Label,
    pub btn_download_soundpack: PushButton,
    pub sound_available: Cell<bool>,
    /// Switch profile names, scanned at creation and reloaded upon install (index == combo index).
    switch_profiles: RefCell<Vec<String>>,
}

impl TabSound {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        // --- Card 1 Controls ---
        let lbl_sound_title = Label::create(
            parent,
            strings.sound_title,
            TITLE_X,
            TITLE_Y,
            CHECK_X - TITLE_X - 5,
            TITLE_H,
            hfont_bold,
        )?;

        let check_sound_enabled = CheckBox::create(
            parent,
            IDC_CHECK_SOUND_ENABLED,
            strings.check_sound_enabled,
            CHECK_X,
            CHECK_Y,
            CHECK_RIGHT - CHECK_X,
            CHECK_H,
            hfont_bold,
        )?;
        check_sound_enabled.set_align_right(true);

        let lbl_switch_type = Label::create(
            parent,
            strings.label_switch_type,
            layout::CARD_CONTENT_X,
            156,
            layout::CARD_CONTENT_WIDTH,
            18,
            hfont_bold,
        )?;

        let combo_switch_type = ComboBox::create(
            parent,
            IDC_COMBO_SWITCH_TYPE,
            layout::CARD_CONTENT_X,
            178,
            410,
            180,
            hfont_normal,
        )?;
        let switch_profiles = crate::engine::config_store::list_switch_profiles();
        let sound_available = !switch_profiles.is_empty();
        for p in &switch_profiles {
            combo_switch_type.add_item(p);
        }
        if switch_profiles.is_empty() {
            combo_switch_type.add_item(strings.sound_profile_default);
        }
        combo_switch_type.set_selected(0);

        let btn_test_sound = PushButton::create(
            parent,
            IDC_BTN_TEST_SOUND,
            strings.btn_test_sound,
            680,
            178,
            110,
            28,
            hfont_normal,
        )?;

        let lbl_volume = Label::create(
            parent,
            strings.label_volume,
            layout::CARD_CONTENT_X,
            220,
            120,
            18,
            hfont_bold,
        )?;

        let lbl_volume_val = Label::create_right_with_id(
            parent,
            IDC_LABEL_VOLUME_VAL,
            "50%",
            730,
            220,
            60,
            18,
            hfont_bold,
        )?;

        let slider_volume = Slider::create_with_value(
            parent,
            IDC_SLIDER_SOUND_VOLUME,
            layout::CARD_CONTENT_X,
            242,
            layout::CARD_CONTENT_WIDTH,
            26,
            0,
            100,
            50,
        )?;

        // --- Card 2 Controls (Tải & Quản lý gói âm thanh switch) ---
        let lbl_soundpack_title = Label::create_with_id(
            parent,
            IDC_LABEL_SOUNDPACK_TITLE,
            strings.soundpack_card_title,
            layout::CARD_CONTENT_X,
            SOUND_CARD2_TOP + 14,
            layout::CARD_CONTENT_WIDTH,
            18,
            hfont_bold,
        )?;

        let lbl_soundpack_note = Label::create_with_id(
            parent,
            IDC_LABEL_SOUNDPACK_STATUS,
            if sound_available {
                strings.soundpack_note_installed
            } else {
                strings.soundpack_note_missing
            },
            layout::CARD_CONTENT_X,
            SOUND_CARD2_TOP + 36,
            layout::CARD_CONTENT_WIDTH,
            18,
            hfont_normal,
        )?;

        let btn_download_soundpack = PushButton::create_with_variant(
            parent,
            IDC_BTN_DOWNLOAD_SOUNDPACK,
            if sound_available {
                strings.btn_update_soundpack
            } else {
                strings.btn_download_soundpack
            },
            layout::CARD_CONTENT_X,
            SOUND_CARD2_TOP + 64,
            260,
            32,
            hfont_normal,
            if sound_available {
                ButtonVariant::Secondary
            } else {
                ButtonVariant::Primary
            },
            None,
        )?;

        let tab = Self {
            lbl_sound_title,
            check_sound_enabled,
            lbl_switch_type,
            combo_switch_type,
            btn_test_sound,
            lbl_volume,
            slider_volume,
            lbl_volume_val,
            lbl_soundpack_title,
            lbl_soundpack_note,
            btn_download_soundpack,
            sound_available: Cell::new(sound_available),
            switch_profiles: RefCell::new(switch_profiles),
        };
        tab.update_sound_controls_state();
        Some(tab)
    }

    pub fn update_sound_controls_state(&self) {
        let is_enabled = self.check_sound_enabled.is_checked() && self.sound_available.get();
        self.combo_switch_type.set_enabled(is_enabled);
        self.slider_volume.set_enabled(is_enabled);
        self.btn_test_sound.set_enabled(is_enabled);
    }

    fn select_switch_profile(&self, name: &str) {
        let profiles = self.switch_profiles.borrow();
        if let Some(pos) = profiles.iter().position(|p| p.eq_ignore_ascii_case(name)) {
            self.combo_switch_type.set_selected(pos);
        }
    }

    fn selected_switch_profile(&self) -> Option<String> {
        let profiles = self.switch_profiles.borrow();
        profiles.get(self.combo_switch_type.get_selected()?).cloned()
    }

    pub fn load_config(&self, config: &EngineConfig) {
        self.check_sound_enabled.set_checked(config.sound_enabled);
        self.slider_volume.set_pos(config.sound_volume as u32);
        self.lbl_volume_val
            .set_text(&format!("{}%", config.sound_volume));
        self.select_switch_profile(&config.sound_profile);
        self.update_sound_controls_state();
    }

    pub fn load_profile(&self, profile: &AppProfile, global_config: &EngineConfig) {
        if profile.is_root {
            self.load_config(global_config);
            return;
        }
        let vol = profile.sound_volume.unwrap_or(global_config.sound_volume);
        let prof_name = profile
            .sound_profile
            .as_deref()
            .unwrap_or(&global_config.sound_profile);

        self.check_sound_enabled
            .set_tri_state(profile.sound_enabled, global_config.sound_enabled);
        self.slider_volume.set_pos(vol as u32);
        self.lbl_volume_val.set_text(&format!("{}%", vol));
        self.select_switch_profile(prof_name);
        self.update_sound_controls_state();
    }

    pub fn read_config(&self, config: &mut EngineConfig) {
        config.sound_enabled = self.check_sound_enabled.is_checked();
        config.sound_volume = self.slider_volume.get_pos() as u8;
        if let Some(name) = self.selected_switch_profile() {
            config.sound_profile = std::sync::Arc::from(name.as_str());
        }
    }

    /// Child: only the checkbox is read here; volume is set in `on_hscroll`,
    /// switch type only when the combo actually changes (keeps them inheriting otherwise).
    pub fn read_profile(&self, profile: &mut AppProfile, combo_changed: bool) {
        profile.sound_enabled = self.check_sound_enabled.tri_state();
        if combo_changed && let Some(name) = self.selected_switch_profile() {
            profile.sound_profile = Some(std::sync::Arc::from(name.as_str()));
        }
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.lbl_sound_title.set_text(strings.sound_title);
        self.check_sound_enabled.set_text(strings.check_sound_enabled);
        self.lbl_switch_type.set_text(strings.label_switch_type);
        self.btn_test_sound.set_text(strings.btn_test_sound);
        self.lbl_volume.set_text(strings.label_volume);
        self.lbl_soundpack_title.set_text(strings.soundpack_card_title);
        if self.sound_available.get() {
            self.btn_download_soundpack.set_text(strings.btn_update_soundpack);
            self.lbl_soundpack_note.set_text(strings.soundpack_note_installed);
        } else {
            self.btn_download_soundpack.set_text(strings.btn_download_soundpack);
            self.lbl_soundpack_note.set_text(strings.soundpack_note_missing);
        }
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        use crate::ui::controller::{self, EditTarget};
        if ctl_id == IDC_BTN_TEST_SOUND {
            crate::platform::win32::sound::play_test_sound();
            return true;
        }

        if ctl_id == IDC_BTN_DOWNLOAD_SOUNDPACK {
            let hwnd_panel = hwnd;
            let strings = crate::language::current();
            self.btn_download_soundpack.set_text(strings.btn_downloading_soundpack);
            self.btn_download_soundpack.set_enabled(false);
            self.lbl_soundpack_note.set_text("Đang kết nối GitHub & tải 13 switch... ⏳");

            std::thread::spawn(move || {
                let status = Box::new(crate::platform::win32::sound::installer::download_and_install_soundpack());
                let ptr = Box::into_raw(status);
                let posted = unsafe {
                    crate::ui::win32::PostMessageW(
                        hwnd_panel,
                        crate::ui::WM_APP_SOUNDPACK_INSTALLED,
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

        let is_check = ctl_id == IDC_CHECK_SOUND_ENABLED && notify_code == 0;
        let is_combo = ctl_id == IDC_COMBO_SWITCH_TYPE && notify_code == CBN_SELCHANGE;

        if is_check || is_combo {
            if is_check {
                self.update_sound_controls_state();
            }
            controller::edit_selected_then(
                |target| match target {
                    EditTarget::Global(cfg) => self.read_config(cfg),
                    EditTarget::Child(child) => self.read_profile(child, is_combo),
                },
                |engine| controller::reconfigure_sound(engine),
            );
            // Preview the newly selected switch (after reconfigure, outside the engine lock).
            if is_combo {
                crate::platform::win32::sound::play_test_sound();
            }
            return true;
        }

        false
    }

    /// UI-thread completion of the background soundpack download & installation
    pub fn on_soundpack_installed(&self, status: &crate::platform::win32::sound::installer::SoundpackInstallStatus) {
        let strings = crate::language::current();
        if let crate::platform::win32::sound::installer::SoundpackInstallStatus::Success { .. } = status {
            let profiles = crate::engine::config_store::list_switch_profiles();
            self.sound_available.set(!profiles.is_empty());

            self.combo_switch_type.clear();
            for p in &profiles {
                self.combo_switch_type.add_item(p);
            }
            if profiles.is_empty() {
                self.combo_switch_type.add_item(strings.sound_profile_default);
            }
            self.combo_switch_type.set_selected(0);
            *self.switch_profiles.borrow_mut() = profiles;

            self.btn_download_soundpack.set_text(strings.btn_update_soundpack);
            self.btn_download_soundpack.set_enabled(true);
            self.lbl_soundpack_note.set_text(strings.soundpack_note_installed);
            self.update_sound_controls_state();

            if let Some(first_name) = self.switch_profiles.borrow().first() {
                let name_arc = std::sync::Arc::from(first_name.as_str());
                crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().sound_profile = name_arc;
                    crate::ui::controller::reconfigure_sound(engine);
                });
            }
        } else {
            let has_sounds = self.sound_available.get();
            if has_sounds {
                self.btn_download_soundpack.set_text(strings.btn_update_soundpack);
                self.lbl_soundpack_note.set_text(strings.soundpack_note_installed);
            } else {
                self.btn_download_soundpack.set_text(strings.btn_download_soundpack);
                self.lbl_soundpack_note.set_text(strings.soundpack_note_missing);
            }
            self.btn_download_soundpack.set_enabled(true);
        }
    }

    /// `WM_HSCROLL` from the volume trackbar. While dragging (`TB_THUMBTRACK`) only the
    /// label is updated; the value is persisted once the thumb is released / keyboard step.
    pub fn on_hscroll(&self, wparam: usize, slider_hwnd: isize) -> bool {
        use crate::ui::controller;
        const TB_THUMBTRACK: usize = 5;
        if slider_hwnd != self.slider_volume.hwnd() {
            return false;
        }
        let pos = self.slider_volume.get_pos();
        self.lbl_volume_val.set_text(&format!("{pos}%"));
        if wparam & 0xFFFF == TB_THUMBTRACK {
            return true;
        }

        let vol = pos as u8;
        controller::with_engine(|engine| {
            let pm = &mut engine.profile_manager;
            let sel_id = pm.selected_id.clone();
            let changed = if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                let cfg = engine.config_mut();
                std::mem::replace(&mut cfg.sound_volume, vol) != vol
            } else if let Some(child) = pm.get_profile_mut(&sel_id) {
                child.sound_volume.replace(vol) != Some(vol)
            } else {
                false
            };
            // TB_THUMBPOSITION and TB_ENDTRACK both arrive on release: act once.
            if changed {
                controller::save(engine);
                controller::reconfigure_sound(engine);
            }
        });
        true
    }

    pub fn update_visibility(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.lbl_sound_title.hwnd(), cmd);
            ShowWindow(self.check_sound_enabled.hwnd(), cmd);
            ShowWindow(self.lbl_switch_type.hwnd(), cmd);
            ShowWindow(self.combo_switch_type.hwnd(), cmd);
            ShowWindow(self.btn_test_sound.hwnd(), cmd);
            ShowWindow(self.lbl_volume.hwnd(), cmd);
            ShowWindow(self.slider_volume.hwnd(), cmd);
            ShowWindow(self.lbl_volume_val.hwnd(), cmd);
            ShowWindow(self.lbl_soundpack_title.hwnd(), cmd);
            ShowWindow(self.lbl_soundpack_note.hwnd(), cmd);
            ShowWindow(self.btn_download_soundpack.hwnd(), cmd);
        }
    }
}
