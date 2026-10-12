//! Key Categorization & Physical Mapping for Mechanical Switch Acoustics
//!
//! Định danh các nhóm phím cơ bản (Space, Enter, Backspace, Modifiers...)
//! và ánh xạ mã phím ảo phần cứng (`Virtual-Key`) hoặc tên file âm thanh tương ứng.

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyTag {
    Normal = 0,
    Esc,
    Shift,
    Space,
    Enter,
    Backspace,
    Tab,
    CapsLock,
    Ctrl,
    Alt,
    Delete,
    Insert,
    Arrow,
    Win,
    Home,
    End,
    PageUp,
    PageDown,
    Fn,
}

pub const KEY_TAG_COUNT: usize = 19;

/// Maps a Windows Virtual-Key (VK) code to a canonical KeyTag enum in O(1) time
pub fn vk_to_tag(vk: u32) -> KeyTag {
    match vk {
        0x1B => KeyTag::Esc,
        0x10 | 0xA0 | 0xA1 => KeyTag::Shift,
        0x20 => KeyTag::Space,
        0x0D => KeyTag::Enter,
        0x08 => KeyTag::Backspace,
        0x09 => KeyTag::Tab,
        0x14 => KeyTag::CapsLock,
        0x11 | 0xA2 | 0xA3 => KeyTag::Ctrl,
        0x12 | 0xA4 | 0xA5 => KeyTag::Alt,
        0x2E => KeyTag::Delete,
        0x2D => KeyTag::Insert,
        0x25..=0x28 => KeyTag::Arrow,
        0x5B | 0x5C => KeyTag::Win,
        0x24 => KeyTag::Home,
        0x23 => KeyTag::End,
        0x21 => KeyTag::PageUp,
        0x22 => KeyTag::PageDown,
        0x70..=0x7B => KeyTag::Fn,
        _ => KeyTag::Normal,
    }
}

/// Matches a sound file stem name strictly to its canonical fixed key tag.
/// Eliminates substring ambiguities and keyword collisions completely.
pub fn match_file_tag(stem: &str) -> Option<KeyTag> {
    match stem {
        "space" => Some(KeyTag::Space),
        "backspace" => Some(KeyTag::Backspace),
        "enter" => Some(KeyTag::Enter),
        "tab" => Some(KeyTag::Tab),
        "delete" => Some(KeyTag::Delete),
        "shift" => Some(KeyTag::Shift),
        "ctrl" => Some(KeyTag::Ctrl),
        "alt" => Some(KeyTag::Alt),
        "win" => Some(KeyTag::Win),
        "capslock" => Some(KeyTag::CapsLock),
        "esc" => Some(KeyTag::Esc),
        "fn" => Some(KeyTag::Fn),
        "arrow" => Some(KeyTag::Arrow),
        "home" => Some(KeyTag::Home),
        "end" => Some(KeyTag::End),
        "pageup" => Some(KeyTag::PageUp),
        "pagedown" => Some(KeyTag::PageDown),
        "insert" => Some(KeyTag::Insert),
        _ => None,
    }
}

