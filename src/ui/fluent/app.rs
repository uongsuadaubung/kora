use super::fonts::setup_fonts;
use super::tabs::tab_about::AboutTabState;
use super::tabs::tab_keys::KeysTabState;
use super::tabs::tab_macro::MacroTabState;
use super::tabs::tab_sound::SoundTabState;
use super::tabs::tab_system::SystemTabState;
use super::tabs::tab_typing::TypingTabState;
use super::theme::{apply_theme, FluentTheme};
use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Profile(usize), // 0: Bộ gõ, 1: Quản lý phím, 2: Âm thanh, 3: Gõ tắt
    SystemSettings,
    About,
}

pub struct KoraFluentApp {
    pub theme: FluentTheme,
    pub view_mode: ViewMode,
    pub selected_profile_id: String,

    // Modal dialogs
    pub show_add_profile_dialog: bool,
    pub new_profile_name: String,
    pub new_profile_process: String,
    pub new_profile_icon: String,

    pub show_delete_confirm_dialog: bool,

    // Tabs
    pub tab_typing: TypingTabState,
    pub tab_keys: KeysTabState,
    pub tab_sound: SoundTabState,
    pub tab_macro: MacroTabState,
    pub tab_system: SystemTabState,
    pub tab_about: AboutTabState,
}

impl Default for KoraFluentApp {
    fn default() -> Self {
        let mut app = Self {
            theme: FluentTheme::dark(),
            view_mode: ViewMode::Profile(0),
            selected_profile_id: "global".to_string(),

            show_add_profile_dialog: false,
            new_profile_name: String::new(),
            new_profile_process: String::new(),
            new_profile_icon: "💻".to_string(),

            show_delete_confirm_dialog: false,

            tab_typing: TypingTabState::default(),
            tab_keys: KeysTabState::default(),
            tab_sound: SoundTabState::default(),
            tab_macro: MacroTabState::default(),
            tab_system: SystemTabState::default(),
            tab_about: AboutTabState::default(),
        };

        app.sync_all();
        app
    }
}

impl KoraFluentApp {
    pub fn sync_all(&mut self) {
        crate::ui::controller::with_engine(|engine| {
            engine.profile_manager.selected_id = self.selected_profile_id.clone();
        });
        self.tab_typing.sync_from_engine();
        self.tab_keys.sync_from_engine();
        self.tab_sound.sync_from_engine();
        self.tab_macro.sync_from_engine();
        self.tab_system.sync_from_engine();
    }
}

