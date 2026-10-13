//! Vietnamese Coda (Phụ âm cuối) State Representation

use super::charset::{BaseVowel, Diacritic, Tone, compose_d, compose_vowel};
use super::inline_list::InlineList;
use super::nucleus::NucleusState;
use super::syllable::Syllable;
use std::fmt;

/// Coda (Phụ âm cuối)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodaState {
    pub nucleus: NucleusState,
    pub coda: InlineList<(char, bool), 4>,
    pub circumflex_across_coda: Option<BaseVowel>,
}

impl CodaState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_single(nucleus: NucleusState, ch: char, is_upper: bool) -> Self {
        Self {
            nucleus,
            coda: InlineList::from_single((ch, is_upper)),
            circumflex_across_coda: None,
        }
    }

    #[inline]
    pub fn to_syllable(&self) -> Syllable {
        self.into()
    }

    #[inline]
    pub fn render_to(&self, out: &mut String) {
        if let Some(ref onset) = self.nucleus.onset {
            onset.render_to(out);
        }

        // When there is a coda, tone is placed according to coda rules:
        // Single vowel: index 0.
        // Triphthong with final diacritic (e.g. uyên, uyết): index 2.
        // Otherwise (diphthongs or general triphthongs): index 1.
        let tone_pos = if self.nucleus.tone == Tone::None {
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
        };

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

        for &(c, is_upper) in &self.coda {
            if is_upper {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c.to_ascii_lowercase());
            }
        }
    }

    #[inline]
    pub fn render_chars_to(&self, out: &mut InlineList<char, 16>) {
        if let Some(ref onset) = self.nucleus.onset {
            onset.render_chars_to(out);
        }

        let tone_pos = if self.nucleus.tone == Tone::None {
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
        };

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

        for &(c, is_upper) in &self.coda {
            if is_upper {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c.to_ascii_lowercase());
            }
        }
    }

    /// Renders coda state directly into a stack-allocated CompactString (0 heap allocations).
    #[inline]
    pub fn render_compact(&self) -> crate::engine::action::CompactString {
        let mut s = crate::engine::action::CompactString::new();
        if let Some(ref onset) = self.nucleus.onset {
            for &(c, is_upper) in &onset.chars {
                if c.eq_ignore_ascii_case(&'d') {
                    s.push(compose_d(onset.is_d_stroke, is_upper));
                } else if is_upper {
                    s.push(c.to_ascii_uppercase());
                } else {
                    s.push(c.to_ascii_lowercase());
                }
            }
        }

        let tone_pos = if self.nucleus.tone == Tone::None {
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
        };

        for (i, v) in self.nucleus.vowels.iter().enumerate() {
            let tone_for_vowel = if i == tone_pos {
                self.nucleus.tone
            } else {
                Tone::None
            };
            s.push(compose_vowel(
                v.base,
                v.diacritic,
                tone_for_vowel,
                v.is_upper,
            ));
        }

        for &(c, is_upper) in &self.coda {
            if is_upper {
                s.push(c.to_ascii_uppercase());
            } else {
                s.push(c.to_ascii_lowercase());
            }
        }
        s
    }

    #[inline]
    pub fn render(&self) -> String {
        let mut s = String::with_capacity(20);
        self.render_to(&mut s);
        s
    }
}

impl fmt::Display for CodaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(20);
        self.render_to(&mut s);
        f.write_str(&s)
    }
}

impl From<&CodaState> for Syllable {
    fn from(coda: &CodaState) -> Self {
        Syllable {
            onset: coda
                .nucleus
                .onset
                .as_ref()
                .map(|o| o.chars.to_vec())
                .unwrap_or_default(),
            d_stroke: coda.nucleus.onset.as_ref().is_some_and(|o| o.is_d_stroke),
            vowels: coda.nucleus.vowels.to_vec(),
            tone: coda.nucleus.tone,
            coda: coda.coda.to_vec(),
        }
    }
}
