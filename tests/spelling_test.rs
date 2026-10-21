use kora::vietnamese::{
    VowelLetter,
    charset::{BaseVowel, Diacritic, Tone},
    spelling::{
        is_valid_coda_pair, is_valid_coda_start, is_valid_onset, is_valid_onset_extension,
        is_valid_vietnamese_shorthand, is_valid_vietnamese_syllable,
    },
    syllable::Syllable,
};

#[test]
fn test_onset_extensions() {
    assert!(is_valid_onset_extension(&[('c', false)], 'h'));
    assert!(is_valid_onset_extension(&[('t', false)], 'r'));
    assert!(is_valid_onset_extension(&[('n', false)], 'g'));
    assert!(is_valid_onset_extension(&[('n', false), ('g', false)], 'h'));
    assert!(!is_valid_onset_extension(&[('b', false)], 't'));
    assert!(!is_valid_onset_extension(
        &[('t', false), ('r', false)],
        'h'
    ));
}

#[test]
fn test_coda_checks() {
    assert!(is_valid_coda_start('n'));
    assert!(is_valid_coda_start('c'));
    assert!(!is_valid_coda_start('g'));
    assert!(!is_valid_coda_start('x'));

    assert!(is_valid_coda_pair('n', 'g'));
    assert!(is_valid_coda_pair('n', 'h'));
    assert!(is_valid_coda_pair('c', 'h'));
    assert!(!is_valid_coda_pair('n', 't'));
    assert!(!is_valid_coda_pair('p', 't'));
}

#[test]
fn test_syllable_with_glides() {
    // "giờ" -> onset: 'g', vowels: ['i', 'ơ']
    let gio = Syllable {
        onset: vec![('g', false)],
        d_stroke: false,
        vowels: vec![
            VowelLetter {
                base: BaseVowel::I,
                diacritic: Diacritic::None,
                is_upper: false,
            },
            VowelLetter {
                base: BaseVowel::O,
                diacritic: Diacritic::Horn,
                is_upper: false,
            },
        ],
        tone: Tone::Grave,
        coda: Vec::new(),
    };
    assert!(is_valid_vietnamese_syllable(&gio));

    // "bạc" with Grave tone is invalid (coda 'c' only accepts Acute or DotBelow)
    let bac_grave = Syllable {
        onset: vec![('b', false)],
        d_stroke: false,
        vowels: vec![VowelLetter {
            base: BaseVowel::A,
            diacritic: Diacritic::None,
            is_upper: false,
        }],
        tone: Tone::Grave,
        coda: vec![('c', false)],
    };
    assert!(!is_valid_vietnamese_syllable(&bac_grave));

    let bac_dot = Syllable {
        onset: vec![('b', false)],
        d_stroke: false,
        vowels: vec![VowelLetter {
            base: BaseVowel::A,
            diacritic: Diacritic::None,
            is_upper: false,
        }],
        tone: Tone::DotBelow,
        coda: vec![('c', false)],
    };
    assert!(is_valid_vietnamese_syllable(&bac_dot));
}

#[test]
fn test_onset_validity() {
    // Valid onsets
    assert!(is_valid_onset(&[], false)); // empty onset is valid
    assert!(is_valid_onset(&[('d', false)], true)); // 'đ'
    assert!(is_valid_onset(&[('b', false)], false));
    assert!(is_valid_onset(&[('t', false), ('r', false)], false));
    assert!(is_valid_onset(
        &[('n', false), ('g', false), ('h', false)],
        false
    ));

    // Invalid non-native consonants in Vietnamese
    assert!(!is_valid_onset(&[('f', false)], false));
    assert!(!is_valid_onset(&[('j', false)], false));
    assert!(!is_valid_onset(&[('w', false)], false));
    assert!(!is_valid_onset(&[('z', false)], false));

    // Invalid onset clusters
    assert!(!is_valid_onset(&[('f', false), ('l', false)], false));
    assert!(!is_valid_onset(&[('s', false), ('t', false)], false));
}

#[test]
fn test_invalid_syllables() {
    // "fĩ" -> onset: 'f', vowel: 'i' with Tilde -> INVALID!
    let fi_tilde = Syllable {
        onset: vec![('f', false)],
        d_stroke: false,
        vowels: vec![VowelLetter {
            base: BaseVowel::I,
            diacritic: Diacritic::None,
            is_upper: false,
        }],
        tone: Tone::Tilde,
        coda: Vec::new(),
    };
    assert!(!is_valid_vietnamese_syllable(&fi_tilde));

    // "q" without "u" -> INVALID!
    let qa = Syllable {
        onset: vec![('q', false)],
        d_stroke: false,
        vowels: vec![VowelLetter {
            base: BaseVowel::A,
            diacritic: Diacritic::None,
            is_upper: false,
        }],
        tone: Tone::None,
        coda: Vec::new(),
    };
    assert!(!is_valid_vietnamese_syllable(&qa));
}

