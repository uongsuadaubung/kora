use crate::ui::fluent::components::*;
use eframe::egui::{self, Color32};

pub struct AboutTabState {
    pub checking_update: bool,
    pub update_status: Option<String>,
}

impl Default for AboutTabState {
    fn default() -> Self {
        Self {
            checking_update: false,
            update_status: None,
        }
    }
}

impl AboutTabState {
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label(
            egui::RichText::new("Thông tin về Kora")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Bộ gõ tiếng Việt thế hệ mới – Siêu nhẹ, siêu nhanh và an toàn tuyệt đối")
                .size(12.0),
        );
        ui.add_space(16.0);

        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⌨").size(36.0).color(Color32::from_rgb(0, 120, 212)));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Kora - Vietnamese Keyboard Suite")
                            .size(16.0)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("Phiên bản v0.11.0 – Windows 11 Fluent Edition")
                            .size(12.0)
                            .color(Color32::from_rgb(160, 160, 160)),
                    );
                });
            });

            card_divider(ui);

            let status_msg = self.update_status.as_deref().unwrap_or("Phiên bản bạn đang sử dụng là phiên bản mới nhất").to_string();
            setting_row(
                ui,
                "✨",
                "Trạng thái cập nhật",
                &status_msg,
                |ui| {
                    if ui.button("🔄 Kiểm tra cập nhật").clicked() {
                        self.update_status = Some("Bạn đang sử dụng phiên bản mới nhất!".to_string());
                    }
                },
            );

            card_divider(ui);

            setting_row(
                ui,
                "🌐",
                "Mã nguồn mở trên GitHub",
                "Đóng góp tính năng, báo cáo lỗi hoặc xem mã nguồn dự án",
                |ui| {
                    if ui.button("Mở GitHub").clicked() {
                        let _ = std::process::Command::new("cmd")
                            .args(["/C", "start", "https://github.com/uongsuadaubung/kora"])
                            .spawn();
                    }
                },
            );

            card_divider(ui);

            setting_row(
                ui,
                "⚖️",
                "Giấy phép phát hành",
                "Phần mềm nguồn mở miễn phí phân phối theo giấy phép MIT License",
                |_ui| {},
            );
        });

        false
    }
}
