//! Vietnamese Onset (Phụ âm đầu) State Representation

use super::charset::compose_d;
use super::inline_list::InlineList;
use std::fmt;

/// Onset (Phụ âm đầu)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnsetState {
    pub chars: InlineList<(char, bool), 4>, // (char, is_upper)
    pub is_d_stroke: bool,                  // 'đ' / 'Đ'
}

impl OnsetState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_single(ch: char, is_upper: bool, is_d_stroke: bool) -> Self {
        Self {
            chars: InlineList::from_single((ch, is_upper)),
            is_d_stroke,
        }
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
    pub fn render_to(&self, out: &mut String) {
        for &(c, is_upper) in &self.chars {
            if c.eq_ignore_ascii_case(&'d') {
                out.push(compose_d(self.is_d_stroke, is_upper));
            } else if is_upper {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c.to_ascii_lowercase());
            }
        }
    }

    #[inline]
    pub fn render_chars_to(&self, out: &mut InlineList<char, 16>) {
        for &(c, is_upper) in &self.chars {
            if c.eq_ignore_ascii_case(&'d') {
                out.push(compose_d(self.is_d_stroke, is_upper));
            } else if is_upper {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c.to_ascii_lowercase());
            }
        }
    }

    #[inline]
    pub fn render_compact(&self) -> crate::engine::action::CompactString {
        let mut s = crate::engine::action::CompactString::new();
        for &(c, is_upper) in &self.chars {
            if c.eq_ignore_ascii_case(&'d') {
                s.push(compose_d(self.is_d_stroke, is_upper));
            } else if is_upper {
                s.push(c.to_ascii_uppercase());
            } else {
                s.push(c.to_ascii_lowercase());
            }
        }
        s
    }

    #[inline]
    pub fn render(&self) -> String {
        let mut s = String::with_capacity(8);
        self.render_to(&mut s);
        s
    }
}

impl fmt::Display for OnsetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(8);
        self.render_to(&mut s);
        f.write_str(&s)
    }
}
