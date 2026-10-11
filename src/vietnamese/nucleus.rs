//! Vietnamese Nucleus (Nguyên âm & Dấu thanh) State Representation

use super::VowelLetter;
use super::charset::{BaseVowel, Diacritic, Tone, compose_vowel};
use super::inline_list::InlineList;
use super::modifier::KeyEffect;
use super::onset::OnsetState;
use super::spelling::is_stop_coda;
use super::syllable::Syllable;
use crate::engine::buffer::RawKey;
use crate::engine::config::EngineConfig;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierOutcome {
    Applied,
    Undone(String),
    NotApplied,
}

/// Nucleus (Nguyên âm & Dấu thanh)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NucleusState {
    pub onset: Option<OnsetState>,
    pub vowels: InlineList<VowelLetter, 4>,
    pub tone: Tone,
}

impl NucleusState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_single(onset: Option<OnsetState>, vowel: VowelLetter, tone: Tone) -> Self {
        Self {
            onset,
            vowels: InlineList::from_single(vowel),
            tone,
        }
    }

    #[inline]
    pub fn render_to(&self, out: &mut String) {
        if let Some(ref onset) = self.onset {
            onset.render_to(out);
        }

        let tone_pos = self.find_tone_position();
        for (i, v) in self.vowels.iter().enumerate() {
            let tone_for_vowel = if i == tone_pos { self.tone } else { Tone::None };
            out.push(compose_vowel(
                v.base,
                v.diacritic,
                tone_for_vowel,
                v.is_upper,
            ));
        }
    }

    #[inline]
    pub fn render(&self) -> String {
        let mut s = String::with_capacity(16);
        self.render_to(&mut s);
        s
    }

    pub fn find_tone_position(&self) -> usize {
        let n = self.vowels.len();
        if n <= 1 {
            return 0;
        }

        if n == 2 {
            let v0 = &self.vowels[0];
            let v1 = &self.vowels[1];

            // If ươ (both have horn) -> tone on ơ (index 1)
            if v0.base == BaseVowel::U
                && v0.diacritic == Diacritic::Horn
                && v1.base == BaseVowel::O
                && v1.diacritic == Diacritic::Horn
            {
                return 1;
            }

            // In any diphthong, if second vowel has a diacritic (iê, yê, uô, uê, oă, uơ) -> index 1!
            if v1.diacritic != Diacritic::None {
                return 1;
            }

            // If first vowel has a diacritic (ưa, âu, ây, êu, ưu, ơi, ưi) -> index 0!
            if v0.diacritic != Diacritic::None {
                return 0;
            }

            // Neither vowel has a diacritic:
            // "oa", "oe", "uy": tone is placed on the first vowel (index 0: hòa, khóe, thủy, thúy)
            if (v0.base == BaseVowel::O && (v1.base == BaseVowel::A || v1.base == BaseVowel::E))
                || (v0.base == BaseVowel::U && v1.base == BaseVowel::Y)
            {
                return 0;
            }

            // "ua", "ia", "ya" -> tone on first vowel
            if v1.base == BaseVowel::A {
                return 0;
            }

            // "ai", "oi", "ui", "ay", "ao", "eo", "au", "iu" -> tone on first vowel
            return 0;
        }

        if n == 3 {
            // Triphthongs: tone on middle vowel (index 1)
            return 1;
        }

        0
    }

    /// Converts this Nucleus into a standard Syllable for validation/inspection
    pub fn to_syllable(&self) -> Syllable {
        self.into()
    }

    /// Free mark 'd' stroke toggle: toggles onset 'd' <-> 'đ'
    pub fn toggle_d_stroke(&mut self) -> bool {
        if let Some(ref mut onset) = self.onset
            && !onset.chars.is_empty()
            && onset.chars[0].0.eq_ignore_ascii_case(&'d')
        {
            onset.is_d_stroke = !onset.is_d_stroke;
            return true;
        }
        false
    }

    /// Applies horn (or breve for 'a') to eligible vowels according to Vietnamese phonotactics:
    /// - Single: a->ă, o->ơ, u->ư
    /// - Diphthong "uo" -> "ươ" (both get horn: lươn, được, nước)
    /// - Diphthong "ua" -> "ưa" (only 'u' gets horn: mưa, chưa, xưa)
    /// - Diphthong "oa" -> "oă" (only 'a' gets breve: hoặc, khoăn)
    /// - Diphthongs "ui", "oi" -> "ưi", "ơi" (first vowel gets horn: gửi, chơi)
    /// - Diphthong "iu" -> "ưu" (second vowel gets horn: cừu, lựu)
    /// - Triphthong "uoi", "uou" -> "ươi", "ươu" (u and o get horn)
    pub fn apply_horn(&mut self, has_coda: bool) -> bool {
        match self.vowels.len() {
            0 => false,
            1 => {
                let v = &mut self.vowels[0];
                match v.base {
                    BaseVowel::A if v.diacritic == Diacritic::None => {
                        v.diacritic = Diacritic::Breve;
                        true
                    }
                    BaseVowel::O if v.diacritic == Diacritic::None => {
                        v.diacritic = Diacritic::Horn;
                        true
                    }
                    BaseVowel::U if v.diacritic == Diacritic::None => {
                        v.diacritic = Diacritic::Horn;
                        true
                    }
                    _ => false,
                }
            }
            2 => {
                let (b0, d0) = (self.vowels[0].base, self.vowels[0].diacritic);
                let (b1, d1) = (self.vowels[1].base, self.vowels[1].diacritic);
                match (b0, b1) {
                    // "uo" -> both get horn: "ươ"
                    // Special exception: in Vietnamese, "thuở" and "huơ" have horn only on 'o' (uơ)
                    // when there is no coda ("thưở" and "hươ" do not exist in Vietnamese).
                    (BaseVowel::U, BaseVowel::O)
                        if d0 == Diacritic::None || d1 == Diacritic::None =>
                    {
                        let is_thuo_or_huo = !has_coda
                            && self.onset.as_ref().is_some_and(|o| match o.chars.as_slice() {
                                [(c, _)] => c.eq_ignore_ascii_case(&'h'),
                                [(c0, _), (c1, _)] => {
                                    c0.eq_ignore_ascii_case(&'t') && c1.eq_ignore_ascii_case(&'h')
                                }
                                _ => false,
                            });
                        if is_thuo_or_huo {
                            self.vowels[1].diacritic = Diacritic::Horn;
                        } else {
                            self.vowels[0].diacritic = Diacritic::Horn;
                            self.vowels[1].diacritic = Diacritic::Horn;
                        }
                        true
                    }
                    // "ua" -> only 'u' gets horn: "ưa" (mưa, chưa)
                    (BaseVowel::U, BaseVowel::A) if d0 == Diacritic::None => {
                        self.vowels[0].diacritic = Diacritic::Horn;
                        true
                    }
                    // "oa" -> only 'a' gets breve: "oă" (hoặc, khoăn)
                    (BaseVowel::O, BaseVowel::A) if d1 == Diacritic::None => {
                        self.vowels[1].diacritic = Diacritic::Breve;
                        true
                    }
                    // "ui" -> only 'u' gets horn: "ưi" (gửi, ngửi)
                    (BaseVowel::U, BaseVowel::I) if d0 == Diacritic::None => {
                        self.vowels[0].diacritic = Diacritic::Horn;
                        true
                    }
                    // "oi" -> only 'o' gets horn: "ơi" (chơi, nơi)
                    (BaseVowel::O, BaseVowel::I) if d0 == Diacritic::None => {
                        self.vowels[0].diacritic = Diacritic::Horn;
                        true
                    }
                    // "iu" -> only 'u' gets horn: "ưu" (cừu, lựu)
                    (BaseVowel::I, BaseVowel::U) if d1 == Diacritic::None => {
                        self.vowels[1].diacritic = Diacritic::Horn;
                        true
                    }
                    _ => false,
                }
            }
            3
                // Triphthong e.g. "uoi" -> "ươi", "uou" -> "ươu"
                if self.vowels[0].base == BaseVowel::U && self.vowels[1].base == BaseVowel::O =>
            {
                self.vowels[0].diacritic = Diacritic::Horn;
                self.vowels[1].diacritic = Diacritic::Horn;
                true
            }
            _ => false,
        }
    }

    /// Reverts horn (or breve for 'a') back to plain vowels (undo toggle)
    pub fn revert_horn(&mut self) -> bool {
        let mut reverted = false;
        for v in self.vowels.iter_mut() {
            if matches!(v.diacritic, Diacritic::Horn | Diacritic::Breve) {
                v.diacritic = Diacritic::None;
                reverted = true;
            }
        }
        reverted
    }

    /// Toggle circumflex (aa->â, ee->ê, oo->ô) with reverse scanning across vowels
    pub fn toggle_circumflex(&mut self, base: BaseVowel) -> bool {
        if let Some(v) = self.vowels.iter_mut().rev().find(|v| v.base == base) {
            v.diacritic = if v.diacritic == Diacritic::Circumflex {
                Diacritic::None
            } else {
                Diacritic::Circumflex
            };
            true
        } else {
            false
        }
    }

    /// Unified modifier application (Tone, Circumflex, Horn, Breve, D-stroke, Undo Toggle).
    /// Used by both NucleusState and CodaState across all input methods (Telex, Simple Telex, VNI).
    pub fn apply_modifier(
        &mut self,
        effect: KeyEffect,
        key: RawKey,
        coda: Option<&[(char, bool)]>,
        config: &EngineConfig,
    ) -> ModifierOutcome {
        let make_raw = |n: &NucleusState| -> String {
            let mut s = String::with_capacity(16);
            n.render_to(&mut s);
            if let Some(coda_chars) = coda {
                for &(c, is_upper) in coda_chars {
                    if is_upper {
                        s.push(c.to_ascii_uppercase());
                    } else {
                        s.push(c.to_ascii_lowercase());
                    }
                }
            }
            s.push(key.ch);
            s
        };

        match effect {
            KeyEffect::Circumflex(target) => {
                if let Some(coda_chars) = coda {
                    // English word guard: "data" (onset 'd' without d-stroke + 'a' + 't' + 'a')
                    // In Vietnamese, 'ât' only combines with 'đ' (đất), never with uncrossed 'd'.
                    if self.tone == Tone::None
                        && target == Some(BaseVowel::A)
                        && coda_chars.len() == 1
                        && coda_chars[0].0.eq_ignore_ascii_case(&'t')
                        && self.onset.as_ref().is_some_and(|o| {
                            !o.is_d_stroke
                                && o.chars.len() == 1
                                && o.chars[0].0.eq_ignore_ascii_case(&'d')
                        })
                    {
                        return ModifierOutcome::NotApplied;
                    }
                }

                match target {
                    Some(target_base) => {
                        // Double vowel circumflex (Telex: aa -> â, ee -> ê, oo -> ô)
                        if let Some(pos) = self.vowels.iter().rposition(|v| {
                            v.base == target_base && v.diacritic == Diacritic::Circumflex
                        }) {
                            self.vowels[pos].diacritic = Diacritic::None;
                            ModifierOutcome::Undone(make_raw(self))
                        } else if self.toggle_circumflex(target_base) {
                            ModifierOutcome::Applied
                        } else {
                            ModifierOutcome::NotApplied
                        }
                    }
                    None => {
                        // Generic circumflex (VNI: 6)
                        if self.toggle_circumflex(BaseVowel::A)
                            || self.toggle_circumflex(BaseVowel::E)
                            || self.toggle_circumflex(BaseVowel::O)
                        {
                            ModifierOutcome::Applied
                        } else {
                            ModifierOutcome::NotApplied
                        }
                    }
                }
            }

            KeyEffect::DStroke => {
                if let Some(ref mut onset) = self.onset {
                    if !onset.chars.is_empty() && onset.chars[0].0.eq_ignore_ascii_case(&'d') {
                        if config.method.is_telex_family() && onset.is_d_stroke {
                            // Telex undo toggle: 3rd 'd' cancels 'đ' into 'dd'
                            onset.is_d_stroke = false;
                            ModifierOutcome::Undone(make_raw(self))
                        } else {
                            onset.is_d_stroke = !onset.is_d_stroke;
                            ModifierOutcome::Applied
                        }
                    } else {
                        ModifierOutcome::NotApplied
                    }
                } else {
                    ModifierOutcome::NotApplied
                }
            }

            KeyEffect::HornShortcut => {
                // Standalone 'w' undo: ww -> w
                if coda.is_none()
                    && self.onset.is_none()
                    && self.vowels.len() == 1
                    && self.vowels[0].base == BaseVowel::U
                    && self.vowels[0].diacritic == Diacritic::Horn
                {
                    let raw_w = if key.is_upper { "W" } else { "w" }.to_string();
                    return ModifierOutcome::Undone(raw_w);
                }

                if self.apply_horn(coda.is_some()) {
                    ModifierOutcome::Applied
                } else if self.revert_horn() {
                    ModifierOutcome::Undone(make_raw(self))
                } else {
                    ModifierOutcome::NotApplied
                }
            }

            KeyEffect::Horn => {
                if self.apply_horn(coda.is_some()) {
                    ModifierOutcome::Applied
                } else {
                    ModifierOutcome::NotApplied
                }
            }

            KeyEffect::Breve => {
                if let Some(v) = self
                    .vowels
                    .iter_mut()
                    .rev()
                    .find(|v| v.base == BaseVowel::A)
                {
                    v.diacritic = if v.diacritic == Diacritic::Breve {
                        Diacritic::None
                    } else {
                        Diacritic::Breve
                    };
                    ModifierOutcome::Applied
                } else {
                    ModifierOutcome::NotApplied
                }
            }

            KeyEffect::Tone(tone) => {
                if self.tone == tone {
                    if config.method.is_telex_family() {
                        self.tone = Tone::None;
                        ModifierOutcome::Undone(make_raw(self))
                    } else {
                        self.tone = Tone::None;
                        ModifierOutcome::Applied
                    }
                } else {
                    self.tone = tone;
                    if let Some(coda_chars) = coda
                        && is_stop_coda(coda_chars)
                        && !matches!(self.tone, Tone::Acute | Tone::DotBelow)
                    {
                        self.tone = Tone::None;
                        return ModifierOutcome::NotApplied;
                    }
                    ModifierOutcome::Applied
                }
            }

            KeyEffect::RemoveTone => {
                if self.tone != Tone::None {
                    self.tone = Tone::None;
                    ModifierOutcome::Applied
                } else if config.method.is_telex_family() {
                    let mut changed = false;
                    for v in self.vowels.iter_mut() {
                        if v.diacritic != Diacritic::None {
                            v.diacritic = Diacritic::None;
                            changed = true;
                        }
                    }
                    if let Some(ref mut onset) = self.onset
                        && onset.is_d_stroke
                    {
                        onset.is_d_stroke = false;
                        changed = true;
                    }
                    if changed {
                        ModifierOutcome::Applied
                    } else {
                        ModifierOutcome::NotApplied
                    }
                } else {
                    ModifierOutcome::NotApplied
                }
            }
        }
    }
}

impl fmt::Display for NucleusState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(16);
        self.render_to(&mut s);
        f.write_str(&s)
    }
}

impl From<&NucleusState> for Syllable {
    fn from(nucleus: &NucleusState) -> Self {
        Syllable {
            onset: nucleus
                .onset
                .as_ref()
                .map(|o| o.chars.to_vec())
                .unwrap_or_default(),
            d_stroke: nucleus.onset.as_ref().is_some_and(|o| o.is_d_stroke),
            vowels: nucleus.vowels.to_vec(),
            tone: nucleus.tone,
            coda: Vec::new(),
        }
    }
}
