//! Soundpack Downloader & Installer for Kora
//! Downloads the 13 mechanical switch soundpack from GitHub and installs it to ~/.config/kora/switches/

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const SOUNDPACK_URL: &str =
    "https://github.com/uongsuadaubung/kora/archive/refs/heads/main.zip";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundpackInstallStatus {
    Success {
        switches_count: usize,
        path: String,
    },
    Error(String),
}

/// Downloads the switches soundpack archive from GitHub and extracts it into the ~/.config/kora/switches/ directory.
pub fn download_and_install_soundpack() -> SoundpackInstallStatus {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(&home).join(".config").join("kora");
    let dest_switches_dir = config_dir.join("switches");

    // 1. Ensure config directory exists
    if let Err(e) = fs::create_dir_all(&config_dir) {
        return SoundpackInstallStatus::Error(format!("Không thể tạo thư mục cấu hình: {e}"));
    }

    let temp_dir = std::env::temp_dir();
    let run_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(12345);
    let temp_zip = temp_dir.join(format!("kora_soundpack_{run_id}.zip"));
    let temp_extract = temp_dir.join(format!("kora_extract_{run_id}"));

    // 2. Download archive using curl.exe
    if let Err(e) = run_curl_download(SOUNDPACK_URL, &temp_zip) {
        cleanup_temp(&temp_zip, &temp_extract);
        return SoundpackInstallStatus::Error(e);
    }

    // 3. Extract archive using tar.exe or powershell Expand-Archive
    if let Err(e) = extract_zip(&temp_zip, &temp_extract) {
        cleanup_temp(&temp_zip, &temp_extract);
        return SoundpackInstallStatus::Error(e);
    }

    // 4. Find extracted switches directory
    let found_switches = find_switches_dir(&temp_extract);
    let source_switches = match found_switches {
        Some(p) => p,
        None => {
            cleanup_temp(&temp_zip, &temp_extract);
            return SoundpackInstallStatus::Error(
                "Không tìm thấy thư mục switches trong file tải về từ GitHub.".to_string(),
            );
        }
    };

    // 5. Copy switches to dest_switches_dir
    if let Err(e) = copy_dir_all(&source_switches, &dest_switches_dir) {
        cleanup_temp(&temp_zip, &temp_extract);
        return SoundpackInstallStatus::Error(format!("Lỗi khi cài đặt thư mục switches: {e}"));
    }

    // 6. Cleanup temp files
    cleanup_temp(&temp_zip, &temp_extract);

    // 7. Verify switches count
    let count = crate::engine::config_store::list_switch_profiles().len();
    if count == 0 {
        return SoundpackInstallStatus::Error(
            "Cài đặt hoàn tất nhưng không phát hiện bộ switch nào trong thư mục.".to_string(),
        );
    }

    SoundpackInstallStatus::Success {
        switches_count: count,
        path: dest_switches_dir.to_string_lossy().to_string(),
    }
}

fn run_curl_download(url: &str, output_path: &Path) -> Result<(), String> {
    let mut cmd = Command::new("curl.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd
        .args([
            "-f",
            "-sL",
            "--max-time",
            "180",
            "-H",
            "User-Agent: Kora-Soundpack-Installer",
            url,
            "-o",
        ])
        .arg(output_path)
        .output()
        .map_err(|e| format!("Không thể chạy curl.exe: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Tải file thất bại (curl mã lỗi {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    if !output_path.is_file() {
        return Err("File tải về không tồn tại.".to_string());
    }

    let size = fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);
    if size < 1000 {
        return Err("File tải về không hợp lệ hoặc kích thước quá nhỏ.".to_string());
    }

    Ok(())
}

fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let _ = fs::create_dir_all(dest_dir);

    // Try tar.exe first (native in Windows 10/11)
    let mut tar_cmd = Command::new("tar.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        tar_cmd.creation_flags(0x08000000);
    }
    tar_cmd.arg("-xf").arg(zip_path).arg("-C").arg(dest_dir);

    if let Ok(output) = tar_cmd.output()
        && output.status.success()
    {
        return Ok(());
    }

    // Fallback: PowerShell Expand-Archive
    let mut ps_cmd = Command::new("powershell.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        ps_cmd.creation_flags(0x08000000);
    }
    let script = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        zip_path.to_string_lossy(),
        dest_dir.to_string_lossy()
    );
    let output = ps_cmd
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
        .map_err(|e| format!("Không thể giải nén file zip: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Giải nén thất bại: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn find_switches_dir(base: &Path) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.eq_ignore_ascii_case("switches") || name.eq_ignore_ascii_case("switchs") {
                    return Some(path);
                }
                if let Some(sub) = find_switches_dir(&path) {
                    return Some(sub);
                }
            }
        }
    }
    None
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn cleanup_temp(zip: &Path, extract: &Path) {
    let _ = fs::remove_file(zip);
    let _ = fs::remove_dir_all(extract);
}

/// Displays an informational or warning MessageBox with the install result
pub fn show_soundpack_install_dialog(hwnd: isize, status: &SoundpackInstallStatus) {
    let strings = crate::language::current();
    match status {
        SoundpackInstallStatus::Success {
            switches_count,
            path,
        } => {
            let msg = format!(
                "{}\n\n• Số lượng: {} bộ switch\n• Thư mục cài đặt: {}\n\nBạn có thể chọn switch và bấm \"Nghe thử\" ngay bây giờ!",
                strings.soundpack_download_success, switches_count, path
            );
            unsafe {
                crate::platform::win32::MessageBoxW(
                    hwnd,
                    crate::ui::win32::to_wide(&msg).as_ptr(),
                    crate::ui::win32::to_wide("Kora — Âm thanh phím cơ").as_ptr(),
                    0x00000040 /* MB_ICONINFORMATION */ | 0x00000000 /* MB_OK */,
                );
            }
        }
        SoundpackInstallStatus::Error(err) => {
            let msg = format!("{}\n\nChi tiết lỗi: {}", strings.soundpack_download_error, err);
            unsafe {
                crate::platform::win32::MessageBoxW(
                    hwnd,
                    crate::ui::win32::to_wide(&msg).as_ptr(),
                    crate::ui::win32::to_wide("Kora — Lỗi tải âm thanh").as_ptr(),
                    0x00000030 /* MB_ICONWARNING */ | 0x00000000 /* MB_OK */,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundpack_url() {
        assert!(SOUNDPACK_URL.starts_with("https://github.com/"));
        assert!(SOUNDPACK_URL.ends_with(".zip"));
    }

    #[test]
    fn test_find_switches_dir() {
        let temp_dir = std::env::temp_dir().join("kora_test_find_switches");
        let switches_sub = temp_dir.join("kora-main").join("switches");
        let _ = fs::create_dir_all(&switches_sub);

        let found = find_switches_dir(&temp_dir);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), switches_sub);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
