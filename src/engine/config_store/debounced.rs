//! Debounced background file saver for EngineConfig, MacroTable, and ProfileManager

use super::codec::serialize_config_all;
use super::paths::get_config_path;
use crate::engine::config::EngineConfig;
use crate::engine::macro_table::MacroTable;
use crate::engine::profile::ProfileManager;
use std::fs;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Serializes EngineConfig, MacroTable, and ProfileManager into unified INI format and saves to disk
pub fn save_config_all(
    config: &EngineConfig,
    macros: &MacroTable,
    profiles: &ProfileManager,
) -> std::io::Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let out = serialize_config_all(config, macros, profiles);
    fs::write(&path, out)?;

    Ok(())
}

/// Serializes EngineConfig and MacroTable into unified INI format and saves to disk
pub fn save_config_and_macros(config: &EngineConfig, macros: &MacroTable) -> std::io::Result<()> {
    let dummy_profiles = ProfileManager::new();
    save_config_all(config, macros, &dummy_profiles)
}

struct DebounceState {
    pending: Option<(EngineConfig, MacroTable, ProfileManager)>,
    last_request: Instant,
    shutdown: bool,
}

static DEBOUNCER: OnceLock<(Mutex<DebounceState>, Condvar)> = OnceLock::new();

fn get_debouncer() -> &'static (Mutex<DebounceState>, Condvar) {
    DEBOUNCER.get_or_init(|| {
        let pair = (
            Mutex::new(DebounceState {
                pending: None,
                last_request: Instant::now(),
                shutdown: false,
            }),
            Condvar::new(),
        );

        let _ = std::thread::Builder::new()
            .name("kora-config-debouncer".to_string())
            .spawn(|| {
                let (lock, cvar) = get_debouncer();
                let debounce_duration = Duration::from_millis(500);

                loop {
                    let mut state = lock.lock().unwrap();

                    while state.pending.is_none() && !state.shutdown {
                        state = cvar.wait(state).unwrap();
                    }

                    if state.shutdown {
                        if let Some((cfg, mac, prof)) = state.pending.take() {
                            let _ = save_config_all(&cfg, &mac, &prof);
                        }
                        break;
                    }

                    let elapsed = state.last_request.elapsed();
                    if elapsed < debounce_duration {
                        let remaining = debounce_duration - elapsed;
                        let (new_state, _) = cvar.wait_timeout(state, remaining).unwrap();
                        state = new_state;
                        continue;
                    }

                    if let Some((cfg, mac, prof)) = state.pending.take() {
                        drop(state);
                        if let Err(e) = save_config_all(&cfg, &mac, &prof) {
                            eprintln!("[Kora] Lỗi lưu cấu hình: {e}");
                        }
                    }
                }
            });

        pair
    })
}

/// Requests an asynchronous, debounced save of EngineConfig, MacroTable, and ProfileManager to disk.
pub fn save_config_all_debounced(
    config: &EngineConfig,
    macros: &MacroTable,
    profiles: &ProfileManager,
) {
    let (lock, cvar) = get_debouncer();
    let mut state = lock.lock().unwrap();
    state.pending = Some((config.clone(), macros.clone(), profiles.clone()));
    state.last_request = Instant::now();
    cvar.notify_one();
}

/// Requests an asynchronous, debounced save of EngineConfig and MacroTable to disk (legacy helper).
pub fn save_config_and_macros_debounced(config: &EngineConfig, macros: &MacroTable) {
    let (lock, cvar) = get_debouncer();
    let mut state = lock.lock().unwrap();
    let profiles = state
        .pending
        .as_ref()
        .map(|(_, _, p)| p.clone())
        .unwrap_or_default();
    state.pending = Some((config.clone(), macros.clone(), profiles));
    state.last_request = Instant::now();
    cvar.notify_one();
}

/// Immediately flushes any pending debounced config saves to disk.
pub fn flush_config_debounced() {
    if let Some((lock, _)) = DEBOUNCER.get() {
        let mut state = lock.lock().unwrap();
        if let Some((cfg, mac, prof)) = state.pending.take() {
            let _ = save_config_all(&cfg, &mac, &prof);
        }
    }
}
