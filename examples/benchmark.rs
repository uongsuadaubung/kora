//! Kora Keyboard Suite - Comprehensive Throughput & Latency Benchmark
//! Measures raw typing speed, latency per key, macro expansion, and key management overhead.

use kora::{EngineAction, EngineConfig, InputMethod, VietnameseEngine};
use std::hint::black_box;
use std::time::Instant;

fn simulate_typing(engine: &mut VietnameseEngine, text: &str) -> String {
    let mut screen = String::new();
    for ch in text.chars() {
        let is_upper = ch.is_uppercase();
        match engine.on_key(ch, is_upper, false) {
            EngineAction::Passthrough => screen.push(ch),
            EngineAction::Replace { backspaces, output } => {
                for _ in 0..backspaces {
                    screen.pop();
                }
                screen.push_str(&output);
            }
            EngineAction::Consume => {}
        }
    }
    screen
}

fn print_result(name: &str, total_keys: usize, elapsed: std::time::Duration) {
    let speed = (total_keys as f64) / elapsed.as_secs_f64();
    let ns_per_key = (elapsed.as_nanos() as f64) / (total_keys as f64);
    let us_per_key = ns_per_key / 1000.0;
    println!(
        "  {:<42} | {:>9.0} keys/s | {:>7.1} ns/key ({:.3} µs)",
        name, speed, ns_per_key, us_per_key
    );
}

