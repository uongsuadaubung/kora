//! JavaScript scripts coordinator

pub mod audio;
pub mod macro_demo;
pub mod playground;
pub mod presets;
pub mod ui;

/// Renders the complete, unified script.js script file
pub fn render_js() -> String {
    let mut js = String::with_capacity(35_000);
    js.push_str(presets::render());
    js.push_str("\n\n");
    js.push_str(audio::render());
    js.push_str("\n\n");
    js.push_str(playground::render());
    js.push_str("\n\n");
    js.push_str(macro_demo::render());
    js.push_str("\n\n");
    js.push_str(ui::render());
    js.push_str("\n\n");
    js
}
