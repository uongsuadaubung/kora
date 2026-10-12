use crate::vietnamese::{
    charset::{BaseVowel, Diacritic, Tone},
    coda::CodaState,
    inline_list::InlineList,
    nucleus::NucleusState,
    onset::OnsetState,
    spelling::{is_valid_onset, is_valid_vietnamese_components},
    syllable::Syllable,
};

/// State pattern: Representation of Vietnamese Syllable Parser State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyllableState {
    #[default]
    Empty,
    Onset(OnsetState),
    Nucleus(NucleusState),
    Coda(CodaState),
    Passthrough(InlineList<char, 16>),
}

impl SyllableState {
    /// Returns a compact summary string of the state for debugging
    pub fn summary(&self) -> String {
        match self {
            SyllableState::Empty => "Empty".to_string(),
            SyllableState::Onset(onset) => format!("Onset(\"{onset}\")"),
            SyllableState::Nucleus(nucleus) => format!("Nucleus(\"{nucleus}\")"),
            SyllableState::Coda(coda) => format!("Coda(\"{coda}\")"),
            SyllableState::Passthrough(raw) => {
                let s: String = raw.iter().collect();
                format!("Passthrough(\"{s}\")")
            }
        }
    }

    /// Checks whether this syllable state represents a valid Vietnamese spelling without heap allocations.
    pub fn is_valid_spelling(&self) -> bool {
        match self {
            SyllableState::Empty => true,
            SyllableState::Onset(onset) => is_valid_onset(&onset.chars, onset.is_d_stroke),
            SyllableState::Nucleus(nucleus) => {
                let onset_slice = nucleus
                    .onset
                    .as_ref()
                    .map(|o| o.chars.as_slice())
                    .unwrap_or(&[]);
                let d_stroke = nucleus.onset.as_ref().is_some_and(|o| o.is_d_stroke);
                is_valid_vietnamese_components(
                    onset_slice,
                    d_stroke,
                    &nucleus.vowels,
                    nucleus.tone,
                    &[],
                )
            }
            SyllableState::Coda(coda) => {
                let onset_slice = coda
                    .nucleus
                    .onset
                    .as_ref()
                    .map(|o| o.chars.as_slice())
                    .unwrap_or(&[]);
                let d_stroke = coda.nucleus.onset.as_ref().is_some_and(|o| o.is_d_stroke);
                is_valid_vietnamese_components(
                    onset_slice,
                    d_stroke,
                    &coda.nucleus.vowels,
                    coda.nucleus.tone,
                    &coda.coda,
                )
            }
            SyllableState::Passthrough(raw) => raw.iter().all(|c| c.is_ascii()),
        }
    }

