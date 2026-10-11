//! Diagnostic logging infrastructure for Kora
//!
//! Provides non-blocking asynchronous file logging to disk on a dedicated worker thread,
//! ensuring zero I/O disk latency on time-critical threads (Windows low-level hook / UI).

use std::sync::mpsc::{channel, Sender};

/// Non-blocking asynchronous log worker that writes debug logs to disk on a dedicated thread,
/// preventing any I/O disk stalls on the Windows low-level hook thread.
#[derive(Debug, Clone)]
pub struct AsyncLogWriter {
    sender: Option<Sender<String>>,
}

impl AsyncLogWriter {
    pub fn new(path: Option<String>) -> Self {
        if let Some(path) = path {
            let (tx, rx) = channel::<String>();
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path);

            if let Ok(mut file) = file {
                let _ = std::thread::Builder::new()
                    .name("kora-async-logger".to_string())
                    .spawn(move || {
                        use std::io::Write;
                        while let Ok(msg) = rx.recv() {
                            let _ = writeln!(file, "{}", msg);
                            let _ = file.flush();
                        }
                    });
                Self { sender: Some(tx) }
            } else {
                Self { sender: None }
            }
        } else {
            Self { sender: None }
        }
    }

    pub fn disabled() -> Self {
        Self { sender: None }
    }
}

impl Default for AsyncLogWriter {
    fn default() -> Self {
        Self::disabled()
    }
}

impl AsyncLogWriter {

    pub fn write_line(&self, line: String) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(line);
        }
    }

    pub fn is_active(&self) -> bool {
        self.sender.is_some()
    }
}

/// Helper trả về chuỗi thời gian hiện tại chính xác đến mili-giây
pub fn current_timestamp_str() -> String {
    #[cfg(target_os = "windows")]
    {
        use crate::platform::win32::types::{GetLocalTime, SystemTime};

        let mut st = std::mem::MaybeUninit::<SystemTime>::zeroed();
        unsafe {
            GetLocalTime(st.as_mut_ptr());
            let st = st.assume_init();
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                st.year, st.month, st.day, st.hour, st.minute, st.second, st.milliseconds
            )
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        format!("{}.{:03}", now.as_secs(), now.subsec_millis())
    }
}
