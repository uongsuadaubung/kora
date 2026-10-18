//! Hotkey definition, VK parsing, and formatting for typing mode switch

/// Key combination used to switch typing modes (Vietnamese / English)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
    pub vk: u32,
}

impl Default for Hotkey {
    fn default() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            win: false,
            vk: 0,
        }
    }
}

impl Hotkey {
    pub const fn new(ctrl: bool, shift: bool, alt: bool, win: bool, vk: u32) -> Self {
        Self {
            ctrl,
            shift,
            alt,
            win,
            vk,
        }
    }

    pub fn is_ctrl_shift(&self) -> bool {
        self.ctrl && self.shift && !self.alt && !self.win && self.vk == 0
    }

    pub fn is_valid(&self) -> bool {
        if self.vk == 0 {
            let mod_count = self.ctrl as u8 + self.shift as u8 + self.alt as u8 + self.win as u8;
            mod_count >= 2
        } else {
            self.ctrl || self.shift || self.alt || self.win || (0x70..=0x7B).contains(&self.vk)
        }
    }

    pub fn display_text(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.win {
            parts.push("Win");
        }
        if self.vk != 0 {
            let name = match self.vk {
                0xC0 => "~",
                _ => vk_to_name(self.vk),
            };
            if !name.is_empty() {
                parts.push(name);
            } else {
                parts.push("Key");
            }
        }
        if parts.is_empty() {
            "Ctrl + Shift".to_string()
        } else {
            parts.join(" + ")
        }
    }

    pub fn to_config_str(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("ctrl".to_string());
        }
        if self.alt {
            parts.push("alt".to_string());
        }
        if self.shift {
            parts.push("shift".to_string());
        }
        if self.win {
            parts.push("win".to_string());
        }
        if self.vk != 0 {
            let name = vk_to_name(self.vk).to_ascii_lowercase();
            if !name.is_empty() {
                parts.push(name);
            }
        }
        if parts.is_empty() {
            "ctrl_shift".to_string()
        } else {
            parts.join("_")
        }
    }

    pub fn from_config_str(s: &str) -> Self {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Self::default();
        }

        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut win = false;
        let mut vk = 0;

        let tokens: Vec<&str> = trimmed
            .split(['+', '_', '-', ' '])
            .filter(|t| !t.is_empty())
            .collect();

        for token in tokens {
            match token {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                "win" | "windows" => win = true,
                other => {
                    if let Some(code) = name_to_vk(other) {
                        vk = code;
                    }
                }
            }
        }

        let res = Self {
            ctrl,
            shift,
            alt,
            win,
            vk,
        };
        if res.is_valid() { res } else { Self::default() }
    }
}

pub fn vk_to_name(vk: u32) -> &'static str {
    match vk {
        0x20 => "Space",
        0xC0 => "~",
        0xBA => ";",
        0xBC => ",",
        0xBE => ".",
        0xBF => "/",
        0xDB => "[",
        0xDD => "]",
        0xDC => "\\",
        0xBD => "-",
        0xBB => "=",
        0x09 => "Tab",
        0x08 => "Backspace",
        0x0D => "Enter",
        0x1B => "Esc",
        0x70 => "F1",
        0x71 => "F2",
        0x72 => "F3",
        0x73 => "F4",
        0x74 => "F5",
        0x75 => "F6",
        0x76 => "F7",
        0x77 => "F8",
        0x78 => "F9",
        0x79 => "F10",
        0x7A => "F11",
        0x7B => "F12",
        0x30 => "0",
        0x31 => "1",
        0x32 => "2",
        0x33 => "3",
        0x34 => "4",
        0x35 => "5",
        0x36 => "6",
        0x37 => "7",
        0x38 => "8",
        0x39 => "9",
        0x41 => "A",
        0x42 => "B",
        0x43 => "C",
        0x44 => "D",
        0x45 => "E",
        0x46 => "F",
        0x47 => "G",
        0x48 => "H",
        0x49 => "I",
        0x4A => "J",
        0x4B => "K",
        0x4C => "L",
        0x4D => "M",
        0x4E => "N",
        0x4F => "O",
        0x50 => "P",
        0x51 => "Q",
        0x52 => "R",
        0x53 => "S",
        0x54 => "T",
        0x55 => "U",
        0x56 => "V",
        0x57 => "W",
        0x58 => "X",
        0x59 => "Y",
        0x5A => "Z",
        0x14 => "Caps Lock",
        0x2D => "Insert",
        0x2E => "Delete",
        0x24 => "Home",
        0x23 => "End",
        0x21 => "Page Up",
        0x22 => "Page Down",
        0x25 => "Left",
        0x26 => "Up",
        0x27 => "Right",
        0x28 => "Down",
        0x5B => "Left Windows",
        0x5C => "Right Windows",
        0x5D => "Apps",
        0x91 => "Scroll Lock",
        0x90 => "Num Lock",
        0x13 => "Pause",
        0x2C => "Print Screen",
        0x11 => "Ctrl",
        0xA2 => "Left Ctrl",
        0xA3 => "Right Ctrl",
        0x12 => "Alt",
        0xA4 => "Left Alt",
        0xA5 => "Right Alt",
        0x10 => "Shift",
        0xA0 => "Left Shift",
        0xA1 => "Right Shift",
        0xAD => "Mute",
        0xAE => "Volume Down",
        0xAF => "Volume Up",
        0xB0 => "Next Track",
        0xB1 => "Prev Track",
        0xB2 => "Media Stop",
        0xB3 => "Play/Pause",
        _ => "",
    }
}

