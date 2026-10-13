use super::{
    VowelLetter,
    charset::{BaseVowel, Diacritic, Tone},
    syllable::Syllable,
};

/// Compact 5-bit packing of a vowel for O(1) bitmask phonotactic lookup:
/// 3 bits for base (0..5) + 2 bits for diacritic (0..3) = 5 bits (0..23).
#[inline(always)]
const fn pack_vowel(base: BaseVowel, diacritic: Diacritic) -> usize {
    ((base as usize) << 2) | (diacritic as usize)
}

/// Compact 15-bit packing of a 3-vowel cluster for O(1) integer matching:
/// 3 vowels * 5 bits each = 15 bits, fits inside a single u16.
#[inline(always)]
const fn pack_3vowels(
    v0: (BaseVowel, Diacritic),
    v1: (BaseVowel, Diacritic),
    v2: (BaseVowel, Diacritic),
) -> u16 {
    ((pack_vowel(v0.0, v0.1) as u16) << 10)
        | ((pack_vowel(v1.0, v1.1) as u16) << 5)
        | (pack_vowel(v2.0, v2.1) as u16)
}

// 15 legitimate completed 3-vowel clusters in Vietnamese
const TRIPHTHONG_IEU: u16 = pack_3vowels((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None));
const TRIPHTHONG_YEU: u16 = pack_3vowels((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None));
const TRIPHTHONG_OAI: u16 = pack_3vowels((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::I, Diacritic::None));
const TRIPHTHONG_OAY: u16 = pack_3vowels((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::Y, Diacritic::None));
const TRIPHTHONG_OAO: u16 = pack_3vowels((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::O, Diacritic::None));
const TRIPHTHONG_OEO: u16 = pack_3vowels((BaseVowel::O, Diacritic::None), (BaseVowel::E, Diacritic::None), (BaseVowel::O, Diacritic::None));
const TRIPHTHONG_UAI: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::I, Diacritic::None));
const TRIPHTHONG_UAY: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::Y, Diacritic::None));
const TRIPHTHONG_UOI_CIR: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Circumflex), (BaseVowel::I, Diacritic::None));
const TRIPHTHONG_UOI_HORN: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Horn), (BaseVowel::I, Diacritic::None));
const TRIPHTHONG_UOI_DOUBLE_HORN: u16 = pack_3vowels((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn), (BaseVowel::I, Diacritic::None));
const TRIPHTHONG_UOU_DOUBLE_HORN: u16 = pack_3vowels((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn), (BaseVowel::U, Diacritic::None));
const TRIPHTHONG_UYA: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None), (BaseVowel::A, Diacritic::None));
const TRIPHTHONG_UYE: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex));
const TRIPHTHONG_UYU: u16 = pack_3vowels((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None), (BaseVowel::U, Diacritic::None));

/// Bitmask lookup table for legitimate completed 2-vowel combinations in Vietnamese.
/// 24 * 24 = 576 possible pairs mapped to 16 u64 words (1024 bits).
/// Eliminates 33 complex tuple match arms into a single O(1) bitwise AND.
const VALID_2VOWELS_MASK: [u64; 16] = {
    let mut table = [0u64; 16];
    let valid_pairs: &[((BaseVowel, Diacritic), (BaseVowel, Diacritic))] = &[
        // ai, ao, au, ay, ây, âu
        ((BaseVowel::A, Diacritic::None), (BaseVowel::I, Diacritic::None)),
        ((BaseVowel::A, Diacritic::None), (BaseVowel::O, Diacritic::None)),
        ((BaseVowel::A, Diacritic::None), (BaseVowel::U, Diacritic::None)),
        ((BaseVowel::A, Diacritic::None), (BaseVowel::Y, Diacritic::None)),
        ((BaseVowel::A, Diacritic::Circumflex), (BaseVowel::Y, Diacritic::None)),
        ((BaseVowel::A, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None)),
        // eo, êu
        ((BaseVowel::E, Diacritic::None), (BaseVowel::O, Diacritic::None)),
        ((BaseVowel::E, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None)),
        // ia, iê, iu
        ((BaseVowel::I, Diacritic::None), (BaseVowel::A, Diacritic::None)),
        ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex)),
        ((BaseVowel::I, Diacritic::None), (BaseVowel::U, Diacritic::None)),
        // oa, oă, oe, oi, ôi, ơi, oo (xoong, boong)
        ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None)),
        ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::Breve)),
        ((BaseVowel::O, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::O, Diacritic::None), (BaseVowel::I, Diacritic::None)),
        ((BaseVowel::O, Diacritic::Circumflex), (BaseVowel::I, Diacritic::None)),
        ((BaseVowel::O, Diacritic::Horn), (BaseVowel::I, Diacritic::None)),
        ((BaseVowel::O, Diacritic::None), (BaseVowel::O, Diacritic::None)),
        // ua, uâ, uô, uơ, uê, ui, uy
        ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::Circumflex)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Circumflex)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Horn)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::I, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None)),
        // ưa, ươ, ưu, ưi
        ((BaseVowel::U, Diacritic::Horn), (BaseVowel::A, Diacritic::None)),
        ((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn)),
        ((BaseVowel::U, Diacritic::Horn), (BaseVowel::U, Diacritic::None)),
        ((BaseVowel::U, Diacritic::Horn), (BaseVowel::I, Diacritic::None)),
        // ya, yê
        ((BaseVowel::Y, Diacritic::None), (BaseVowel::A, Diacritic::None)),
        ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex)),
    ];

    let mut i = 0;
    while i < valid_pairs.len() {
        let (v0, v1) = valid_pairs[i];
        let idx = (pack_vowel(v0.0, v0.1) << 5) | pack_vowel(v1.0, v1.1);
        table[idx >> 6] |= 1u64 << (idx & 63);
        i += 1;
    }
    table
};

