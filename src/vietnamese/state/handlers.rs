use std::marker::PhantomData;

use super::types::{Coda, Empty, Nucleus, Onset, Passthrough, State, SyllableState};
use crate::engine::{
    action::{CompactString, EngineAction},
    buffer::RawKey,
    config::EngineConfig,
};
use crate::vietnamese::{
    charset::{BaseVowel, Diacritic, Tone, decompose_vowel, is_d_stroke},
    coda::CodaState,
    inline_list::InlineList,
    modifier::{KeyEffect, match_modifier_key},
    nucleus::{ModifierOutcome, NucleusState},
    onset::OnsetState,
    spelling::{
        can_vowels_accept_coda, is_special_k_coda_allowed_during_typing, is_valid_coda_pair,
        is_valid_coda_start, is_valid_coda_start_for_vowels,
        is_valid_intermediate_vowel_combination, is_valid_onset_extension,
        is_valid_onset_vowel_orthography, is_valid_vowel_combination,
    },
    syllable::VowelLetter,
};

// ============================================================================
// 1. Typestate: State<Empty>
// ============================================================================

impl State<Empty> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            data: (),
            _state: PhantomData,
        }
    }

    /// Consumes `State<Empty>` and transitions into `State<Onset>`
    #[inline(always)]
    pub fn into_onset(self, ch: char, is_upper: bool, is_d_stroke: bool) -> State<Onset> {
        State {
            data: OnsetState::from_single(ch, is_upper, is_d_stroke),
            _state: PhantomData,
        }
    }

    /// Consumes `State<Empty>` and transitions directly into `State<Nucleus>`
    #[inline(always)]
    pub fn into_nucleus(
        self,
        vowel: VowelLetter,
        tone: Tone,
        is_shorthand: bool,
    ) -> State<Nucleus> {
        let mut nucleus = NucleusState::from_single(None, vowel, tone);
        nucleus.is_shorthand = is_shorthand;
        State {
            data: nucleus,
            _state: PhantomData,
        }
    }

    /// Consumes `State<Empty>` and transitions into `State<Passthrough>`
    #[inline(always)]
    pub fn into_passthrough(self, ch: char) -> State<Passthrough> {
        State {
            data: InlineList::from_single(ch),
            _state: PhantomData,
        }
    }

    #[inline(always)]
    fn try_bracket_w_shortcut(
        self,
        key: RawKey,
        config: &EngineConfig,
    ) -> Option<(SyllableState, EngineAction)> {
        if config.bracket_w && config.method.has_bracket_shortcuts() {
            let base = match key.ch {
                '[' => BaseVowel::U,
                ']' => BaseVowel::O,
                _ => return None,
            };
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
                true,
            );
            let output = nucleus.render_compact();
            return Some((
                nucleus.into(),
                EngineAction::Replace {
                    backspaces: 0,
                    output,
                },
            ));
        }
        None
    }

    /// Feeds a key into `State<Empty>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(self, key: RawKey, config: &EngineConfig) -> (SyllableState, EngineAction) {
        let ch_lower = key.ch.to_ascii_lowercase();

        // 1. Bracket W shortcuts: [ -> ư, ] -> ơ
        if let Some(res) = self.try_bracket_w_shortcut(key, config) {
            return res;
        }

        // 2. Standalone 'w' -> 'ư'
        if config.method.has_standalone_w() && ch_lower == 'w' {
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: BaseVowel::U,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
                true,
            );
            let output = nucleus.render_compact();
            return (
                nucleus.into(),
                EngineAction::Replace {
                    backspaces: 0,
                    output,
                },
            );
        }

        // 3. Auto-uppercased first letter (key.is_upper is true, but key.ch is lowercase)
        if key.is_upper && key.ch.is_lowercase() && key.ch.is_alphabetic() {
            let upper_char = key.ch.to_uppercase().next().unwrap_or(key.ch);
            if let Some(base_vowel) = BaseVowel::from_char(key.ch) {
                let nucleus = self.into_nucleus(
                    VowelLetter {
                        base: base_vowel,
                        diacritic: Diacritic::None,
                        is_upper: true,
                    },
                    Tone::None,
                    false,
                );
                return (
                    nucleus.into(),
                    EngineAction::Replace {
                        backspaces: 0,
                        output: upper_char.into(),
                    },
                );
            } else {
                let onset = self.into_onset(upper_char, true, false);
                return (
                    onset.into(),
                    EngineAction::Replace {
                        backspaces: 0,
                        output: upper_char.into(),
                    },
                );
            }
        }

        // 4. Vowel -> Transition Empty -> Nucleus
        if let Some((base_vowel, diacritic, tone)) = decompose_vowel(key.ch) {
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                },
                tone,
                false,
            );
            return (nucleus.into(), EngineAction::Passthrough);
        }

        // 5. 'đ' or 'Đ' -> Transition Empty -> Onset
        if is_d_stroke(key.ch) {
            return (
                self.into_onset('d', key.is_upper, true).into(),
                EngineAction::Passthrough,
            );
        }

        // 6. Non-Vietnamese initial consonants: 'f', 'j', 'z', and non-standalone 'w'
        let is_non_vi_consonant = matches!(ch_lower, 'f' | 'j' | 'z')
            || (!config.method.has_standalone_w() && ch_lower == 'w');
        if is_non_vi_consonant {
            return (
                self.into_passthrough(key.ch).into(),
                EngineAction::Passthrough,
            );
        }

        // 7. Standard consonant -> Transition Empty -> Onset
        if key.ch.is_alphabetic() {
            return (
                self.into_onset(key.ch, key.is_upper, false).into(),
                EngineAction::Passthrough,
            );
        }

        // 8. Anything else -> Transition Empty -> Passthrough
        (
            self.into_passthrough(key.ch).into(),
            EngineAction::Passthrough,
        )
    }
}

