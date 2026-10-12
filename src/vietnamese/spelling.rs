use super::{
    VowelLetter,
    charset::{BaseVowel, Diacritic, Tone},
    syllable::Syllable,
};

/// Validates whether a sequence of vowels forms a legitimate Vietnamese vowel cluster
#[inline]
pub fn is_valid_vowel_combination(vowels: &[VowelLetter]) -> bool {
    match vowels.len() {
        0 | 1 => true,
        2 => {
            let v0 = (vowels[0].base, vowels[0].diacritic);
            let v1 = (vowels[1].base, vowels[1].diacritic);
            matches!(
                (v0, v1),
                // ai, ao, au, ay, ây, âu
                ((BaseVowel::A, Diacritic::None), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::A, Diacritic::None), (BaseVowel::O, Diacritic::None))
                | ((BaseVowel::A, Diacritic::None), (BaseVowel::U, Diacritic::None))
                | ((BaseVowel::A, Diacritic::None), (BaseVowel::Y, Diacritic::None))
                | ((BaseVowel::A, Diacritic::Circumflex), (BaseVowel::Y, Diacritic::None))
                | ((BaseVowel::A, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None))
                // eo, êu
                | ((BaseVowel::E, Diacritic::None), (BaseVowel::O, Diacritic::None))
                | ((BaseVowel::E, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None))
                // ia, iê, iu
                | ((BaseVowel::I, Diacritic::None), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
                | ((BaseVowel::I, Diacritic::None), (BaseVowel::U, Diacritic::None))
                // oa, oă, oe, oi, ôi, ơi
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::Breve))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::E, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::O, Diacritic::Circumflex), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::O, Diacritic::Horn), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::O, Diacritic::None)) // oo (xoong, boong)
                // ua, uâ, uô, uơ, uê, ui, uy
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Horn))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None))
                // ưa, ươ, ưu, ưi
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn))
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::U, Diacritic::None))
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::I, Diacritic::None))
                // ya, yê
                | ((BaseVowel::Y, Diacritic::None), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
            )
        }
        3 => {
            let v0 = (vowels[0].base, vowels[0].diacritic);
            let v1 = (vowels[1].base, vowels[1].diacritic);
            let v2 = (vowels[2].base, vowels[2].diacritic);
            matches!(
                (v0, v1, v2),
                // iêu, yêu
                ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None))
                | ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex), (BaseVowel::U, Diacritic::None))
                // oai, oay, oao, oeo
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::Y, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::O, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::E, Diacritic::None), (BaseVowel::O, Diacritic::None))
                // uai, uay, uôi, uơi, ươi, ươu, uya, uyê, uyu
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None), (BaseVowel::Y, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Circumflex), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Horn), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn), (BaseVowel::I, Diacritic::None))
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn), (BaseVowel::U, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None), (BaseVowel::U, Diacritic::None))
            )
        }
        _ => false,
    }
}