/// Bitmask lookup table for valid intermediate 2-vowel typing combinations.
/// Seamlessly includes all completed pairs plus typing bases (ie, ye, uo, ue, uu, ee, eu).
const VALID_INTERMEDIATE_2VOWELS_MASK: [u64; 16] = {
    let mut table = VALID_2VOWELS_MASK;
    let extra_intermediate: &[((BaseVowel, Diacritic), (BaseVowel, Diacritic))] = &[
        // Intermediate typing bases for iê, yê, uô, ươ, uê, ê, êu, ưu
        ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::U, Diacritic::None)),
        ((BaseVowel::E, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::E, Diacritic::None), (BaseVowel::U, Diacritic::None)),
    ];

    let mut i = 0;
    while i < extra_intermediate.len() {
        let (v0, v1) = extra_intermediate[i];
        let idx = (pack_vowel(v0.0, v0.1) << 5) | pack_vowel(v1.0, v1.1);
        table[idx >> 6] |= 1u64 << (idx & 63);
        i += 1;
    }
    table
};

/// Validates whether a sequence of vowels forms a legitimate Vietnamese vowel cluster.
/// Uses O(1) compile-time bitmask lookup for 2-vowel diphthongs (replaces 33 match arms)
/// and packed integer comparisons for 3-vowel triphthongs (replaces 15 tuple match arms).
#[inline]
pub fn is_valid_vowel_combination(vowels: &[VowelLetter]) -> bool {
    match vowels.len() {
        0 | 1 => true,
        2 => {
            // O(1) bitwise AND against compile-time bitmask: zero branches, zero jumps!
            let idx = (pack_vowel(vowels[0].base, vowels[0].diacritic) << 5)
                | pack_vowel(vowels[1].base, vowels[1].diacritic);
            (VALID_2VOWELS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0
        }
        3 => {
            // O(1) packed u16 integer matching against 15 constant triphthongs
            let code = pack_3vowels(
                (vowels[0].base, vowels[0].diacritic),
                (vowels[1].base, vowels[1].diacritic),
                (vowels[2].base, vowels[2].diacritic),
            );
            matches!(
                code,
                TRIPHTHONG_IEU
                    | TRIPHTHONG_YEU
                    | TRIPHTHONG_OAI
                    | TRIPHTHONG_OAY
                    | TRIPHTHONG_OAO
                    | TRIPHTHONG_OEO
                    | TRIPHTHONG_UAI
                    | TRIPHTHONG_UAY
                    | TRIPHTHONG_UOI_CIR
                    | TRIPHTHONG_UOI_HORN
                    | TRIPHTHONG_UOI_DOUBLE_HORN
                    | TRIPHTHONG_UOU_DOUBLE_HORN
                    | TRIPHTHONG_UYA
                    | TRIPHTHONG_UYE
                    | TRIPHTHONG_UYU
            )
        }
        _ => false,
    }
}

/// Validates whether a sequence of vowels forms a legitimate intermediate or completed Vietnamese vowel cluster during typing.
/// Uses O(1) compile-time bitmask for 2-vowel pairs and packed base comparison for 3-vowel triphthongs.
#[inline]
pub fn is_valid_intermediate_vowel_combination(vowels: &[VowelLetter]) -> bool {
    match vowels.len() {
        0 | 1 => true,
        2 => {
            // O(1) bitwise AND against intermediate bitmask: checks completed + typing bases in 1 instruction!
            let idx = (pack_vowel(vowels[0].base, vowels[0].diacritic) << 5)
                | pack_vowel(vowels[1].base, vowels[1].diacritic);
            (VALID_INTERMEDIATE_2VOWELS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0
        }
        3 => {
            if is_valid_vowel_combination(vowels) {
                return true;
            }
            // Intermediate typing bases for triphthongs (before diacritics/horns are added):
            // uye, ieu, yeu, uoi, uou
            let base_code = ((vowels[0].base as u16) << 6)
                | ((vowels[1].base as u16) << 3)
                | (vowels[2].base as u16);
            const UYE: u16 = ((BaseVowel::U as u16) << 6) | ((BaseVowel::Y as u16) << 3) | (BaseVowel::E as u16);
            const IEU: u16 = ((BaseVowel::I as u16) << 6) | ((BaseVowel::E as u16) << 3) | (BaseVowel::U as u16);
            const YEU: u16 = ((BaseVowel::Y as u16) << 6) | ((BaseVowel::E as u16) << 3) | (BaseVowel::U as u16);
            const UOI: u16 = ((BaseVowel::U as u16) << 6) | ((BaseVowel::O as u16) << 3) | (BaseVowel::I as u16);
            const UOU: u16 = ((BaseVowel::U as u16) << 6) | ((BaseVowel::O as u16) << 3) | (BaseVowel::U as u16);
            matches!(base_code, UYE | IEU | YEU | UOI | UOU)
        }
        _ => false,
    }
}

/// Bitmask for valid 1-letter Vietnamese onsets: b, c, d, g, h, k, l, m, n, p, q, r, s, t, v, x.
const SINGLE_ONSET_MASK: u32 = {
    let mut mask = 0u32;
    let chars = b"bcdghklmnpqrstvx";
    let mut i = 0;
    while i < chars.len() {
        mask |= 1u32 << (chars[i] - b'a');
        i += 1;
    }
    mask
};

/// Bitmask table for valid 2-letter Vietnamese onsets:
/// ch, gh, gi, kh, nh, ng, ph, qu, th, tr.
/// Indexed by ((c0 - 'a') << 5) | (c1 - 'a') in 0..825 (16 u64 words = 1024 bits = 128 bytes).
const ONSET_2LETTERS_MASK: [u64; 16] = {
    let mut table = [0u64; 16];
    let pairs: &[(u8, u8)] = &[
        (b'c', b'h'), (b'g', b'h'), (b'g', b'i'), (b'k', b'h'),
        (b'n', b'h'), (b'n', b'g'), (b'p', b'h'), (b'q', b'u'),
        (b't', b'h'), (b't', b'r'),
    ];
    let mut i = 0;
    while i < pairs.len() {
        let (c0, c1) = pairs[i];
        let idx = ((c0 - b'a') as usize) << 5 | ((c1 - b'a') as usize);
        table[idx >> 6] |= 1u64 << (idx & 63);
        i += 1;
    }
    table
};

/// Validates whether an onset consonant sequence is a legitimate Vietnamese onset.
/// Evaluated with O(1) bitmasks: zero branch mispredictions!
#[inline]
pub fn is_valid_onset(onset: &[(char, bool)], is_d_stroke: bool) -> bool {
    if onset.is_empty() {
        return true;
    }
    if is_d_stroke {
        return onset.len() == 1 && onset[0].0.eq_ignore_ascii_case(&'d');
    }
    match onset.len() {
        1 => {
            let c = onset[0].0.to_ascii_lowercase();
            if c.is_ascii_lowercase() {
                (SINGLE_ONSET_MASK & (1u32 << (c as u32 - 'a' as u32))) != 0
            } else {
                false
            }
        }
        2 => {
            let c0 = onset[0].0.to_ascii_lowercase();
            let c1 = onset[1].0.to_ascii_lowercase();
            if c0.is_ascii_lowercase() && c1.is_ascii_lowercase() {
                let idx = ((c0 as usize - 'a' as usize) << 5) | (c1 as usize - 'a' as usize);
                (ONSET_2LETTERS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0
            } else {
                false
            }
        }
        3 => {
            let c0 = onset[0].0.to_ascii_lowercase();
            let c1 = onset[1].0.to_ascii_lowercase();
            let c2 = onset[2].0.to_ascii_lowercase();
            c0 == 'n' && c1 == 'g' && c2 == 'h'
        }
        _ => false,
    }
}

/// Validates whether syllable components satisfy Vietnamese spelling / phonotactic rules without heap allocation.
pub fn is_valid_vietnamese_components(
    onset: &[(char, bool)],
    d_stroke: bool,
    vowels: &[VowelLetter],
    tone: Tone,
    coda: &[(char, bool)],
) -> bool {
    // 1. Validate onset
    if !is_valid_onset(onset, d_stroke) {
        return false;
    }

    if vowels.is_empty() {
        return true;
    }

    // 2. Validate onset + vowel orthography
    if !is_valid_onset_vowel_orthography(onset, d_stroke, vowels[0].base) {
        return false;
    }

    // Special glides:
    // If onset is 'g' and first vowel is 'i' (e.g. "gió", "giờ", "giúp", "giường", "giếng"),
    // or onset is 'q' and first vowel is 'u' (e.g. "qua", "quê", "quốc", "quần"),
    // the 'i' or 'u' acts as the glide consonant extension, and the vowel cluster starts at index 1!
    let last_onset = onset.last().map(|(c, _)| c.to_ascii_lowercase());
    let vowels_to_check = if vowels.len() >= 2 {
        if (last_onset == Some('g') && vowels[0].base == BaseVowel::I)
            || (last_onset == Some('q') && vowels[0].base == BaseVowel::U)
        {
            &vowels[1..]
        } else {
            vowels
        }
    } else {
        vowels
    };

    // 3. Valid vowel cluster
    if !is_valid_vowel_combination(vowels_to_check) {
        return false;
    }

    // 4. Validate coda
    if !coda.is_empty() {
        if !can_vowels_accept_coda(vowels_to_check) {
            return false;
        }

        let is_qu = (onset.len() == 2
            && onset[0].0.eq_ignore_ascii_case(&'q')
            && onset[1].0.eq_ignore_ascii_case(&'u'))
            || (onset.len() == 1
                && onset[0].0.eq_ignore_ascii_case(&'q')
                && !vowels.is_empty()
                && vowels[0].base == BaseVowel::U);

        match coda.len() {
            1 => {
                let c = coda[0].0.to_ascii_lowercase();
                if !is_valid_coda_start_for_vowels(is_qu, vowels_to_check, c) {
                    return false;
                }
                // In completed syllable, front vowel 'i' never takes single 'c' (must be 'ch': thích, lịch, ích).
                if vowels_to_check.len() == 1
                    && vowels_to_check[0].base == BaseVowel::I
                    && c == 'c'
                {
                    return false;
                }
                if c == 'k' {
                    if !is_special_k_coda_allowed_completed(onset, d_stroke, vowels_to_check) {
                        return false;
                    }
                } else if !matches!(c, 'c' | 'm' | 'n' | 'p' | 't' | 'g' | 'h') {
                    return false;
                }
            }
            2 => {
                if !is_valid_coda_pair(coda[0].0, coda[1].0) {
                    return false;
                }
            }
            _ => return false,
        }

        // Stop codas (c, ch, p, t, k) only accept Acute (Sắc) or DotBelow (Nặng), or Tone::None (for Dak Lak loanwords / typing bases).
        // Nasal coda 'nh' accepts all 6 tones.
        if is_stop_coda(coda) {
            match tone {
                Tone::None | Tone::Acute | Tone::DotBelow => {}
                _ => return false,
            }
        }
    }

    true
}

/// Validates whether a syllable satisfies Vietnamese spelling / phonotactic rules.
#[inline]
pub fn is_valid_vietnamese_syllable(syllable: &Syllable) -> bool {
    is_valid_vietnamese_components(
        &syllable.onset,
        syllable.d_stroke,
        &syllable.vowels,
        syllable.tone,
        &syllable.coda,
    )
}

/// Bitmask for 1-letter stop codas in Vietnamese: 'c', 'p', 't', 'k'.
/// Stop codas only accept Acute (Sắc) or DotBelow (Nặng) tones, or Tone::None.
const STOP_CODA_CHARS_MASK: u32 = {
    let mut mask = 0u32;
    let chars = b"cptk";
    let mut i = 0;
    while i < chars.len() {
        mask |= 1u32 << (chars[i] - b'a');
        i += 1;
    }
    mask
};

/// Returns true if the coda is a stop coda ('c', 'ch', 'p', 't', 'k').
/// Evaluated with O(1) bitmask: zero branch mispredictions on 1-letter codas.
#[inline]
pub fn is_stop_coda(coda: &[(char, bool)]) -> bool {
    match coda.len() {
        1 => {
            let c = coda[0].0.to_ascii_lowercase();
            if c.is_ascii_lowercase() {
                (STOP_CODA_CHARS_MASK & (1u32 << (c as u32 - 'a' as u32))) != 0
            } else {
                false
            }
        }
        2 => {
            let c1 = coda[0].0.to_ascii_lowercase();
            let c2 = coda[1].0.to_ascii_lowercase();
            c1 == 'c' && c2 == 'h'
        }
        _ => false,
    }
}

/// Validates whether an onset consonant sequence can be extended with next_ch.
/// Vietnamese multi-letter onsets:
/// - 2 letters: ch, gh, gi, kh, nh, ng, ph, qu, th, tr
/// - 3 letters: ngh
#[inline]
pub fn is_valid_onset_extension(current_onset: &[(char, bool)], next_ch: char) -> bool {
    let next_lower = next_ch.to_ascii_lowercase();
    if !next_lower.is_ascii_lowercase() {
        return false;
    }
    match current_onset.len() {
        1 => {
            let first = current_onset[0].0.to_ascii_lowercase();
            if first.is_ascii_lowercase() {
                let idx = ((first as usize - 'a' as usize) << 5) | (next_lower as usize - 'a' as usize);
                (ONSET_2LETTERS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0
            } else {
                false
            }
        }
        2 => {
            let first = current_onset[0].0.to_ascii_lowercase();
            let second = current_onset[1].0.to_ascii_lowercase();
            first == 'n' && second == 'g' && next_lower == 'h'
        }
        _ => false,
    }
}

/// Validates whether 'k' is permitted as a coda for this syllable/nucleus during typing.
/// In standard Vietnamese, 'k' is never a coda. However, in official Vietnamese administrative
/// and geographical names (e.g. Đắk Lắk, Đắk Nông, Đắk Hà, Đăk Đoa, hồ Lắk), 'k' is used:
/// - Onset 'd'/'đ' with vowel 'a' (user may type 'w' or 'd' later: dawkd, dakswd, etc.)
/// - Onset 'l'/'L' with vowel 'a' (user may type 'w' or 's' later: lakws, laksw, etc.)
pub fn is_special_k_coda_allowed_during_typing(
    onset_chars: &[(char, bool)],
    is_d_stroke: bool,
    vowels: &[VowelLetter],
) -> bool {
    if vowels.len() != 1 || vowels[0].base != BaseVowel::A {
        return false;
    }

    is_d_stroke
        || (!onset_chars.is_empty() && onset_chars[0].0.eq_ignore_ascii_case(&'d'))
        || (onset_chars.len() == 1 && onset_chars[0].0.eq_ignore_ascii_case(&'l'))
}

/// Validates whether a completed syllable with 'k' coda is valid Vietnamese spelling.
/// For completed syllables:
/// - 'đ' onset allows 'a' (Đak) or 'ă' (Đăk, Đắk)
/// - 'l' onset requires 'ă' (Lăk, Lắk)
pub fn is_special_k_coda_allowed_completed(
    onset_chars: &[(char, bool)],
    is_d_stroke: bool,
    vowels: &[VowelLetter],
) -> bool {
    if vowels.len() != 1 || vowels[0].base != BaseVowel::A {
        return false;
    }

    if is_d_stroke {
        vowels[0].diacritic == Diacritic::Breve || vowels[0].diacritic == Diacritic::None
    } else if onset_chars.len() == 1 && onset_chars[0].0.eq_ignore_ascii_case(&'l') {
        vowels[0].diacritic == Diacritic::Breve
    } else {
        false
    }
}

/// Bitmask for valid 1-letter coda start consonants in Vietnamese: c, m, n, p, t.
const CODA_START_CHARS_MASK: u32 = {
    let mut mask = 0u32;
    let chars = b"cmnpt";
    let mut i = 0;
    while i < chars.len() {
        mask |= 1u32 << (chars[i] - b'a');
        i += 1;
    }
    mask
};

/// Validates whether a consonant can legitimately begin a Vietnamese coda (c, m, n, p, t).
/// Evaluated via O(1) bitwise test against CODA_START_CHARS_MASK.
#[inline]
pub fn is_valid_coda_start(ch: char) -> bool {
    let c = ch.to_ascii_lowercase();
    if c.is_ascii_lowercase() {
        (CODA_START_CHARS_MASK & (1u32 << (c as u32 - 'a' as u32))) != 0
    } else {
        false
    }
}

/// Bitmask table for valid 2-letter Vietnamese codas: ng, nh, ch.
/// Indexed by ((c0 - 'a') << 5) | (c1 - 'a') in 0..825 (16 u64 words = 1024 bits = 128 bytes).
const CODA_2LETTERS_MASK: [u64; 16] = {
    let mut table = [0u64; 16];
    let pairs: &[(u8, u8)] = &[(b'n', b'g'), (b'n', b'h'), (b'c', b'h')];
    let mut i = 0;
    while i < pairs.len() {
        let (c0, c1) = pairs[i];
        let idx = ((c0 - b'a') as usize) << 5 | ((c1 - b'a') as usize);
        table[idx >> 6] |= 1u64 << (idx & 63);
        i += 1;
    }
    table
};

/// Validates whether two consonants form a legitimate 2-letter Vietnamese coda ('ng', 'nh', 'ch').
/// Evaluated with O(1) bitmask table lookup: zero branches, zero jumps.
#[inline]
pub fn is_valid_coda_pair(first: char, second: char) -> bool {
    let c0 = first.to_ascii_lowercase();
    let c1 = second.to_ascii_lowercase();
    if c0.is_ascii_lowercase() && c1.is_ascii_lowercase() {
        let idx = ((c0 as usize - 'a' as usize) << 5) | (c1 as usize - 'a' as usize);
        (CODA_2LETTERS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0
    } else {
        false
    }
}

/// Validates whether a nucleus vowel combination can legally accept a coda consonant in Vietnamese.
/// In Vietnamese phonotactics:
/// - Single vowels can accept a coda: an, em, in, on, un, etc.
/// - Centering diphthongs can accept a coda: iê/yê, uô, ươ, uâ, oă, oa, oe, uê, uy.
/// - Off-glide diphthongs CANNOT accept a coda: ai, oi, ôi, ơi, ui, ưi, ay, ây, ao, eo, au, âu, iu, ưu, ia, ya, ua, ưa.
/// - Triphthongs ending in semivowels CANNOT accept a coda: iêu, yêu, oai, oay, oao, oeo, uai, uay, uôi, uơi, ươi, ươu, uya, uyu.
/// - Only triphthong 'uyê' can accept a coda: uyên, uyết.

/// Bitmask lookup table for 2-vowel combinations that can legally accept a coda.
/// Total possible pairs: 24 * 24 = 576 entries -> 576 bits = 9 u64 words (allocated 16 u64 words = 1024 bits).
/// This eliminates 16 pattern branches down to a single O(1) bitwise AND instruction.
const CODA_ACCEPT_2VOWELS_MASK: [u64; 16] = {
    let mut table = [0u64; 16];
    let valid_pairs: &[((BaseVowel, Diacritic), (BaseVowel, Diacritic))] = &[
        // iê, yê, and intermediate typing bases (ie, ye for liene -> liên, yens -> yến)
        ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex)),
        ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex)),
        ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        // uô, ươ, uo (uo is the typing base for uô/ươ when free_mark is used)
        ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Circumflex)),
        ((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::None)),
        // uâ, ua (ua is the typing base for uâ in xuana -> xuân)
        ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::Circumflex)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None)),
        // oă
        ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::Breve)),
        // oa, oe, uê, uy, and ue (typing base for quene -> quên)
        ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None)),
        ((BaseVowel::O, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::None)),
        ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None)),
        // oo (xoong, boong)
        ((BaseVowel::O, Diacritic::None), (BaseVowel::O, Diacritic::None)),
    ];

    let mut i = 0;
    while i < valid_pairs.len() {
        let (v0, v1) = valid_pairs[i];
        let idx = (pack_vowel(v0.0, v0.1) << 5) | pack_vowel(v1.0, v1.1);
        table[idx >> 6] |= 1u64 << (idx & 63);
        i += 1;
    }
    table
};

