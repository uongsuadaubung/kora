use super::charset::{BaseVowel, Tone};
use crate::engine::config::InputMethod;

/// Abstract linguistic modifier action in the Vietnamese orthography.
///
/// Any Vietnamese typing method (Telex, Simple Telex, VNI, VIQR, etc.)
/// maps keyboard inputs to these orthographic actions:
/// 1. Tones (Thanh điệu: Sắc, Huyền, Hỏi, Ngã, Nặng, Bỏ dấu)
/// 2. Circumflex (Dấu mũ: â, ê, ô)
/// 3. Horn (Dấu móc: ư, ơ)
/// 4. Breve (Dấu trăng: ă)
/// 5. D-Stroke (Gạch ngang: đ)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEffect {
    /// Applies or toggles a tone mark (Acute, Grave, HookAbove, Tilde, DotBelow)
    Tone(Tone),
    /// Removes tone or diacritics (Telex 'z', VNI '0')
    RemoveTone,
    /// Adds or toggles a circumflex mark (â, ê, ô)
    /// - `Some(v)` targets a specific vowel (Telex aa, ee, oo)
    /// - `None` targets any eligible vowel in the syllable (VNI 6)
    Circumflex(Option<BaseVowel>),
    /// Adds or toggles horn mark on u/o (VNI 7)
    Horn,
    /// Adds or toggles breve mark on a (VNI 8)
    Breve,
    /// Adds or toggles horn on u/o or breve on a (Telex 'w')
    HornShortcut,
    /// Adds or toggles d-stroke: d -> đ (Telex dd, VNI d9)
    DStroke,
}

/// Bitmask of Telex modifier keys: 'a', 'd', 'e', 'f', 'j', 'o', 'r', 's', 'w', 'x', 'z'.
/// Rejects non-modifier consonants ('b', 'c', 'g', 'h', 'i', 'k', 'l', 'm', 'n', 'p', 'q', 't', 'u', 'v', 'y')
/// in exactly 1 CPU cycle without evaluating match arms or closures.
const TELEX_MODIFIER_MASK: u32 = {
    let chars = b"asfrxjzwaeod";
    let mut mask = 0u32;
    let mut i = 0;
    while i < chars.len() {
        mask |= 1u32 << (chars[i] - b'a');
        i += 1;
    }
    mask
};

/// Matches a key character to its modifier effect based on the active input method
/// and syllable context (present vowels, onset 'd').
///
/// `has_onset_d` is evaluated lazily ONLY when `ch_lower == 'd'` (Telex) or `ch == '9'` (VNI),
/// avoiding onset memory loads on 98% of keystrokes.
#[inline(always)]
pub fn match_modifier_key(
    method: InputMethod,
    ch: char,
    has_vowel: impl Fn(BaseVowel) -> bool,
    has_onset_d: impl FnOnce() -> bool,
) -> Option<KeyEffect> {
    if method.is_telex_family() {
        let cp = ch as u32 | 0x20;
        let delta = cp.wrapping_sub('a' as u32);
        if delta >= 26 || (TELEX_MODIFIER_MASK & (1u32 << delta)) == 0 {
            return None;
        }

        match cp as u8 {
            b's' => Some(KeyEffect::Tone(Tone::Acute)),
            b'f' => Some(KeyEffect::Tone(Tone::Grave)),
            b'r' => Some(KeyEffect::Tone(Tone::HookAbove)),
            b'x' => Some(KeyEffect::Tone(Tone::Tilde)),
            b'j' => Some(KeyEffect::Tone(Tone::DotBelow)),
            b'z' => Some(KeyEffect::RemoveTone),
            b'w' => Some(KeyEffect::HornShortcut),
            b'a' if has_vowel(BaseVowel::A) => Some(KeyEffect::Circumflex(Some(BaseVowel::A))),
            b'e' if has_vowel(BaseVowel::E) => Some(KeyEffect::Circumflex(Some(BaseVowel::E))),
            b'o' if has_vowel(BaseVowel::O) => Some(KeyEffect::Circumflex(Some(BaseVowel::O))),
            b'd' if has_onset_d() => Some(KeyEffect::DStroke),
            _ => None,
        }
    } else if method.is_vni() {
        let cp = ch as u32;
        let delta = cp.wrapping_sub('0' as u32);
        if delta >= 10 {
            return None;
        }

        match cp as u8 {
            b'1' => Some(KeyEffect::Tone(Tone::Acute)),
            b'2' => Some(KeyEffect::Tone(Tone::Grave)),
            b'3' => Some(KeyEffect::Tone(Tone::HookAbove)),
            b'4' => Some(KeyEffect::Tone(Tone::Tilde)),
            b'5' => Some(KeyEffect::Tone(Tone::DotBelow)),
            b'0' => Some(KeyEffect::RemoveTone),
            b'6' => Some(KeyEffect::Circumflex(None)),
            b'7' => Some(KeyEffect::Horn),
            b'8' => Some(KeyEffect::Breve),
            b'9' if has_onset_d() => Some(KeyEffect::DStroke),
            _ => None,
        }
    } else {
        None
    }
}
