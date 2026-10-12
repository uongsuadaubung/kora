use super::{
    VowelLetter,
    charset::{BaseVowel, Diacritic, Tone, decompose_vowel, is_d_stroke},
    coda::CodaState,
    inline_list::InlineList,
    modifier::{KeyEffect, match_modifier_key},
    nucleus::{ModifierOutcome, NucleusState},
    onset::OnsetState,
    spelling::{
        can_vowels_accept_coda, is_special_k_coda_allowed_during_typing, is_valid_coda_pair,
        is_valid_coda_start, is_valid_coda_start_for_vowels,
        is_valid_intermediate_vowel_combination, is_valid_onset, is_valid_onset_extension,
        is_valid_onset_vowel_orthography, is_valid_vietnamese_components,
        is_valid_vowel_combination,
    },
    syllable::Syllable,
};
use crate::engine::{action::EngineAction, buffer::RawKey, config::EngineConfig};
use std::fmt;

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

impl fmt::Display for SyllableState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(16);
        self.render_to(&mut s);
        f.write_str(&s)
    }
}

impl SyllableState {
    /// Renders current state directly to a pre-allocated buffer with zero formatting overhead
    #[inline]
    pub fn render_to(&self, out: &mut String) {
        match self {
            SyllableState::Empty => {}
            SyllableState::Onset(onset) => onset.render_to(out),
            SyllableState::Nucleus(nucleus) => nucleus.render_to(out),
            SyllableState::Coda(coda) => coda.render_to(out),
            SyllableState::Passthrough(raw) => {
                for &ch in raw.iter() {
                    out.push(ch);
                }
            }
        }
    }

    /// Renders current state to a displayed string with pre-allocated capacity
    #[inline]
    pub fn render(&self) -> String {
        let mut s = String::with_capacity(16);
        self.render_to(&mut s);
        s
    }

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

    /// Parses an iterator of characters into the appropriate SyllableState
    pub fn parse_prefix_chars(
        raw: impl IntoIterator<Item = char>,
        config: &EngineConfig,
    ) -> SyllableState {
        let mut state = SyllableState::Empty;
        let mut current_len = 0;
        for ch in raw {
            let key = RawKey {
                ch,
                is_upper: ch.is_uppercase(),
            };
            let (next_state, action) = state.feed_key(key, current_len, config);
            state = next_state;
            match action {
                EngineAction::Replace { output, .. } => {
                    current_len = output.chars().count();
                }
                EngineAction::Passthrough => {
                    current_len += 1;
                }
                EngineAction::Consume => {}
            }
        }
        state
    }

    /// Parses a string into the appropriate SyllableState (Onset, Nucleus, Coda, or Passthrough)
    pub fn parse_prefix(raw: &str, config: &EngineConfig) -> SyllableState {
        Self::parse_prefix_chars(raw.chars(), config)
    }

