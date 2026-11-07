//! Kora Local Inter-Process Communication (IPC) Server
//!
//! Provides a zero-dependency, ultra-lightweight HTTP/1.1 listener on 127.0.0.1:13497.
//! Enables external applications (AutoCAD AutoLISP, Word macros, Neovim/VS Code, scripts, stream decks, etc.)
//! to dynamically configure Kora per-process or globally in real time (mode, method, charset, sound, macro, auto_caps).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

/// Port 13497 (Signifying author's birthdate 13/04/1997)
pub const KORA_IPC_PORT: u16 = 13497;
pub const KORA_IPC_ADDR: &str = "127.0.0.1:13497";

/// Decodes standard percent-encoded URLs (e.g. `%20` -> `' '`, `+` -> `' '`).
pub fn url_decode(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len());
    let mut iter = input.bytes().peekable();
    while let Some(b) = iter.next() {
        match b {
            b'+' => bytes.push(b' '),
            b'%' => {
                let h1 = iter.next();
                let h2 = iter.next();
                if let (Some(h1), Some(h2)) = (h1, h2) {
                    if let Ok(num) = u8::from_str_radix(std::str::from_utf8(&[h1, h2]).unwrap_or(""), 16) {
                        bytes.push(num);
                        continue;
                    }
                    bytes.push(b'%');
                    bytes.push(h1);
                    bytes.push(h2);
                } else {
                    bytes.push(b'%');
                    if let Some(h1) = h1 {
                        bytes.push(h1);
                    }
                }
            }
            _ => bytes.push(b),
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Parses a query string (e.g. `name=acad.exe&font=.VnTime`) into key-value pairs.
pub fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        let trimmed = pair.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            map.insert(url_decode(k), url_decode(v));
        } else {
            map.insert(url_decode(trimmed), String::new());
        }
    }
    map
}

/// Parses an incoming HTTP request (line, headers, and optional body).
/// Returns (Method, Path, QueryAndBodyParams).
pub fn parse_http_request(request: &str) -> Option<(&str, &str, HashMap<String, String>)> {
    let (header_part, body_part) = match request.split_once("\r\n\r\n") {
        Some((h, b)) => (h, b),
        None => match request.split_once("\n\n") {
            Some((h, b)) => (h, b),
            None => (request, ""),
        },
    };

    let first_line = header_part.lines().next()?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?;
    let uri = parts.next()?;

    let (path, mut params) = match uri.split_once('?') {
        Some((p, q)) => (p, parse_query(q)),
        None => (uri, HashMap::new()),
    };

    if !body_part.trim().is_empty() {
        let body_params = parse_query(body_part.trim());
        for (k, v) in body_params {
            params.insert(k, v);
        }
    }

    Some((method, path, params))
}

/// Alias for backwards compatibility
pub fn parse_http_request_line(request: &str) -> Option<(&str, &str, HashMap<String, String>)> {
    parse_http_request(request)
}

fn format_config_json(target: &str, is_active: bool, cfg: &crate::engine::EngineConfig) -> String {
    format!(
        "{{\"status\":\"ok\",\"target\":\"{}\",\"active\":{},\"config\":{{\"enabled\":{},\"mode\":\"{}\",\"method\":\"{}\",\"charset\":\"{}\",\"sound_enabled\":{},\"sound_profile\":\"{}\",\"sound_volume\":{},\"use_macro\":{},\"auto_uppercase_first_char\":{}}}}}\n",
        target,
        is_active,
        cfg.enabled,
        if cfg.enabled { "vietnamese" } else { "english" },
        cfg.method.as_str(),
        cfg.charset.as_str(),
        cfg.sound_enabled,
        cfg.sound_profile,
        cfg.sound_volume,
        cfg.use_macro,
        cfg.auto_uppercase_first_char
    )
}

