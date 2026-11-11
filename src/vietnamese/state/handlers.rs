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
    modifier::{KeyEffect, match_modifier_key},
    nucleus::{ModifierOutcome, NucleusState},
    onset::OnsetState,
    spelling::{
        can_vowels_accept_coda, is_special_k_coda_allowed_during_typing_ascii, is_valid_coda_pair,
        is_valid_coda_start, is_valid_coda_start_for_vowels,
        is_valid_intermediate_vowel_combination, is_valid_onset_extension_ascii,
        is_valid_onset_vowel_orthography_ascii, is_valid_vowel_combination,
    },
    syllable::VowelLetter,
};

// ============================================================================
// Compile-Time ASCII Initial Dispatch Table for State<Empty>
// ============================================================================

const INIT_PASSTHROUGH: u8 = 0;
const INIT_BRACKET_OPEN: u8 = 1;
const INIT_BRACKET_CLOSE: u8 = 2;
const INIT_W: u8 = 3;
const INIT_NON_VI_CONSONANT: u8 = 4;
const INIT_VI_CONSONANT: u8 = 5;
const INIT_VOWEL_BASE: u8 = 10;

const INITIAL_ASCII_LUT: [u8; 128] = {
    let mut lut = [INIT_PASSTHROUGH; 128];
    lut[b'[' as usize] = INIT_BRACKET_OPEN;
    lut[b']' as usize] = INIT_BRACKET_CLOSE;

    let mut c = b'a';
    while c <= b'z' {
        let tag = match c {
            b'a' => INIT_VOWEL_BASE + BaseVowel::A as u8,
            b'e' => INIT_VOWEL_BASE + BaseVowel::E as u8,
            b'i' => INIT_VOWEL_BASE + BaseVowel::I as u8,
            b'o' => INIT_VOWEL_BASE + BaseVowel::O as u8,
            b'u' => INIT_VOWEL_BASE + BaseVowel::U as u8,
            b'y' => INIT_VOWEL_BASE + BaseVowel::Y as u8,
            b'w' => INIT_W,
            b'f' | b'j' | b'z' => INIT_NON_VI_CONSONANT,
            _ => INIT_VI_CONSONANT,
        };
        lut[c as usize] = tag;
        lut[(c - 32) as usize] = tag;
        c += 1;
    }
    lut
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

    /// Feeds a key into `State<Empty>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(self, key: RawKey, config: &EngineConfig) -> (SyllableState, EngineAction) {
        let ch = key.ch();
        let is_upper = key.is_upper();
        let cp = ch as u32;

        // O(1) ASCII Initial Fast-Path (>99% of initial keystrokes)
        if cp < 128 {
            let b_ch = cp as u8;
            match INITIAL_ASCII_LUT[cp as usize] {
                INIT_VI_CONSONANT => {
                    if is_upper && b_ch.is_ascii_lowercase() {
                        let upper = (b_ch - 32) as char;
                        return (
                            self.into_onset(upper, true, false).into(),
                            EngineAction::Replace {
                                backspaces: 0,
                                output: upper.into(),
                            },
                        );
                    }
                    return (
                        self.into_onset(ch, is_upper, false).into(),
                        EngineAction::Passthrough,
                    );
                }
                tag if tag >= INIT_VOWEL_BASE => {
                    let base = match tag - INIT_VOWEL_BASE {
                        0 => BaseVowel::A,
                        1 => BaseVowel::E,
                        2 => BaseVowel::I,
                        3 => BaseVowel::O,
                        4 => BaseVowel::U,
                        _ => BaseVowel::Y,
                    };
                    let nucleus = self.into_nucleus(
                        VowelLetter {
                            base,
                            diacritic: Diacritic::None,
                            is_upper,
                        },
                        Tone::None,
                        false,
                    );
                    if is_upper && b_ch.is_ascii_lowercase() {
                        let upper = (b_ch - 32) as char;
                        return (
                            nucleus.into(),
                            EngineAction::Replace {
                                backspaces: 0,
                                output: upper.into(),
                            },
                        );
                    }
                    return (nucleus.into(), EngineAction::Passthrough);
                }
                INIT_NON_VI_CONSONANT => {
                    if is_upper && b_ch.is_ascii_lowercase() {
                        let upper = (b_ch - 32) as char;
                        return (
                            SyllableState::passthrough(),
                            EngineAction::Replace {
                                backspaces: 0,
                                output: upper.into(),
                            },
                        );
                    }
                    return (SyllableState::passthrough(), EngineAction::Passthrough);
                }
                INIT_W => {
                    if config.method.has_standalone_w() {
                        let nucleus = self.into_nucleus(
                            VowelLetter {
                                base: BaseVowel::U,
                                diacritic: Diacritic::Horn,
                                is_upper,
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
                    if is_upper && b_ch == b'w' {
                        return (
                            SyllableState::passthrough(),
                            EngineAction::Replace {
                                backspaces: 0,
                                output: 'W'.into(),
                            },
                        );
                    }
                    return (SyllableState::passthrough(), EngineAction::Passthrough);
                }
                tag @ (INIT_BRACKET_OPEN | INIT_BRACKET_CLOSE) => {
                    if config.bracket_w && config.method.has_bracket_shortcuts() {
                        let base = if tag == INIT_BRACKET_OPEN {
                            BaseVowel::U
                        } else {
                            BaseVowel::O
                        };
                        let nucleus = self.into_nucleus(
                            VowelLetter {
                                base,
                                diacritic: Diacritic::Horn,
                                is_upper,
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
                    return (SyllableState::passthrough(), EngineAction::Passthrough);
                }
                _ => return (SyllableState::passthrough(), EngineAction::Passthrough),
            }
        }

        // Non-ASCII initial characters (pre-composed Vietnamese vowels, 'đ'/'Đ', or foreign Unicode)
        if is_upper && ch.is_lowercase() && ch.is_alphabetic() {
            let upper_char = ch.to_uppercase().next().unwrap_or(ch);
            if let Some((base_vowel, diacritic, tone)) = decompose_vowel(upper_char) {
                let nucleus = self.into_nucleus(
                    VowelLetter {
                        base: base_vowel,
                        diacritic,
                        is_upper: true,
                    },
                    tone,
                    false,
                );
                return (
                    nucleus.into(),
                    EngineAction::Replace {
                        backspaces: 0,
                        output: upper_char.into(),
                    },
                );
            }
            if is_d_stroke(upper_char) {
                return (
                    self.into_onset('d', true, true).into(),
                    EngineAction::Replace {
                        backspaces: 0,
                        output: upper_char.into(),
                    },
                );
            }
            return (
                SyllableState::passthrough(),
                EngineAction::Replace {
                    backspaces: 0,
                    output: upper_char.into(),
                },
            );
        }

        if let Some((base_vowel, diacritic, tone)) = decompose_vowel(ch) {
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper,
                },
                tone,
                false,
            );
            return (nucleus.into(), EngineAction::Passthrough);
        }

        if is_d_stroke(ch) {
            return (
                self.into_onset('d', is_upper, true).into(),
                EngineAction::Passthrough,
            );
        }

        (SyllableState::passthrough(), EngineAction::Passthrough)
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

    #[inline(always)]
    fn try_bracket_w_shortcut(
        self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> Option<(SyllableState, EngineAction)> {
        let ch = key.ch();
        if ch != '[' && ch != ']' {
            return None;
        }
        if config.bracket_w && config.method.has_bracket_shortcuts() {
            let base = if ch == '[' {
                BaseVowel::U
            } else {
                BaseVowel::O
            };
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base,
                    diacritic: Diacritic::Horn,
                    is_upper: key.is_upper(),
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
        let ch = key.ch();
        let is_upper = key.is_upper();
        let ch_lower = ch.to_ascii_lowercase();

        // 1. Bracket W shortcuts: [ -> ư, ] -> ơ
        if let Some(res) = self.try_bracket_w_shortcut(key, current_len, config) {
            return res;
        }

        // 2. D-Stroke in Onset (Telex 'dd' or VNI 'd9' -> 'đ')
        if let Some(KeyEffect::DStroke) = match_modifier_key(
            config.method,
            ch,
            |_| false,
            || !self.chars.is_empty() && (self.chars[0] | 0x20) == b'd',
        ) {
            if config.method.is_telex_family() && self.is_d_stroke {
                // 3rd 'd' cancels 'đ' and restores double 'dd' into Passthrough!
                let first_d = self.chars[0] as char;
                let mut output = CompactString::new();
                output.push(first_d);
                output.push(ch);
                return (
                    SyllableState::passthrough(),
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
            if !is_valid_onset_vowel_orthography_ascii(&self.chars, self.is_d_stroke, BaseVowel::U)
            {
                return (SyllableState::passthrough(), EngineAction::Passthrough);
            }
            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: BaseVowel::U,
                    diacritic: Diacritic::Horn,
                    is_upper,
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
        if let Some((base_vowel, diacritic, tone)) = decompose_vowel(ch) {
            if !is_valid_onset_vowel_orthography_ascii(&self.chars, self.is_d_stroke, base_vowel) {
                return (SyllableState::passthrough(), EngineAction::Passthrough);
            }
            // Promote entire onset to uppercase for ALL CAPS words (e.g. "QUA", "PHONG")
            let mut onset_promoted = false;
            if is_upper && !self.chars.is_empty() && self.chars[0].is_ascii_uppercase() {
                for b in &mut self.chars {
                    if b.is_ascii_lowercase() {
                        *b -= 32;
                        onset_promoted = true;
                    }
                }
            }

            let nucleus = self.into_nucleus(
                VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper,
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
        if ch.is_ascii_alphabetic() && is_valid_onset_extension_ascii(&self.chars, ch) {
            self.push_ascii(ch, is_upper);
            return (self.into(), EngineAction::Passthrough);
        }

        // 6. Invalid onset combo -> Transition Onset -> Passthrough
        (SyllableState::passthrough(), EngineAction::Passthrough)
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

    /// Feeds a key into `State<Nucleus>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(
        mut self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (SyllableState, EngineAction) {
        let ch = key.ch();
        let is_upper = key.is_upper();
        let is_telex = config.method.is_telex_family();

        // 1. Bracket W shortcuts: [[ -> [, ]] -> ]
        if (ch == '[' || ch == ']')
            && config.bracket_w
            && config.method.has_bracket_shortcuts()
            && self.onset.is_none()
            && self.vowels.len() == 1
            && self.is_shorthand
        {
            if ch == '['
                && self.vowels[0].base == BaseVowel::U
                && self.vowels[0].diacritic == Diacritic::Horn
            {
                return (
                    SyllableState::passthrough(),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output: "[".into(),
                    },
                );
            }
            if ch == ']'
                && self.vowels[0].base == BaseVowel::O
                && self.vowels[0].diacritic == Diacritic::Horn
            {
                return (
                    SyllableState::passthrough(),
                    EngineAction::Replace {
                        backspaces: current_len,
                        output: "]".into(),
                    },
                );
            }
        }

        // 2. Unified Modifiers (Tone, Circumflex, Horn, Breve, D-stroke, Undo Toggle)
        if let Some(effect) = match_modifier_key(
            config.method,
            ch,
            |b| self.vowels.iter().any(|v| v.base == b),
            || {
                self.onset
                    .as_ref()
                    .is_some_and(|o| !o.chars.is_empty() && (o.chars[0] | 0x20) == b'd')
            },
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
                        SyllableState::passthrough(),
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
        if let Some((base_vowel, mut diacritic, tone)) = decompose_vowel(ch) {
            // Glide absorption: "gi" or "qu" followed by another vowel
            if self.vowels.len() == 1
                && let Some(ref mut onset) = self.data.onset
                && onset.chars.len() == 1
            {
                let c0 = onset.chars[0] | 0x20;
                if c0 == b'g'
                    && self.data.vowels[0].base == BaseVowel::I
                    && self.data.vowels[0].diacritic == Diacritic::None
                {
                    onset.push_ascii('i', self.data.vowels[0].is_upper);
                    self.data.vowels.clear();
                } else if c0 == b'q'
                    && self.data.vowels[0].base == BaseVowel::U
                    && self.data.vowels[0].diacritic == Diacritic::None
                {
                    onset.push_ascii('u', self.data.vowels[0].is_upper);
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

                // Upgrade "uơ" to "ươ" when an additional vowel arrives to form a triphthong (e.g. "huơ" + "u" -> "hươu", "huơ" + "i" -> "hươi")
                let mut upgraded_u_horn = false;
                if self.vowels.len() == 2
                    && self.vowels[0].base == BaseVowel::U
                    && self.vowels[0].diacritic == Diacritic::None
                    && self.vowels[1].base == BaseVowel::O
                    && self.vowels[1].diacritic == Diacritic::Horn
                    && matches!(base_vowel, BaseVowel::U | BaseVowel::I)
                {
                    self.vowels[0].diacritic = Diacritic::Horn;
                    upgraded_u_horn = true;
                }

                self.vowels.push(VowelLetter {
                    base: base_vowel,
                    diacritic,
                    is_upper,
                });

                let is_valid = if self.tone != Tone::None {
                    is_valid_vowel_combination(self.vowels.as_slice())
                } else {
                    is_valid_intermediate_vowel_combination(self.vowels.as_slice())
                };

                if !is_valid {
                    return (SyllableState::passthrough(), EngineAction::Passthrough);
                }

                let prev_has_tone = self.tone != Tone::None;
                if tone != Tone::None && self.tone == Tone::None {
                    self.tone = tone;
                }

                // Fast path: plain vowel with no tone shift and no u horn upgrade
                if !upgraded_u_horn
                    && diacritic == Diacritic::None
                    && !prev_has_tone
                    && tone == Tone::None
                {
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
        let is_valid_start = is_valid_coda_start(ch);
        let is_k = ch.eq_ignore_ascii_case(&'k');
        if !is_valid_start && !is_k {
            return (SyllableState::passthrough(), EngineAction::Passthrough);
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
                is_special_k_coda_allowed_during_typing_ascii(&o.chars, o.is_d_stroke, &self.vowels)
            });
        let is_onset_qu = self.onset.as_ref().is_some_and(|o| {
            o.chars.len() == 2 && (o.chars[0] | 0x20) == b'q' && (o.chars[1] | 0x20) == b'u'
        });
        let can_accept_coda = can_vowels_accept_coda(&self.vowels)
            && is_valid_coda_start_for_vowels(is_onset_qu, &self.vowels, ch);
        let is_stop = matches!(ch.to_ascii_lowercase(), 'c' | 'p' | 't' | 'k');
        let tone_incompatible = config.restore_on_wrong_spelling
            && is_stop
            && matches!(self.tone, Tone::Grave | Tone::HookAbove | Tone::Tilde);
        if can_accept_coda && (is_valid_start || is_special_k) && !tone_incompatible {
            let coda = self.into_coda(ch, is_upper);

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
        (SyllableState::passthrough(), EngineAction::Passthrough)
    }
}

// ============================================================================
// 4. Typestate: State<Coda>
// ============================================================================

impl State<Coda> {
    /// Feeds a key into `State<Coda>`, consuming `self` and transitioning to the next state.
    #[inline(always)]
    pub fn feed_key(
        mut self,
        key: RawKey,
        current_len: usize,
        config: &EngineConfig,
    ) -> (SyllableState, EngineAction) {
        let ch = key.ch();
        // 1. Unified Modifiers across Coda (Telex & VNI)
        if let Some(effect) = match_modifier_key(
            config.method,
            ch,
            |b| self.nucleus.vowels.iter().any(|v| v.base == b),
            || {
                self.nucleus
                    .onset
                    .as_ref()
                    .is_some_and(|o| !o.chars.is_empty() && (o.chars[0] | 0x20) == b'd')
            },
        ) {
            let coda_copy = self.data.coda;
            let across_coda = self.data.circumflex_across_coda;
            match self.data.nucleus.apply_modifier(
                effect,
                key,
                Some(coda_copy.as_slice()),
                across_coda,
                config,
            ) {
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
                        SyllableState::passthrough(),
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
        if self.coda.len() == 1 && is_valid_coda_pair(self.coda[0] as char, ch) {
            let b = (ch as u8).to_ascii_lowercase();
            self.coda
                .push(if key.is_upper() { b.to_ascii_uppercase() } else { b });
            return (self.into(), EngineAction::Passthrough);
        }

        // 3. Otherwise -> Transition Coda -> Passthrough
        (SyllableState::passthrough(), EngineAction::Passthrough)
    }
}

// ============================================================================
// 5. Typestate: State<Passthrough>
// ============================================================================

impl State<Passthrough> {
    #[inline(always)]
    pub fn feed_key(self, _key: RawKey) -> (SyllableState, EngineAction) {
        (self.into(), EngineAction::Passthrough)
    }
}

// ============================================================================
// Runtime Dispatch Delegation on SyllableState
// ============================================================================

impl SyllableState {
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