impl eframe::App for KoraFluentApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Windows 11 Left Sidebar: Multi-Profile Workspace Hub (220px)
        egui::Panel::left("kora_fluent_sidebar")
            .exact_size(220.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(self.theme.bg_sidebar)
                    .inner_margin(egui::Margin::symmetric(10, 14)),
            )
            .show(ui, |ui| {
                // Sidebar Header: HỒ SƠ ỨNG DỤNG + Button "+"
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("HỒ SƠ ỨNG DỤNG")
                            .size(11.0)
                            .strong()
                            .color(self.theme.text_secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new("+").size(15.0).strong())
                            .on_hover_text("Thêm hồ sơ ứng dụng mới")
                            .clicked()
                        {
                            self.new_profile_name.clear();
                            self.new_profile_process.clear();
                            self.new_profile_icon = "💻".to_string();
                            self.show_add_profile_dialog = true;
                        }
                    });
                });

                ui.add_space(8.0);

                // Fetch all profiles from Engine
                let profiles_info: Vec<(String, String, String, String)> =
                    crate::ui::controller::with_engine(|e| {
                        e.profile_manager
                            .all_profiles()
                            .iter()
                            .map(|p| match p {
                                crate::engine::profile::ProfileRef::Root(r) => (
                                    "global".to_string(),
                                    if r.icon.is_empty() {
                                        "🌐".to_string()
                                    } else {
                                        r.icon.clone()
                                    },
                                    "Mặc định".to_string(),
                                    "Toàn bộ hệ thống".to_string(),
                                ),
                                crate::engine::profile::ProfileRef::Child(c) => (
                                    c.id.clone(),
                                    if c.icon.is_empty() {
                                        "💻".to_string()
                                    } else {
                                        c.icon.clone()
                                    },
                                    c.name.clone(),
                                    if c.process_names.is_empty() {
                                        "Chưa gán ứng dụng".to_string()
                                    } else {
                                        c.process_names.join(", ")
                                    },
                                ),
                            })
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        vec![(
                            "global".to_string(),
                            "🌐".to_string(),
                            "Mặc định".to_string(),
                            "Toàn bộ hệ thống".to_string(),
                        )]
                    });

                let current_subtab = match self.view_mode {
                    ViewMode::Profile(t) => t,
                    _ => 0,
                };

                // Bottom links take ~86px (separator + 2 links of 32px + spacing)
                let bottom_height = 86.0;
                let available_for_scroll = (ui.available_height() - bottom_height).max(60.0);

                // Scrollable Profiles List
                egui::ScrollArea::vertical()
                    .max_height(available_for_scroll)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (id, icon, name, desc) in profiles_info {
                            let is_selected = matches!(self.view_mode, ViewMode::Profile(_))
                                && self.selected_profile_id == id;

                            let desired_size = Vec2::new(ui.available_width(), 46.0);
                            let (rect, response) =
                                ui.allocate_exact_size(desired_size, egui::Sense::click());
                            if response.clicked() {
                                self.selected_profile_id = id.clone();
                                self.view_mode = ViewMode::Profile(current_subtab);
                                self.sync_all();
                            }

                            let hover = response.hovered();
                            let bg_color = if is_selected {
                                self.theme.bg_card
                            } else if hover {
                                Color32::from_white_alpha(10)
                            } else {
                                Color32::TRANSPARENT
                            };

                            ui.painter()
                                .rect_filled(rect, CornerRadius::same(6), bg_color);
                            if is_selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    CornerRadius::same(6),
                                    Stroke::new(1.0, self.theme.stroke_card),
                                    egui::StrokeKind::Inside,
                                );
                                // Left active indicator pill
                                let pill = Rect::from_min_size(
                                    Pos2::new(rect.left() + 2.0, rect.top() + 13.0),
                                    Vec2::new(3.0, 20.0),
                                );
                                ui.painter().rect_filled(
                                    pill,
                                    CornerRadius::same(2),
                                    self.theme.accent,
                                );
                            }

                            // Profile Icon
                            ui.painter().text(
                                Pos2::new(rect.left() + 12.0, rect.center().y - 10.0),
                                egui::Align2::LEFT_TOP,
                                &icon,
                                egui::FontId::proportional(16.0),
                                self.theme.text_primary,
                            );

                            // Profile Name
                            let text_color = if is_selected {
                                self.theme.accent
                            } else {
                                self.theme.text_primary
                            };
                            ui.painter().text(
                                Pos2::new(rect.left() + 38.0, rect.top() + 6.0),
                                egui::Align2::LEFT_TOP,
                                &name,
                                egui::FontId::proportional(13.0),
                                text_color,
                            );

                            // Process / Description
                            ui.painter().text(
                                Pos2::new(rect.left() + 38.0, rect.top() + 24.0),
                                egui::Align2::LEFT_TOP,
                                &desc,
                                egui::FontId::proportional(11.0),
                                self.theme.text_secondary,
                            );

                            ui.add_space(2.0);
                        }
                    });

                // Bottom Sidebar Area (Anchored to bottom without overflow)
                let remaining_space = (ui.available_height() - bottom_height).max(0.0);
                if remaining_space > 0.0 {
                    ui.add_space(remaining_space);
                }
                ui.separator();
                ui.add_space(4.0);

                // Bottom Link: Cài đặt hệ thống
                let is_settings = self.view_mode == ViewMode::SystemSettings;
                if render_sidebar_link(ui, &self.theme, "⚙", "Cài đặt hệ thống", is_settings) {
                    self.view_mode = ViewMode::SystemSettings;
                }
                ui.add_space(2.0);

                // Bottom Link: Thông tin
                let is_about = self.view_mode == ViewMode::About;
                if render_sidebar_link(ui, &self.theme, "ℹ", "Thông tin", is_about) {
                    self.view_mode = ViewMode::About;
                }
            });

        // Bottom Footer Panel (Pinned to bottom of the remaining right area)
        egui::Panel::bottom("kora_fluent_footer")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.bg_app)
                    .inner_margin(egui::Margin {
                        left: 24,
                        right: 24,
                        top: 8,
                        bottom: 14,
                    }),
            )
            .show(ui, |ui| {
                ui.separator();
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("⏻ Thoát Kora")
                                    .color(Color32::from_rgb(255, 90, 90)),
                            )
                            .min_size(Vec2::new(96.0, 28.0)),
                        )
                        .clicked()
                    {
                        crate::engine::config_store::flush_config_debounced();
                        unsafe {
                            crate::ui::win32::PostQuitMessage(0);
                        }
                        std::process::exit(0);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Đóng").color(Color32::WHITE),
                                )
                                .min_size(Vec2::new(80.0, 28.0)),
                            )
                            .clicked()
                        {
                            IS_OPEN.store(false, std::sync::atomic::Ordering::SeqCst);
                            crate::ui::hide_control_panel();
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            });

        // Right Workspace Panel
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.bg_app)
                    .inner_margin(egui::Margin {
                        left: 24,
                        right: 24,
                        top: 16,
                        bottom: 4,
                    }),
            )
            .show(ui, |ui| match self.view_mode {
                ViewMode::Profile(subtab) => {
                    // 1. Profile Banner Header
                    let (p_icon, p_name, p_desc, is_child) =
                        crate::ui::controller::with_engine(|e| {
                            let sel = &e.profile_manager.selected_id;
                            if sel == "global" {
                                (
                                    "🌐".to_string(),
                                    "Mặc định".to_string(),
                                    "Hồ sơ gốc áp dụng cho tất cả ứng dụng".to_string(),
                                    false,
                                )
                            } else if let Some(c) = e.profile_manager.get_profile(sel) {
                                let proc_info = if c.process_names.is_empty() {
                                    "Chưa gán ứng dụng".to_string()
                                } else {
                                    format!("Đang áp dụng: {}", c.process_names.join(", "))
                                };
                                (c.icon.clone(), c.name.clone(), proc_info, true)
                            } else {
                                (
                                    "🌐".to_string(),
                                    "Mặc định".to_string(),
                                    "".to_string(),
                                    false,
                                )
                            }
                        })
                        .unwrap_or_else(|| {
                            (
                                "🌐".to_string(),
                                "Mặc định".to_string(),
                                "".to_string(),
                                false,
                            )
                        });

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&p_icon).size(24.0));
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(&p_name)
                                    .size(17.0)
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            ui.label(
                                egui::RichText::new(&p_desc)
                                    .size(12.0)
                                    .color(self.theme.text_secondary),
                            );
                        });

                        if is_child {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(
                                            egui::RichText::new("🗑 Xóa hồ sơ")
                                                .color(Color32::from_rgb(255, 90, 90)),
                                        )
                                        .clicked()
                                    {
                                        self.show_delete_confirm_dialog = true;
                                    }
                                },
                            );
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 2. Subtab Bar (4 tabs: Bộ gõ, Quản lý phím, Âm thanh, Gõ tắt)
                    let subtabs = [
                        ("⌨", "Bộ gõ"),
                        ("🔀", "Quản lý phím"),
                        ("🔊", "Âm thanh"),
                        ("📝", "Gõ tắt"),
                    ];

                    ui.horizontal(|ui| {
                        for (i, (icon, label)) in subtabs.iter().enumerate() {
                            let is_tab_active = subtab == i;
                            let text_color = if is_tab_active {
                                self.theme.accent
                            } else {
                                self.theme.text_secondary
                            };
                            let bg = if is_tab_active {
                                self.theme.bg_card
                            } else {
                                Color32::TRANSPARENT
                            };

                            let btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} {}", icon, label))
                                        .size(13.0)
                                        .color(text_color)
                                        .strong(),
                                )
                                .fill(bg)
                                .corner_radius(6.0),
                            );
                            if btn.clicked() {
                                self.view_mode = ViewMode::Profile(i);
                            }
                        }
                    });

                    ui.add_space(10.0);

                    // 3. Subtab Content Area
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| match subtab {
                            0 => {
                                if self.tab_typing.render(ui) {
                                    self.tab_typing.save_to_engine();
                                }
                            }
                            1 => {
                                if self.tab_keys.render(ui) {
                                    self.tab_keys.save_to_engine();
                                }
                            }
                            2 => {
                                if self.tab_sound.render(ui) {
                                    self.tab_sound.save_to_engine();
                                }
                            }
                            _ => {
                                if self.tab_macro.render(ui) {
                                    self.tab_macro.save_to_engine();
                                }
                            }
                        });
                }
                ViewMode::SystemSettings => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.tab_system.render(ui) {
                                self.tab_system.save_to_engine();
                            }
                        });
                }
                ViewMode::About => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            self.tab_about.render(ui);
                        });
                }
            });

        // Modal: Thêm hồ sơ mới
        if self.show_add_profile_dialog {
            egui::Window::new("Thêm hồ sơ ứng dụng mới")
                .collapsible(false)
                .resizable(false)
                .fixed_size([340.0, 240.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label("Tên hồ sơ:");
                    ui.text_edit_singleline(&mut self.new_profile_name);

                    ui.add_space(6.0);
                    ui.label("Tên tiến trình ứng dụng (.exe):");
                    ui.text_edit_singleline(&mut self.new_profile_process);

                    ui.add_space(6.0);
                    ui.label("Biểu tượng:");
                    let quick_icons = ["💻", "🎮", "📝", "🌐", "🎨", "⚡", "🛠️", "🎯", "🎵"];
                    ui.horizontal_wrapped(|ui| {
                        for icon in quick_icons {
                            let is_sel = self.new_profile_icon == icon;
                            if ui.selectable_label(is_sel, icon).clicked() {
                                self.new_profile_icon = icon.to_string();
                            }
                        }
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Hủy").clicked() {
                            self.show_add_profile_dialog = false;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can_add = !self.new_profile_name.trim().is_empty()
                                && !self.new_profile_process.trim().is_empty();
                            if ui
                                .add_enabled(can_add, egui::Button::new("Thêm hồ sơ"))
                                .clicked()
                            {
                                let proc = self.new_profile_process.trim().to_string();
                                let base_id = proc
                                    .trim_end_matches(".exe")
                                    .replace([' ', '-', '.'], "_")
                                    .to_lowercase();
                                let mut id = if base_id.is_empty() {
                                    "app".to_string()
                                } else {
                                    base_id.clone()
                                };
                                let mut counter = 1;

                                crate::ui::controller::with_engine(|engine| {
                                    while id == "global"
                                        || engine.profile_manager.get_profile(&id).is_some()
                                    {
                                        id = format!("{}_{}", base_id, counter);
                                        counter += 1;
                                    }
                                    let new_prof = crate::engine::profile::AppProfile::new_child(
                                        id.clone(),
                                        self.new_profile_name.trim().to_string(),
                                        proc,
                                        self.new_profile_icon.as_str(),
                                    );
                                    engine.profile_manager.add_profile(new_prof);
                                    engine.profile_manager.selected_id = id.clone();
                                    crate::ui::controller::save(engine);
                                });

                                self.selected_profile_id = id;
                                self.sync_all();
                                self.show_add_profile_dialog = false;
                            }
                        });
                    });
                });
        }

        // Modal: Xác nhận xóa hồ sơ
        if self.show_delete_confirm_dialog {
            egui::Window::new("Xác nhận xóa hồ sơ")
                .collapsible(false)
                .resizable(false)
                .fixed_size([300.0, 140.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.add_space(8.0);
                    ui.label("Bạn có chắc chắn muốn xóa hồ sơ này không?");
                    ui.label("Toàn bộ cấu hình tùy biến của hồ sơ sẽ bị xóa.");

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Hủy").clicked() {
                            self.show_delete_confirm_dialog = false;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(
                                    egui::RichText::new("Xóa")
                                        .color(Color32::from_rgb(255, 90, 90)),
                                )
                                .clicked()
                            {
                                if self.selected_profile_id != "global" {
                                    crate::ui::controller::with_engine(|engine| {
                                        engine
                                            .profile_manager
                                            .remove_profile(&self.selected_profile_id);
                                        engine.profile_manager.selected_id = "global".to_string();
                                        crate::ui::controller::save(engine);
                                    });
                                    self.selected_profile_id = "global".to_string();
                                    self.sync_all();
                                }
                                self.show_delete_confirm_dialog = false;
                            }
                        });
                    });
                });
        }
    }
}