// ============================================================================
// 2. Typestate: State<Onset>
// ============================================================================

impl State<Onset> {
    /// Consumes `State<Onset>` and transitions into `State<Nucleus>`
    #[inline(always)]
    pub fn into_nucleus(
        self,
        vowel: VowelLetter,
        tone: Tone,
        is_shorthand: bool,
    ) -> State<Nucleus> {
        let mut nucleus = NucleusState::from_single(Some(self.data), vowel, tone);
        nucleus.is_shorthand = is_shorthand;
        State {
            data: nucleus,
            _state: PhantomData,
        }
    }

    /// Consumes `State<Onset>` and transitions into `State<Passthrough>` with trailing `ch`
    #[inline(always)]
    pub fn into_passthrough(self, ch: char) -> State<Passthrough> {
        let mut raw = InlineList::new();
        self.data.render_chars_to(&mut raw);
        raw.push(ch);
        State {
            data: raw,
            _state: PhantomData,
        }
    }

    /// Consumes `State<Onset>` on Backspace, returning `Empty` or updated `State<Onset>`
    #[inline(always)]
    pub fn pop_char(mut self) -> SyllableState {
        self.data.chars.pop();
        if self.data.chars.is_empty() {
            SyllableState::Empty
        } else {
            self.into()
        }
    }

    #[inline(always)]
    fn try_bracket_w_shortcut(
        self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> Option<(SyllableState, EngineAction)> {
        if config.bracket_w && config.method.has_bracket_shortcuts() {
            let base = match key.ch {
                '[' => BaseVowel::U,
                ']' => BaseVowel::O,
                _ => return None,
            };
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
                true,
            );
            let output = nucleus.render_compact();
            return Some((
                nucleus.into(),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            ));
        }
        None
    }

