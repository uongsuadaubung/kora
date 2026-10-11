mod components;
mod scripts;
mod styles;

use std::fs;
use std::path::Path;

fn main() {
    println!("=== Building Kora Landing Page ===");

    let out_dir = Path::new("landing-page");
    if !out_dir.exists() {
        fs::create_dir_all(out_dir).expect("create landing-page dir");
    }

    // 1. Build index.html
    let html_content = components::render_page();
    let html_path = out_dir.join("index.html");
    fs::write(&html_path, &html_content).expect("write index.html");
    println!(
        "✓ Successfully built {} ({} bytes)",
        html_path.display(),
        html_content.len()
    );

    // 2. Build style.css
    let css_content = styles::render_css();
    let css_path = out_dir.join("style.css");
    fs::write(&css_path, &css_content).expect("write style.css");
    println!(
        "✓ Successfully built {} ({} bytes)",
        css_path.display(),
        css_content.len()
    );

    // 3. Build script.js
    let js_content = scripts::render_js();
    let js_path = out_dir.join("script.js");
    fs::write(&js_path, &js_content).expect("write script.js");
    println!(
        "✓ Successfully built {} ({} bytes)",
        js_path.display(),
        js_content.len()
    );

    println!("=== Build finished successfully! ===");
}
