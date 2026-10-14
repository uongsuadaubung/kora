#![allow(non_snake_case)]
#![windows_subsystem = "windows"]

use kora::platform::{alloc_console, attach_parent_console, ensure_single_instance};
use kora::{EngineAction, VietnameseEngine};
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_cli = args.iter().any(|a| a == "--cli");
    let is_autostart = args.iter().any(|a| a == "--autostart" || a == "--silent");

    // Single-instance guard: prevent multiple running instances of Kora
    if !ensure_single_instance(is_cli) {
        return;
    }

    // Clean up previous executable left over from in-place self-update
    if let Ok(exe) = std::env::current_exe() {
        let old = exe.with_extension("exe.old");
        if old.exists() && std::fs::remove_file(&old).is_err() {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = std::fs::remove_file(old);
            });
        }
    }

    // Attach to existing parent console if launched from terminal (cmd/PowerShell)
    attach_parent_console();

    // Autostart (Run key) starts us with CWD = System32. Pin the GUI's CWD to the exe directory
    // so relative paths (debug log, ./switches) resolve the same as a normal double-click launch.
    if !is_cli
        && let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        let _ = std::env::set_current_dir(dir);
    }

    // Load configuration, macros & profiles unified from ~/.config/kora/config.ini
    let (mut config, macros, profiles) = kora::load_config_all();

    // Command-line flag overrides
    if args.iter().any(|a| a == "--no-debug") {
        config.debug = false;
    }
    if args.iter().any(|a| a == "--debug") {
        config.debug = true;
    }
    if args.iter().any(|a| a == "--auto-caps") {
        config.auto_uppercase_first_char = true;
    }

    let is_debug = config.debug;
    let mut engine = VietnameseEngine::new(config);
    engine.macro_table = macros;
    engine.profile_manager = profiles;

    if is_cli {
        alloc_console();
        run_cli(engine);
    } else {
        #[cfg(target_os = "windows")]
        {
            println!("========================================================");
            println!("         Kora - Next-Gen Keyboard & Input Suite         ");
            println!("========================================================");
            let log_path = engine
                .config()
                .debug_file_path
                .as_deref()
                .unwrap_or("Không lưu");
            println!(
                "* Debug Keylogger: {}",
                if is_debug {
                    "BẬT (Ghi vết chi tiết từng phím & state)"
                } else {
                    "TẮT"
                }
            );
            println!(
                "* File lưu nhật ký: {}",
                if is_debug { log_path } else { "TẮT" }
            );
            println!(
                "* Gõ tắt (Macro): {}",
                if engine.config().use_macro {
                    "BẬT"
                } else {
                    "TẮT (Mặc định tắt)"
                }
            );
            println!("* Số lượng từ gõ tắt: {}", engine.macro_table.len());
            println!(
                "* Tự động viết hoa đầu câu: {}",
                if engine.config().auto_uppercase_first_char {
                    "BẬT"
                } else {
                    "TẮT"
                }
            );
            println!(
                "* Phím tắt chuyển chế độ Việt [V] / Anh [E]: [{}]",
                if engine.config().switch_with_ctrl_shift {
                    "Ctrl + Shift"
                } else {
                    "Alt + Z"
                }
            );
            println!("* Phím tắt ngoặc: [ -> ư, ] -> ơ");
            println!("* Cấu hình lưu tại: {}", kora::get_config_path().display());
            println!("--------------------------------------------------------");

            if is_debug && let Some(ref path) = engine.config().debug_file_path {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&**path)
                {
                    let _ = writeln!(
                        f,
                        "\n========================================================"
                    );
                    let _ = writeln!(
                        f,
                        "Kora Session Started: {}",
                        kora::logger::current_timestamp_str()
                    );
                    let _ = writeln!(
                        f,
                        "Method: {:?} | Macro: {}",
                        engine.config().method,
                        engine.config().use_macro
                    );
                    let _ = writeln!(
                        f,
                        "========================================================"
                    );
                }
            }

            let show_dialog_on_startup = !is_autostart && engine.config().show_dialog_on_startup;
            kora::platform::win32::set_mode_change_callback(kora::ui::post_mode_change);
            kora::platform::win32::set_message_filter(kora::ui::views::dialogs::pre_translate_message);
            kora::platform::win32::run_hook_loop(engine, move || {
                kora::ui::init_ui();
                if show_dialog_on_startup {
                    kora::ui::show_control_panel();
                } else {
                    kora::platform::win32::trim_working_set();
                }
            });
        }

        #[cfg(not(target_os = "windows"))]
        {
            run_cli(engine);
        }
    }
}

fn run_cli(mut engine: VietnameseEngine) {
    println!("========================================================");
    println!("          Kora Vietnamese Engine - CLI Demo             ");
    println!("========================================================");
    println!("Gõ các phím tiếng Việt (Telex) để kiểm tra hoạt động:");
    println!("Ví dụ: 'tooi gox tieengs vieetj' -> 'tôi gõ tiếng việt'");
    println!("(Nhập 'exit' hoặc 'quit' để thoát)");
    println!("--------------------------------------------------------");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line == "exit" || line == "quit" {
            break;
        }

        let mut output = String::new();
        engine.reset();

        for ch in line.chars() {
            let is_upper = ch.is_uppercase();
            match engine.on_key(ch, is_upper, false) {
                EngineAction::Passthrough => {
                    output.push(ch);
                }
                EngineAction::Replace {
                    backspaces,
                    output: new_text,
                } => {
                    for _ in 0..backspaces {
                        output.pop();
                    }
                    output.push_str(&new_text);
                }
                EngineAction::Consume => {}
            }
        }

        println!("Kết quả: {}", output);
    }
}