    /// Checks whether this state represents an unambiguous, completed Vietnamese syllable
    /// that must NEVER be destructively rolled back into raw keys when an invalid key arrives.
    ///
    /// When an extra character arrives after an unambiguous Vietnamese syllable (e.g. "nấu" + 'c' -> "nấuc",
    /// "hiện" + 'e' -> "hiệne", "bạn" + 'k' -> "bạnk", "được" + 'w' -> "đượcw"),
    /// the syllable is preserved as passthrough so that hitting Backspace immediately restores
    /// the original Vietnamese word without screen flicker or raw key destruction.
    pub fn is_unambiguous_completed_vietnamese(&self) -> bool {
        if !self.is_valid_spelling() {
            return false;
        }

        let (nucleus, coda) = match self {
            SyllableState::Nucleus(n) => (n, &[][..]),
            SyllableState::Coda(c) => (&c.nucleus, c.coda.as_slice()),
            _ => return false,
        };

        let onset = nucleus.onset.as_ref();
        let vowels = &nucleus.vowels;
        let tone = nucleus.tone;

        // 1. D-stroke ('đ'): purely Vietnamese, impossible in English.
        if onset.is_some_and(|o| o.is_d_stroke) {
            return true;
        }

        // 2. Horn ('ơ', 'ư') or Breve ('ă'): purely Vietnamese diacritics.
        // Exception for English words starting with 'w' (which Telex interprets as standalone 'ư'):
        // When onset is None, words starting with 'w' are English unless they form a valid Vietnamese
        // word (with coda like "ước", "ương", or "ưa", or with tone like "ừ", "ứ").
        // Bare 'w' in "what", "write", "win", "word", "work" must rollback cleanly.
        if vowels.iter().any(|v| {
            matches!(
                v.diacritic,
                Diacritic::Horn | Diacritic::Breve
            )
        }) {
            if onset.is_none() && vowels[0].base == BaseVowel::U && vowels[0].diacritic == Diacritic::Horn {
                if coda.is_empty() {
                    let is_ua = vowels.len() == 2 && vowels[1].base == BaseVowel::A;
                    let is_valid_standalone_u = vowels.len() == 1
                        && matches!(tone, Tone::Grave | Tone::Acute);
                    if !is_ua && !is_valid_standalone_u {
                        return false;
                    }
                }
            } else {
                let is_bare_uo_horn = vowels.len() == 2
                    && vowels[0].base == BaseVowel::U
                    && vowels[0].diacritic == Diacritic::Horn
                    && vowels[1].base == BaseVowel::O
                    && vowels[1].diacritic == Diacritic::Horn;

                if is_bare_uo_horn && coda.is_empty() {
                    return false;
                }
            }
            return true;
        }

        // 3. Circumflex on 'A' ('â' from Telex 'aa' / VNI '6'): purely Vietnamese.
        if vowels.iter().any(|v| {
            v.base == BaseVowel::A && v.diacritic == Diacritic::Circumflex
        }) {
            return true;
        }

        // 4. Tone DotBelow (nặng / Telex 'j' / VNI '5'): purely Vietnamese.
        if tone == Tone::DotBelow {
            return true;
        }

        // 5. Circumflex on 'E' or 'O' ('ê', 'ô'):
        // In English, 'ee' and 'oo' are common (e.g. "meet", "good", "cool").
        // But once 'ê' or 'ô' has a tone (e.g. "bố", "đến", "mến") or a coda (e.g. "công", "tiếng", "bông"),
        // it cannot be an English 'good' or 'meet'!
        if vowels.iter().any(|v| {
            matches!(v.base, BaseVowel::E | BaseVowel::O) && v.diacritic == Diacritic::Circumflex
        }) && (tone != Tone::None || !coda.is_empty())
        {
            return true;
        }

        // 6. Vietnamese Diphthong / Triphthong with Tone (e.g. "nấu", "mái", "chào", "nghỉ", "ngày"):
        // English words with two plain vowels like "mail", "boat", "coin" have Tone::None.
        // A multi-vowel syllable with a tone is unambiguous Vietnamese.
        if vowels.len() >= 2 && tone != Tone::None {
            if onset.is_none()
                && vowels[0].base == BaseVowel::U
                && coda.is_empty()
                && vowels[1].base != BaseVowel::A
            {
                return false;
            }
            let is_bare_uo = vowels.len() == 2
                && vowels[0].base == BaseVowel::U
                && vowels[1].base == BaseVowel::O;

            if !(is_bare_uo && coda.is_empty()) {
                return true;
            }
        }

        // 7. Completed Coda with Tone (e.g. "bóng", "sách", "toán", "chính", "mát"):
        // Exception: English '-st' words (e.g. "test", "list", "post") where a single plain vowel
        // was marked with acute tone 's' followed by coda 't' (e.g. "tét" in "testing", "lít" in "listing").
        if !coda.is_empty() && tone != Tone::None {
            let is_english_st_pattern = vowels.len() == 1
                && vowels[0].diacritic == Diacritic::None
                && tone == Tone::Acute
                && coda.len() == 1
                && coda[0].0.eq_ignore_ascii_case(&'t');

            if !is_english_st_pattern {
                return true;
            }
        }

        false
    }

    /// Converts this state into a Syllable for validation, if it is a structured syllable state
    pub fn to_syllable(&self) -> Option<Syllable> {
        match self {
            SyllableState::Empty => None,
            SyllableState::Onset(onset) => Some(Syllable {
                onset: onset.chars.to_vec(),
                d_stroke: onset.is_d_stroke,
                vowels: Vec::new(),
                tone: Tone::None,
                coda: Vec::new(),
            }),
            SyllableState::Nucleus(nucleus) => Some(nucleus.into()),
            SyllableState::Coda(coda) => Some(coda.into()),
            SyllableState::Passthrough(_) => None,
        }
    }
}

