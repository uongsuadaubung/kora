//! Native Windows Audio Session API (WASAPI) COM Wrapper & Audio Session
//!
//! Khai báo C-ABI COM Interfaces cho hệ thống âm thanh Windows:
//! - Quản lý thiết bị xuất âm thanh (`IMMDeviceEnumerator`, `IMMDevice`).
//! - Điều khiển Audio Stream (`IAudioClient`, `IAudioRenderClient`).
//! - Bộ trộn đa âm Polyphonic Audio Session (`WasapiSession`).

use super::wav::SoundBank;
use std::ptr::null_mut;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub const CLSID_MM_DEVICE_ENUMERATOR: GUID = GUID {
    data1: 0xBCDE0395,
    data2: 0xE52F,
    data3: 0x467C,
    data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E],
};

pub const IID_IMM_DEVICE_ENUMERATOR: GUID = GUID {
    data1: 0xA95664D2,
    data2: 0x9614,
    data3: 0x4F35,
    data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
};

pub const IID_IAUDIO_CLIENT: GUID = GUID {
    data1: 0x1CB9AD4C,
    data2: 0xDBFA,
    data3: 0x4C32,
    data4: [0xB1, 0x78, 0xC2, 0xF5, 0x68, 0xA7, 0x03, 0xB2],
};

pub const IID_IAUDIO_RENDER_CLIENT: GUID = GUID {
    data1: 0xF294ACFC,
    data2: 0x3146,
    data3: 0x4483,
    data4: [0xA7, 0xBF, 0xAD, 0xDC, 0xA7, 0xC2, 0x60, 0xE2],
};

pub const CLSCTX_ALL: u32 = 23;
pub const COINIT_MULTITHREADED: u32 = 0x0;
pub const AUDCLNT_SHAREMODE_SHARED: u32 = 0;

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
pub struct WAVEFORMATEX {
    pub w_format_tag: u16,
    pub n_channels: u16,
    pub n_samples_per_sec: u32,
    pub n_avg_bytes_per_sec: u32,
    pub n_block_align: u16,
    pub w_bits_per_sample: u16,
    pub cb_size: u16,
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn CoInitializeEx(pv_reserved: *mut std::ffi::c_void, dw_co_init: u32) -> i32;
    pub fn CoCreateInstance(
        rclsid: *const GUID,
        p_unk_outer: *mut std::ffi::c_void,
        dw_cls_context: u32,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> i32;
    pub fn CoTaskMemFree(pv: *mut std::ffi::c_void);
}

#[repr(C)]
pub struct IMMDeviceEnumeratorVtbl {
    pub query_interface: usize,
    pub add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub enum_audio_endpoints: usize,
    pub get_default_audio_endpoint: unsafe extern "system" fn(
        *mut std::ffi::c_void,
        i32,
        i32,
        *mut *mut std::ffi::c_void,
    ) -> i32,
}

#[repr(C)]
pub struct IMMDeviceVtbl {
    pub query_interface: usize,
    pub add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub activate: unsafe extern "system" fn(
        *mut std::ffi::c_void,
        *const GUID,
        u32,
        *mut std::ffi::c_void,
        *mut *mut std::ffi::c_void,
    ) -> i32,
}

#[repr(C)]
pub struct IAudioClientVtbl {
    pub query_interface: usize,
    pub add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub initialize: unsafe extern "system" fn(
        *mut std::ffi::c_void,
        u32,
        u32,
        i64,
        i64,
        *const WAVEFORMATEX,
        *const GUID,
    ) -> i32,
    pub get_buffer_size: unsafe extern "system" fn(*mut std::ffi::c_void, *mut u32) -> i32,
    pub get_stream_latency: usize,
    pub get_current_padding: unsafe extern "system" fn(*mut std::ffi::c_void, *mut u32) -> i32,
    pub is_format_supported: usize,
    pub get_mix_format:
        unsafe extern "system" fn(*mut std::ffi::c_void, *mut *mut WAVEFORMATEX) -> i32,
    pub get_device_period: usize,
    pub start: unsafe extern "system" fn(*mut std::ffi::c_void) -> i32,
    pub stop: unsafe extern "system" fn(*mut std::ffi::c_void) -> i32,
    pub reset: unsafe extern "system" fn(*mut std::ffi::c_void) -> i32,
    pub set_event_handle: usize,
    pub get_service: unsafe extern "system" fn(
        *mut std::ffi::c_void,
        *const GUID,
        *mut *mut std::ffi::c_void,
    ) -> i32,
}

#[repr(C)]
pub struct IAudioRenderClientVtbl {
    pub query_interface: usize,
    pub add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub get_buffer: unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut *mut u8) -> i32,
    pub release_buffer: unsafe extern "system" fn(*mut std::ffi::c_void, u32, u32) -> i32,
}

