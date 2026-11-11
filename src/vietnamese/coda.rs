//! Vietnamese Coda (Phụ âm cuối) State Representation

use super::charset::{BaseVowel, Diacritic, Tone, compose_vowel, compose_vowel_u16};
use super::inline_list::InlineList;
use super::nucleus::NucleusState;
use super::syllable::Syllable;
use std::fmt;

/// Coda (Phụ âm cuối) — compact 25-byte stack representation.
/// Vietnamese codas have at most 2 ASCII consonants (`ng`, `nh`, `ch`), stored with exact case in `[u8; 2]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodaState {
    pub nucleus: NucleusState,
    pub coda: InlineList<u8, 2>,
    pub circumflex_across_coda: Option<BaseVowel>,
}

impl CodaState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn from_single(nucleus: NucleusState, ch: char, is_upper: bool) -> Self {
        let b = (ch as u8).to_ascii_lowercase();
        let cased = if is_upper { b.to_ascii_uppercase() } else { b };
        Self {
            nucleus,
            coda: InlineList::from_single(cased),
            circumflex_across_coda: None,
        }
    }

    #[inline]
    pub fn to_syllable(&self) -> Syllable {
        self.into()
    }

    /// When there is a coda, tone is placed according to coda rules:
    /// Single vowel: index 0.
    /// Triphthong with final diacritic (e.g. uyên, uyết): index 2.
    /// Otherwise (diphthongs or general triphthongs): index 1.
    #[inline(always)]
    fn tone_position(&self) -> usize {
        if self.nucleus.tone == Tone::None {
            0
        } else {
            let n = self.nucleus.vowels.len();
            if n <= 1 {
                0
            } else if n == 3 && self.nucleus.vowels[2].diacritic != Diacritic::None {
                2
            } else {
                1
            }
        }
    }

    #[inline]
    pub fn render_chars_to(&self, out: &mut InlineList<char, 16>) {
        if let Some(ref onset) = self.nucleus.onset {
            onset.render_chars_to(out);
        }

        let tone_pos = self.tone_position();
        for (i, v) in self.nucleus.vowels.iter().enumerate() {
            let tone_for_vowel = if i == tone_pos {
                self.nucleus.tone
            } else {
                Tone::None
            };
            out.push(compose_vowel(
                v.base,
                v.diacritic,
                tone_for_vowel,
                v.is_upper,
            ));
        }

        for &b in &self.coda {
            out.push(b as char);
        }
    }

    /// Renders coda state directly into a stack-allocated CompactString (0 heap allocations).
    #[inline(always)]
    pub fn render_compact(&self) -> crate::engine::action::CompactString {
        let mut s = crate::engine::action::CompactString::new();
        if let Some(ref onset) = self.nucleus.onset {
            onset.render_compact_to(&mut s);
        }

        let tone_pos = self.tone_position();
        for (i, v) in self.nucleus.vowels.iter().enumerate() {
            let tone_for_vowel = if i == tone_pos {
                self.nucleus.tone
            } else {
                Tone::None
            };
            s.push_bmp_u16(compose_vowel_u16(
                v.base,
                v.diacritic,
                tone_for_vowel,
                v.is_upper,
            ));
        }

        s.push_ascii_bytes(self.coda.as_slice());
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

impl fmt::Display for CodaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render_compact())
    }
}

impl From<&CodaState> for Syllable {
    fn from(coda: &CodaState) -> Self {
        Syllable {
            onset: coda
                .nucleus
                .onset
                .as_ref()
                .map(|o| {
                    o.chars
                        .iter()
                        .map(|&b| (b as char, b.is_ascii_uppercase()))
                        .collect()
                })
                .unwrap_or_default(),
            d_stroke: coda.nucleus.onset.as_ref().is_some_and(|o| o.is_d_stroke),
            vowels: coda.nucleus.vowels.to_vec(),
            tone: coda.nucleus.tone,
            coda: coda
                .coda
                .iter()
                .map(|&b| (b as char, b.is_ascii_uppercase()))
                .collect(),
        }
    }
}
