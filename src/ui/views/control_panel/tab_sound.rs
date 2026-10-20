//! Tab 2: Mechanical Keyboard Typing Sound (Bàn phím cơ & Âm thanh)
//! Uses CardLayout (auto-HWND & auto-i18n), Slider::parse_scroll, and semantic event matchers.

use crate::engine::EngineConfig;
use crate::engine::profile::{AppProfile, Child};
use crate::language::LanguageStrings;
use crate::ui::prelude::*;
use std::cell::{Cell, RefCell};

pub struct TabSound {
    layout: CardLayout,
    pub check_sound_enabled: CheckBox<Inheritable>,
    pub combo_switch_type: ComboBox,
    pub btn_test_sound: PushButton,
    pub slider_volume: Slider,
    pub lbl_volume_val: Label,
    pub lbl_soundpack_note: Label,
    pub btn_download_soundpack: PushButton,
    pub sound_available: Cell<bool>,
    /// Switch profile names, scanned at creation and reloaded upon install (index == combo index).
    switch_profiles: RefCell<Vec<String>>,
}

impl TabSound {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();
        let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);
        let switch_profiles = crate::engine::config_store::list_switch_profiles();
        let sound_available = !switch_profiles.is_empty();

        // --- Card 1: Thiết lập Âm thanh gõ phím cơ ---
        let (check_sound_enabled, combo_switch_type, btn_test_sound, lbl_volume_val, slider_volume) =
            ly.card_with_pad(spacing::SPACE_14, 16, |card| {
                let ((x_title, w_title), (x_chk, w_chk)) = card.split_right(290, 5);
                let y_title = card.row(22, 8);
                card.label_at(|s| s.sound_title, x_title, y_title + 1, w_title, 20, true)?;

                let check_sound_enabled = card.inheritable_checkbox_at(
                    |s| s.check_sound_enabled,
                    x_chk,
                    y_title,
                    w_chk,
                    22,
                    true,
                )?;
                check_sound_enabled.set_align_right(true);

                card.label_row(|s| s.label_switch_type, 18, 4, true)?;

                let ((x_cmb, w_cmb), (x_btn, w_btn)) = card.split_right(110, 10);
                let y_cmb_sw = card.row(28, 14);
                let combo_switch_type = card.track(ComboBox::create(
                    parent,
                    0,
                    x_cmb,
                    y_cmb_sw,
                    w_cmb,
                    180,
                    hfont_normal,
                )?);
                for p in &switch_profiles {
                    combo_switch_type.add_item(p);
                }
                if switch_profiles.is_empty() {
                    combo_switch_type.add_item(strings.sound_profile_default);
                }
                combo_switch_type.set_selected(0);

                let btn_test_sound = card.button_at(
                    |s| s.btn_test_sound,
                    x_btn,
                    y_cmb_sw,
                    w_btn,
                    28,
                    false,
                    ButtonVariant::Secondary,
                )?;

                let ((x_vol_lbl, _), (x_vol_val, w_vol_val)) = card.split_right(60, 10);
                let y_vol_lbl = card.row(18, 4);
                card.label_at(|s| s.label_volume, x_vol_lbl, y_vol_lbl, 120, 18, true)?;

                let lbl_volume_val = card.track(
                    Label::create_right(
                        parent, "50%", x_vol_val, y_vol_lbl, w_vol_val, 18, hfont_bold,
                    )?
                    .with_tone(LabelTone::Accent),
                );

                let y_slider = card.row(26, 0);
                let slider_volume = card.track(Slider::create_with_value(
                    parent,
                    0,
                    card.x(),
                    y_slider,
                    card.width(),
                    26,
                    0,
                    100,
                    50,
                )?);
                Some((
                    check_sound_enabled,
                    combo_switch_type,
                    btn_test_sound,
                    lbl_volume_val,
                    slider_volume,
                ))
            })?;

        // --- Card 2: Tải & Quản lý gói âm thanh switch ---
        let (lbl_soundpack_note, btn_download_soundpack) =
            ly.card_with_pad(spacing::SPACE_14, 20, |card| {
                card.label_row(|s| s.soundpack_card_title, 18, 4, true)?;

                let y_sp_note = card.row(18, 10);
                let lbl_soundpack_note = card.track(
                    Label::create(
                        parent,
                        if sound_available {
                            strings.soundpack_note_installed
                        } else {
                            strings.soundpack_note_missing
                        },
                        card.x(),
                        y_sp_note,
                        card.width(),
                        18,
                        hfont_normal,
                    )?
                    .with_tone(LabelTone::Secondary),
                );

                let y_sp_btn = card.row(32, 0);
                let btn_download_soundpack = card.track(PushButton::create_with_variant(
                    parent,
                    0,
                    if sound_available {
                        strings.btn_update_soundpack
                    } else {
                        strings.btn_download_soundpack
                    },
                    card.x(),
                    y_sp_btn,
                    260,
                    32,
                    hfont_normal,
                    if sound_available {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Primary
                    },
                    None,
                )?);
                Some((lbl_soundpack_note, btn_download_soundpack))
            })?;

        let tab = Self {
            layout: ly,
            check_sound_enabled,
            combo_switch_type,
            btn_test_sound,
            slider_volume,
            lbl_volume_val,
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
        profiles
            .get(self.combo_switch_type.get_selected()?)
            .cloned()
    }

    pub fn load_profile(&self, child: Option<&AppProfile<Child>>, global_config: &EngineConfig) {
        let vol = child
            .and_then(|c| c.sound_volume)
            .unwrap_or(global_config.sound_volume);
        let prof_name = child
            .and_then(|c| c.sound_profile.as_deref())
            .unwrap_or(&global_config.sound_profile);

        self.check_sound_enabled.set_inherited(
            child.is_none(),
            child.and_then(|c| c.sound_enabled),
            global_config.sound_enabled,
        );
        self.slider_volume.set_pos(vol as u32);
        self.lbl_volume_val.set_text(&format!("{}%", vol));
        self.select_switch_profile(prof_name);
        self.update_sound_controls_state();
    }

    /// Saves controls back into the active target (`Global` config or `Child` profile).
    /// Child: volume is set in `on_hscroll`, switch type only when the combo actually changes
    /// (keeps them inheriting otherwise).
    fn sync_to_target(
        &self,
        mut target: crate::ui::controller::EditTarget<'_>,
        combo_changed: bool,
    ) {
        use crate::ui::controller::EditTarget;
        target.sync_checkbox(
            &self.check_sound_enabled,
            |c| &mut c.sound_enabled,
            |p| &mut p.sound_enabled,
        );
        match target {
            EditTarget::Global(config) => {
                config.sound_volume = self.slider_volume.get_pos() as u8;
                if let Some(name) = self.selected_switch_profile() {
                    config.sound_profile = std::sync::Arc::from(name.as_str());
                }
            }
            EditTarget::Child(profile) => {
                if combo_changed && let Some(name) = self.selected_switch_profile() {
                    profile.sound_profile = Some(std::sync::Arc::from(name.as_str()));
                }
            }
        }
    }

    fn refresh_soundpack_labels(&self, strings: &LanguageStrings) {
        let (btn_txt, note_txt) = if self.sound_available.get() {
            (
                strings.btn_update_soundpack,
                strings.soundpack_note_installed,
            )
        } else {
            (
                strings.btn_download_soundpack,
                strings.soundpack_note_missing,
            )
        };
        self.btn_download_soundpack.set_text(btn_txt);
        self.lbl_soundpack_note.set_text(note_txt);
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);
        self.refresh_soundpack_labels(strings);
        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((child, cfg)) = snap {
            self.load_profile(child.as_ref(), &cfg);
        }
    }

    pub fn on_command(&self, hwnd: isize, ctl_id: u32, notify_code: u32) -> bool {
        use crate::ui::controller;
        if self.btn_test_sound.is_clicked(ctl_id, notify_code) {
            crate::platform::win32::sound::play_test_sound();
            return true;
        }

        if self.btn_download_soundpack.is_clicked(ctl_id, notify_code) {
            let hwnd_panel = hwnd;
            let strings = crate::language::current();
            self.btn_download_soundpack
                .set_text(strings.btn_downloading_soundpack);
            self.btn_download_soundpack.set_enabled(false);
            self.lbl_soundpack_note
                .set_text("Đang kết nối GitHub & tải 13 switch... ⏳");

            std::thread::spawn(move || {
                let status = Box::new(
                    crate::platform::win32::sound::installer::download_and_install_soundpack(),
                );
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

        let is_check = self.check_sound_enabled.is_toggled(ctl_id, notify_code);
        let is_combo = self.combo_switch_type.is_changed(ctl_id, notify_code);

        if is_check || is_combo {
            if is_check {
                self.update_sound_controls_state();
            }
            controller::edit_selected_then(
                |target| self.sync_to_target(target, is_combo),
                |engine| controller::reconfigure_sound(engine),
            );
            if is_combo {
                crate::platform::win32::sound::play_test_sound();
            }
            return true;
        }

        false
    }

    /// UI-thread completion of the background soundpack download & installation
    pub fn on_soundpack_installed(
        &self,
        status: &crate::platform::win32::sound::installer::SoundpackInstallStatus,
    ) {
        let strings = crate::language::current();
        if let crate::platform::win32::sound::installer::SoundpackInstallStatus::Success {
            ..
        } = status
        {
            let profiles = crate::engine::config_store::list_switch_profiles();
            self.sound_available.set(!profiles.is_empty());

            self.combo_switch_type.clear();
            for p in &profiles {
                self.combo_switch_type.add_item(p);
            }
            if profiles.is_empty() {
                self.combo_switch_type
                    .add_item(strings.sound_profile_default);
            }
            self.combo_switch_type.set_selected(0);
            *self.switch_profiles.borrow_mut() = profiles;
            self.update_sound_controls_state();

            if let Some(first_name) = self.switch_profiles.borrow().first() {
                let name_arc = std::sync::Arc::from(first_name.as_str());
                crate::ui::controller::with_engine(|engine| {
                    engine.config_mut().sound_profile = name_arc;
                    crate::ui::controller::reconfigure_sound(engine);
                });
            }
        }
        self.refresh_soundpack_labels(strings);
        self.btn_download_soundpack.set_enabled(true);
    }

    /// `WM_HSCROLL` from the volume trackbar. While dragging (`!committed`) only the
    /// label is updated; the value is persisted once the thumb is released / keyboard step.
    pub fn on_hscroll(&self, wparam: usize, slider_hwnd: isize) -> bool {
        use crate::ui::controller::{self, EditTarget};
        let Some((pos, committed)) = self.slider_volume.parse_scroll(wparam, slider_hwnd) else {
            return false;
        };
        self.lbl_volume_val.set_text(&format!("{pos}%"));
        if committed {
            let vol = pos as u8;
            controller::edit_selected_then(
                |target| match target {
                    EditTarget::Global(cfg) => cfg.sound_volume = vol,
                    EditTarget::Child(child) => child.sound_volume = Some(vol),
                },
                |engine| controller::reconfigure_sound(engine),
            );
        }
        true
    }

    pub fn paint(&self, hdc: isize, right_bound: i32) {
        self.layout.paint_cards(hdc, layout::CARD_X, right_bound);
    }

    pub fn update_visibility(&self, visible: bool) {
        self.layout.set_visible(visible);
    }
}
