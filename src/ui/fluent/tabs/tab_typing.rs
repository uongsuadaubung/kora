use crate::engine::config::InputMethod;
use crate::ui::fluent::components::*;
use crate::vietnamese::Charset;
use eframe::egui::{self};

pub struct TypingTabState {
    pub enabled: bool,
    pub method: usize,
    pub charset: usize,
    pub auto_upper: bool,
    pub restore_wrong: bool,
    pub bracket_w: bool,
    pub use_macro: bool,
    pub use_macro_en: bool,
    pub remember_space: bool,
    pub target_capture: bool,
}

impl Default for TypingTabState {
    fn default() -> Self {
        Self {
            enabled: true,
            method: 0,
            charset: 0,
            auto_upper: false,
            restore_wrong: true,
            bracket_w: true,
            use_macro: false,
            use_macro_en: true,
            remember_space: true,
            target_capture: true,
        }
    }
}

impl TypingTabState {
    pub fn sync_from_engine(&mut self) {
        if let Some(cfg) = crate::ui::controller::with_engine(|e| {
            let sel = &e.profile_manager.selected_id;
            e.profile_manager.resolve_effective_config(sel, e.config())
        }) {
            self.enabled = cfg.enabled;
            self.method = match cfg.method {
                InputMethod::Telex => 0,
                InputMethod::Vni => 1,
                InputMethod::SimpleTelex1 => 2,
                InputMethod::SimpleTelex2 => 3,
            };
            self.charset = match cfg.charset {
                Charset::Unicode => 0,
                Charset::Tcvn3 => 1,
            };
            self.auto_upper = cfg.auto_uppercase_first_char;
            self.restore_wrong = cfg.restore_on_wrong_spelling;
            self.bracket_w = cfg.bracket_w;
            self.use_macro = cfg.use_macro;
            self.use_macro_en = cfg.use_macro_in_english_mode;
            self.remember_space = cfg.remember_history_across_space;
            self.target_capture = cfg.target_capture_enabled;
        }
    }