/// Validates whether a sequence of vowels forms a legitimate intermediate or completed Vietnamese vowel cluster during typing.
#[inline]
pub fn is_valid_intermediate_vowel_combination(vowels: &[VowelLetter]) -> bool {
    match vowels.len() {
        0 | 1 => true,
        2 => {
            if is_valid_vowel_combination(vowels) {
                return true;
            }
            let v0 = (vowels[0].base, vowels[0].diacritic);
            let v1 = (vowels[1].base, vowels[1].diacritic);
            matches!(
                (v0, v1),
                // Intermediate typing bases for iê, yê, uô, ươ, uê, ê, êu, ưu
                (
                    (BaseVowel::I, Diacritic::None),
                    (BaseVowel::E, Diacritic::None)
                ) | (
                    (BaseVowel::Y, Diacritic::None),
                    (BaseVowel::E, Diacritic::None)
                ) | (
                    (BaseVowel::U, Diacritic::None),
                    (BaseVowel::O, Diacritic::None)
                ) | (
                    (BaseVowel::U, Diacritic::None),
                    (BaseVowel::E, Diacritic::None)
                ) | (
                    (BaseVowel::U, Diacritic::None),
                    (BaseVowel::U, Diacritic::None)
                ) | (
                    (BaseVowel::E, Diacritic::None),
                    (BaseVowel::E, Diacritic::None)
                ) | (
                    (BaseVowel::E, Diacritic::None),
                    (BaseVowel::U, Diacritic::None)
                )
            )
        }
        3 => {
            if is_valid_vowel_combination(vowels) {
                return true;
            }
            let v0 = vowels[0].base;
            let v1 = vowels[1].base;
            let v2 = vowels[2].base;
            matches!(
                (v0, v1, v2),
                (BaseVowel::U, BaseVowel::Y, BaseVowel::E)
                    | (BaseVowel::I, BaseVowel::E, BaseVowel::U)
                    | (BaseVowel::Y, BaseVowel::E, BaseVowel::U)
                    | (BaseVowel::U, BaseVowel::O, BaseVowel::I)
                    | (BaseVowel::U, BaseVowel::O, BaseVowel::U)
            )
        }
        _ => false,
    }
}

