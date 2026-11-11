//! Vietnamese Onset (Phụ âm đầu) State Representation

use super::charset::compose_d;
use super::inline_list::InlineList;
use std::fmt;

/// Onset (Phụ âm đầu) — compact 6-byte stack representation (`[u8; 4]` + `len` + `is_d_stroke`).
/// Each `u8` stores the ASCII consonant with its exact case preserved (`'b'`/`'B'`, `'n'`/`'N'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnsetState {
    pub chars: InlineList<u8, 4>,
    pub is_d_stroke: bool, // 'đ' / 'Đ'
}

impl OnsetState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn from_single(ch: char, is_upper: bool, is_d_stroke: bool) -> Self {
        let b = (ch as u8).to_ascii_lowercase();
        let cased = if is_upper { b.to_ascii_uppercase() } else { b };
        Self {
            chars: InlineList::from_single(cased),
            is_d_stroke,
        }
    }

    #[inline(always)]
    pub fn push_ascii(&mut self, ch: char, is_upper: bool) {
        let b = (ch as u8).to_ascii_lowercase();
        let cased = if is_upper { b.to_ascii_uppercase() } else { b };
        self.chars.push(cased);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    #[inline]
    pub fn render_chars_to(&self, out: &mut InlineList<char, 16>) {
        if self.is_d_stroke {
            for &b in &self.chars {
                if (b | 0x20) == b'd' {
                    out.push(compose_d(true, b.is_ascii_uppercase()));
                } else {
                    out.push(b as char);
                }
            }
        } else {
            for &b in &self.chars {
                out.push(b as char);
            }
        }
    }

    #[inline(always)]
    pub fn render_compact_to(&self, out: &mut crate::engine::action::CompactString) {
        if !self.is_d_stroke {
            out.push_ascii_bytes(self.chars.as_slice());
        } else {
            for &b in &self.chars {
                if (b | 0x20) == b'd' {
                    out.push_bmp_u16(if b.is_ascii_uppercase() { 0x0110 } else { 0x0111 });
                } else {
                    out.push_bmp_u16(b as u16);
                }
            }
        }
    }

    #[inline]
    pub fn render_compact(&self) -> crate::engine::action::CompactString {
        let mut s = crate::engine::action::CompactString::new();
        self.render_compact_to(&mut s);
        s
    }

    #[inline]
    pub fn render_to(&self, out: &mut String) {
        out.push_str(&self.render_compact());
    }

    #[inline]
    pub fn render(&self) -> String {
        self.render_compact().to_string()
    }
}

impl fmt::Display for OnsetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render_compact())
    }
}