pub fn name_to_vk(name: &str) -> Option<u32> {
    let lower = name.trim().to_ascii_lowercase();
    match lower.as_str() {
        "space" => Some(0x20),
        "~" | "`" | "tilde" | "grave" | "backtick" => Some(0xC0),
        ";" | "semicolon" => Some(0xBA),
        "," | "comma" => Some(0xBC),
        "." | "period" => Some(0xBE),
        "/" | "slash" => Some(0xBF),
        "[" => Some(0xDB),
        "]" => Some(0xDD),
        "\\" => Some(0xDC),
        "-" | "minus" => Some(0xBD),
        "=" | "plus" => Some(0xBB),
        "tab" => Some(0x09),
        "enter" => Some(0x0D),
        "esc" | "escape" => Some(0x1B),
        "caps" | "caps lock" | "capslock" => Some(0x14),
        "insert" | "ins" => Some(0x2D),
        "delete" | "del" => Some(0x2E),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "page up" | "pageup" | "pgup" => Some(0x21),
        "page down" | "pagedown" | "pgdn" => Some(0x22),
        "left" => Some(0x25),
        "up" => Some(0x26),
        "right" => Some(0x27),
        "down" => Some(0x28),
        "win" | "windows" | "left windows" | "lwin" => Some(0x5B),
        "right windows" | "rwin" => Some(0x5C),
        "apps" | "menu" => Some(0x5D),
        "scroll lock" | "scrolllock" => Some(0x91),
        "num lock" | "numlock" => Some(0x90),
        "pause" => Some(0x13),
        "print screen" | "printscreen" | "prtscn" => Some(0x2C),
        "ctrl" | "control" => Some(0x11),
        "left ctrl" | "lctrl" => Some(0xA2),
        "right ctrl" | "rctrl" => Some(0xA3),
        "alt" => Some(0x12),
        "left alt" | "lalt" => Some(0xA4),
        "right alt" | "ralt" => Some(0xA5),
        "shift" => Some(0x10),
        "left shift" | "lshift" => Some(0xA0),
        "right shift" | "rshift" => Some(0xA1),
        "mute" => Some(0xAD),
        "volume down" | "volumedown" => Some(0xAE),
        "volume up" | "volumeup" => Some(0xAF),
        "next track" => Some(0xB0),
        "prev track" => Some(0xB1),
        "stop" | "media stop" => Some(0xB2),
        "play/pause" | "play pause" | "play" => Some(0xB3),
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        "f3" => Some(0x72),
        "f4" => Some(0x73),
        "f5" => Some(0x74),
        "f6" => Some(0x75),
        "f7" => Some(0x76),
        "f8" => Some(0x77),
        "f9" => Some(0x78),
        "f10" => Some(0x79),
        "f11" => Some(0x7A),
        "f12" => Some(0x7B),
        s if s.len() == 1 => {
            let b = s.as_bytes()[0];
            if b.is_ascii_lowercase() {
                Some((b - b'a' + 0x41) as u32)
            } else if b.is_ascii_digit() {
                Some((b - b'0' + 0x30) as u32)
            } else {
                None
            }
        }
        _ => None,
    }
}