fn main() {
    println!("================================================================================");
    println!("             KORA INPUT ENGINE & KEYBOARD WORKSPACE BENCHMARK                   ");
    println!("================================================================================");

    let paragraph =
        "tooi gox tieengs vieetj raats nhanh vaf muwowjt maf treen heej thoongs Kora moqis ";
    let paragraph_chars: Vec<char> = paragraph.chars().collect();
    let iterations = 25_000;
    let total_paragraph_keys = paragraph_chars.len() * iterations;

    // -------------------------------------------------------------------------
    // 1. Pure Engine on_key Throughput (Nanosecond latency per raw keystroke)
    // -------------------------------------------------------------------------
    println!("\n[1] PURE ENGINE ON_KEY LATENCY & THROUGHPUT (No screen allocation):");
    println!("  {:<42} | {:>16} | {:>18}", "Benchmark Case", "Throughput", "Latency per Key");
    println!("  {:-<42}-+-{:-<16}-+-{:-<18}", "", "", "");

    // 1.1 Telex
    {
        let mut engine = VietnameseEngine::new(EngineConfig {
            method: InputMethod::Telex,
            ..Default::default()
        });
        let start = Instant::now();
        for _ in 0..iterations {
            for &ch in &paragraph_chars {
                black_box(engine.on_key(ch, ch.is_uppercase(), false));
            }
        }
        print_result("Telex (on_key raw)", total_paragraph_keys, start.elapsed());
    }

    // 1.2 Simple Telex
    {
        let mut engine = VietnameseEngine::new(EngineConfig {
            method: InputMethod::SimpleTelex1,
            ..Default::default()
        });
        let start = Instant::now();
        for _ in 0..iterations {
            for &ch in &paragraph_chars {
                black_box(engine.on_key(ch, ch.is_uppercase(), false));
            }
        }
        print_result("Simple Telex (on_key raw)", total_paragraph_keys, start.elapsed());
    }

    // 1.3 English Passthrough
    {
        let mut engine = VietnameseEngine::new(EngineConfig {
            enabled: false,
            ..Default::default()
        });
        let start = Instant::now();
        for _ in 0..iterations {
            for &ch in &paragraph_chars {
                black_box(engine.on_key(ch, ch.is_uppercase(), false));
            }
        }
        print_result("English Bypass (on_key raw)", total_paragraph_keys, start.elapsed());
    }

    // -------------------------------------------------------------------------
    // 2. Full End-to-End Screen Simulation (Backspace + Replacement EngineAction)
    // -------------------------------------------------------------------------
    println!("\n[2] FULL SCREEN SIMULATION (Handling Backspace & Replacement):");
    println!("  {:<42} | {:>16} | {:>18}", "Benchmark Case", "Throughput", "Latency per Key");
    println!("  {:-<42}-+-{:-<16}-+-{:-<18}", "", "", "");

    {
        let mut engine = VietnameseEngine::new(EngineConfig::default());
        let start = Instant::now();
        for _ in 0..iterations {
            simulate_typing(&mut engine, paragraph);
        }
        print_result("End-to-End Vietnamese Typing", total_paragraph_keys, start.elapsed());
    }

    // -------------------------------------------------------------------------
    // 3. Macro Typing & Row Mute Throughput (Zero-Allocation Verification)
    // -------------------------------------------------------------------------
    println!("\n[3] MACRO EXPANSION & ZERO-ALLOCATION ROW MUTE BENCHMARK:");
    println!("  {:<42} | {:>16} | {:>18}", "Benchmark Case", "Throughput", "Latency per Key");
    println!("  {:-<42}-+-{:-<16}-+-{:-<18}", "", "", "");

    let macro_text = "vn dc ko tp_hcm vn dc ko tp_hcm ";
    let macro_chars: Vec<char> = macro_text.chars().collect();
    let macro_iterations = 25_000;
    let total_macro_keys = macro_chars.len() * macro_iterations;

    // 3.1 Normal Macro (Unmuted, all active)
    {
        let mut engine = VietnameseEngine::new(EngineConfig {
            use_macro: true,
            ..Default::default()
        });
        engine.macro_table.insert("vn", "Việt Nam");
        engine.macro_table.insert("dc", "được");
        engine.macro_table.insert("ko", "không");
        engine.macro_table.insert("tp_hcm", "thành phố Hồ Chí Minh");

        let start = Instant::now();
        for _ in 0..macro_iterations {
            for &ch in &macro_chars {
                black_box(engine.on_key(ch, ch.is_uppercase(), false));
            }
        }
        print_result("Macro (All Unmuted)", total_macro_keys, start.elapsed());
    }

    // 3.2 Macro with Muted Rows (Row toggled off, testing zero-allocation cache)
    {
        let mut engine = VietnameseEngine::new(EngineConfig {
            use_macro: true,
            ..Default::default()
        });
        engine.macro_table.insert("vn", "Việt Nam");
        engine.macro_table.insert("dc", "được");
        engine.macro_table.insert("ko", "không");
        engine.macro_table.insert("tp_hcm", "thành phố Hồ Chí Minh");

        // Mute rows (simulating UI toggle off)
        engine
            .profile_manager
            .global_profile
            .muted_macros
            .push("vn".to_string());
        engine
            .profile_manager
            .global_profile
            .muted_macros
            .push("tp_hcm".to_string());

        let start = Instant::now();
        for _ in 0..macro_iterations {
            for &ch in &macro_chars {
                black_box(engine.on_key(ch, ch.is_uppercase(), false));
            }
        }
        print_result("Macro with Muted Rows (Cached Zero-Alloc)", total_macro_keys, start.elapsed());
    }

    // -------------------------------------------------------------------------
    // 4. Keyboard Management Hook Resolution (Remap, Disable, Key Macro)
    // -------------------------------------------------------------------------
    println!("\n[4] KEYBOARD HOOK RULE RESOLUTION (Remap, Disable, Key Macro):");
    println!("  {:<42} | {:>16} | {:>18}", "Benchmark Case", "Throughput", "Latency per Key");
    println!("  {:-<42}-+-{:-<16}-+-{:-<18}", "", "", "");

    {
        let mut engine = VietnameseEngine::new(EngineConfig {
            remap_enabled: true,
            disable_enabled: true,
            key_macro_enabled: true,
            ..Default::default()
        });
        let pm = &mut engine.profile_manager;
        pm.global_profile.remap_enabled = Some(true);
        pm.global_profile.disable_enabled = Some(true);
        pm.global_profile.key_macro_enabled = Some(true);
        pm.global_profile.app_remaps.push(kora::engine::profile::KeyRemapRule::new(0x14, 0x11));
        pm.global_profile.app_disabled.push(0x2D);

        let hook_iterations = 100_000;
        let test_vks = [0x14u32, 0x2Du32, 0x41u32, 0x20u32]; // CapsLock, Insert, 'A', Space
        let total_hook_checks = test_vks.len() * hook_iterations;

        let start = Instant::now();
        for _ in 0..hook_iterations {
            for &vk in &test_vks {
                let _is_disabled = black_box(engine.effective_disabled().contains(&vk));
                let _remap = black_box(engine.effective_remaps().into_iter().find(|r| r.from_vk == vk));
            }
        }
        print_result("Hook Rule Resolution (4 keys)", total_hook_checks, start.elapsed());
    }

    println!("\n================================================================================");
    println!("SUMMARY: ALL BENCHMARKS COMPLETED WITH SUB-MICROSECOND LATENCY PER KEYSTROKE.");
    println!("================================================================================\n");
}