    /// Feeds a key into `State<Onset>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(
        mut self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (SyllableState, EngineAction) {
        let ch_lower = key.ch.to_ascii_lowercase();

        // 1. Bracket W shortcuts: [ -> ư, ] -> ơ
        if let Some(res) = self.try_bracket_w_shortcut(key, current_len, config) {
            return res;
        }

        // 2. D-Stroke in Onset (Telex 'dd' or VNI 'd9' -> 'đ')
        let has_onset_d = !self.chars.is_empty() && self.chars[0].0.eq_ignore_ascii_case(&'d');
        if let Some(KeyEffect::DStroke) =
            match_modifier_key(config.method, key.ch, |_| false, has_onset_d)
        {
            if config.method.is_telex_family() && self.is_d_stroke {
                // 3rd 'd' cancels 'đ' and restores double 'dd' into Passthrough!
                let first_d = if self.chars[0].1 { 'D' } else { 'd' };
                let mut output = CompactString::new();
                output.push(first_d);
                output.push(key.ch);
                let mut raw = InlineList::new();
                raw.push(first_d);
                raw.push(key.ch);
                return (
                    SyllableState::passthrough(raw),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output,
                    },
                );
            } else {
                self.is_d_stroke = !self.is_d_stroke;
                let output = self.render_compact();
                return (
                    self.into(),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output,
                    },
                );
            }
        }

        // 3. Incoming shorthand 'w' -> Transition Onset -> Nucleus!
        if config.method.has_standalone_w() && ch_lower == 'w' {
            if !is_valid_onset_vowel_orthography(&self.chars, self.is_d_stroke, BaseVowel::U) {
                return (
                    self.into_passthrough(key.ch).into(),
                    EngineAction::Passthrough,
                );
            }
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: BaseVowel::U,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper,
                },
                Tone::None,
                true,
            );
            let output = nucleus.render_compact();
            return (
                nucleus.into(),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            );
        }

        // 4. Incoming vowel -> Transition Onset -> Nucleus!
        if let Some((base_vowel, diacritic, tone)) = decompose_vowel(key.ch) {
            if !is_valid_onset_vowel_orthography(&self.chars, self.is_d_stroke, base_vowel) {
                return (
                    self.into_passthrough(key.ch).into(),
                    EngineAction::Passthrough,
                );
            }
            // Promote entire onset to uppercase for ALL CAPS words (e.g. "QUA", "PHONG")
            let mut onset_promoted = false;
            if key.is_upper && !self.chars.is_empty() && self.chars[0].1 {
                for c in &mut self.chars {
                    if !c.1 {
                        c.1 = true;
                        onset_promoted = true;
                    }
                }
            }

            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                },
                tone,
                false,
            );

            // Fast path: If onset casing was not changed and vowel is plain, pass through directly
            if !onset_promoted && diacritic == Diacritic::None && tone == Tone::None {
                return (nucleus.into(), EngineAction::Passthrough);
            }

            let output = nucleus.render_compact();
            return (
                nucleus.into(),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            );
        }

        // 5. Additional consonant in Onset
        if key.ch.is_alphabetic() && is_valid_onset_extension(&self.chars, key.ch) {
            self.chars.push((key.ch, key.is_upper));
            return (self.into(), EngineAction::Passthrough);
        }

        // 6. Invalid onset combo -> Transition Onset -> Passthrough
        (
            self.into_passthrough(key.ch).into(),
            EngineAction::Passthrough,
        )
    }
}

// ============================================================================
// 3. Typestate: State<Nucleus>
// ============================================================================

impl State<Nucleus> {
    /// Consumes `State<Nucleus>` and transitions into `State<Coda>`
    #[inline(always)]
    pub fn into_coda(self, ch: char, is_upper: bool) -> State<Coda> {
        State {
            data: CodaState::from_single(self.data, ch, is_upper),
            _state: PhantomData,
        }
    }

    /// Consumes `State<Nucleus>` and transitions into `State<Passthrough>` with trailing `ch`
    #[inline(always)]
    pub fn into_passthrough(self, ch: char) -> State<Passthrough> {
        let mut raw = InlineList::new();
        self.data.render_chars_to(&mut raw);
        raw.push(ch);
        State {
            data: raw,
            _state: PhantomData,
        }
    }

    /// Consumes `State<Nucleus>` and transitions into `State<Passthrough>` from undone raw string
    #[inline(always)]
    pub fn into_passthrough_raw(self, raw: &str) -> State<Passthrough> {
        State {
            data: InlineList::from(raw),
            _state: PhantomData,
        }
    }