/// Validates whether an onset consonant sequence is a legitimate Vietnamese onset.
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
            matches!(
                c,
                'b' | 'c'
                    | 'd'
                    | 'g'
                    | 'h'
                    | 'k'
                    | 'l'
                    | 'm'
                    | 'n'
                    | 'p'
                    | 'q'
                    | 'r'
                    | 's'
                    | 't'
                    | 'v'
                    | 'x'
            )
        }
        2 => {
            let c0 = onset[0].0.to_ascii_lowercase();
            let c1 = onset[1].0.to_ascii_lowercase();
            matches!(
                (c0, c1),
                ('c', 'h')
                    | ('g', 'h')
                    | ('g', 'i')
                    | ('k', 'h')
                    | ('n', 'h')
                    | ('n', 'g')
                    | ('p', 'h')
                    | ('q', 'u')
                    | ('t', 'h')
                    | ('t', 'r')
            )
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

/// Returns true if the coda is a stop coda ('c', 'ch', 'p', 't', 'k').
/// Stop codas in Vietnamese phonotactics only accept Acute (Sắc) or DotBelow (Nặng) tones,
/// and do not accept additional vowels when followed by tone.
#[inline]
pub fn is_stop_coda(coda: &[(char, bool)]) -> bool {
    match coda {
        [(c, _)] => matches!(c.to_ascii_lowercase(), 'c' | 'p' | 't' | 'k'),
        [(c1, _), (c2, _)] => c1.eq_ignore_ascii_case(&'c') && c2.eq_ignore_ascii_case(&'h'),
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
    match current_onset.len() {
        1 => {
            let first = current_onset[0].0.to_ascii_lowercase();
            matches!(
                (first, next_lower),
                ('c', 'h')
                    | ('g', 'h')
                    | ('g', 'i')
                    | ('k', 'h')
                    | ('n', 'h')
                    | ('n', 'g')
                    | ('p', 'h')
                    | ('q', 'u')
                    | ('t', 'h')
                    | ('t', 'r')
            )
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

/// Validates whether a consonant can legitimately begin a Vietnamese coda.
/// Standard coda start letters: c, m, n, p, t
#[inline]
pub fn is_valid_coda_start(ch: char) -> bool {
    let ch_lower = ch.to_ascii_lowercase();
    matches!(ch_lower, 'c' | 'm' | 'n' | 'p' | 't')
}

/// Validates whether two consonants form a legitimate 2-letter Vietnamese coda.
/// In Vietnamese, only 'ng', 'nh', and 'ch' are valid 2-letter codas.
#[inline]
pub fn is_valid_coda_pair(first: char, second: char) -> bool {
    let c0 = first.to_ascii_lowercase();
    let c1 = second.to_ascii_lowercase();
    (c0 == 'n' && (c1 == 'g' || c1 == 'h')) || (c0 == 'c' && c1 == 'h')
}

/// Validates whether a nucleus vowel combination can legally accept a coda consonant in Vietnamese.
/// In Vietnamese phonotactics:
/// - Single vowels can accept a coda: an, em, in, on, un, etc.
/// - Centering diphthongs can accept a coda: iê/yê, uô, ươ, uâ, oă, oa, oe, uê, uy.
/// - Off-glide diphthongs CANNOT accept a coda: ai, oi, ôi, ơi, ui, ưi, ay, ây, ao, eo, au, âu, iu, ưu, ia, ya, ua, ưa.
/// - Triphthongs ending in semivowels CANNOT accept a coda: iêu, yêu, oai, oay, oao, oeo, uai, uay, uôi, uơi, ươi, ươu, uya, uyu.
/// - Only triphthong 'uyê' can accept a coda: uyên, uyết.
#[inline]
pub fn can_vowels_accept_coda(vowels: &[VowelLetter]) -> bool {
    match vowels.len() {
        0 => false,
        1 => true,
        2 => {
            let v0 = (vowels[0].base, vowels[0].diacritic);
            let v1 = (vowels[1].base, vowels[1].diacritic);
            matches!(
                (v0, v1),
                // iê, yê, and intermediate typing bases (ie, ye for liene -> liên, yens -> yến)
                ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
                | ((BaseVowel::I, Diacritic::None), (BaseVowel::E, Diacritic::None))
                | ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
                | ((BaseVowel::Y, Diacritic::None), (BaseVowel::E, Diacritic::None))
                // uô, ươ, uo (uo is the typing base for uô/ươ when free_mark is used)
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::Horn), (BaseVowel::O, Diacritic::Horn))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::O, Diacritic::None))
                // uâ, ua (ua is the typing base for uâ in xuana -> xuân)
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::A, Diacritic::None))
                // oă
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::Breve))
                // oa, oe, uê, uy, and ue (typing base for quene -> quên)
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::A, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::E, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::Circumflex))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::E, Diacritic::None))
                | ((BaseVowel::U, Diacritic::None), (BaseVowel::Y, Diacritic::None))
                | ((BaseVowel::O, Diacritic::None), (BaseVowel::O, Diacritic::None)) // oo (xoong, boong)
            )
        }
        3 => {
            let v0 = (vowels[0].base, vowels[0].diacritic);
            let v1 = (vowels[1].base, vowels[1].diacritic);
            let v2 = (vowels[2].base, vowels[2].diacritic);
            matches!(
                (v0, v1, v2),
                // uyê (uyên, uyết) and intermediate typing base uye (uyene -> uyên, chuyens -> chuyến)
                (
                    (BaseVowel::U, Diacritic::None),
                    (BaseVowel::Y, Diacritic::None),
                    (BaseVowel::E, Diacritic::Circumflex)
                ) | (
                    (BaseVowel::U, Diacritic::None),
                    (BaseVowel::Y, Diacritic::None),
                    (BaseVowel::E, Diacritic::None)
                )
            )
        }
        _ => false,
    }
}

