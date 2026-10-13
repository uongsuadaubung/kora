//! Asynchronous Key Macro Automation Executor
//!
//! Executes macro action sequences on a dedicated background worker thread to
//! completely prevent LL keyboard hook timeout (WH_KEYBOARD_LL ~200ms threshold)
//! and UI stuttering.

use crate::engine::config::hotkey::name_to_vk;
use crate::engine::profile::MacroAction;
use crate::platform::win32::injector::{send_unicode_string, send_vk_event};
use std::sync::mpsc::{channel, Sender};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

static SENDER: OnceLock<Sender<Vec<MacroAction>>> = OnceLock::new();

fn get_sender() -> &'static Sender<Vec<MacroAction>> {
    SENDER.get_or_init(|| {
        let (tx, rx) = channel::<Vec<MacroAction>>();
        thread::Builder::new()
            .name("kora_macro_executor".to_string())
            .spawn(move || {
                while let Ok(actions) = rx.recv() {
                    run_actions(&actions);
                }
            })
            .expect("Failed to spawn kora_macro_executor thread");
        tx
    })
}

/// Queues a sequence of macro actions for asynchronous background execution
pub fn execute_actions(actions: Vec<MacroAction>) {
    if actions.is_empty() {
        return;
    }
    let _ = get_sender().send(actions);
}

/// Internal execution logic running on the worker thread
fn run_actions(actions: &[MacroAction]) {
    for action in actions {
        match action {
            MacroAction::Press(combo) => {
                execute_press(combo);
            }
            MacroAction::KeyDown(key) => {
                if let Some(vk) = name_to_vk(key) {
                    send_vk_event(vk, false);
                }
            }
            MacroAction::KeyUp(key) => {
                if let Some(vk) = name_to_vk(key) {
                    send_vk_event(vk, true);
                }
            }
            MacroAction::Delay(ms) => {
                if *ms > 0 {
                    thread::sleep(Duration::from_millis(*ms as u64));
                }
            }
            MacroAction::Text(text) => {
                send_unicode_string(text);
            }
        }
    }
}

/// Executes a combined or single key press (Down then Up)
fn execute_press(combo: &str) {
    let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return;
    }

    let mut modifier_vks = Vec::new();
    let mut main_vk = None;

    for part in parts {
        let part_lower = part.to_ascii_lowercase();
        match part_lower.as_str() {
            "ctrl" | "control" => modifier_vks.push(0x11),
            "alt" => modifier_vks.push(0x12),
            "shift" => modifier_vks.push(0x10),
            "win" | "windows" => modifier_vks.push(0x5B),
            _ => {
                if let Some(vk) = name_to_vk(part) {
                    main_vk = Some(vk);
                }
            }
        }
    }

    // 1. Press modifiers down
    for &mod_vk in &modifier_vks {
        send_vk_event(mod_vk, false);
    }

    // 2. Press main key down then up (or just tap modifier if no main key)
    if let Some(vk) = main_vk {
        send_vk_event(vk, false);
        thread::sleep(Duration::from_millis(10));
        send_vk_event(vk, true);
    } else if let Some(&last_mod) = modifier_vks.last() {
        send_vk_event(last_mod, true);
    }

    // 3. Release modifiers in reverse order
    for &mod_vk in modifier_vks.iter().rev() {
        send_vk_event(mod_vk, true);
    }
}