    /// Consumes `State<Nucleus>` on Backspace, returning `Empty`, `State<Onset>`, or `State<Nucleus>`
    #[inline(always)]
    pub fn pop_vowel(mut self) -> SyllableState {
        let tone_pos_before = if self.tone == Tone::None {
            None
        } else {
            Some(self.find_tone_position())
        };

        self.vowels.pop();

        if self.vowels.is_empty() {
            if let Some(mut onset) = self.onset {
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
                        self.onset = Some(onset);
                        self.vowels = InlineList::from_single(VowelLetter {
                            base,
                            diacritic: Diacritic::None,
                            is_upper: glide.1,
                        });
                        return self.into();
                    }
                }
                State::<Onset>::from_data(onset).into()
            } else {
                SyllableState::Empty
            }
        } else {
            // If the popped vowel carried the tone mark, clear tone
            if let Some(pos) = tone_pos_before
                && pos >= self.vowels.len()
            {
                self.tone = Tone::None;
            }
            self.into()
        }
    }

    /// Feeds a key into `State<Nucleus>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(
        mut self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (SyllableState, EngineAction) {
        let is_telex = config.method.is_telex_family();

        // 1. Bracket W shortcuts: [[ -> [, ]] -> ]
        if config.bracket_w
            && config.method.has_bracket_shortcuts()
            && self.onset.is_none()
            && self.vowels.len() == 1
            && self.is_shorthand
        {
            if key.ch == '['
                && self.vowels[0].base == BaseVowel::U
                && self.vowels[0].diacritic == Diacritic::Horn
            {
                return (
                    SyllableState::passthrough(InlineList::from_single('[')),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output: "[".into(),
                    },
                );
            }
            if key.ch == ']'
                && self.vowels[0].base == BaseVowel::O
                && self.vowels[0].diacritic == Diacritic::Horn
            {
                return (
                    SyllableState::passthrough(InlineList::from_single(']')),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output: "]".into(),
                    },
                );
            }
        }

        // 2. Unified Modifiers (Tone, Circumflex, Horn, Breve, D-stroke, Undo Toggle)
        let has_onset_d = self
            .onset
            .as_ref()
            .is_some_and(|o| !o.chars.is_empty() && o.chars[0].0.eq_ignore_ascii_case(&'d'));
        if let Some(effect) = match_modifier_key(
            config.method,
            key.ch,
            |b| self.vowels.iter().any(|v| v.base == b),
            has_onset_d,
        ) {
            match self.apply_modifier(effect, key, None, None, config) {
                ModifierOutcome::Applied => {
                    let output = self.render_compact();
                    return (
                        self.into(),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output,
                        },
                    );
                }
                ModifierOutcome::Undone(raw) => {
                    return (
                        self.into_passthrough_raw(raw.as_str()).into(),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output: raw,
                        },
                    );
                }
                ModifierOutcome::NotApplied => {}
            }
        }

        // 3. Additional vowel in Nucleus (up to 3 vowels)
        if let Some((base_vowel, mut diacritic, tone)) = decompose_vowel(key.ch) {
            // Glide absorption: "gi" or "qu" followed by another vowel
            if self.vowels.len() == 1
                && let Some(ref mut onset) = self.data.onset
                && onset.chars.len() == 1
            {
                let c0 = onset.chars[0].0.to_ascii_lowercase();
                if c0 == 'g'
                    && self.data.vowels[0].base == BaseVowel::I
                    && self.data.vowels[0].diacritic == Diacritic::None
                {
                    onset.chars.push(('i', self.data.vowels[0].is_upper));
                    self.data.vowels.clear();
                } else if c0 == 'q'
                    && self.data.vowels[0].base == BaseVowel::U
                    && self.data.vowels[0].diacritic == Diacritic::None
                {
                    onset.chars.push(('u', self.data.vowels[0].is_upper));
                    self.data.vowels.clear();
                }
            }

            if self.vowels.len() < 3 {
                // Telex automatic coupling: 'ư' + 'o' -> 'ươ'
                if is_telex
                    && self.tone == Tone::None
                    && base_vowel == BaseVowel::O
                    && diacritic == Diacritic::None
                    && self.vowels.len() == 1
                    && self.vowels[0].base == BaseVowel::U
                    && self.vowels[0].diacritic == Diacritic::Horn
                {
                    diacritic = Diacritic::Horn;
                }

                let mut candidate_vowels = self.vowels;
                candidate_vowels.push(VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                });

                let is_valid = if self.tone != Tone::None {
                    is_valid_vowel_combination(candidate_vowels.as_slice())
                } else {
                    is_valid_intermediate_vowel_combination(candidate_vowels.as_slice())
                };

                if !is_valid {
                    return (
                        self.into_passthrough(key.ch).into(),
                        EngineAction::Passthrough,
                    );
                }

                let prev_has_tone = self.tone != Tone::None;
                self.vowels.push(VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper: key.is_upper,
                });
                if tone != Tone::None && self.tone == Tone::None {
                    self.tone = tone;
                }

                // Fast path: plain vowel with no tone shift
                if diacritic == Diacritic::None && !prev_has_tone && tone == Tone::None {
                    return (self.into(), EngineAction::Passthrough);
                }

                let output = self.render_compact();
                return (
                    self.into(),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output,
                    },
                );
            }
        }

        // 4. Consonant -> Transition from Nucleus to Coda!
        let is_valid_start = is_valid_coda_start(key.ch);
        let is_k = key.ch.eq_ignore_ascii_case(&'k');
        if !is_valid_start && !is_k {
            return (
                self.into_passthrough(key.ch).into(),
                EngineAction::Passthrough,
            );
        }

        // Upgrade "uơ" to "ươ" when a coda arrives (e.g. "thước", "thương")
        let mut upgraded_u_horn = false;
        if self.vowels.len() == 2
            && self.vowels[0].base == BaseVowel::U
            && self.vowels[0].diacritic == Diacritic::None
            && self.vowels[1].base == BaseVowel::O
            && self.vowels[1].diacritic == Diacritic::Horn
            && is_valid_start
        {
            self.vowels[0].diacritic = Diacritic::Horn;
            upgraded_u_horn = true;
        }

        let is_special_k = is_k
            && self.onset.as_ref().is_some_and(|o| {
                is_special_k_coda_allowed_during_typing(&o.chars, o.is_d_stroke, &self.vowels)
            });
        let is_onset_qu = self.onset.as_ref().is_some_and(|o| {
            o.chars.len() == 2
                && o.chars[0].0.eq_ignore_ascii_case(&'q')
                && o.chars[1].0.eq_ignore_ascii_case(&'u')
        });
        let can_accept_coda = can_vowels_accept_coda(&self.vowels)
            && is_valid_coda_start_for_vowels(is_onset_qu, &self.vowels, key.ch);
        let is_stop = matches!(key.ch.to_ascii_lowercase(), 'c' | 'p' | 't' | 'k');
        let tone_incompatible = config.restore_on_wrong_spelling
            && is_stop
            && matches!(self.tone, Tone::Grave | Tone::HookAbove | Tone::Tilde);
        if can_accept_coda && (is_valid_start || is_special_k) && !tone_incompatible {
            let coda = self.into_coda(key.ch, key.is_upper);

            // Fast path: if horn was not upgraded and there is no tone mark to shift
            if !upgraded_u_horn && coda.nucleus.tone == Tone::None {
                return (coda.into(), EngineAction::Passthrough);
            }

            let output = coda.render_compact();
            return (
                coda.into(),
                EngineAction::Replace {
                    backspaces: current_len,
                    output,
                },
            );
        }

        // Non-coda consonant or non-alphabetic -> Transition Nucleus -> Passthrough
        (
            self.into_passthrough(key.ch).into(),
            EngineAction::Passthrough,
        )
    }
}