fn render_sidebar_link(
    ui: &mut egui::Ui,
    theme: &FluentTheme,
    icon: &str,
    label: &str,
    is_active: bool,
) -> bool {
    let desired_size = Vec2::new(ui.available_width(), 32.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let hover = response.hovered();

    let bg_color = if is_active {
        theme.bg_card
    } else if hover {
        Color32::from_white_alpha(10)
    } else {
        Color32::TRANSPARENT
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same(6), bg_color);
    if is_active {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.0, theme.stroke_card),
            egui::StrokeKind::Inside,
        );
        let pill = Rect::from_min_size(
            Pos2::new(rect.left() + 2.0, rect.top() + 8.0),
            Vec2::new(3.0, 16.0),
        );
        ui.painter()
            .rect_filled(pill, CornerRadius::same(2), theme.accent);
    }

    let text_color = if is_active {
        theme.accent
    } else {
        theme.text_secondary
    };
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.center().y - 8.0),
        egui::Align2::LEFT_TOP,
        icon,
        egui::FontId::proportional(14.0),
        text_color,
    );

    ui.painter().text(
        Pos2::new(rect.left() + 34.0, rect.center().y - 8.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    response.clicked()
}

static IS_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Spawns or runs the Windows 11 Fluent Control Panel in a dedicated UI thread
pub fn show_fluent_control_panel() {
    if IS_OPEN.swap(true, std::sync::atomic::Ordering::SeqCst) {
        unsafe {
            let title = crate::ui::win32::to_wide("Kora - Bảng điều khiển");
            let hwnd = crate::ui::win32::FindWindowW(std::ptr::null(), title.as_ptr());
            if hwnd != 0 {
                crate::ui::win32::ShowWindow(hwnd, 9);
                crate::ui::win32::SetForegroundWindow(hwnd);
            }
        }
        return;
    }

    std::thread::spawn(move || {
        unsafe {
            type SetThreadDpiAwarenessContextFn = unsafe extern "system" fn(isize) -> isize;
            let user32_name = crate::ui::win32::to_wide("user32.dll");
            let h_mod = crate::ui::win32::GetModuleHandleW(user32_name.as_ptr());
            if h_mod != 0 {
                let proc = crate::ui::win32::GetProcAddress(
                    h_mod,
                    b"SetThreadDpiAwarenessContext\0".as_ptr(),
                );
                if !proc.is_null() {
                    let set_dpi: SetThreadDpiAwarenessContextFn = std::mem::transmute(proc);
                    set_dpi(-4); // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
                }
            }
        }

        use winit::platform::windows::EventLoopBuilderExtWindows;

        let mut options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([820.0, 590.0])
                .with_min_inner_size([720.0, 520.0])
                .with_title("Kora - Bảng điều khiển"),
            ..Default::default()
        };

        options.event_loop_builder = Some(Box::new(|builder| {
            builder.with_any_thread(true);
            builder.with_dpi_aware(true);
        }));

        let _ = eframe::run_native(
            "KoraFluentSettings",
            options,
            Box::new(|cc| {
                let app = KoraFluentApp::default();
                setup_fonts(&cc.egui_ctx);
                apply_theme(&cc.egui_ctx, &app.theme);
                Ok(Box::new(app))
            }),
        );

        IS_OPEN.store(false, std::sync::atomic::Ordering::SeqCst);
        crate::ui::hide_control_panel();
    });
}