pub struct ActiveVoice {
    pub vk: u32,
    pub variation: usize,
    pub pos: usize,
    pub volume: f32,
}

pub struct WasapiSession {
    p_client: *mut std::ffi::c_void,
    p_render: *mut std::ffi::c_void,
    buffer_frame_count: u32,
    channels: usize,
    is_float: bool,
    block_align: usize,
    mix_buf: Vec<f32>,
}

impl WasapiSession {
    pub fn open() -> Result<Self, i32> {
        unsafe {
            let mut p_enum: *mut std::ffi::c_void = null_mut();
            let hr = CoCreateInstance(
                &CLSID_MM_DEVICE_ENUMERATOR,
                null_mut(),
                CLSCTX_ALL,
                &IID_IMM_DEVICE_ENUMERATOR,
                &mut p_enum,
            );
            if hr < 0 {
                return Err(hr);
            }

            let enum_vtbl = *(p_enum as *mut *mut IMMDeviceEnumeratorVtbl);
            let mut p_device: *mut std::ffi::c_void = null_mut();
            let hr = ((*enum_vtbl).get_default_audio_endpoint)(p_enum, 0, 0, &mut p_device);
            ((*enum_vtbl).release)(p_enum);
            if hr < 0 {
                return Err(hr);
            }

            let dev_vtbl = *(p_device as *mut *mut IMMDeviceVtbl);
            let mut p_client: *mut std::ffi::c_void = null_mut();
            let hr = ((*dev_vtbl).activate)(
                p_device,
                &IID_IAUDIO_CLIENT,
                CLSCTX_ALL,
                null_mut(),
                &mut p_client,
            );
            ((*dev_vtbl).release)(p_device);
            if hr < 0 {
                return Err(hr);
            }

            let client_vtbl = *(p_client as *mut *mut IAudioClientVtbl);
            let mut p_format: *mut WAVEFORMATEX = null_mut();
            let hr = ((*client_vtbl).get_mix_format)(p_client, &mut p_format);
            if hr < 0 {
                ((*client_vtbl).release)(p_client);
                return Err(hr);
            }

            let fmt = &*p_format;
            let channels = fmt.n_channels as usize;
            let is_float = fmt.w_bits_per_sample == 32;
            let block_align = fmt.n_block_align as usize;

            // 50ms buffer duration in 100ns units
            let hns_buffer_duration = 50 * 10000;
            let hr = ((*client_vtbl).initialize)(
                p_client,
                AUDCLNT_SHAREMODE_SHARED,
                0,
                hns_buffer_duration,
                0,
                p_format,
                null_mut(),
            );
            CoTaskMemFree(p_format as *mut std::ffi::c_void);
            if hr < 0 {
                ((*client_vtbl).release)(p_client);
                return Err(hr);
            }

            let mut buffer_frame_count = 0u32;
            let hr = ((*client_vtbl).get_buffer_size)(p_client, &mut buffer_frame_count);
            if hr < 0 {
                ((*client_vtbl).release)(p_client);
                return Err(hr);
            }

            let mut p_render: *mut std::ffi::c_void = null_mut();
            let hr =
                ((*client_vtbl).get_service)(p_client, &IID_IAUDIO_RENDER_CLIENT, &mut p_render);
            if hr < 0 {
                ((*client_vtbl).release)(p_client);
                return Err(hr);
            }

            let render_vtbl = *(p_render as *mut *mut IAudioRenderClientVtbl);

            // Pre-fill initial buffer with silence
            let mut p_data: *mut u8 = null_mut();
            let hr = ((*render_vtbl).get_buffer)(p_render, buffer_frame_count, &mut p_data);
            if hr >= 0 {
                std::ptr::write_bytes(p_data, 0, buffer_frame_count as usize * block_align);
                let _ = ((*render_vtbl).release_buffer)(p_render, buffer_frame_count, 0);
            }

            // Start audio stream
            let hr = ((*client_vtbl).start)(p_client);
            if hr < 0 {
                ((*render_vtbl).release)(p_render);
                ((*client_vtbl).release)(p_client);
                return Err(hr);
            }

            Ok(Self {
                p_client,
                p_render,
                buffer_frame_count,
                channels,
                is_float,
                block_align,
                mix_buf: vec![0.0f32; buffer_frame_count as usize],
            })
        }
    }

