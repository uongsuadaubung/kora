use super::common::simulate_typing;
use kora::engine::{EngineConfig, VietnameseEngine};

#[test]
fn test_debug_logger() {
    let test_log_file = "test_debug.log";
    if std::path::Path::new(test_log_file).exists() {
        let _ = std::fs::remove_file(test_log_file);
    }

    let mut engine = VietnameseEngine::new(EngineConfig {
        debug: true,
        debug_file_path: Some(std::sync::Arc::from(test_log_file)),
        ..Default::default()
    });

    simulate_typing(&mut engine, "tieengs ");
    let logs = engine.debug_log();
    assert!(!logs.is_empty());
    assert!(logs.iter().any(|l| l.contains("[DBG][KEY]")));
    assert!(logs.iter().any(|l| l.contains("[DBG][BREAK]")));

    // Allow background worker thread to write and flush (poll up to 500ms)
    let mut file_content = String::new();
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        if let Ok(content) = std::fs::read_to_string(test_log_file)
            && content.contains("[DBG][KEY]")
        {
            file_content = content;
            break;
        }
    }

    // Verify file exists on disk and contains log content
    assert!(std::path::Path::new(test_log_file).exists());
    assert!(file_content.contains("[DBG][KEY]"));
    assert!(file_content.contains("tieengs"));

    // Cleanup
    let _ = std::fs::remove_file(test_log_file);
}

#[test]
fn test_app_theme_detection() {
    use kora::ui::tray::is_windows_dark_apps;

    // Checks that registry query runs smoothly and returns a bool
    let is_dark = is_windows_dark_apps();
    println!("Windows apps use dark theme: {}", is_dark);
}

#[test]
fn test_optimized_png_icon_loading() {
    use kora::ui::tray::{load_icon_from_memory, safe_destroy_icon};

    static VIET_BYTES: &[u8] = include_bytes!("../../assets/vi.ico");
    static ENG_BYTES: &[u8] = include_bytes!("../../assets/en.ico");
    static APP_BYTES: &[u8] = include_bytes!("../../assets/icon.ico");

    assert!(VIET_BYTES.len() < 3000, "vi.ico should be tiny (< 3KB)");
    assert!(ENG_BYTES.len() < 3000, "en.ico should be tiny (< 3KB)");
    assert!(APP_BYTES.len() < 10000, "icon.ico should be tiny (< 10KB)");

    for sz in [16, 20, 24, 32] {
        let h_viet = load_icon_from_memory(VIET_BYTES, sz, sz);
        assert!(
            h_viet.is_some() && h_viet.unwrap() != 0,
            "Failed to load vi.ico at size {}",
            sz
        );
        safe_destroy_icon(h_viet.unwrap());

        let h_eng = load_icon_from_memory(ENG_BYTES, sz, sz);
        assert!(
            h_eng.is_some() && h_eng.unwrap() != 0,
            "Failed to load en.ico at size {}",
            sz
        );
        safe_destroy_icon(h_eng.unwrap());
    }

    for sz in [16, 24, 32, 48, 256] {
        let h_app = load_icon_from_memory(APP_BYTES, sz, sz);
        assert!(
            h_app.is_some() && h_app.unwrap() != 0,
            "Failed to load icon.ico at size {}",
            sz
        );
        safe_destroy_icon(h_app.unwrap());
    }
}

#[test]
fn test_dynamic_debug_toggle() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        debug: false,
        ..Default::default()
    });

    simulate_typing(&mut engine, "tieengs ");
    assert!(
        engine.debug_log.is_empty(),
        "Logs should be empty when debug is disabled"
    );

    // Dynamically enable debug
    engine.config_mut().debug = true;
    simulate_typing(&mut engine, "vieetj ");
    assert!(
        !engine.debug_log.is_empty(),
        "Logs should be recorded after dynamically enabling debug"
    );

    // Dynamically disable debug
    let log_count = engine.debug_log.len();
    engine.config_mut().debug = false;
    simulate_typing(&mut engine, "nam ");
    assert_eq!(
        engine.debug_log.len(),
        log_count,
        "No new logs should be recorded when disabled again"
    );
}



