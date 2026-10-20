//! RIFF/WAVE Audio File Parser & SoundBank Cache Manager
//!
//! Chịu trách nhiệm:
//! - Giải mã file âm thanh PCM WAVE 8/16/24/32-bit (Mono/Stereo) và resample về 48kHz.
//! - Nạp và quản lý bộ nhớ đệm âm thanh switch bàn phím cơ (`SoundBank`).

use super::tags::{KEY_TAG_COUNT, KeyTag, match_file_tag, vk_to_tag};

pub struct SoundBank {
    pub(crate) slots: [Vec<Vec<f32>>; KEY_TAG_COUNT],
}

impl SoundBank {
    pub const fn empty() -> Self {
        const EMPTY_VEC: Vec<Vec<f32>> = Vec::new();
        Self {
            slots: [EMPTY_VEC; KEY_TAG_COUNT],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_empty())
    }

    #[inline]
    pub fn get_samples(&self, vk: u32, variation: usize) -> &[f32] {
        let tag = vk_to_tag(vk) as usize;
        let list = &self.slots[tag];
        if !list.is_empty() {
            return &list[variation % list.len()];
        }

        let normal = &self.slots[KeyTag::Normal as usize];
        if !normal.is_empty() {
            return &normal[variation % normal.len()];
        }

        // Graceful fallback to any sound in the pack if normal is empty
        for slot in &self.slots {
            if !slot.is_empty() {
                return &slot[variation % slot.len()];
            }
        }

        &[]
    }

    pub fn load_from_disk(profile: &str) -> Self {
        const EMPTY_VEC: Vec<Vec<f32>> = Vec::new();
        let mut slots: [Vec<Vec<f32>>; KEY_TAG_COUNT] = [EMPTY_VEC; KEY_TAG_COUNT];

        if let Some(base_dir) = crate::engine::config_store::get_switches_dir() {
            let mut target_dir = base_dir.join(profile);
            if !target_dir.is_dir()
                && let Ok(entries) = std::fs::read_dir(&base_dir)
            {
                for entry in entries.flatten() {
                    if let Ok(ft) = entry.file_type()
                        && ft.is_dir()
                    {
                        let name = entry.file_name();
                        if name.to_string_lossy().eq_ignore_ascii_case(profile) {
                            target_dir = entry.path();
                            break;
                        }
                    }
                }
            }

            if target_dir.is_dir()
                && let Ok(entries) = std::fs::read_dir(&target_dir)
            {
                let mut wav_files = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str())
                        && (ext.eq_ignore_ascii_case("wav") || ext.eq_ignore_ascii_case("wave"))
                    {
                        wav_files.push(path);
                    }
                }
                wav_files.sort();

                for path in wav_files {
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();

                    if let Ok(bytes) = std::fs::read(&path)
                        && let Some(samples) = parse_wav(&bytes)
                    {
                        if samples.is_empty() {
                            continue;
                        }

                        let tag = match_file_tag(&stem);
                        if let Some(t) = tag {
                            slots[t as usize].push(samples);
                        } else {
                            slots[KeyTag::Normal as usize].push(samples);
                        }
                    }
                }
            }
        }

        Self { slots }
    }
}

