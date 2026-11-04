use super::theme::FluentTheme;
use eframe::egui::{self, Color32, Pos2, Stroke, StrokeKind, Vec2};

/// Windows 11 Animated Fluent Toggle Switch
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = Vec2::new(40.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    let animation_t = ui.ctx().animate_bool(response.id, *on);
    let theme = FluentTheme::dark();
    let is_hovered = response.hovered();

    // Background pill
    let radius = rect.height() / 2.0;
    let bg_color = if *on {
        if is_hovered {
            theme.accent_hover
        } else {
            theme.accent
        }
    } else {
        if is_hovered {
            Color32::from_rgb(52, 52, 52)
        } else {
            Color32::from_rgb(38, 38, 38)
        }
    };

    let border_stroke = if *on {
        Stroke::NONE
    } else {
        let border_color = if is_hovered {
            Color32::from_rgb(180, 180, 180)
        } else {
            Color32::from_rgb(130, 130, 130)
        };
        Stroke::new(1.0, border_color)
    };

    ui.painter()
        .rect(rect, radius, bg_color, border_stroke, StrokeKind::Inside);

    // Thumb glide animation
    let thumb_radius = radius - 3.5;
    let min_x = rect.left() + radius;
    let max_x = rect.right() - radius;
    let thumb_x = egui::lerp(min_x..=max_x, animation_t);

    let thumb_color = if *on {
        Color32::WHITE
    } else if is_hovered {
        Color32::from_rgb(240, 240, 240)
    } else {
        Color32::from_rgb(205, 205, 205)
    };

    ui.painter()
        .circle_filled(Pos2::new(thumb_x, rect.center().y), thumb_radius, thumb_color);

    response
}

/// Windows 11 Settings Card Container (with subtle border and 8px rounding)
pub fn card_frame<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let theme = FluentTheme::dark();
    egui::Frame::new()
        .fill(theme.bg_card)
        .corner_radius(8.0)
        .stroke(Stroke::new(1.0, theme.stroke_card))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, add_contents)
        .inner
}

/// Generic Settings Row with Left Title/Desc and Right Control
pub fn setting_row<R>(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    subtitle: &str,
    control: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let theme = FluentTheme::dark();
    ui.horizontal(|ui| {
        if !icon.is_empty() {
            ui.label(egui::RichText::new(icon).size(18.0));
            ui.add_space(8.0);
        }

        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(13.0)
                    .color(theme.text_primary),
            );
            if !subtitle.is_empty() {
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            control(ui)
        })
        .inner
    })
    .inner
}

/// Shorthand for a Setting Row with a Toggle Switch
pub fn setting_toggle(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    subtitle: &str,
    value: &mut bool,
) -> egui::Response {
    setting_row(ui, icon, title, subtitle, |ui| {
        toggle_switch(ui, value)
    })
}

/// Shorthand for a Setting Row with a Dropdown ComboBox
pub fn setting_combo<T: ToString + PartialEq>(
    ui: &mut egui::Ui,
    id_salt: &str,
    icon: &str,
    title: &str,
    subtitle: &str,
    options: &[T],
    selected_idx: &mut usize,
) {
    setting_row(ui, icon, title, subtitle, |ui| {
        let current_text = options.get(*selected_idx).map(|s| s.to_string()).unwrap_or_default();
        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(current_text)
            .show_ui(ui, |ui| {
                for (i, opt) in options.iter().enumerate() {
                    ui.selectable_value(selected_idx, i, opt.to_string());
                }
            });
    });
}

/// Section Heading above a group of Cards
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    let theme = FluentTheme::dark();
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(title)
            .size(14.0)
            .strong()
            .color(theme.text_primary),
    );
    ui.add_space(4.0);
}

/// Subtle 1px divider inside a Card between rows
pub fn card_divider(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
}
