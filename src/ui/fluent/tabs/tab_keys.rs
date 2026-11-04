use crate::ui::fluent::components::*;
use eframe::egui::{self};

pub struct KeysTabState {
    pub remap_enabled: bool,
    pub disable_enabled: bool,
    pub key_macro_enabled: bool,
}

impl Default for KeysTabState {
    fn default() -> Self {
        Self {
            remap_enabled: false,
            disable_enabled: false,
            key_macro_enabled: false,
        }
    }
}

impl KeysTabState {
    pub fn sync_from_engine(&mut self) {
        if let Some(cfg) = crate::ui::controller::with_engine(|e| e.config().clone()) {
            self.remap_enabled = cfg.remap_enabled;
            self.disable_enabled = cfg.disable_enabled;
            self.key_macro_enabled = cfg.key_macro_enabled;
        }
    }

    pub fn save_to_engine(&self) {
        crate::ui::controller::with_engine(|e| {
            let cfg = e.config_mut();
            cfg.remap_enabled = self.remap_enabled;
            cfg.disable_enabled = self.disable_enabled;
            cfg.key_macro_enabled = self.key_macro_enabled;

            crate::ui::controller::save(e);
        });
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label(
            egui::RichText::new("Quản lý & Tùy biến Phím")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Đổi chức năng phím, khóa các phím dễ bấm nhầm và thiết lập phím macro tự động")
                .size(12.0),
        );
        ui.add_space(16.0);

        // Section 1: Đổi phím
        section_header(ui, "Đổi chức năng phím (Key Remapping)");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🔀",
                "Bật đổi phím",
                "Gán một phím bấm vật lý thành chức năng của phím khác (ví dụ: CapsLock ➔ Ctrl)",
                &mut self.remap_enabled,
            ).changed() {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Section 2: Khóa phím
        section_header(ui, "Khóa phím (Disable Keys)");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🔒",
                "Bật khóa phím",
                "Chặn tín hiệu của các phím dễ bấm nhầm (ví dụ: phím Windows khi chơi game, phím Insert)",
                &mut self.disable_enabled,
            ).changed() {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Section 3: Phím Macro
        section_header(ui, "Phím Macro (Key Macro)");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🤖",
                "Bật phím Macro",
                "Thực thi một chuỗi phím tự động chỉ với một lần nhấn",
                &mut self.key_macro_enabled,
            ).changed() {
                changed = true;
            }
        });

        changed
    }
}
