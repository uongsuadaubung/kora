use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::vietnamese::{
    charset::{BaseVowel, Diacritic, Tone},
    coda::CodaState,
    inline_list::InlineList,
    nucleus::NucleusState,
    onset::OnsetState,
    spelling::{is_valid_onset, is_valid_vietnamese_components},
    syllable::Syllable,
};

// ============================================================================
// Compile-Time Typestate Markers (Zero-Sized Types — 0 bytes at runtime)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Empty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Onset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Nucleus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Coda;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Passthrough;

/// Sealed-style trait associating each compile-time Typestate marker `S`
/// with its exact state payload `S::Data`.
pub trait Phase: Copy + Clone + PartialEq + Eq + Default {
    type Data: Copy + Clone + PartialEq + Eq + Default + fmt::Debug;
}

impl Phase for Empty {
    type Data = ();
}

impl Phase for Onset {
    type Data = OnsetState;
}

impl Phase for Nucleus {
    type Data = NucleusState;
}

impl Phase for Coda {
    type Data = CodaState;
}

impl Phase for Passthrough {
    type Data = InlineList<char, 16>;
}

/// Zero-cost Typestate container parameterized over phase marker `S`.
/// `#[repr(transparent)]` guarantees identical memory layout and register usage
/// to `S::Data` alone, while `_state: PhantomData<S>` enforces compile-time phase transitions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State<S: Phase> {
    pub data: S::Data,
    pub(crate) _state: PhantomData<S>,
}

impl<S: Phase> State<S> {
    #[inline(always)]
    pub const fn from_data(data: S::Data) -> Self {
        Self {
            data,
            _state: PhantomData,
        }
    }
}

impl<S: Phase> Deref for State<S> {
    type Target = S::Data;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<S: Phase> DerefMut for State<S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<S: Phase> fmt::Display for State<S>
where
    S::Data: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl From<State<Empty>> for SyllableState {
    #[inline(always)]
    fn from(_: State<Empty>) -> Self {
        SyllableState::Empty
    }
}

impl From<State<Onset>> for SyllableState {
    #[inline(always)]
    fn from(s: State<Onset>) -> Self {
        SyllableState::Onset(s)
    }
}

impl From<State<Nucleus>> for SyllableState {
    #[inline(always)]
    fn from(s: State<Nucleus>) -> Self {
        SyllableState::Nucleus(s)
    }
}

impl From<State<Coda>> for SyllableState {
    #[inline(always)]
    fn from(s: State<Coda>) -> Self {
        SyllableState::Coda(s)
    }
}

impl From<State<Passthrough>> for SyllableState {
    #[inline(always)]
    fn from(s: State<Passthrough>) -> Self {
        SyllableState::Passthrough(s)
    }
}

/// Runtime dispatch sum-type wrapping the compile-time `State<S>` typestates
/// for storage across OS keystroke callbacks in `TypingBuffer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyllableState {
    #[default]
    Empty,
    Onset(State<Onset>),
    Nucleus(State<Nucleus>),
    Coda(State<Coda>),
    Passthrough(State<Passthrough>),
}

impl SyllableState {
    #[inline(always)]
    pub const fn passthrough(raw: InlineList<char, 16>) -> Self {
        SyllableState::Passthrough(State::from_data(raw))
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
    pub fn is_unambiguous_completed_vietnamese(&self) -> bool {
        if !self.is_valid_spelling() {
            return false;
        }

        let (nucleus, coda) = match self {
            SyllableState::Nucleus(n) => (&n.data, &[][..]),
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

        // 5. Circumflex on 'E' or 'O' ('ê', 'ô') with tone or coda:
        if vowels.iter().any(|v| {
            matches!(v.base, BaseVowel::E | BaseVowel::O) && v.diacritic == Diacritic::Circumflex
        }) && (tone != Tone::None || !coda.is_empty())
        {
            return true;
        }

        // 6. Vietnamese Diphthong / Triphthong with Tone:
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

        // 7. Completed Coda with Tone (excluding English '-st' pattern):
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
            SyllableState::Nucleus(nucleus) => Some((&nucleus.data).into()),
            SyllableState::Coda(coda) => Some((&coda.data).into()),
            SyllableState::Passthrough(_) => None,
        }
    }
}