#[test]
fn test_onset_vowel_orthography() {
    use kora::vietnamese::spelling::{
        is_valid_coda_start_for_vowels, is_valid_onset_vowel_orthography,
    };

    // 'c' only allows a, o, u; rejects e, i, y
    assert!(is_valid_onset_vowel_orthography(
        &[('c', false)],
        false,
        BaseVowel::A
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('c', false)],
        false,
        BaseVowel::O
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('c', false)],
        false,
        BaseVowel::U
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('c', false)],
        false,
        BaseVowel::E
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('c', false)],
        false,
        BaseVowel::I
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('c', false)],
        false,
        BaseVowel::Y
    ));

    // 'k' only allows e, i, y; rejects a, o, u
    assert!(is_valid_onset_vowel_orthography(
        &[('k', false)],
        false,
        BaseVowel::E
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('k', false)],
        false,
        BaseVowel::I
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('k', false)],
        false,
        BaseVowel::Y
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('k', false)],
        false,
        BaseVowel::A
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('k', false)],
        false,
        BaseVowel::O
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('k', false)],
        false,
        BaseVowel::U
    ));

    // 'q' must precede 'u'
    assert!(is_valid_onset_vowel_orthography(
        &[('q', false)],
        false,
        BaseVowel::U
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('q', false)],
        false,
        BaseVowel::A
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('q', false)],
        false,
        BaseVowel::E
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('q', false)],
        false,
        BaseVowel::I
    ));

    // 'g' rejects 'e' (must use 'gh')
    assert!(!is_valid_onset_vowel_orthography(
        &[('g', false)],
        false,
        BaseVowel::E
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('g', false)],
        false,
        BaseVowel::A
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('g', false)],
        false,
        BaseVowel::I
    ));

    // 'gh' only accepts 'e', 'i'
    assert!(is_valid_onset_vowel_orthography(
        &[('g', false), ('h', false)],
        false,
        BaseVowel::E
    ));
    assert!(is_valid_onset_vowel_orthography(
        &[('g', false), ('h', false)],
        false,
        BaseVowel::I
    ));
    assert!(!is_valid_onset_vowel_orthography(
        &[('g', false), ('h', false)],
        false,
        BaseVowel::A
    ));

    // Coda constraints for special vowels:
    let oo_vowels = [
        VowelLetter {
            base: BaseVowel::O,
            diacritic: Diacritic::None,
            is_upper: false,
        },
        VowelLetter {
            base: BaseVowel::O,
            diacritic: Diacritic::None,
            is_upper: false,
        },
    ];
    // 'oo' only allows 'n' (to form 'ng' in xoong, boong), rejects 'm', 't', 'p', 'c'
    assert!(is_valid_coda_start_for_vowels(false, &oo_vowels, 'n'));
    assert!(!is_valid_coda_start_for_vowels(false, &oo_vowels, 'm'));
    assert!(!is_valid_coda_start_for_vowels(false, &oo_vowels, 't'));
    assert!(!is_valid_coda_start_for_vowels(false, &oo_vowels, 'p'));

    let oe_vowels = [
        VowelLetter {
            base: BaseVowel::O,
            diacritic: Diacritic::None,
            is_upper: false,
        },
        VowelLetter {
            base: BaseVowel::E,
            diacritic: Diacritic::None,
            is_upper: false,
        },
    ];
    // 'oe' allows 't' (khoét) and 'n' (hoen), rejects 'm' (poem)
    assert!(is_valid_coda_start_for_vowels(false, &oe_vowels, 't'));
    assert!(is_valid_coda_start_for_vowels(false, &oe_vowels, 'n'));
    assert!(!is_valid_coda_start_for_vowels(false, &oe_vowels, 'm'));

    let y_vowels = [VowelLetter {
        base: BaseVowel::Y,
        diacritic: Diacritic::None,
        is_upper: false,
    }];
    // 'y' with onset 'qu' allows 't' (quýt), 'n' (quỳnh), 'c' (quỵch)
    assert!(is_valid_coda_start_for_vowels(true, &y_vowels, 't'));
    assert!(is_valid_coda_start_for_vowels(true, &y_vowels, 'n'));
    assert!(is_valid_coda_start_for_vowels(true, &y_vowels, 'c'));
    // 'y' without 'qu' (mỹ, lý, tỷ) rejects all codas
    assert!(!is_valid_coda_start_for_vowels(false, &y_vowels, 't'));
    assert!(!is_valid_coda_start_for_vowels(false, &y_vowels, 'n'));
}

#[test]
fn test_vietnamese_shorthand() {
    assert!(is_valid_vietnamese_shorthand(&['đ', 'c']));
    assert!(is_valid_vietnamese_shorthand(&['Đ', 'c']));
    assert!(is_valid_vietnamese_shorthand(&['Đ', 'C']));
    assert!(is_valid_vietnamese_shorthand(&['đ', 'C']));

    assert!(!is_valid_vietnamese_shorthand(&['đ']));
    assert!(!is_valid_vietnamese_shorthand(&['c']));
    assert!(!is_valid_vietnamese_shorthand(&['d', 'c']));
    assert!(!is_valid_vietnamese_shorthand(&['đ', 'c', 's']));
    assert!(!is_valid_vietnamese_shorthand(&['đ', 'k']));
}