    /// Transitions backward when Backspace is pressed:
    /// Drops the last rendered character on screen and transitions to the previous state cleanly.
    pub fn handle_backspace(self, config: &EngineConfig) -> SyllableState {
        match self {
            SyllableState::Empty => SyllableState::Empty,
            SyllableState::Onset(mut onset) => {
                onset.chars.pop();
                if onset.chars.is_empty() {
                    SyllableState::Empty
                } else {
                    SyllableState::Onset(onset)
                }
            }
            SyllableState::Nucleus(mut nucleus) => {
                let tone_pos_before = if nucleus.tone == Tone::None {
                    None
                } else {
                    Some(nucleus.find_tone_position())
                };

                nucleus.vowels.pop();

                if nucleus.vowels.is_empty() {
                    if let Some(mut onset) = nucleus.onset {
                        // If onset absorbed glide "gi" or "qu", break it back down into onset + single vowel
                        if onset.chars.len() == 2 {
                            let c0 = onset.chars[0].0.to_ascii_lowercase();
                            let c1 = onset.chars[1].0.to_ascii_lowercase();
                            if (c0 == 'g' && c1 == 'i') || (c0 == 'q' && c1 == 'u') {
                                let glide = onset.chars.pop().unwrap();
                                let base = if glide.0.eq_ignore_ascii_case(&'i') {
                                    BaseVowel::I
                                } else {
                                    BaseVowel::U
                                };
                                nucleus.onset = Some(onset);
                                nucleus.vowels = InlineList::from_single(VowelLetter {
                                    base,
                                    diacritic: Diacritic::None,
                                    is_upper: glide.1,
                                });
                                return SyllableState::Nucleus(nucleus);
                            }
                        }
                        SyllableState::Onset(onset)
                    } else {
                        SyllableState::Empty
                    }
                } else {
                    // If the popped vowel carried the tone mark, clear tone
                    if let Some(pos) = tone_pos_before
                        && pos >= nucleus.vowels.len()
                    {
                        nucleus.tone = Tone::None;
                    }
                    SyllableState::Nucleus(nucleus)
                }
            }
            SyllableState::Coda(mut coda) => {
                coda.coda.pop();
                if coda.coda.is_empty() {
                    SyllableState::Nucleus(coda.nucleus)
                } else {
                    SyllableState::Coda(coda)
                }
            }
            SyllableState::Passthrough(mut raw) => {
                raw.pop();
                if raw.is_empty() {
                    SyllableState::Empty
                } else {
                    Self::parse_prefix_chars(raw.iter().copied(), config)
                }
            }
        }
    }

