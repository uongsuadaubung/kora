//! HTML Components coordinator

pub mod comparison;
pub mod download;
pub mod faq;
pub mod features;
pub mod footer;
pub mod head;
pub mod hero;
pub mod interface;
pub mod macro_demo;
pub mod navbar;
pub mod performance;
pub mod playground;

/// Renders the complete, unified index.html landing page
pub fn render_page() -> String {
    let mut page = String::with_capacity(70_000);
    page.push_str(head::render());
    page.push('\n');
    page.push_str(navbar::render());
    page.push_str("\n\n");
    page.push_str(hero::render());
    page.push_str("\n\n");
    page.push_str(playground::render());
    page.push_str("\n\n");
    page.push_str(macro_demo::render());
    page.push_str("\n\n");
    page.push_str(interface::render());
    page.push_str("\n\n");
    page.push_str(features::render());
    page.push_str("\n\n");
    page.push_str(performance::render());
    page.push_str("\n\n");
    page.push_str(comparison::render());
    page.push_str("\n\n");
    page.push_str(faq::render());
    page.push_str("\n\n");
    page.push_str(download::render());
    page.push_str("\n\n");
    page.push_str(footer::render());
    page.push('\n');
    page
}
