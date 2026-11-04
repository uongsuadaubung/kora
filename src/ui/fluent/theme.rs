use eframe::egui::{self, Color32, CornerRadius, Stroke, Vec2};

#[derive(Clone, Debug)]
pub struct FluentTheme {
    pub bg_app: Color32,
    pub bg_sidebar: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub stroke_card: Color32,
    pub stroke_divider: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub is_dark: bool,
}

impl Default for FluentTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl FluentTheme {
    pub fn dark() -> Self {
        Self {
            bg_app: Color32::from_rgb(32, 32, 32),
            bg_sidebar: Color32::from_rgb(26, 26, 26),
            bg_card: Color32::from_rgb(39, 39, 39),
            bg_card_hover: Color32::from_rgb(46, 46, 46),
            stroke_card: Color32::from_white_alpha(14),
            stroke_divider: Color32::from_white_alpha(10),
            accent: Color32::from_rgb(0, 120, 212),
            accent_hover: Color32::from_rgb(20, 135, 225),
            text_primary: Color32::from_rgb(245, 245, 245),
            text_secondary: Color32::from_rgb(160, 160, 160),
            is_dark: true,
        }
    }

    pub fn light() -> Self {
        Self {
            bg_app: Color32::from_rgb(243, 243, 243),
            bg_sidebar: Color32::from_rgb(238, 238, 238),
            bg_card: Color32::from_rgb(255, 255, 255),
            bg_card_hover: Color32::from_rgb(248, 248, 248),
            stroke_card: Color32::from_black_alpha(16),
            stroke_divider: Color32::from_black_alpha(10),
            accent: Color32::from_rgb(0, 103, 192),
            accent_hover: Color32::from_rgb(0, 90, 170),
            text_primary: Color32::from_rgb(26, 26, 26),
            text_secondary: Color32::from_rgb(94, 94, 94),
            is_dark: false,
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, theme: &FluentTheme) {
    let mut visuals = if theme.is_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = theme.bg_app;
    visuals.window_fill = theme.bg_app;
    visuals.faint_bg_color = theme.bg_card;
    visuals.extreme_bg_color = if theme.is_dark {
        Color32::from_rgb(22, 22, 22)
    } else {
        Color32::from_rgb(250, 250, 250)
    };

    // Modern Fluent Corner Radiuses (8px cards, 5-6px controls)
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);
    visuals.widgets.open.corner_radius = CornerRadius::same(6);

    // Control background fills & strokes
    visuals.widgets.inactive.bg_fill = theme.bg_card;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.stroke_card);
    visuals.widgets.hovered.bg_fill = theme.bg_card_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_white_alpha(30));
    visuals.widgets.active.bg_fill = theme.accent;
    visuals.selection.bg_fill = theme.accent;

    ctx.set_visuals(visuals);

    ctx.tessellation_options_mut(|opt| {
        opt.round_text_to_pixels = true;
        opt.round_line_segments_to_pixels = true;
        opt.round_rects_to_pixels = true;
    });

    let mut style = (*ctx.style_of(if theme.is_dark { egui::Theme::Dark } else { egui::Theme::Light })).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 10.0);
    style.spacing.button_padding = Vec2::new(14.0, 7.0);

    // Crisp pixel-aligned typography hierarchy
    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(11.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(13.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(13.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(18.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(13.0)),
    ]
    .into();

    ctx.set_style_of(if theme.is_dark { egui::Theme::Dark } else { egui::Theme::Light }, style);
}