    /// Process a new key and transit to the next state
    pub fn feed_key(
        self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (Self, EngineAction) {
        match self {
            SyllableState::Empty => Self::handle_empty(key, config),
            SyllableState::Onset(onset) => Self::handle_onset(onset, key, current_len, config),
            SyllableState::Nucleus(nucleus) => {
                Self::handle_nucleus(nucleus, key, current_len, config)
            }
            SyllableState::Coda(coda) => Self::handle_coda(coda, key, current_len, config),
            SyllableState::Passthrough(mut raw) => {
                raw.push(key.ch);
                (SyllableState::Passthrough(raw), EngineAction::Passthrough)
            }
        }
    }

    #[inline]
    fn try_bracket_w_shortcut(
        onset: Option<OnsetState>,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> Option<(Self, EngineAction)> {
        if config.bracket_w && config.method.has_bracket_shortcuts() {
            let base = match key.ch {
                '[' => BaseVowel::U,
                ']' => BaseVowel::O,
                _ => return None,
            };
            let nucleus = NucleusState::from_single(
                onset,
                VowelLetter {
                    base,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
            );
            let output = nucleus.render();
            return Some((
                SyllableState::Nucleus(nucleus),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            ));
        }
        None
    }

    fn handle_empty(key: RawKey, config: &EngineConfig) -> (Self, EngineAction) {
        let ch_lower = key.ch.to_ascii_lowercase();

        // 1. Bracket W shortcuts: [ -> ư, ] -> ơ
        if let Some(res) = Self::try_bracket_w_shortcut(None, key, 0, config) {
            return res;
        }

        // Standalone 'w' -> 'ư'
        if config.method.has_standalone_w() && ch_lower == 'w' {
            let nucleus = NucleusState::from_single(
                None,
                VowelLetter {
                    base: BaseVowel::U,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
            );
            let output = nucleus.render();
            return (
                SyllableState::Nucleus(nucleus),
                EngineAction::Replace {
                    backspaces: 0,
                    output,
                },
            );
        }

        // If auto-uppercased first letter (key.is_upper is true, but key.ch is lowercase)
        if key.is_upper && key.ch.is_lowercase() && key.ch.is_alphabetic() {
            let upper_char = key.ch.to_uppercase().next().unwrap_or(key.ch);
            if let Some(base_vowel) = BaseVowel::from_char(key.ch) {
                let nucleus = NucleusState::from_single(
                    None,
                    VowelLetter {
                        base: base_vowel,
                        diacritic: Diacritic::None,
                        is_upper: true,
                    },
                    Tone::None,
                );
                return (
                    SyllableState::Nucleus(nucleus),
                    EngineAction::Replace {
                        backspaces: 0,
                        output: String::from(upper_char),
                    },
                );
            } else {
                let onset = OnsetState::from_single(upper_char, true, false);
                return (
                    SyllableState::Onset(onset),
                    EngineAction::Replace {
                        backspaces: 0,
                        output: String::from(upper_char),
                    },
                );
            }
        }

        // If vowel
        if let Some((base_vowel, diacritic, tone)) = decompose_vowel(key.ch) {
            let nucleus = NucleusState::from_single(
                None,
                VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                },
                tone,
            );
            return (SyllableState::Nucleus(nucleus), EngineAction::Passthrough);
        }

        // 'đ' or 'Đ'
        if is_d_stroke(key.ch) {
            let onset = OnsetState::from_single('d', key.is_upper, true);
            return (SyllableState::Onset(onset), EngineAction::Passthrough);
        }

        // Non-Vietnamese initial consonants: 'f', 'j', 'z', and non-standalone 'w'
        let is_non_vi_consonant = matches!(ch_lower, 'f' | 'j' | 'z')
            || (!config.method.has_standalone_w() && ch_lower == 'w');
        if is_non_vi_consonant {
            let mut raw = InlineList::new();
            raw.push(key.ch);
            return (SyllableState::Passthrough(raw), EngineAction::Passthrough);
        }

        // If consonant
        if key.ch.is_alphabetic() {
            let onset = OnsetState::from_single(key.ch, key.is_upper, false);
            return (SyllableState::Onset(onset), EngineAction::Passthrough);
        }

        // Anything else -> Passthrough
        let mut raw = InlineList::new();
        raw.push(key.ch);
        (SyllableState::Passthrough(raw), EngineAction::Passthrough)
    }

    fn handle_onset(
        mut onset: OnsetState,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (Self, EngineAction) {
        let ch_lower = key.ch.to_ascii_lowercase();

        // 1. Bracket W shortcuts: [ -> ư, ] -> ơ
        if let Some(res) = Self::try_bracket_w_shortcut(Some(onset), key, current_len, config) {
            return res;
        }

        // 2. D-Stroke in Onset (Telex 'dd' or VNI 'd9' -> 'đ')
        let has_onset_d = !onset.chars.is_empty() && onset.chars[0].0.eq_ignore_ascii_case(&'d');
        if let Some(KeyEffect::DStroke) =
            match_modifier_key(config.method, key.ch, |_| false, has_onset_d)
        {
            if config.method.is_telex_family() && onset.is_d_stroke {
                // 3rd 'd' cancels 'đ' and restores double 'dd' into Passthrough!
                let first_d = if onset.chars[0].1 { 'D' } else { 'd' };
                let mut output = String::with_capacity(2);
                output.push(first_d);
                output.push(key.ch);
                let mut raw = InlineList::new();
                raw.push(first_d);
                raw.push(key.ch);
                return (
                    SyllableState::Passthrough(raw),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output,
                    },
                );
            } else {
                onset.is_d_stroke = !onset.is_d_stroke;
                let output = onset.render();
                return (
                    SyllableState::Onset(onset),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output,
                    },
                );
            }
        }

        // 2. Incoming vowel -> Transition from Onset to Nucleus!
        if config.method.has_standalone_w() && ch_lower == 'w' {
            if !is_valid_onset_vowel_orthography(&onset.chars, onset.is_d_stroke, BaseVowel::U) {
                let mut raw = InlineList::from(onset.render().as_str());
                raw.push(key.ch);
                return (SyllableState::Passthrough(raw), EngineAction::Passthrough);
            }
            // e.g. "tw" -> "tư" in Standard Telex
            let nucleus = NucleusState::from_single(
                Some(onset),
                VowelLetter {
                    base: BaseVowel::U,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
            );
            let output = nucleus.render();
            return (
                SyllableState::Nucleus(nucleus),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            );
        }

        if let Some((base_vowel, diacritic, tone)) = decompose_vowel(key.ch) {
            if !is_valid_onset_vowel_orthography(&onset.chars, onset.is_d_stroke, base_vowel) {
                let mut raw = InlineList::from(onset.render().as_str());
                raw.push(key.ch);
                return (SyllableState::Passthrough(raw), EngineAction::Passthrough);
            }
            // If onset was capitalized (e.g. quick consonant "Qu", "Ph", "Gi") and this vowel is also uppercase,
            // promote entire onset to uppercase for ALL CAPS words (e.g. "QUA", "PHONG")
            let mut onset_promoted = false;
            if key.is_upper && !onset.chars.is_empty() && onset.chars[0].1 {
                for c in &mut onset.chars {
                    if !c.1 {
                        c.1 = true;
                        onset_promoted = true;
                    }
                }
            }

            let nucleus = NucleusState::from_single(
                Some(onset),
                VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                },
                tone,
            );

            // Fast path: If onset casing was not changed and vowel is plain (no diacritic, no tone),
            // the rendered screen character is identical to simply appending key.ch!
            if !onset_promoted && diacritic == Diacritic::None && tone == Tone::None {
                return (SyllableState::Nucleus(nucleus), EngineAction::Passthrough);
            }

            let output = nucleus.render();
            return (
                SyllableState::Nucleus(nucleus),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            );
        }

        // 3. Additional consonant in Onset
        if key.ch.is_alphabetic() && is_valid_onset_extension(&onset.chars, key.ch) {
            onset.chars.push((key.ch, key.is_upper));
            return (SyllableState::Onset(onset), EngineAction::Passthrough);
        }

        // Invalid onset combo -> Passthrough
        let mut raw = InlineList::from(onset.render().as_str());
        raw.push(key.ch);
        (SyllableState::Passthrough(raw), EngineAction::Passthrough)
    }

    fn handle_nucleus(
        mut nucleus: NucleusState,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (Self, EngineAction) {
        let is_telex = config.method.is_telex_family();

        // 1. Bracket W shortcuts: [[ -> [, ]] -> ]
        if config.bracket_w
            && config.method.has_bracket_shortcuts()
            && nucleus.onset.is_none()
            && nucleus.vowels.len() == 1
        {
            if key.ch == '['
                && nucleus.vowels[0].base == BaseVowel::U
                && nucleus.vowels[0].diacritic == Diacritic::Horn
            {
                return (
                    SyllableState::Passthrough(InlineList::from_single('[')),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output: "[".to_string(),
                    },
                );
            }
            if key.ch == ']'
                && nucleus.vowels[0].base == BaseVowel::O
                && nucleus.vowels[0].diacritic == Diacritic::Horn
            {
                return (
                    SyllableState::Passthrough(InlineList::from_single(']')),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output: "]".to_string(),
                    },
                );
            }
        }

        // 2. Unified Modifiers (Tone, Circumflex, Horn, Breve, D-stroke, Undo Toggle)
        let has_onset_d = nucleus
            .onset
            .as_ref()
            .is_some_and(|o| !o.chars.is_empty() && o.chars[0].0.eq_ignore_ascii_case(&'d'));
        if let Some(effect) = match_modifier_key(
            config.method,
            key.ch,
            |b| nucleus.vowels.iter().any(|v| v.base == b),
            has_onset_d,
        ) {
            match nucleus.apply_modifier(effect, key, None, config) {
                ModifierOutcome::Applied => {
                    let output = nucleus.render();
                    return (
                        SyllableState::Nucleus(nucleus),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output,
                        },
                    );
                }
                ModifierOutcome::Undone(raw) => {
                    return (
                        SyllableState::Passthrough(InlineList::from(raw.as_str())),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output: raw,
                        },
                    );
                }
                ModifierOutcome::NotApplied => {}
            }
        }