/// Pure Safe Rust WAV / RIFF Audio Parser (supports PCM 8/16/24-bit, 32-bit float, Mono/Stereo, Resampled to 48kHz)
pub fn parse_wav(data: &[u8]) -> Option<Vec<f32>> {
    if data.len() < 44 {
        return None;
    }
    if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return None;
    }

    let mut pos = 12;
    let mut format_tag = 1u16; // 1 = PCM, 3 = IEEE float
    let mut channels = 1u16;
    let mut sample_rate = 44100u32;
    let mut bits_per_sample = 16u16;
    let mut pcm_data: Option<&[u8]> = None;

    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_len =
            u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
        pos += 8;

        if chunk_id == b"fmt " && pos + chunk_len <= data.len() && chunk_len >= 16 {
            format_tag = u16::from_le_bytes([data[pos], data[pos + 1]]);
            channels = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            sample_rate =
                u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            bits_per_sample = u16::from_le_bytes([data[pos + 14], data[pos + 15]]);
        } else if chunk_id == b"data" {
            let len = chunk_len.min(data.len().saturating_sub(pos));
            pcm_data = Some(&data[pos..pos + len]);
        }
        pos += chunk_len;
        if !chunk_len.is_multiple_of(2) {
            pos += 1;
        }
    }

    let raw = pcm_data?;
    if channels == 0 || sample_rate == 0 {
        return None;
    }

    let num_channels = channels as usize;
    let mut mono_samples: Vec<f32> = Vec::new();

    if format_tag == 1 {
        match bits_per_sample {
            16 => {
                let frame_size = 2 * num_channels;
                let frames = raw.len() / frame_size;
                mono_samples.reserve(frames);
                for i in 0..frames {
                    let mut sum = 0.0f32;
                    for ch in 0..num_channels {
                        let offset = i * frame_size + ch * 2;
                        let s = i16::from_le_bytes([raw[offset], raw[offset + 1]]) as f32 / 32768.0;
                        sum += s;
                    }
                    mono_samples.push(sum / num_channels as f32);
                }
            }
            8 => {
                let frame_size = num_channels;
                let frames = raw.len() / frame_size;
                mono_samples.reserve(frames);
                for i in 0..frames {
                    let mut sum = 0.0f32;
                    for ch in 0..num_channels {
                        let offset = i * frame_size + ch;
                        let s = (raw[offset] as f32 - 128.0) / 128.0;
                        sum += s;
                    }
                    mono_samples.push(sum / num_channels as f32);
                }
            }
            24 => {
                let frame_size = 3 * num_channels;
                let frames = raw.len() / frame_size;
                mono_samples.reserve(frames);
                for i in 0..frames {
                    let mut sum = 0.0f32;
                    for ch in 0..num_channels {
                        let offset = i * frame_size + ch * 3;
                        let b = [raw[offset], raw[offset + 1], raw[offset + 2]];
                        let s = i32::from_le_bytes([
                            b[0],
                            b[1],
                            b[2],
                            if b[2] & 0x80 != 0 { 0xFF } else { 0x00 },
                        ]) as f32
                            / 8388608.0;
                        sum += s;
                    }
                    mono_samples.push(sum / num_channels as f32);
                }
            }
            _ => return None,
        }
    } else if format_tag == 3 && bits_per_sample == 32 {
        let frame_size = 4 * num_channels;
        let frames = raw.len() / frame_size;
        mono_samples.reserve(frames);
        for i in 0..frames {
            let mut sum = 0.0f32;
            for ch in 0..num_channels {
                let offset = i * frame_size + ch * 4;
                let s = f32::from_le_bytes([
                    raw[offset],
                    raw[offset + 1],
                    raw[offset + 2],
                    raw[offset + 3],
                ]);
                sum += s;
            }
            mono_samples.push(sum / num_channels as f32);
        }
    } else {
        return None;
    }

    const TARGET_FS: f32 = 48000.0;
    if sample_rate == 48000 {
        Some(mono_samples)
    } else {
        let ratio = sample_rate as f32 / TARGET_FS;
        let target_len = (mono_samples.len() as f32 / ratio) as usize;
        let mut resampled = Vec::with_capacity(target_len);
        for i in 0..target_len {
            let src_idx = i as f32 * ratio;
            let idx0 = src_idx.floor() as usize;
            let frac = src_idx - idx0 as f32;
            let s0 = mono_samples.get(idx0).copied().unwrap_or(0.0);
            let s1 = mono_samples.get(idx0 + 1).copied().unwrap_or(s0);
            resampled.push(s0 + frac * (s1 - s0));
        }
        Some(resampled)
    }
}