// ============================================================================
// 4. Typestate: State<Coda>
// ============================================================================

impl State<Coda> {
    /// Consumes `State<Coda>` and transitions into `State<Passthrough>` with trailing `ch`
    #[inline(always)]
    pub fn into_passthrough(self, ch: char) -> State<Passthrough> {
        let mut raw = InlineList::new();
        self.data.render_chars_to(&mut raw);
        raw.push(ch);
        State {
            data: raw,
            _state: PhantomData,
        }
    }

    /// Consumes `State<Coda>` and transitions into `State<Passthrough>` from undone raw string
    #[inline(always)]
    pub fn into_passthrough_raw(self, raw: &str) -> State<Passthrough> {
        State {
            data: InlineList::from(raw),
            _state: PhantomData,
        }
    }

    /// Consumes `State<Coda>` on Backspace, transitioning back to `State<Nucleus>` when coda is emptied
    #[inline(always)]
    pub fn pop_coda(mut self) -> SyllableState {
        self.data.coda.pop();
        if self.data.coda.is_empty() {
            State::<Nucleus>::from_data(self.data.nucleus).into()
        } else {
            self.into()
        }
    }

    /// Feeds a key into `State<Coda>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(
        mut self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (SyllableState, EngineAction) {
        // 1. Unified Modifiers across Coda (Telex & VNI)
        let has_onset_d = self
            .nucleus
            .onset
            .as_ref()
            .is_some_and(|o| !o.chars.is_empty() && o.chars[0].0.eq_ignore_ascii_case(&'d'));
        if let Some(effect) = match_modifier_key(
            config.method,
            key.ch,
            |b| self.nucleus.vowels.iter().any(|v| v.base == b),
            has_onset_d,
        ) {
            let coda_copy = self.data.coda;
            let across_coda = self.data.circumflex_across_coda;
            match self
                .data
                .nucleus
                .apply_modifier(effect, key, Some(&coda_copy), across_coda, config)
            {
                ModifierOutcome::Applied => {
                    if let KeyEffect::Circumflex(Some(target_base)) = effect {
                        self.circumflex_across_coda = Some(target_base);
                    } else {
                        self.circumflex_across_coda = None;
                    }
                    let output = self.render_compact();
                    return (
                        self.into(),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output,
                        },
                    );
                }
                ModifierOutcome::Undone(raw) => {
                    return (
                        self.into_passthrough_raw(raw.as_str()).into(),
                        EngineAction::Replace {
                            backspaces: current_len,
                            output: raw,
                        },
                    );
                }
                ModifierOutcome::NotApplied => {}
            }
        }

