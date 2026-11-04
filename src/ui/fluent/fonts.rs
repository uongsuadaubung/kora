use eframe::egui;

/// Dynamically loads Windows system fonts (Segoe UI + Segoe UI Symbol + Segoe UI Emoji) into egui.
/// This avoids bundling ~2 MB of static font files inside the binary while giving authentic Windows 11 visuals.
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::empty();

    // 1. Primary Text Font: Segoe UI (fallback to Arial)
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf")
        .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\arial.ttf"))
    {
        fonts.font_data.insert(
            "segoe_ui".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("segoe_ui".to_owned());
    }

    // 2. Monospace Font: Consolas (fallback to Segoe UI)
    if let Ok(mono_bytes) = std::fs::read("C:\\Windows\\Fonts\\consola.ttf") {
        fonts.font_data.insert(
            "consolas".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(mono_bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("consolas".to_owned());
    } else {
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("segoe_ui".to_owned());
    }

    // 3. Symbols Font: Segoe UI Symbol
    if let Ok(sym_bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
        fonts.font_data.insert(
            "segoe_ui_symbol".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(sym_bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("segoe_ui_symbol".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("segoe_ui_symbol".to_owned());
    }

    // 4. Emoji Font: Segoe UI Emoji
    if let Ok(emj_bytes) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
        fonts.font_data.insert(
            "segoe_ui_emoji".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(emj_bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("segoe_ui_emoji".to_owned());
    }

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loading() {
        let ctx = egui::Context::default();
        setup_fonts(&ctx);
    }
}

