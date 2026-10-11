//! Automatic update checker for Kora using native system curl.exe
//! Queries GitHub Releases to detect newer versions and displays interactive update prompts.

use std::process::Command;

pub const GITHUB_REPO: &str = "uongsuadaubung/kora";
pub const GITHUB_API_URL: &str = "https://api.github.com/repos/uongsuadaubung/kora/releases/latest";
pub const GITHUB_RELEASES_URL: &str = "https://github.com/uongsuadaubung/kora/releases/latest";

/// Status of the version check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate(String),
    NewVersionAvailable {
        current_version: String,
        latest_version: String,
        release_url: String,
    },
    Error(String),
}

/// Checks GitHub for newer versions of Kora using system curl.exe
pub fn check_for_updates() -> UpdateStatus {
    let current_version = env!("CARGO_PKG_VERSION");

    // 1. Try querying GitHub Releases API via curl.exe
    match run_curl_api() {
        Ok(json) => {
            if let Some(tag_name) = parse_tag_name(&json) {
                let release_url =
                    parse_html_url(&json).unwrap_or_else(|| GITHUB_RELEASES_URL.to_string());
                return evaluate_version(current_version, &tag_name, release_url);
            }
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            eprintln!("[Updater] API query failed: {_e}");
        }
    }

    // 2. Fallback: Query HTTP headers via `curl -sI` (redirect location), avoiding API rate limits
    match run_curl_redirect() {
        Ok(headers) => {
            if let Some(tag_name) = parse_tag_from_location(&headers) {
                return evaluate_version(
                    current_version,
                    &tag_name,
                    GITHUB_RELEASES_URL.to_string(),
                );
            }
        }
        Err(e) => {
            return UpdateStatus::Error(format!("Không thể kết nối đến máy chủ GitHub: {e}"));
        }
    }

    UpdateStatus::Error("Không thể tìm thấy thông tin phiên bản mới nhất từ GitHub.".to_string())
}

fn evaluate_version(current_version: &str, latest_tag: &str, release_url: String) -> UpdateStatus {
    if is_newer_version(current_version, latest_tag) {
        UpdateStatus::NewVersionAvailable {
            current_version: current_version.to_string(),
            latest_version: latest_tag.to_string(),
            release_url,
        }
    } else {
        UpdateStatus::UpToDate(current_version.to_string())
    }
}

fn run_curl_api() -> std::io::Result<String> {
    let mut cmd = Command::new("curl.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd
        .args([
            "-s",
            "--max-time",
            "10",
            "-H",
            "User-Agent: Kora-Updater",
            GITHUB_API_URL,
        ])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::other(format!(
            "curl exited with code {:?}",
            output.status.code()
        )))
    }
}

fn run_curl_redirect() -> std::io::Result<String> {
    let mut cmd = Command::new("curl.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd
        .args([
            "-sI",
            "--max-time",
            "10",
            "-H",
            "User-Agent: Kora-Updater",
            GITHUB_RELEASES_URL,
        ])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::other(format!(
            "curl exited with code {:?}",
            output.status.code()
        )))
    }
}

pub fn parse_tag_name(json: &str) -> Option<String> {
    let key = "\"tag_name\"";
    let pos = json.find(key)?;
    let after_key = &json[pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = &after_key[colon_pos + 1..];
    let quote_start = after_colon.find('"')?;
    let rest = &after_colon[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].trim().to_string())
}

pub fn parse_html_url(json: &str) -> Option<String> {
    let key = "\"html_url\"";
    let pos = json.find(key)?;
    let after_key = &json[pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = &after_key[colon_pos + 1..];
    let quote_start = after_colon.find('"')?;
    let rest = &after_colon[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].trim().to_string())
}

pub fn parse_tag_from_location(headers: &str) -> Option<String> {
    for line in headers.lines() {
        let line = line.trim();
        if line.to_ascii_lowercase().starts_with("location:")
            && let Some(pos) = line.rfind("/tag/")
        {
            return Some(line[pos + 5..].trim().to_string());
        }
    }
    None
}

pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let mut parts = s.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u32>().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some((c_maj, c_min, c_pat)), Some((l_maj, l_min, l_pat))) => {
            (l_maj, l_min, l_pat) > (c_maj, c_min, c_pat)
        }
        _ => false,
    }
}

/// Displays an interactive Windows MessageBox with the update result
#[cfg(windows)]
pub fn show_update_result_dialog(hwnd: isize, status: &UpdateStatus) {
    use crate::platform::win32::types::{
        IDYES, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_YESNO, MessageBoxW,
    };
    use crate::ui::win32::to_wide;

    let strings = crate::language::current();

    match status {
        UpdateStatus::NewVersionAvailable {
            current_version,
            latest_version,
            release_url,
        } => {
            let msg = strings
                .update_available_msg
                .replace("{latest}", latest_version)
                .replace("{current}", current_version);
            let title = strings.update_title;

            let ret = unsafe {
                MessageBoxW(
                    hwnd,
                    to_wide(&msg).as_ptr(),
                    to_wide(title).as_ptr(),
                    MB_YESNO | MB_ICONINFORMATION,
                )
            };

            if ret == IDYES {
                let _ = std::process::Command::new("rundll32.exe")
                    .args(["url.dll,FileProtocolHandler", release_url])
                    .spawn();
            }
        }
        UpdateStatus::UpToDate(current_version) => {
            let msg = strings
                .update_latest_msg
                .replace("{current}", current_version);
            let title = strings.update_title;

            unsafe {
                MessageBoxW(
                    hwnd,
                    to_wide(&msg).as_ptr(),
                    to_wide(title).as_ptr(),
                    MB_OK | MB_ICONINFORMATION,
                );
            }
        }
        UpdateStatus::Error(err) => {
            let msg = strings
                .update_error_msg
                .replace("{error}", err);
            let title = strings.update_error_title;

            unsafe {
                MessageBoxW(
                    hwnd,
                    to_wide(&msg).as_ptr(),
                    to_wide(title).as_ptr(),
                    MB_OK | MB_ICONWARNING,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_version("v0.3.0"), Some((0, 3, 0)));
        assert_eq!(parse_version("V1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.0"), Some((1, 0, 0)));
        assert_eq!(parse_version("invalid"), None);
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.3.0", "v0.3.1"));
        assert!(is_newer_version("0.3.0", "v0.4.0"));
        assert!(is_newer_version("0.3.0", "v1.0.0"));
        assert!(!is_newer_version("0.3.0", "v0.3.0"));
        assert!(!is_newer_version("0.3.0", "v0.2.9"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
    }

    #[test]
    fn test_parse_json_tag_and_url() {
        let json = r#"{"tag_name":"v0.3.5","html_url":"https://github.com/uongsuadaubung/kora/releases/tag/v0.3.5"}"#;
        assert_eq!(parse_tag_name(json), Some("v0.3.5".to_string()));
        assert_eq!(
            parse_html_url(json),
            Some("https://github.com/uongsuadaubung/kora/releases/tag/v0.3.5".to_string())
        );
    }

    #[test]
    fn test_parse_location_header() {
        let headers = "HTTP/1.1 302 Found\r\nLocation: https://github.com/uongsuadaubung/kora/releases/tag/v0.4.0\r\nContent-Length: 0";
        assert_eq!(parse_tag_from_location(headers), Some("v0.4.0".to_string()));
    }
}