        // 2. Additional consonant in Coda (only "ng", "nh", "ch" are valid 2-letter codas in Vietnamese)
        if self.coda.len() == 1 && is_valid_coda_pair(self.coda[0].0, key.ch) {
            self.coda.push((key.ch, key.is_upper));
            return (self.into(), EngineAction::Passthrough);
        }

        // 3. Otherwise -> Transition Coda -> Passthrough
        (
            self.into_passthrough(key.ch).into(),
            EngineAction::Passthrough,
        )
    }
}

// ============================================================================
// 5. Typestate: State<Passthrough>
// ============================================================================

impl State<Passthrough> {
    #[inline(always)]
    pub fn feed_key(mut self, key: RawKey) -> (SyllableState, EngineAction) {
        self.data.push(key.ch);
        (self.into(), EngineAction::Passthrough)
    }

    #[inline(always)]
    pub fn pop_char(mut self, config: &EngineConfig) -> SyllableState {
        self.data.pop();
        if self.data.is_empty() {
            SyllableState::Empty
        } else {
            SyllableState::parse_prefix_chars(self.data.iter().copied(), config)
        }
    }
}

// ============================================================================
// Runtime Dispatch Delegation on SyllableState
// ============================================================================

impl SyllableState {
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

    /// Transitions backward when Backspace is pressed by delegating to each Typestate
    #[inline(always)]
    pub fn handle_backspace(self, config: &EngineConfig) -> SyllableState {
        match self {
            SyllableState::Empty => SyllableState::Empty,
            SyllableState::Onset(onset) => onset.pop_char(),
            SyllableState::Nucleus(nucleus) => nucleus.pop_vowel(),
            SyllableState::Coda(coda) => coda.pop_coda(),
            SyllableState::Passthrough(raw) => raw.pop_char(config),
        }
    }

    /// Process a new key and transit to the next state by delegating to each Typestate
    #[inline(always)]
    pub fn feed_key(
        self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (Self, EngineAction) {
        match self {
            SyllableState::Empty => State::<Empty>::new().feed_key(key, config),
            SyllableState::Onset(onset) => onset.feed_key(key, current_len, config),
            SyllableState::Nucleus(nucleus) => nucleus.feed_key(key, current_len, config),
            SyllableState::Coda(coda) => coda.feed_key(key, current_len, config),
            SyllableState::Passthrough(raw) => raw.feed_key(key),
        }
    }
}
