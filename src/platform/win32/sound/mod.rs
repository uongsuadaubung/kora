//! Mechanical Keyboard Sound Engine using Native Windows Audio Session API (WASAPI)
//! Pure Win32 COM, Shared Mode, Polyphonic Voice Mixer, Zero External Dependencies.
//!
//! Submodules:
//! - [`tags`]: Phân loại phím cơ & mapping file âm thanh (`KeyTag`, `vk_to_tag`, `match_file_tag`).
//! - [`wav`]: Giải mã file âm thanh PCM WAVE 8/16/24/32-bit & bộ nhớ đệm `SoundBank`.
//! - [`wasapi`]: Giao diện COM Low-level WASAPI & phiên làm việc `WasapiSession`.

pub mod installer;
pub mod tags;
pub mod wasapi;
pub mod wav;

pub use installer::*;
pub use tags::*;
pub use wasapi::*;
pub use wav::*;

use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::Duration;

pub(crate) enum SoundCmd {
    Play(u32),
    Test,
    Reconfigure {
        enabled: bool,
        profile: String,
        volume: u8,
    },
}

static SOUND_ENABLED: AtomicBool = AtomicBool::new(false);
static SOUND_VOLUME: AtomicU8 = AtomicU8::new(50);
static SOUND_CHANNEL: Mutex<Option<SyncSender<SoundCmd>>> = Mutex::new(None);

static KEY_DOWN_BITS: [AtomicU64; 4] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

/// Releases the physical key state so the next press can trigger sound
pub fn release_key_sound(vk: u32) {
    if (vk as usize) < 256 {
        let word_idx = (vk as usize) / 64;
        let mask = 1u64 << ((vk as usize) % 64);
        KEY_DOWN_BITS[word_idx].fetch_and(!mask, Ordering::Relaxed);
    }
}

/// Resets all physical key down tracking bits (e.g. after sleep / lock / mode change)
pub fn reset_key_sound_bits() {
    for bits in &KEY_DOWN_BITS {
        bits.store(0, Ordering::Relaxed);
    }
}

/// Global trigger called directly from the Low-Level Keyboard Hook.
/// Non-blocking try_send ensures hook latency remains strictly ~0.05 µs.
pub fn trigger_key_sound(vk: u32) {
    if !SOUND_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    // Filter out OS auto-repeat events when a physical key is held down (e.g. holding Backspace)
    if (vk as usize) < 256 {
        let word_idx = (vk as usize) / 64;
        let mask = 1u64 << ((vk as usize) % 64);
        let prev = KEY_DOWN_BITS[word_idx].fetch_or(mask, Ordering::Relaxed);
        if prev & mask != 0 {
            return;
        }
    }
    if let Ok(guard) = SOUND_CHANNEL.try_lock()
        && let Some(ref sender) = *guard
    {
        let _ = sender.try_send(SoundCmd::Play(vk));
    }
}

/// Plays a test sound immediately for the currently configured switch profile and volume
pub fn play_test_sound() {
    if let Ok(guard) = SOUND_CHANNEL.try_lock()
        && let Some(ref sender) = *guard
    {
        let _ = sender.try_send(SoundCmd::Test);
    }
}

/// Dynamically updates the active sound profile and volume without blocking
pub fn reconfigure_sound(enabled: bool, profile: &str, volume: u8) {
    SOUND_ENABLED.store(enabled, Ordering::Relaxed);
    SOUND_VOLUME.store(volume, Ordering::Relaxed);

    if let Ok(guard) = SOUND_CHANNEL.try_lock()
        && let Some(ref sender) = *guard
    {
        let _ = sender.try_send(SoundCmd::Reconfigure {
            enabled,
            profile: profile.to_string(),
            volume,
        });
    }
}

/// Initializes the WASAPI Shared Mode audio background worker
pub fn init_sound(enabled: bool, profile: &str, volume: u8) {
    SOUND_ENABLED.store(enabled, Ordering::Relaxed);
    SOUND_VOLUME.store(volume, Ordering::Relaxed);

    let (sender, receiver) = mpsc::sync_channel::<SoundCmd>(16);
    {
        let mut guard = SOUND_CHANNEL.lock().unwrap();
        *guard = Some(sender);
    }

    let prof = profile.to_string();
    thread::Builder::new()
        .name("kora-wasapi-audio".into())
        .spawn(move || {
            wasapi_audio_worker(receiver, enabled, prof, volume);
        })
        .expect("Failed to spawn WASAPI audio worker thread");
}