#[inline]
pub fn can_vowels_accept_coda(vowels: &[VowelLetter]) -> bool {
    match vowels.len() {
        0 => false,
        1 => true,
        2 => {
            // O(1) bitwise AND against compile-time bitmask table: zero branches, zero jumps!
            let idx = (pack_vowel(vowels[0].base, vowels[0].diacritic) << 5)
                | pack_vowel(vowels[1].base, vowels[1].diacritic);
            (CODA_ACCEPT_2VOWELS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0
        }
        3 => {
            // In Vietnamese, only 'uyê' (uyên, uyết) and intermediate typing base 'uye' can accept a coda
            vowels[0].base == BaseVowel::U
                && vowels[0].diacritic == Diacritic::None
                && vowels[1].base == BaseVowel::Y
                && vowels[1].diacritic == Diacritic::None
                && vowels[2].base == BaseVowel::E
                && (vowels[2].diacritic == Diacritic::Circumflex
                    || vowels[2].diacritic == Diacritic::None)
        }
        _ => false,
    }
}

// 6-bit masks representing allowed BaseVowel sets for onset consonants.
// Bit 0: A (0), Bit 1: E (1), Bit 2: I (2), Bit 3: O (3), Bit 4: U (4), Bit 5: Y (5).
const VOWEL_MASK_E: u8 = 1 << (BaseVowel::E as u8);
const VOWEL_MASK_I: u8 = 1 << (BaseVowel::I as u8);
const VOWEL_MASK_U: u8 = 1 << (BaseVowel::U as u8);
const VOWEL_MASK_Y: u8 = 1 << (BaseVowel::Y as u8);

const VOWEL_ALL: u8 = 0b0011_1111;
const VOWEL_NO_Y: u8 = VOWEL_ALL & !VOWEL_MASK_Y;
const VOWEL_FRONT: u8 = VOWEL_MASK_I | VOWEL_MASK_Y | VOWEL_MASK_E; // k goes with i, y, e/ê
const VOWEL_NON_FRONT: u8 = VOWEL_ALL & !VOWEL_FRONT;               // c goes with a, o, u (never i, y, e)
const VOWEL_G: u8 = VOWEL_ALL & !(VOWEL_MASK_E | VOWEL_MASK_Y);     // g goes with a, o, u, i (never e, y)
const VOWEL_Q: u8 = VOWEL_MASK_U;                                   // q always requires u
const VOWEL_GH_NGH: u8 = VOWEL_MASK_I | VOWEL_MASK_E;               // gh, ngh only go with i, e/ê
const VOWEL_NG: u8 = VOWEL_ALL & !(VOWEL_MASK_I | VOWEL_MASK_E | VOWEL_MASK_Y); // ng never with i, e, y

/// Lookup table of allowed vowels for 1-letter onsets: [u8; 26] (26 bytes).
const SINGLE_ONSET_ALLOWED_VOWELS: [u8; 26] = {
    let mut table = [VOWEL_ALL; 26];
    table[(b'q' - b'a') as usize] = VOWEL_Q;
    table[(b'k' - b'a') as usize] = VOWEL_FRONT;
    table[(b'c' - b'a') as usize] = VOWEL_NON_FRONT;
    table[(b'g' - b'a') as usize] = VOWEL_G;
    table[(b'b' - b'a') as usize] = VOWEL_NO_Y;
    table[(b'd' - b'a') as usize] = VOWEL_NO_Y;
    table[(b'r' - b'a') as usize] = VOWEL_NO_Y;
    table
};

/// Lookup table of allowed vowels for 2-letter onsets: [u8; 1024] (1 KB).
/// Indexed by ((c0 - 'a') << 5) | (c1 - 'a').
const DOUBLE_ONSET_ALLOWED_VOWELS: [u8; 1024] = {
    let mut table = [VOWEL_ALL; 1024];
    table[(((b'g' - b'a') as usize) << 5) | ((b'h' - b'a') as usize)] = VOWEL_GH_NGH;
    table[(((b'n' - b'a') as usize) << 5) | ((b'g' - b'a') as usize)] = VOWEL_NG;
    table[(((b'p' - b'a') as usize) << 5) | ((b'h' - b'a') as usize)] = VOWEL_NO_Y;
    table[(((b'k' - b'a') as usize) << 5) | ((b'h' - b'a') as usize)] = VOWEL_NO_Y;
    table
};

/// Validates whether an onset consonant and the first vowel base satisfy Vietnamese orthography rules.
/// Evaluated with O(1) bitwise AND against compile-time mask tables: zero pattern-matching branches!
#[inline]
pub fn is_valid_onset_vowel_orthography(
    onset: &[(char, bool)],
    d_stroke: bool,
    first_vowel_base: BaseVowel,
) -> bool {
    if onset.is_empty() || d_stroke {
        return true;
    }

    let allowed_mask = match onset.len() {
        1 => {
            let c = onset[0].0.to_ascii_lowercase();
            if c.is_ascii_lowercase() {
                SINGLE_ONSET_ALLOWED_VOWELS[(c as usize) - ('a' as usize)]
            } else {
                VOWEL_ALL
            }
        }
        2 => {
            let c0 = onset[0].0.to_ascii_lowercase();
            let c1 = onset[1].0.to_ascii_lowercase();
            if c0.is_ascii_lowercase() && c1.is_ascii_lowercase() {
                let idx = ((c0 as usize - 'a' as usize) << 5) | (c1 as usize - 'a' as usize);
                DOUBLE_ONSET_ALLOWED_VOWELS[idx]
            } else {
                VOWEL_ALL
            }
        }
        3 => {
            let c0 = onset[0].0.to_ascii_lowercase();
            let c1 = onset[1].0.to_ascii_lowercase();
            let c2 = onset[2].0.to_ascii_lowercase();
            if c0 == 'n' && c1 == 'g' && c2 == 'h' {
                VOWEL_GH_NGH
            } else {
                VOWEL_ALL
            }
        }
        _ => VOWEL_ALL,
    };

    (allowed_mask & (1u8 << (first_vowel_base as u8))) != 0
}

/// Allowed coda start characters for single 'y' when onset is 'qu' ('t', 'n', 'c'): quýt, quỳnh, quỵch.
const CODA_START_QU_Y_MASK: u32 = (1u32 << (b't' - b'a')) | (1u32 << (b'n' - b'a')) | (1u32 << (b'c' - b'a'));

/// Lookup table of allowed coda start characters for 2-vowel pairs: [u32; 36] (144 bytes).
/// Indexed by (v0.base * 6) + v1.base in 0..35.
/// Defaults to u32::MAX (all codas accepted) for 32 pairs, and restricts only the 4 phonotactically
/// constrained pairs: 'oo' (only 'n'), 'oe' ('t', 'n'), 'uy' ('n', 't', 'p', 'c'), 'uê' ('n', 'c').
const CODA_START_2VOWELS_MASK: [u32; 36] = {
    let mut table = [u32::MAX; 36];
    // 'oo' accepts only 'n' (to allow typing 2-letter coda 'ng' in xoong, boong)
    table[(BaseVowel::O as usize) * 6 + (BaseVowel::O as usize)] = 1u32 << (b'n' - b'a');
    // 'oe' accepts 't' (khoét, loét) or 'n' (hoen)
    table[(BaseVowel::O as usize) * 6 + (BaseVowel::E as usize)] =
        (1u32 << (b't' - b'a')) | (1u32 << (b'n' - b'a'));
    // 'uy' accepts 'n' (huynh), 't' (quýt), 'p' (tuýp), 'c' (quỵch)
    table[(BaseVowel::U as usize) * 6 + (BaseVowel::Y as usize)] =
        (1u32 << (b'n' - b'a')) | (1u32 << (b't' - b'a')) | (1u32 << (b'p' - b'a')) | (1u32 << (b'c' - b'a'));
    // 'uê' accepts 'n' (for nh in huênh) or 'c' (for ch in nghuệch)
    table[(BaseVowel::U as usize) * 6 + (BaseVowel::E as usize)] =
        (1u32 << (b'n' - b'a')) | (1u32 << (b'c' - b'a'));
    table
};

/// Validates whether a specific single coda consonant can legitimately begin a coda following this vowel cluster.
/// Evaluated with O(1) bitmask: eliminates nested vowel-coda branches.
#[inline]
pub fn is_valid_coda_start_for_vowels(
    onset_is_qu: bool,
    vowels: &[VowelLetter],
    coda_ch: char,
) -> bool {
    let c = coda_ch.to_ascii_lowercase();
    match vowels.len() {
        1 => {
            if vowels[0].base == BaseVowel::Y {
                onset_is_qu
                    && c.is_ascii_lowercase()
                    && (CODA_START_QU_Y_MASK & (1u32 << (c as u32 - 'a' as u32))) != 0
            } else {
                true
            }
        }
        2 => {
            let idx = (vowels[0].base as usize) * 6 + (vowels[1].base as usize);
            let mask = CODA_START_2VOWELS_MASK[idx];
            if mask == u32::MAX {
                true
            } else if c.is_ascii_lowercase() {
                (mask & (1u32 << (c as u32 - 'a' as u32))) != 0
            } else {
                false
            }
        }
        _ => true,
    }
}