        // 5. Additional vowel in Nucleus (up to 3 vowels)
        if let Some((base_vowel, mut diacritic, tone)) = decompose_vowel(key.ch) {
            // Glide absorption:
            // When onset is 'g' and single vowel is 'i' (e.g. "gi"),
            // or onset is 'q' and single vowel is 'u' (e.g. "qu"),
            // the arrival of another vowel indicates 'i' or 'u' was the consonant glide!
            // Absorb 'i' / 'u' into onset (making onset "gi" or "qu"), leaving vowels fresh for the incoming vowel!
            if nucleus.vowels.len() == 1
                && let Some(ref mut onset) = nucleus.onset
                && onset.chars.len() == 1
            {
                let c0 = onset.chars[0].0.to_ascii_lowercase();
                if c0 == 'g'
                    && nucleus.vowels[0].base == BaseVowel::I
                    && nucleus.vowels[0].diacritic == Diacritic::None
                {
                    onset.chars.push(('i', nucleus.vowels[0].is_upper));
                    nucleus.vowels.clear();
                } else if c0 == 'q'
                    && nucleus.vowels[0].base == BaseVowel::U
                    && nucleus.vowels[0].diacritic == Diacritic::None
                {
                    onset.chars.push(('u', nucleus.vowels[0].is_upper));
                    nucleus.vowels.clear();
                }
            }

            if nucleus.vowels.len() < 3 {
                // Telex automatic coupling: 'ư' + 'o' -> 'ươ'
                // In Vietnamese Telex typing, typing 'o' after 'ư' automatically couples into 'ươ'
                // e.g. "đư" + 'o' -> "đươ", "bư" + 'o' -> "bươ", "tư" + 'o' -> "tươ"
                if is_telex
                    && nucleus.tone == Tone::None
                    && base_vowel == BaseVowel::O
                    && diacritic == Diacritic::None
                    && nucleus.vowels.len() == 1
                    && nucleus.vowels[0].base == BaseVowel::U
                    && nucleus.vowels[0].diacritic == Diacritic::Horn
                {
                    diacritic = Diacritic::Horn;
                }

                let mut candidate_vowels = nucleus.vowels;
                candidate_vowels.push(VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                });

                let is_valid = if nucleus.tone != Tone::None {
                    is_valid_vowel_combination(candidate_vowels.as_slice())
                } else {
                    is_valid_intermediate_vowel_combination(candidate_vowels.as_slice())
                };

                if !is_valid {
                    let mut raw = InlineList::from(nucleus.render().as_str());
                    raw.push(key.ch);
                    return (SyllableState::Passthrough(raw), EngineAction::Passthrough);
                }

                let prev_has_tone = nucleus.tone != Tone::None;
                nucleus.vowels.push(VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                });
                if tone != Tone::None && nucleus.tone == Tone::None {
                    nucleus.tone = tone;
                }