fn wasapi_audio_worker(
    receiver: mpsc::Receiver<SoundCmd>,
    initial_enabled: bool,
    initial_profile: String,
    initial_volume: u8,
) {
    unsafe {
        let hr = CoInitializeEx(null_mut(), COINIT_MULTITHREADED);
        if hr < 0 && hr != -2147417850 {
            // RPC_E_CHANGED_MODE is benign
            return;
        }
    }

    let mut cur_enabled = initial_enabled;
    let mut cur_profile = initial_profile;
    let mut cur_volume = initial_volume;
    let mut sound_bank = if cur_enabled {
        SoundBank::load_from_disk(&cur_profile)
    } else {
        SoundBank::empty()
    };
    let mut var_counter: usize = 0;
    let mut active_voices: Vec<ActiveVoice> = Vec::with_capacity(16);

    'outer: loop {
        // When sound is disabled, do not open WASAPI audio session or spin mixer.
        // Sleep indefinitely on receiver until enabled (0% CPU, 0 wakeups, 0 bytes audio buffer).
        if !cur_enabled {
            match receiver.recv() {
                Ok(cmd) => {
                    process_sound_cmd(
                        cmd,
                        &mut active_voices,
                        &mut sound_bank,
                        &mut cur_enabled,
                        &mut cur_profile,
                        &mut cur_volume,
                        &mut var_counter,
                    );
                    continue 'outer;
                }
                Err(_) => break 'outer,
            }
        }

        // Initialize WASAPI device session
        let mut session = match WasapiSession::open() {
            Ok(s) => s,
            Err(_) => {
                // If no audio device is currently connected, sleep and wait for events
                thread::sleep(Duration::from_millis(500));
                // Drain channel so it doesn't build up
                while let Ok(cmd) = receiver.try_recv() {
                    process_sound_cmd(
                        cmd,
                        &mut active_voices,
                        &mut sound_bank,
                        &mut cur_enabled,
                        &mut cur_profile,
                        &mut cur_volume,
                        &mut var_counter,
                    );
                }
                continue 'outer;
            }
        };

        // Main audio mixing loop while enabled
        while cur_enabled {
            // 1. Drain incoming command queue
            while let Ok(cmd) = receiver.try_recv() {
                process_sound_cmd(
                    cmd,
                    &mut active_voices,
                    &mut sound_bank,
                    &mut cur_enabled,
                    &mut cur_profile,
                    &mut cur_volume,
                    &mut var_counter,
                );
            }

            if !cur_enabled {
                break;
            }

            // 2. Feed WASAPI Shared Mode buffer
            match session.render_mix(&mut active_voices, &sound_bank) {
                Ok(_) => {}
                Err(_) => {
                    // Audio device invalidated / unplugged -> break inner loop to re-open
                    session.close();
                    thread::sleep(Duration::from_millis(250));
                    continue 'outer;
                }
            }

            // 3. Smart scheduling:
            // When active voices are playing, tick every 5ms for smooth polyphonic mixing.
            // When idle, sleep up to 20ms or wake INSTANTLY (0ms latency) if a keystroke arrives!
            if active_voices.is_empty() {
                if let Ok(cmd) = receiver.recv_timeout(Duration::from_millis(20)) {
                    process_sound_cmd(
                        cmd,
                        &mut active_voices,
                        &mut sound_bank,
                        &mut cur_enabled,
                        &mut cur_profile,
                        &mut cur_volume,
                        &mut var_counter,
                    );
                }
            } else {
                thread::sleep(Duration::from_millis(5));
            }
        }

        session.close();
    }
}

fn process_sound_cmd(
    cmd: SoundCmd,
    active_voices: &mut Vec<ActiveVoice>,
    sound_bank: &mut SoundBank,
    cur_enabled: &mut bool,
    cur_profile: &mut String,
    cur_volume: &mut u8,
    var_counter: &mut usize,
) {
    match cmd {
        SoundCmd::Play(vk) => {
            if !*cur_enabled {
                return;
            }
            let ratio = (*cur_volume as f32 / 100.0).clamp(0.0, 1.0);
            // Perceptual quadratic volume curve (approximates logarithmic human hearing)
            let vol = ratio * ratio;
            if active_voices.len() >= 16 {
                active_voices.remove(0); // Evict oldest voice
            }
            active_voices.push(ActiveVoice {
                vk,
                variation: *var_counter,
                pos: 0,
                volume: vol,
            });
            *var_counter = var_counter.wrapping_add(1);
        }
        SoundCmd::Test => {
            if sound_bank.is_empty() {
                *sound_bank = SoundBank::load_from_disk(cur_profile);
            }
            let ratio = (*cur_volume as f32 / 100.0).clamp(0.0, 1.0);
            let vol = ratio * ratio;
            if active_voices.len() >= 16 {
                active_voices.remove(0);
            }
            active_voices.push(ActiveVoice {
                vk: 0x20, // Spacebar test sound
                variation: *var_counter,
                pos: 0,
                volume: vol,
            });
            *var_counter = var_counter.wrapping_add(1);
        }
        SoundCmd::Reconfigure {
            enabled,
            profile,
            volume,
        } => {
            let was_enabled = *cur_enabled;
            *cur_enabled = enabled;
            *cur_volume = volume;

            if !enabled {
                // Free sample memory and voice queue completely
                active_voices.clear();
                *sound_bank = SoundBank::empty();
                // Request OS to reclaim unmapped physical memory pages
                crate::platform::win32::trim_working_set();
            } else if !was_enabled || profile != *cur_profile || sound_bank.is_empty() {
                *cur_profile = profile;
                *sound_bank = SoundBank::load_from_disk(cur_profile);
            }
        }
    }
}