    pub fn save_to_engine(&self) {
        crate::ui::controller::with_engine(|e| {
            let sel = e.profile_manager.selected_id.clone();
            if sel == "global" {
                let cfg = e.config_mut();
                cfg.enabled = self.enabled;
                cfg.method = match self.method {
                    0 => InputMethod::Telex,
                    1 => InputMethod::Vni,
                    2 => InputMethod::SimpleTelex1,
                    _ => InputMethod::SimpleTelex2,
                };
                cfg.charset = match self.charset {
                    1 => Charset::Tcvn3,
                    _ => Charset::Unicode,
                };
                cfg.auto_uppercase_first_char = self.auto_upper;
                cfg.restore_on_wrong_spelling = self.restore_wrong;
                cfg.bracket_w = self.bracket_w;
                cfg.use_macro = self.use_macro;
                cfg.use_macro_in_english_mode = self.use_macro_en;
                cfg.remember_history_across_space = self.remember_space;
                cfg.target_capture_enabled = self.target_capture;
            } else if let Some(child) = e.profile_manager.get_profile_mut(&sel) {
                child.mode = if self.enabled {
                    crate::engine::profile::LanguageMode::Vietnamese
                } else {
                    crate::engine::profile::LanguageMode::English
                };
                child.method = match self.method {
                    0 => crate::engine::profile::MethodMode::Specific(InputMethod::Telex),
                    1 => crate::engine::profile::MethodMode::Specific(InputMethod::Vni),
                    2 => crate::engine::profile::MethodMode::Specific(InputMethod::SimpleTelex1),
                    _ => crate::engine::profile::MethodMode::Specific(InputMethod::SimpleTelex2),
                };
                child.charset = match self.charset {
                    1 => crate::engine::profile::CharsetMode::Specific(Charset::Tcvn3),
                    _ => crate::engine::profile::CharsetMode::Specific(Charset::Unicode),
                };
                child.auto_uppercase_first_char = Some(self.auto_upper);
                child.restore_on_wrong_spelling = Some(self.restore_wrong);
                child.bracket_w = Some(self.bracket_w);
                child.use_macro = Some(self.use_macro);
                child.use_macro_in_english_mode = Some(self.use_macro_en);
                child.remember_history_across_space = Some(self.remember_space);
            }

            crate::ui::controller::save(e);
        });
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label(
            egui::RichText::new("Cài đặt Bộ gõ")
                .size(22.0)
                .strong(),
        );
        ui.label(
            egui::RichText::new("Tùy chỉnh phương thức gõ tiếng Việt, bảng mã và các tiện ích xử lý từ ngữ")
                .size(12.0),
        );
        ui.add_space(16.0);

        // Card Group 1: Thiết lập cốt lõi
        section_header(ui, "Thiết lập cốt lõi");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🇻🇳",
                "Bật gõ tiếng Việt",
                "Bật hoặc tắt bộ gõ tiếng Việt cho toàn hệ thống",
                &mut self.enabled,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            let old_method = self.method;
            setting_combo(
                ui,
                "typing_method_cb",
                "⌨",
                "Kiểu gõ chính",
                "Chọn phương thức gõ phù hợp với bạn",
                &["Telex", "VNI", "Simple Telex 1", "Simple Telex 2"],
                &mut self.method,
            );
            if self.method != old_method {
                changed = true;
            }

            card_divider(ui);

            let old_charset = self.charset;
            setting_combo(
                ui,
                "typing_charset_cb",
                "🔤",
                "Bảng mã",
                "Bảng mã ký tự văn bản",
                &["Unicode (Dựng sẵn)", "TCVN3 (ABC)"],
                &mut self.charset,
            );
            if self.charset != old_charset {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Card Group 2: Tính năng thông minh
        section_header(ui, "Tính năng thông minh");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🔠",
                "Tự động viết hoa đầu câu",
                "Viết hoa chữ cái đầu tiên sau dấu kết thúc câu (. ! ?)",
                &mut self.auto_upper,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            if setting_toggle(
                ui,
                "🔄",
                "Tự động phục hồi từ gõ sai",
                "Nhấn lặp lại phím bỏ dấu để huỷ dấu sai",
                &mut self.restore_wrong,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            if setting_toggle(
                ui,
                "📐",
                "Phím tắt ngoặc vuông: [ ➔ ư, ] ➔ ơ",
                "Gõ nhanh ký tự ư và ơ bằng phím ngoặc vuông",
                &mut self.bracket_w,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            if setting_toggle(
                ui,
                "⚡",
                "Gõ tắt (Macro)",
                "Bật tính năng tự động thay thế từ viết tắt",
                &mut self.use_macro,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            if setting_toggle(
                ui,
                "🌐",
                "Gõ tắt cả khi ở chế độ tiếng Anh",
                "Cho phép thay thế từ viết tắt ngay cả khi bộ gõ đang ở chế độ [E]",
                &mut self.use_macro_en,
            ).changed() {
                changed = true;
            }

            card_divider(ui);

            if setting_toggle(
                ui,
                "🔙",
                "Khôi phục từ khi bấm Backspace xoá khoảng trắng",
                "Cho phép chỉnh sửa tiếp từ vừa gõ sau khi bấm xoá dấu cách",
                &mut self.remember_space,
            ).changed() {
                changed = true;
            }
        });

        ui.add_space(16.0);

        // Card Group 3: Tương thích ứng dụng
        section_header(ui, "Tương thích ứng dụng");
        card_frame(ui, |ui| {
            if setting_toggle(
                ui,
                "🎯",
                "Nhận diện ô nhập liệu thông minh (Target Capture)",
                "Tự động phát hiện ô gõ mật khẩu, ô tìm kiếm hoặc Terminal để tối ưu hóa thao tác phím",
                &mut self.target_capture,
            ).changed() {
                changed = true;
            }
        });

        changed
    }
}