                // Fast path: If vowel is plain, no diacritic added, and syllable had no tone before and no tone added,
                // appending a plain vowel never changes preceding vowels or shifts any marks!
                if diacritic == Diacritic::None && !prev_has_tone && tone == Tone::None {
                    return (SyllableState::Nucleus(nucleus), EngineAction::Passthrough);
                }

                let output = nucleus.render();
                return (
                    SyllableState::Nucleus(nucleus),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output,
                    },
                );
            }
        }

        // 6. Consonant -> Transition from Nucleus to Coda!
        // If vowels are "uơ" (e.g. from typing 'thuow' or 'huow') and a coda arrives (e.g. 'c' in "thước" or 'n' in "thương"),
        // upgrade "uơ" to "ươ" because "uơ" cannot accept codas while "ươ" does!
        let mut upgraded_u_horn = false;
        if nucleus.vowels.len() == 2
            && nucleus.vowels[0].base == BaseVowel::U
            && nucleus.vowels[0].diacritic == Diacritic::None
            && nucleus.vowels[1].base == BaseVowel::O
            && nucleus.vowels[1].diacritic == Diacritic::Horn
            && is_valid_coda_start(key.ch)
        {
            nucleus.vowels[0].diacritic = Diacritic::Horn;
            upgraded_u_horn = true;
        }

        let is_special_k = key.ch.eq_ignore_ascii_case(&'k')
            && nucleus.onset.as_ref().is_some_and(|o| {
                is_special_k_coda_allowed_during_typing(&o.chars, o.is_d_stroke, &nucleus.vowels)
            });
        let is_onset_qu = nucleus.onset.as_ref().is_some_and(|o| {
            o.chars.len() == 2
                && o.chars[0].0.eq_ignore_ascii_case(&'q')
                && o.chars[1].0.eq_ignore_ascii_case(&'u')
        });
        let can_accept_coda = can_vowels_accept_coda(&nucleus.vowels)
            && is_valid_coda_start_for_vowels(is_onset_qu, &nucleus.vowels, key.ch);
        let is_stop = matches!(key.ch.to_ascii_lowercase(), 'c' | 'p' | 't' | 'k');
        let tone_incompatible = config.restore_on_wrong_spelling
            && is_stop
            && matches!(nucleus.tone, Tone::Grave | Tone::HookAbove | Tone::Tilde);
        if can_accept_coda && (is_valid_coda_start(key.ch) || is_special_k) && !tone_incompatible {
            let coda = CodaState::from_single(nucleus, key.ch, key.is_upper);

            // Fast path: if horn was not upgraded and there is no tone mark to shift,
            // adding a single coda consonant will never shift marks or mutate vowels.
            if !upgraded_u_horn && coda.nucleus.tone == Tone::None {
                return (SyllableState::Coda(coda), EngineAction::Passthrough);
            }

            let output = coda.render();
            return (
                SyllableState::Coda(coda),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            );
        }

        // Non-coda consonant or non-alphabetic -> Passthrough
        let mut raw = InlineList::from(nucleus.render().as_str());
        raw.push(key.ch);
        (SyllableState::Passthrough(raw), EngineAction::Passthrough)
    }

    fn handle_coda(
        mut coda: CodaState,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (Self, EngineAction) {
        // 1. Unified Modifiers across Coda (Telex & VNI)
        let has_onset_d = coda
            .nucleus
            .onset
            .as_ref()
            .is_some_and(|o| !o.chars.is_empty() && o.chars[0].0.eq_ignore_ascii_case(&'d'));
        if let Some(effect) = match_modifier_key(
            config.method,
            key.ch,
            |b| coda.nucleus.vowels.iter().any(|v| v.base == b),
            has_onset_d,
        ) {
            match coda
                .nucleus
                .apply_modifier(effect, key, Some(&coda.coda), config)
            {
                ModifierOutcome::Applied => {
                    let output = coda.render();
                    return (
                        SyllableState::Coda(coda),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output,
                        },
                    );
                }
                ModifierOutcome::Undone(raw) => {
                    return (
                        SyllableState::Passthrough(InlineList::from(raw.as_str())),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output: raw,
                        },
                    );
                }
                ModifierOutcome::NotApplied => {}
            }
        }

        // 3. Additional consonant in Coda (only "ng", "nh", "ch" are valid 2-letter codas in Vietnamese)
        if coda.coda.len() == 1 && is_valid_coda_pair(coda.coda[0].0, key.ch) {
            coda.coda.push((key.ch, key.is_upper));
            return (SyllableState::Coda(coda), EngineAction::Passthrough);
        }

        // Otherwise -> Passthrough
        let mut raw = InlineList::from(coda.render().as_str());
        raw.push(key.ch);
        (SyllableState::Passthrough(raw), EngineAction::Passthrough)
    }
}
