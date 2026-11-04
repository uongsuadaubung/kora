use crate::engine::macro_table::MacroEntry;
use crate::ui::fluent::components::*;
use eframe::egui::{self, Color32, Stroke};

pub struct MacroTabState {
    pub use_macro: bool,
    pub use_macro_en: bool,
    pub new_key: String,
    pub new_value: String,
    pub search_query: String,
    pub entries: Vec<MacroEntry>,
}

impl Default for MacroTabState {
    fn default() -> Self {
        Self {
            use_macro: false,
            use_macro_en: true,
            new_key: String::new(),
            new_value: String::new(),
            search_query: String::new(),
            entries: Vec::new(),
        }
    }
}

impl MacroTabState {
    pub fn sync_from_engine(&mut self) {
        if let Some((use_macro, use_macro_en, entries)) = crate::ui::controller::with_engine(|e| {
            (
                e.config().use_macro,
                e.config().use_macro_in_english_mode,
                e.macro_table.get_sorted_entries(),
            )
        }) {
            self.use_macro = use_macro;
            self.use_macro_en = use_macro_en;
            self.entries = entries;
        }
    }

    pub fn save_to_engine(&self) {
        crate::ui::controller::with_engine(|e| {
            e.config_mut().use_macro = self.use_macro;
            e.config_mut().use_macro_in_english_mode = self.use_macro_en;
            crate::ui::controller::save(e);
        });
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label(
            egui::RichText::new("Quản lý Gõ tắt (Macro)")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Tự động thay thế từ viết tắt thành văn bản đầy đủ theo danh sách định sẵn")
                .size(12.0),
        );
        ui.add_space(16.0);

        // Card Group 1: Thiết lập gõ tắt
        section_header(ui, "Thiết lập chung");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "⚡",
                "Bật tính năng gõ tắt",
                "Kích hoạt cơ chế tự động mở rộng từ viết tắt",
                &mut self.use_macro,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            if setting_toggle(
                ui,
                "🌐",
                "Gõ tắt trong chế độ tiếng Anh",
                "Cho phép gõ tắt ngay cả khi đang gõ tiếng Anh [E]",
                &mut self.use_macro_en,
            ).changed() {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Card Group 2: Thêm từ gõ tắt mới
        section_header(ui, "Thêm từ gõ tắt mới");
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Từ viết tắt").size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_key)
                            .hint_text("ví dụ: vn")
                            .desired_width(120.0),
                    );
                });

                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Từ thay thế").size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_value)
                            .hint_text("ví dụ: Việt Nam")
                            .desired_width(280.0),
                    );
                });

                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.add_space(18.0);
                    let can_add = !self.new_key.trim().is_empty() && !self.new_value.trim().is_empty();
                    if ui.add_enabled(can_add, egui::Button::new("➕ Thêm vào bảng")).clicked() {
                        let key = self.new_key.trim().to_lowercase();
                        let val = self.new_value.trim().to_string();

                        crate::ui::controller::with_engine(|e| {
                            e.macro_table.insert(key.clone(), val.clone());
                            crate::ui::controller::save(e);
                        });

                        self.new_key.clear();
                        self.new_value.clear();
                        self.sync_from_engine();
                    }
                });
            });
        });

        ui.add_space(16.0);

        // Card Group 3: Danh sách từ gõ tắt
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Danh sách từ viết tắt ({})", self.entries.len()))
                    .size(14.0)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("🔍 Tìm kiếm...")
                        .desired_width(160.0),
                );
            });
        });
        ui.add_space(6.0);

        let query = self.search_query.trim().to_lowercase();
        let filtered_entries: Vec<MacroEntry> = self
            .entries
            .iter()
            .filter(|e| query.is_empty() || e.key.contains(&query) || e.value.to_lowercase().contains(&query))
            .cloned()
            .collect();

        card_frame(ui, |ui| {
            if filtered_entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("Chưa có từ viết tắt nào")
                            .color(Color32::from_rgb(140, 140, 140)),
                    );
                    ui.add_space(16.0);
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete: Option<String> = None;

                        for (idx, entry) in filtered_entries.iter().enumerate() {
                            if idx > 0 {
                                card_divider(ui);
                            }

                            ui.horizontal(|ui| {
                                // Key pill
                                let pill_bg = Color32::from_rgb(48, 48, 48);
                                egui::Frame::new()
                                    .fill(pill_bg)
                                    .corner_radius(4.0)
                                    .inner_margin(egui::Margin::symmetric(8, 3))
                                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(15)))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(&entry.key)
                                                .monospace()
                                                .strong()
                                                .color(Color32::from_rgb(96, 205, 255)),
                                        );
                                    });

                                ui.label("➔");

                                ui.label(
                                    egui::RichText::new(&entry.value)
                                        .size(13.0),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑 Xoá").clicked() {
                                        to_delete = Some(entry.key.clone());
                                    }
                                });
                            });
                        }

                        if let Some(key) = to_delete {
                            crate::ui::controller::with_engine(|e| {
                                e.macro_table.remove(&key);
                                crate::ui::controller::save(e);
                            });
                            self.sync_from_engine();
                        }
                    });
            }
        });

        changed
    }
}
