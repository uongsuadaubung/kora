//! CSS styles coordinator

pub mod base;
pub mod buttons;
pub mod comparison;
pub mod download;
pub mod faq;
pub mod features;
pub mod footer;
pub mod headers;
pub mod hero;
pub mod interface;
pub mod macro_demo;
pub mod navbar;
pub mod performance;
pub mod playground;
pub mod responsive;

/// Renders the complete, unified style.css stylesheet
pub fn render_css() -> String {
    let mut css = String::with_capacity(60_000);
    css.push_str(base::render());
    css.push_str("\n\n");
    css.push_str(navbar::render());
    css.push_str("\n\n");
    css.push_str(buttons::render());
    css.push_str("\n\n");
    css.push_str(hero::render());
    css.push_str("\n\n");
    css.push_str(performance::render());
    css.push_str("\n\n");
    css.push_str(headers::render());
    css.push_str("\n\n");
    css.push_str(macro_demo::render());
    css.push_str("\n\n");
    css.push_str(playground::render());
    css.push_str("\n\n");
    css.push_str(comparison::render());
    css.push_str("\n\n");
    css.push_str(interface::render());
    css.push_str("\n\n");
    css.push_str(features::render());
    css.push_str("\n\n");
    css.push_str(faq::render());
    css.push_str("\n\n");
    css.push_str(download::render());
    css.push_str("\n\n");
    css.push_str(footer::render());
    css.push_str("\n\n");
    css.push_str(responsive::render());
    css.push('\n');
    css
}