/// Handles an incoming HTTP request string and produces (HTTP_STATUS_CODE, RESPONSE_BODY).
pub fn handle_ipc_command(
    method: &str,
    path: &str,
    params: &HashMap<String, String>,
) -> (u16, String) {
    if method != "GET" && method != "POST" {
        return (405, "{\"status\":\"error\",\"message\":\"Method Not Allowed\"}\n".to_string());
    }

    match path {
        // ====================================================================
        // UNIFIED CONFIG API: /config (GET to read, GET with params to update)
        // ====================================================================
        "/config" => {
            let target_opt = params
                .get("name")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("global"));

            let is_mutation = params.keys().any(|k| {
                matches!(
                    k.as_str(),
                    "mode"
                        | "method"
                        | "charset"
                        | "font"
                        | "sound"
                        | "sound_profile"
                        | "sound_volume"
                        | "use_macro"
                        | "auto_caps"
                )
            });

            #[cfg(target_os = "windows")]
            {
                if is_mutation {
                    if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
                        if let Some(ref mut engine) = *guard {
                            if let Some(target) = target_opt {
                                let fg = crate::platform::win32::get_foreground_process_name();
                                let proc_with_exe = if target.to_lowercase().ends_with(".exe") {
                                    target.to_lowercase()
                                } else {
                                    format!("{}.exe", target.to_lowercase())
                                };
                                if fg.eq_ignore_ascii_case(target) || fg.eq_ignore_ascii_case(&proc_with_exe) {
                                    engine.switch_profile_by_process(target);
                                }
                            }

                            let (is_active, effective) = engine.apply_config_update(target_opt, params);
                            if is_active {
                                crate::platform::win32::sound::reconfigure_sound(
                                    effective.sound_enabled,
                                    &effective.sound_profile,
                                    effective.sound_volume,
                                );
                                crate::platform::win32::notify_mode_change(effective.enabled);
                                crate::platform::win32::app_tracker::notify_profile_change(
                                    &engine.active_profile_id().to_string(),
                                );
                            }

                            let target_label = target_opt.unwrap_or("global");
                            return (200, format_config_json(target_label, is_active, &effective));
                        }
                    }
                } else if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
                    if let Some(ref engine) = *guard {
                        let (target_name, effective, is_active) = if let Some(target) = target_opt {
                            let eff = engine
                                .profile_manager
                                .find_profile_by_process(target)
                                .map(|p| engine.profile_manager.resolve_effective_config(&p.id, engine.config()))
                                .unwrap_or_else(|| engine.config().clone());
                            let active = engine
                                .profile_manager
                                .find_profile_by_process(target)
                                .map(|p| p.id == engine.active_profile_id())
                                .unwrap_or(false);
                            (target, eff, active)
                        } else {
                            (
                                "global",
                                engine.effective_config().clone(),
                                engine.active_profile_id() == "global",
                            )
                        };

                        return (200, format_config_json(target_name, is_active, &effective));
                    }
                }
            }

            // Fallback for tests or non-windows
            let target_label = target_opt.unwrap_or("global");
            let mut cfg = crate::engine::EngineConfig::default();
            if let Some(m) = params.get("method").and_then(|s| s.parse().ok()) {
                cfg.method = m;
            }
            if let Some(font) = params.get("font") {
                cfg.charset = crate::vietnamese::charset::detect_charset_from_font(font);
            }
            (200, format_config_json(target_label, false, &cfg))
        }

        // ====================================================================
        // PROFILES LIST: /profiles
        // ====================================================================
        "/profiles" => {
            #[cfg(target_os = "windows")]
            {
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
                    if let Some(ref engine) = *guard {
                        let active_id = engine.active_profile_id();
                        let mut items = Vec::new();
                        items.push(format!(
                            "{{\"id\":\"global\",\"name\":\"Global\",\"is_active\":{}}}",
                            active_id == "global"
                        ));
                        for p in &engine.profile_manager.profiles {
                            let processes = p
                                .process_names
                                .iter()
                                .map(|n| format!("\"{}\"", n))
                                .collect::<Vec<_>>()
                                .join(",");
                            items.push(format!(
                                "{{\"id\":\"{}\",\"name\":\"{}\",\"process_names\":[{}],\"is_active\":{}}}",
                                p.id,
                                p.name,
                                processes,
                                active_id == p.id
                            ));
                        }
                        return (
                            200,
                            format!(
                                "{{\"status\":\"ok\",\"active_id\":\"{}\",\"profiles\":[{}]}}\n",
                                active_id,
                                items.join(",")
                            ),
                        );
                    }
                }
            }

            (
                200,
                "{\"status\":\"ok\",\"active_id\":\"global\",\"profiles\":[{\"id\":\"global\",\"name\":\"Global\",\"is_active\":true}]}\n"
                    .to_string(),
            )
        }

        // ====================================================================
        // STATUS & HEALTH: /status, /health, /
        // ====================================================================
        "/status" | "/health" | "/" => {
            let mut active_pid = "global".to_string();
            let mut active_charset = "unicode".to_string();

            #[cfg(target_os = "windows")]
            {
                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
                    if let Some(ref engine) = *guard {
                        active_pid = engine.active_profile_id().to_string();
                        active_charset = engine.effective_config().charset.as_str().to_string();
                    }
                }
            }

            (
                200,
                format!(
                    "{{\"status\":\"ok\",\"service\":\"kora-ipc\",\"port\":{},\"active_profile\":\"{}\",\"active_charset\":\"{}\"}}\n",
                    KORA_IPC_PORT, active_pid, active_charset
                ),
            )
        }

        _ => (
            404,
            "{\"status\":\"error\",\"message\":\"Not Found\"}\n".to_string(),
        ),
    }
}

/// Serves a single incoming client TCP stream
fn handle_client(stream: &mut TcpStream) {
    let mut buffer = [0u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request_str = match std::str::from_utf8(&buffer[..bytes_read]) {
        Ok(s) => s,
        Err(_) => return,
    };

    let (status_code, body) = match parse_http_request(request_str) {
        Some((method, path, params)) => handle_ipc_command(method, path, &params),
        None => (400, "{\"status\":\"error\",\"message\":\"Bad Request\"}\n".to_string()),
    };

    let status_line = match status_code {
        200 => "HTTP/1.1 200 OK",
        400 => "HTTP/1.1 400 Bad Request",
        404 => "HTTP/1.1 404 Not Found",
        405 => "HTTP/1.1 405 Method Not Allowed",
        _ => "HTTP/1.1 500 Internal Server Error",
    };

    let response = format!(
        "{}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        status_line,
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Starts the Kora IPC Server on the default port `13497`.
/// Runs on a dedicated background daemon thread.
pub fn start_ipc_server() -> std::io::Result<()> {
    start_ipc_server_on_port(KORA_IPC_PORT)
}

/// Starts the Kora IPC Server on a specific port.
pub fn start_ipc_server_on_port(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    std::thread::Builder::new()
        .name("kora-ipc-server".to_string())
        .spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                    handle_client(&mut stream);
                }
            }
        })?;
    Ok(())
}
