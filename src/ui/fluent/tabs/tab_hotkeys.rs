use crate::ui::fluent::components::*;
use eframe::egui::{self};

pub struct HotkeysTabState {
    pub switch_enabled: bool,
    pub switch_with_ctrl_shift: bool,
    pub target_capture_enabled: bool,
}

impl Default for HotkeysTabState {
    fn default() -> Self {
        Self {
            switch_enabled: true,
            switch_with_ctrl_shift: true,
            target_capture_enabled: true,
        }
    }
}

impl HotkeysTabState {
    pub fn sync_from_engine(&mut self) {
        if let Some(cfg) = crate::ui::controller::with_engine(|e| e.config().clone()) {
            self.switch_enabled = cfg.switch_key_enabled;
            self.switch_with_ctrl_shift = cfg.switch_with_ctrl_shift;
            self.target_capture_enabled = cfg.target_capture_enabled;
        }
    }

    pub fn save_to_engine(&self) {
        crate::ui::controller::with_engine(|e| {
            let cfg = e.config_mut();
            cfg.switch_key_enabled = self.switch_enabled;
            cfg.switch_with_ctrl_shift = self.switch_with_ctrl_shift;
            cfg.target_capture_enabled = self.target_capture_enabled;

            crate::ui::controller::save(e);
        });
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label(
            egui::RichText::new("Phím tắt hệ thống")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Cấu hình tổ hợp phím chuyển đổi chế độ gõ và thao tác nhanh")
                .size(12.0),
        );
        ui.add_space(16.0);

        section_header(ui, "Chuyển đổi chế độ gõ");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "⚡",
                "Bật phím tắt chuyển đổi",
                "Cho phép dùng phím tắt để bật/tắt tiếng Việt nhanh",
                &mut self.switch_enabled,
            ).changed() {
                changed = true;
            }

            if self.switch_enabled {
                card_divider(ui);

                setting_row(
                    ui,
                    "🔄",
                    "Tổ hợp phím chuyển đổi",
                    "Chuyển qua lại giữa chế độ Tiếng Việt [V] và Tiếng Anh [E]",
                    |ui| {
                        let mut idx = if self.switch_with_ctrl_shift { 0 } else { 1 };
                        let options = ["Ctrl + Shift", "Alt + Z"];
                        let old_idx = idx;
                        egui::ComboBox::from_id_salt("switch_hotkey_picker")
                            .selected_text(options[idx])
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut idx, 0, options[0]);
                                ui.selectable_value(&mut idx, 1, options[1]);
                            });
                        if idx != old_idx {
                            self.switch_with_ctrl_shift = idx == 0;
                            changed = true;
                        }
                    },
                );
            }
        });

        ui.add_space(16.0);

        section_header(ui, "Nhận diện ô văn bản");
        card_frame(ui, |ui| {
            setting_row(
                ui,
                "🎯",
                "Phím tắt kích hoạt Target Capture",
                "Tổ hợp phím mặc định: Ctrl + ` (dấu huyền)",
                |ui| {
                    ui.label(
                        egui::RichText::new("Ctrl + `")
                            .monospace()
                            .strong(),
                    );
                },
            );
        });

        changed
    }
}