/// Validates whether an onset consonant and the first vowel base satisfy Vietnamese orthography rules.
#[inline]
pub fn is_valid_onset_vowel_orthography(
    onset: &[(char, bool)],
    d_stroke: bool,
    first_vowel_base: BaseVowel,
) -> bool {
    if onset.is_empty() || d_stroke {
        return true;
    }

    match onset {
        [(c, _)] if c.eq_ignore_ascii_case(&'q') => first_vowel_base == BaseVowel::U,
        [(c, _)] if c.eq_ignore_ascii_case(&'k') => {
            matches!(first_vowel_base, BaseVowel::I | BaseVowel::Y | BaseVowel::E)
        }
        [(c, _)] if c.eq_ignore_ascii_case(&'c') => {
            !matches!(first_vowel_base, BaseVowel::I | BaseVowel::Y | BaseVowel::E)
        }
        [(c0, _), (c1, _)] if c0.eq_ignore_ascii_case(&'g') && c1.eq_ignore_ascii_case(&'h') => {
            matches!(first_vowel_base, BaseVowel::I | BaseVowel::E)
        }
        [(c0, _), (c1, _), (c2, _)]
            if c0.eq_ignore_ascii_case(&'n')
                && c1.eq_ignore_ascii_case(&'g')
                && c2.eq_ignore_ascii_case(&'h') =>
        {
            matches!(first_vowel_base, BaseVowel::I | BaseVowel::E)
        }
        [(c0, _), (c1, _)] if c0.eq_ignore_ascii_case(&'n') && c1.eq_ignore_ascii_case(&'g') => {
            !matches!(first_vowel_base, BaseVowel::I | BaseVowel::E | BaseVowel::Y)
        }
        [(c, _)]
            if c.eq_ignore_ascii_case(&'g')
                // 'g' + 'i' is the valid 'gi' glide (gió, giờ, giúp)
                // but 'g' + 'e' / 'ê' / 'y' is invalid (must use 'gh' for e/ê; gy doesn't exist)
                && matches!(first_vowel_base, BaseVowel::E | BaseVowel::Y) =>
        {
            false
        }
        // Non-Vietnamese onsets with 'y': by, dy, ry, phy, khy do not exist in Vietnamese
        // (Vietnamese uses bi, di, ri, phi, khi).
        [(c, _)]
            if matches!(c.to_ascii_lowercase(), 'b' | 'd' | 'r')
                && first_vowel_base == BaseVowel::Y =>
        {
            false
        }
        [(c0, _), (c1, _)]
            if (c0.eq_ignore_ascii_case(&'p') && c1.eq_ignore_ascii_case(&'h')
                || c0.eq_ignore_ascii_case(&'k') && c1.eq_ignore_ascii_case(&'h'))
                && first_vowel_base == BaseVowel::Y =>
        {
            false
        }
        _ => true,
    }
}

/// Validates whether a specific single coda consonant can legitimately begin a coda following this vowel cluster.
#[inline]
pub fn is_valid_coda_start_for_vowels(
    onset_is_qu: bool,
    vowels: &[VowelLetter],
    coda_ch: char,
) -> bool {
    let c = coda_ch.to_ascii_lowercase();
    match vowels.len() {
        1 => {
            let v = vowels[0].base;
            // Single 'y' only accepts codas if onset is 'qu' (quýt, quỳnh, quỵch).
            // Without 'qu', single 'y' (lý, mỹ, tỷ, kỳ, hy, sỹ) never takes codas in Vietnamese.
            if v == BaseVowel::Y {
                return onset_is_qu && matches!(c, 't' | 'n' | 'c');
            }
        }
        2 => {
            let v0 = vowels[0].base;
            let v1 = vowels[1].base;
            // 'oo' in Vietnamese only accepts 2-letter coda 'ng' (xoong, boong), never single 'c', 'm', 'p', 't'
            if v0 == BaseVowel::O && v1 == BaseVowel::O {
                return c == 'n'; // to allow typing 'ng' in xoong, boong
            }
            // 'oe' only accepts 't' (khoét, loét) or 'n' (hoen)
            if v0 == BaseVowel::O && v1 == BaseVowel::E {
                return c == 't' || c == 'n';
            }
            // 'uy' accepts 'n' (huynh), 't' (quýt), 'p' (tuýp), 'c' (quỵch), never 'm'
            if v0 == BaseVowel::U && v1 == BaseVowel::Y {
                return matches!(c, 'n' | 't' | 'p' | 'c');
            }
            // 'uê' only accepts 'n' (for nh in huênh) or 'c' (for ch in nghuệch)
            if v0 == BaseVowel::U && v1 == BaseVowel::E {
                return c == 'n' || c == 'c';
            }
        }
        _ => {}
    }
    true
}
