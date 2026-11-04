use crate::ui::fluent::components::*;
use eframe::egui::{self};

pub struct SoundTabState {
    pub sound_enabled: bool,
    pub sound_profile: String,
    pub sound_volume: f32,
    pub available_profiles: Vec<String>,
    pub selected_idx: usize,
}

impl Default for SoundTabState {
    fn default() -> Self {
        let profiles = crate::engine::config_store::list_switch_profiles();
        Self {
            sound_enabled: false,
            sound_profile: profiles.first().cloned().unwrap_or_else(|| "default".to_string()),
            sound_volume: 50.0,
            available_profiles: profiles,
            selected_idx: 0,
        }
    }
}

impl SoundTabState {
    pub fn sync_from_engine(&mut self) {
        if let Some((enabled, profile, volume)) = crate::ui::controller::with_engine(|e| {
            (
                e.config().sound_enabled,
                e.config().sound_profile.to_string(),
                e.config().sound_volume as f32,
            )
        }) {
            self.sound_enabled = enabled;
            self.sound_profile = profile;
            self.sound_volume = volume;
            self.available_profiles = crate::engine::config_store::list_switch_profiles();

            if let Some(pos) = self.available_profiles.iter().position(|p| p == &self.sound_profile) {
                self.selected_idx = pos;
            } else {
                self.selected_idx = 0;
            }
        }
    }

    pub fn save_to_engine(&mut self) {
        if let Some(name) = self.available_profiles.get(self.selected_idx) {
            self.sound_profile = name.clone();
        }

        let profile_arc: std::sync::Arc<str> = self.sound_profile.clone().into();
        let volume_u8 = self.sound_volume.clamp(0.0, 100.0) as u8;

        crate::ui::controller::with_engine(|e| {
            e.config_mut().sound_enabled = self.sound_enabled;
            e.config_mut().sound_profile = profile_arc;
            e.config_mut().sound_volume = volume_u8;

            crate::ui::controller::reconfigure_sound(e);
            crate::ui::controller::save(e);
        });
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label(
            egui::RichText::new("Hiệu ứng Âm thanh Phím cơ")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Mô phỏng âm thanh gõ phím cơ sống động với độ trễ cực thấp (< 0.05ms) qua WASAPI")
                .size(12.0),
        );
        ui.add_space(16.0);

        section_header(ui, "Cài đặt âm thanh");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🔊",
                "Bật âm thanh gõ phím",
                "Phát âm thanh mô phỏng theo từng phím gõ",
                &mut self.sound_enabled,
            ).changed() {
                changed = true;
            }

            if self.sound_enabled {
                card_divider(ui);

                if !self.available_profiles.is_empty() {
                    let old_idx = self.selected_idx;
                    setting_combo(
                        ui,
                        "switch_profile_picker",
                        "🎛",
                        "Loại Switch bàn phím",
                        "Chọn cấu hình âm thanh switch mong muốn",
                        &self.available_profiles,
                        &mut self.selected_idx,
                    );
                    if self.selected_idx != old_idx {
                        changed = true;
                    }

                    card_divider(ui);
                }

                setting_row(
                    ui,
                    "🎚",
                    "Âm lượng hiệu ứng",
                    "Điều chỉnh mức âm lượng phát ra khi gõ",
                    |ui| {
                        if ui.add(egui::Slider::new(&mut self.sound_volume, 0.0..=100.0).text("%")).changed() {
                            changed = true;
                        }
                    },
                );

                card_divider(ui);

                setting_row(
                    ui,
                    "🎵",
                    "Nghe thử âm thanh",
                    "Phát thử một tiếng gõ phím mẫu theo âm lượng hiện tại",
                    |ui| {
                        if ui.button("▶ Nghe thử").clicked() {
                            crate::platform::win32::sound::play_test_sound();
                        }
                    },
                );
            }
        });

        changed
    }
}