    pub fn render_mix(
        &mut self,
        voices: &mut Vec<ActiveVoice>,
        bank: &SoundBank,
    ) -> Result<(), i32> {
        unsafe {
            let client_vtbl = *(self.p_client as *mut *mut IAudioClientVtbl);
            let mut padding = 0u32;
            let hr = ((*client_vtbl).get_current_padding)(self.p_client, &mut padding);
            if hr < 0 {
                return Err(hr);
            }

            let frames_available = self.buffer_frame_count.saturating_sub(padding);
            if frames_available == 0 {
                return Ok(());
            }

            let render_vtbl = *(self.p_render as *mut *mut IAudioRenderClientVtbl);
            let mut p_data: *mut u8 = null_mut();
            let hr = ((*render_vtbl).get_buffer)(self.p_render, frames_available, &mut p_data);
            if hr < 0 {
                return Err(hr);
            }

            if voices.is_empty() {
                // No active sounds playing -> fill silence
                std::ptr::write_bytes(p_data, 0, frames_available as usize * self.block_align);
            } else {
                // Zero-heap polyphonic mixing using preallocated scratch buffer
                let frames = (frames_available as usize).min(self.mix_buf.len());
                let mix_slice = &mut self.mix_buf[..frames];
                mix_slice.fill(0.0);

                for v in voices.iter_mut() {
                    let samples = bank.get_samples(v.vk, v.variation);
                    let remaining = samples.len().saturating_sub(v.pos);
                    let count = remaining.min(frames);
                    let vol = v.volume;
                    let src = &samples[v.pos..v.pos + count];
                    let dst = &mut mix_slice[..count];
                    for (d, &s) in dst.iter_mut().zip(src.iter()) {
                        *d = s.mul_add(vol, *d);
                    }
                    v.pos += count;
                }

                if self.is_float {
                    let float_buf =
                        std::slice::from_raw_parts_mut(p_data as *mut f32, frames * self.channels);
                    if self.channels == 2 {
                        for (pair, &sample) in float_buf.chunks_exact_mut(2).zip(mix_slice.iter()) {
                            let clamped = sample.clamp(-1.0, 1.0);
                            pair[0] = clamped;
                            pair[1] = clamped;
                        }
                    } else {
                        for (chunk, &sample) in float_buf
                            .chunks_exact_mut(self.channels)
                            .zip(mix_slice.iter())
                        {
                            let clamped = sample.clamp(-1.0, 1.0);
                            chunk.fill(clamped);
                        }
                    }
                } else {
                    let i16_buf =
                        std::slice::from_raw_parts_mut(p_data as *mut i16, frames * self.channels);
                    if self.channels == 2 {
                        for (pair, &sample) in i16_buf.chunks_exact_mut(2).zip(mix_slice.iter()) {
                            let clamped = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
                            pair[0] = clamped;
                            pair[1] = clamped;
                        }
                    } else {
                        for (chunk, &sample) in i16_buf
                            .chunks_exact_mut(self.channels)
                            .zip(mix_slice.iter())
                        {
                            let clamped = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
                            chunk.fill(clamped);
                        }
                    }
                }
                // Retain only voices that still have remaining samples
                voices.retain(|v| v.pos < bank.get_samples(v.vk, v.variation).len());
            }

            let hr = ((*render_vtbl).release_buffer)(self.p_render, frames_available, 0);
            if hr < 0 {
                return Err(hr);
            }

            Ok(())
        }
    }

    pub fn close(&mut self) {
        self.mix_buf = Vec::new();
        unsafe {
            if !self.p_client.is_null() {
                let client_vtbl = *(self.p_client as *mut *mut IAudioClientVtbl);
                let _ = ((*client_vtbl).stop)(self.p_client);
            }
            if !self.p_render.is_null() {
                let render_vtbl = *(self.p_render as *mut *mut IAudioRenderClientVtbl);
                let _ = ((*render_vtbl).release)(self.p_render);
                self.p_render = null_mut();
            }
            if !self.p_client.is_null() {
                let client_vtbl = *(self.p_client as *mut *mut IAudioClientVtbl);
                let _ = ((*client_vtbl).release)(self.p_client);
                self.p_client = null_mut();
            }
        }
    }
}

impl Drop for WasapiSession {
    fn drop(&mut self) {
        self.close();
    }
}
