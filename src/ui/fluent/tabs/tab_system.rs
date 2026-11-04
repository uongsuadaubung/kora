use crate::engine::config::UiTheme;
use crate::language::Language;
use crate::ui::fluent::components::*;
use eframe::egui::{self};

pub struct SystemTabState {
    pub autostart: bool,
    pub show_dialog_on_startup: bool,
    pub switch_with_ctrl_shift: bool,
    pub theme_idx: usize,
    pub lang_idx: usize,
    pub debug: bool,
}

impl Default for SystemTabState {
    fn default() -> Self {
        Self {
            autostart: crate::platform::registry::is_windows_autostart_enabled(),
            show_dialog_on_startup: false,
            switch_with_ctrl_shift: true,
            theme_idx: 0,
            lang_idx: 0,
            debug: false,
        }
    }
}

impl SystemTabState {
    pub fn sync_from_engine(&mut self) {
        if let Some(cfg) = crate::ui::controller::with_engine(|e| e.config().clone()) {
            self.autostart = crate::platform::registry::is_windows_autostart_enabled();
            self.show_dialog_on_startup = cfg.show_dialog_on_startup;
            self.switch_with_ctrl_shift = cfg.switch_with_ctrl_shift;
            self.theme_idx = match cfg.theme {
                UiTheme::Auto => 0,
                UiTheme::Dark => 1,
                UiTheme::Light => 2,
            };
            self.lang_idx = match cfg.language {
                Language::Vietnamese => 0,
                Language::English => 1,
            };
            self.debug = cfg.debug;
        }
    }

    pub fn save_to_engine(&self) {
        crate::ui::controller::with_engine(|e| {
            let cfg = e.config_mut();
            cfg.show_dialog_on_startup = self.show_dialog_on_startup;
            cfg.switch_with_ctrl_shift = self.switch_with_ctrl_shift;
            cfg.theme = match self.theme_idx {
                1 => UiTheme::Dark,
                2 => UiTheme::Light,
                _ => UiTheme::Auto,
            };
            cfg.language = match self.lang_idx {
                1 => Language::English,
                _ => Language::Vietnamese,
            };
            cfg.debug = self.debug;

            crate::ui::controller::save(e);
        });
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label(
            egui::RichText::new("Cài đặt Hệ thống")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Tùy chọn khởi động, giao diện hiển thị, ngôn ngữ và nhật ký gỡ lỗi")
                .size(12.0),
        );
        ui.add_space(16.0);

        // Card Group 1: Phím chuyển & Khởi động
        section_header(ui, "Phím tắt & Khởi động");
        card_frame(ui, |ui| {
            let mut switch_idx = if self.switch_with_ctrl_shift { 0 } else { 1 };
            let old_switch = switch_idx;
            setting_combo(
                ui,
                "sys_switch_hotkey_cb",
                "⚡",
                "Phím tắt chuyển chế độ Việt / Anh",
                "Phím tắt nhanh chuyển đổi giữa tiếng Việt [V] và tiếng Anh [E]",
                &["Ctrl + Shift", "Alt + Z"],
                &mut switch_idx,
            );
            if switch_idx != old_switch {
                self.switch_with_ctrl_shift = switch_idx == 0;
                changed = true;
            }

            card_divider(ui);

            setting_row(
                ui,
                "🚀",
                "Khởi động cùng Windows",
                "Tự động chạy Kora khi bật máy tính",
                |ui| {
                    if toggle_switch(ui, &mut self.autostart).changed() {
                        crate::platform::registry::set_windows_autostart(self.autostart);
                    }
                },
            );

            card_divider(ui);

            if setting_toggle(
                ui,
                "🖥",
                "Hiện bảng điều khiển khi khởi động",
                "Tự động mở cửa sổ cài đặt này mỗi khi bật Kora",
                &mut self.show_dialog_on_startup,
            ).changed() {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Card Group 2: Giao diện & Ngôn ngữ
        section_header(ui, "Giao diện & Ngôn ngữ");
        card_frame(ui, |ui| {
            let old_theme = self.theme_idx;
            setting_combo(
                ui,
                "sys_theme_cb",
                "🎨",
                "Chế độ giao diện",
                "Lựa chọn chủ đề màu sắc hiển thị",
                &["Tự động (Theo Windows)", "Tối (Dark Mode)", "Sáng (Light Mode)"],
                &mut self.theme_idx,
            );
            if self.theme_idx != old_theme {
                changed = true;
            }

            card_divider(ui);

            let old_lang = self.lang_idx;
            setting_combo(
                ui,
                "sys_lang_cb",
                "🌐",
                "Ngôn ngữ hiển thị",
                "Ngôn ngữ của bảng điều khiển và thông báo",
                &["Tiếng Việt", "English"],
                &mut self.lang_idx,
            );
            if self.lang_idx != old_lang {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Card Group 3: Gỡ lỗi & Dữ liệu
        section_header(ui, "Gỡ lỗi & Dữ liệu cấu hình");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🐛",
                "Ghi nhận vết gỡ lỗi (Debug Log)",
                "Ghi lại chi tiết từng thao tác phím để kiểm tra lỗi",
                &mut self.debug,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            setting_row(
                ui,
                "📁",
                "Thư mục cấu hình Kora",
                "Mở thư mục chứa file cấu hình config.ini và từ điển",
                |ui| {
                    if ui.button("📂 Mở thư mục").clicked() {
                        let config_dir = crate::engine::config_store::get_config_path();
                        if let Some(parent) = config_dir.parent() {
                            let _ = std::process::Command::new("explorer")
                                .arg(parent)
                                .spawn();
                        }
                    }
                },
            );

            card_divider(ui);

            setting_row(
                ui,
                "🔄",
                "Khôi phục cài đặt gốc",
                "Đặt lại toàn bộ thiết lập về trạng thái mặc định ban đầu",
                |ui| {
                    if ui.button("⚠️ Khôi phục mặc định").clicked() {
                        let def_cfg = crate::engine::EngineConfig::default();
                        crate::ui::controller::with_engine(|e| {
                            *e.config_mut() = def_cfg;
                            crate::ui::controller::save(e);
                        });
                        self.sync_from_engine();
                        changed = true;
                    }
                },
            );
        });

        changed
    }
}
