/// Vietnamese Tones (Dấu thanh)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    None,
    Acute,     // Sắc  (á, é, ó...)
    Grave,     // Huyền (à, è, ò...)
    HookAbove, // Hỏi  (ả, ẻ, ỏ...)
    Tilde,     // Ngã  (ã, ẽ, õ...)
    DotBelow,  // Nặng (ạ, ẹ, ọ...)
}

/// Vietnamese Diacritics (Dấu phụ / mũ / móc)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Diacritic {
    #[default]
    None,
    Circumflex, // Mũ (â, ê, ô)
    Breve,      // Trăng (ă)
    Horn,       // Móc (ơ, ư)
    DStroke,    // Gạch ngang (đ)
}

/// Base vowel letter without diacritics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseVowel {
    #[default]
    A,
    E,
    I,
    O,
    U,
    Y,
}

/// Bảng tra cứu tĩnh compile-time O(1) cho 128 ký tự ASCII (128 bytes trong .rodata).
/// Triệt tiêu hoàn toàn rẽ nhánh (branching) và chuyển đổi chữ hoa/thường (to_ascii_lowercase),
/// ánh xạ tức thì ký tự ASCII sang BaseVowel chỉ trong đúng 1 chu kỳ CPU.
pub const ASCII_VOWELS: [Option<BaseVowel>; 128] = {
    let mut table = [None; 128];
    table[b'a' as usize] = Some(BaseVowel::A);
    table[b'A' as usize] = Some(BaseVowel::A);
    table[b'e' as usize] = Some(BaseVowel::E);
    table[b'E' as usize] = Some(BaseVowel::E);
    table[b'i' as usize] = Some(BaseVowel::I);
    table[b'I' as usize] = Some(BaseVowel::I);
    table[b'o' as usize] = Some(BaseVowel::O);
    table[b'O' as usize] = Some(BaseVowel::O);
    table[b'u' as usize] = Some(BaseVowel::U);
    table[b'U' as usize] = Some(BaseVowel::U);
    table[b'y' as usize] = Some(BaseVowel::Y);
    table[b'Y' as usize] = Some(BaseVowel::Y);
    table
};

impl TryFrom<char> for BaseVowel {
    type Error = ();

    #[inline(always)]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        let cp = c as u32;
        if cp < 128 {
            ASCII_VOWELS[cp as usize].ok_or(())
        } else {
            decompose_vowel(c).map(|(v, _, _)| v).ok_or(())
        }
    }
}

impl BaseVowel {
    #[inline(always)]
    pub fn from_char(c: char) -> Option<Self> {
        let cp = c as u32;
        if cp < 128 {
            ASCII_VOWELS[cp as usize]
        } else {
            decompose_vowel(c).map(|(v, _, _)| v)
        }
    }
}

/// Builds a precomposed UTF-8 Unicode character from base vowel, diacritic, tone, and uppercase flag.
/// Internal helper computing (lowercase, uppercase) for a given vowel combination
const fn compute_vowel_pair(base: BaseVowel, diacritic: Diacritic, tone: Tone) -> (char, char) {
    match (base, diacritic, tone) {
        // --- A ---
        (BaseVowel::A, Diacritic::None, Tone::None) => ('a', 'A'),
        (BaseVowel::A, Diacritic::None, Tone::Acute) => ('á', 'Á'),
        (BaseVowel::A, Diacritic::None, Tone::Grave) => ('à', 'À'),
        (BaseVowel::A, Diacritic::None, Tone::HookAbove) => ('ả', 'Ả'),
        (BaseVowel::A, Diacritic::None, Tone::Tilde) => ('ã', 'Ã'),
        (BaseVowel::A, Diacritic::None, Tone::DotBelow) => ('ạ', 'Ạ'),

        (BaseVowel::A, Diacritic::Circumflex, Tone::None) => ('â', 'Â'),
        (BaseVowel::A, Diacritic::Circumflex, Tone::Acute) => ('ấ', 'Ấ'),
        (BaseVowel::A, Diacritic::Circumflex, Tone::Grave) => ('ầ', 'Ầ'),
        (BaseVowel::A, Diacritic::Circumflex, Tone::HookAbove) => ('ẩ', 'Ẩ'),
        (BaseVowel::A, Diacritic::Circumflex, Tone::Tilde) => ('ẫ', 'Ẫ'),
        (BaseVowel::A, Diacritic::Circumflex, Tone::DotBelow) => ('ậ', 'Ậ'),

        (BaseVowel::A, Diacritic::Breve, Tone::None) => ('ă', 'Ă'),
        (BaseVowel::A, Diacritic::Breve, Tone::Acute) => ('ắ', 'Ắ'),
        (BaseVowel::A, Diacritic::Breve, Tone::Grave) => ('ằ', 'Ằ'),
        (BaseVowel::A, Diacritic::Breve, Tone::HookAbove) => ('ẳ', 'Ẳ'),
        (BaseVowel::A, Diacritic::Breve, Tone::Tilde) => ('ẵ', 'Ẵ'),
        (BaseVowel::A, Diacritic::Breve, Tone::DotBelow) => ('ặ', 'Ặ'),

        // --- E ---
        (BaseVowel::E, Diacritic::None, Tone::None) => ('e', 'E'),
        (BaseVowel::E, Diacritic::None, Tone::Acute) => ('é', 'É'),
        (BaseVowel::E, Diacritic::None, Tone::Grave) => ('è', 'È'),
        (BaseVowel::E, Diacritic::None, Tone::HookAbove) => ('ẻ', 'Ẻ'),
        (BaseVowel::E, Diacritic::None, Tone::Tilde) => ('ẽ', 'Ẽ'),
        (BaseVowel::E, Diacritic::None, Tone::DotBelow) => ('ẹ', 'Ẹ'),

        (BaseVowel::E, Diacritic::Circumflex, Tone::None) => ('ê', 'Ê'),
        (BaseVowel::E, Diacritic::Circumflex, Tone::Acute) => ('ế', 'Ế'),
        (BaseVowel::E, Diacritic::Circumflex, Tone::Grave) => ('ề', 'Ề'),
        (BaseVowel::E, Diacritic::Circumflex, Tone::HookAbove) => ('ể', 'Ể'),
        (BaseVowel::E, Diacritic::Circumflex, Tone::Tilde) => ('ễ', 'Ễ'),
        (BaseVowel::E, Diacritic::Circumflex, Tone::DotBelow) => ('ệ', 'Ệ'),

        // --- I ---
        (BaseVowel::I, _, Tone::None) => ('i', 'I'),
        (BaseVowel::I, _, Tone::Acute) => ('í', 'Í'),
        (BaseVowel::I, _, Tone::Grave) => ('ì', 'Ì'),
        (BaseVowel::I, _, Tone::HookAbove) => ('ỉ', 'Ỉ'),
        (BaseVowel::I, _, Tone::Tilde) => ('ĩ', 'Ĩ'),
        (BaseVowel::I, _, Tone::DotBelow) => ('ị', 'Ị'),

        // --- O ---
        (BaseVowel::O, Diacritic::None, Tone::None) => ('o', 'O'),
        (BaseVowel::O, Diacritic::None, Tone::Acute) => ('ó', 'Ó'),
        (BaseVowel::O, Diacritic::None, Tone::Grave) => ('ò', 'Ò'),
        (BaseVowel::O, Diacritic::None, Tone::HookAbove) => ('ỏ', 'Ỏ'),
        (BaseVowel::O, Diacritic::None, Tone::Tilde) => ('õ', 'Õ'),
        (BaseVowel::O, Diacritic::None, Tone::DotBelow) => ('ọ', 'Ọ'),

        (BaseVowel::O, Diacritic::Circumflex, Tone::None) => ('ô', 'Ô'),
        (BaseVowel::O, Diacritic::Circumflex, Tone::Acute) => ('ố', 'Ố'),
        (BaseVowel::O, Diacritic::Circumflex, Tone::Grave) => ('ồ', 'Ồ'),
        (BaseVowel::O, Diacritic::Circumflex, Tone::HookAbove) => ('ổ', 'Ổ'),
        (BaseVowel::O, Diacritic::Circumflex, Tone::Tilde) => ('ỗ', 'Ỗ'),
        (BaseVowel::O, Diacritic::Circumflex, Tone::DotBelow) => ('ộ', 'Ộ'),

        (BaseVowel::O, Diacritic::Horn, Tone::None) => ('ơ', 'Ơ'),
        (BaseVowel::O, Diacritic::Horn, Tone::Acute) => ('ớ', 'Ớ'),
        (BaseVowel::O, Diacritic::Horn, Tone::Grave) => ('ờ', 'Ờ'),
        (BaseVowel::O, Diacritic::Horn, Tone::HookAbove) => ('ở', 'Ở'),
        (BaseVowel::O, Diacritic::Horn, Tone::Tilde) => ('ỡ', 'Ỡ'),
        (BaseVowel::O, Diacritic::Horn, Tone::DotBelow) => ('ợ', 'Ợ'),

        // --- U ---
        (BaseVowel::U, Diacritic::None, Tone::None) => ('u', 'U'),
        (BaseVowel::U, Diacritic::None, Tone::Acute) => ('ú', 'Ú'),
        (BaseVowel::U, Diacritic::None, Tone::Grave) => ('ù', 'Ù'),
        (BaseVowel::U, Diacritic::None, Tone::HookAbove) => ('ủ', 'Ủ'),
        (BaseVowel::U, Diacritic::None, Tone::Tilde) => ('ũ', 'Ũ'),
        (BaseVowel::U, Diacritic::None, Tone::DotBelow) => ('ụ', 'Ụ'),

        (BaseVowel::U, Diacritic::Horn, Tone::None) => ('ư', 'Ư'),
        (BaseVowel::U, Diacritic::Horn, Tone::Acute) => ('ứ', 'Ứ'),
        (BaseVowel::U, Diacritic::Horn, Tone::Grave) => ('ừ', 'Ừ'),
        (BaseVowel::U, Diacritic::Horn, Tone::HookAbove) => ('ử', 'Ử'),
        (BaseVowel::U, Diacritic::Horn, Tone::Tilde) => ('ữ', 'Ữ'),
        (BaseVowel::U, Diacritic::Horn, Tone::DotBelow) => ('ự', 'Ự'),

        // --- Y ---
        (BaseVowel::Y, _, Tone::None) => ('y', 'Y'),
        (BaseVowel::Y, _, Tone::Acute) => ('ý', 'Ý'),
        (BaseVowel::Y, _, Tone::Grave) => ('ỳ', 'Ỳ'),
        (BaseVowel::Y, _, Tone::HookAbove) => ('ỷ', 'Ỷ'),
        (BaseVowel::Y, _, Tone::Tilde) => ('ỹ', 'Ỹ'),
        (BaseVowel::Y, _, Tone::DotBelow) => ('ỵ', 'Ỵ'),

        // Fallbacks for invalid combinations (e.g. horn on A)
        _ => match base {
            BaseVowel::A => ('a', 'A'),
            BaseVowel::E => ('e', 'E'),
            BaseVowel::I => ('i', 'I'),
            BaseVowel::O => ('o', 'O'),
            BaseVowel::U => ('u', 'U'),
            BaseVowel::Y => ('y', 'Y'),
        },
    }
}

/// Compile-time lookup table mapping (BaseVowel, Diacritic, Tone, uppercase) -> UTF-16 codepoint.
/// Dimensions: 6 BaseVowels * 5 Diacritics * 6 Tones * 2 Cases = 360 entries (720 bytes).
/// Eliminates the 72-branch match in `compose_vowel` down to a single O(1) array load.
const VOWEL_COMPOSE_LUT: [u16; 360] = {
    let bases = [
        BaseVowel::A,
        BaseVowel::E,
        BaseVowel::I,
        BaseVowel::O,
        BaseVowel::U,
        BaseVowel::Y,
    ];
    let diacritics = [
        Diacritic::None,
        Diacritic::Circumflex,
        Diacritic::Breve,
        Diacritic::Horn,
        Diacritic::DStroke,
    ];
    let tones = [
        Tone::None,
        Tone::Acute,
        Tone::Grave,
        Tone::HookAbove,
        Tone::Tilde,
        Tone::DotBelow,
    ];

    let mut lut = [0u16; 360];
    let mut b = 0;
    while b < 6 {
        let mut d = 0;
        while d < 5 {
            let mut t = 0;
            while t < 6 {
                let (lower, upper) = compute_vowel_pair(bases[b], diacritics[d], tones[t]);
                let base_idx = ((b * 5 + d) * 6 + t) << 1;
                lut[base_idx] = lower as u16;
                lut[base_idx | 1] = upper as u16;
                t += 1;
            }
            d += 1;
        }
        b += 1;
    }
    lut
};

/// Builds a precomposed UTF-8 Unicode character from base vowel, diacritic, tone, and uppercase flag.
/// Evaluated via O(1) compile-time LUT `VOWEL_COMPOSE_LUT` with zero branches.
#[inline(always)]
pub const fn compose_vowel(
    base: BaseVowel,
    diacritic: Diacritic,
    tone: Tone,
    uppercase: bool,
) -> char {
    let idx = (((base as usize * 5 + diacritic as usize) * 6 + tone as usize) << 1)
        | (uppercase as usize);
    unsafe { char::from_u32_unchecked(VOWEL_COMPOSE_LUT[idx] as u32) }
}

/// Compose the letter 'd' / 'đ'
pub fn compose_d(has_stroke: bool, uppercase: bool) -> char {
    match (has_stroke, uppercase) {
        (true, false) => 'đ',
        (true, true) => 'Đ',
        (false, false) => 'd',
        (false, true) => 'D',
    }
}

/// Decomposes a character into (BaseVowel, Diacritic, Tone) if it is a Vietnamese vowel
#[inline]
pub fn decompose_vowel(c: char) -> Option<(BaseVowel, Diacritic, Tone)> {
    let cp = c as u32;
    if cp < 128 {
        return ASCII_VOWELS[cp as usize].map(|v| (v, Diacritic::None, Tone::None));
    }

    match c {
        'á' | 'Á' => Some((BaseVowel::A, Diacritic::None, Tone::Acute)),
        'à' | 'À' => Some((BaseVowel::A, Diacritic::None, Tone::Grave)),
        'ả' | 'Ả' => Some((BaseVowel::A, Diacritic::None, Tone::HookAbove)),
        'ã' | 'Ã' => Some((BaseVowel::A, Diacritic::None, Tone::Tilde)),
        'ạ' | 'Ạ' => Some((BaseVowel::A, Diacritic::None, Tone::DotBelow)),

        'â' | 'Â' => Some((BaseVowel::A, Diacritic::Circumflex, Tone::None)),
        'ấ' | 'Ấ' => Some((BaseVowel::A, Diacritic::Circumflex, Tone::Acute)),
        'ầ' | 'Ầ' => Some((BaseVowel::A, Diacritic::Circumflex, Tone::Grave)),
        'ẩ' | 'Ẩ' => Some((BaseVowel::A, Diacritic::Circumflex, Tone::HookAbove)),
        'ẫ' | 'Ẫ' => Some((BaseVowel::A, Diacritic::Circumflex, Tone::Tilde)),
        'ậ' | 'Ậ' => Some((BaseVowel::A, Diacritic::Circumflex, Tone::DotBelow)),

        'ă' | 'Ă' => Some((BaseVowel::A, Diacritic::Breve, Tone::None)),
        'ắ' | 'Ắ' => Some((BaseVowel::A, Diacritic::Breve, Tone::Acute)),
        'ằ' | 'Ằ' => Some((BaseVowel::A, Diacritic::Breve, Tone::Grave)),
        'ẳ' | 'Ẳ' => Some((BaseVowel::A, Diacritic::Breve, Tone::HookAbove)),
        'ẵ' | 'Ẵ' => Some((BaseVowel::A, Diacritic::Breve, Tone::Tilde)),
        'ặ' | 'Ặ' => Some((BaseVowel::A, Diacritic::Breve, Tone::DotBelow)),

        'é' | 'É' => Some((BaseVowel::E, Diacritic::None, Tone::Acute)),
        'è' | 'È' => Some((BaseVowel::E, Diacritic::None, Tone::Grave)),
        'ẻ' | 'Ẻ' => Some((BaseVowel::E, Diacritic::None, Tone::HookAbove)),
        'ẽ' | 'Ẽ' => Some((BaseVowel::E, Diacritic::None, Tone::Tilde)),
        'ẹ' | 'Ẹ' => Some((BaseVowel::E, Diacritic::None, Tone::DotBelow)),

        'ê' | 'Ê' => Some((BaseVowel::E, Diacritic::Circumflex, Tone::None)),
        'ế' | 'Ế' => Some((BaseVowel::E, Diacritic::Circumflex, Tone::Acute)),
        'ề' | 'Ề' => Some((BaseVowel::E, Diacritic::Circumflex, Tone::Grave)),
        'ể' | 'Ể' => Some((BaseVowel::E, Diacritic::Circumflex, Tone::HookAbove)),
        'ễ' | 'Ễ' => Some((BaseVowel::E, Diacritic::Circumflex, Tone::Tilde)),
        'ệ' | 'Ệ' => Some((BaseVowel::E, Diacritic::Circumflex, Tone::DotBelow)),

        'í' | 'Í' => Some((BaseVowel::I, Diacritic::None, Tone::Acute)),
        'ì' | 'Ì' => Some((BaseVowel::I, Diacritic::None, Tone::Grave)),
        'ỉ' | 'Ỉ' => Some((BaseVowel::I, Diacritic::None, Tone::HookAbove)),
        'ĩ' | 'Ĩ' => Some((BaseVowel::I, Diacritic::None, Tone::Tilde)),
        'ị' | 'Ị' => Some((BaseVowel::I, Diacritic::None, Tone::DotBelow)),

        'ó' | 'Ó' => Some((BaseVowel::O, Diacritic::None, Tone::Acute)),
        'ò' | 'Ò' => Some((BaseVowel::O, Diacritic::None, Tone::Grave)),
        'ỏ' | 'Ỏ' => Some((BaseVowel::O, Diacritic::None, Tone::HookAbove)),
        'õ' | 'Õ' => Some((BaseVowel::O, Diacritic::None, Tone::Tilde)),
        'ọ' | 'Ọ' => Some((BaseVowel::O, Diacritic::None, Tone::DotBelow)),

        'ô' | 'Ô' => Some((BaseVowel::O, Diacritic::Circumflex, Tone::None)),
        'ố' | 'Ố' => Some((BaseVowel::O, Diacritic::Circumflex, Tone::Acute)),
        'ồ' | 'Ồ' => Some((BaseVowel::O, Diacritic::Circumflex, Tone::Grave)),
        'ổ' | 'Ổ' => Some((BaseVowel::O, Diacritic::Circumflex, Tone::HookAbove)),
        'ỗ' | 'Ỗ' => Some((BaseVowel::O, Diacritic::Circumflex, Tone::Tilde)),
        'ộ' | 'Ộ' => Some((BaseVowel::O, Diacritic::Circumflex, Tone::DotBelow)),

        'ơ' | 'Ơ' => Some((BaseVowel::O, Diacritic::Horn, Tone::None)),
        'ớ' | 'Ớ' => Some((BaseVowel::O, Diacritic::Horn, Tone::Acute)),
        'ờ' | 'Ờ' => Some((BaseVowel::O, Diacritic::Horn, Tone::Grave)),
        'ở' | 'Ở' => Some((BaseVowel::O, Diacritic::Horn, Tone::HookAbove)),
        'ỡ' | 'Ỡ' => Some((BaseVowel::O, Diacritic::Horn, Tone::Tilde)),
        'ợ' | 'Ợ' => Some((BaseVowel::O, Diacritic::Horn, Tone::DotBelow)),

        'ú' | 'Ú' => Some((BaseVowel::U, Diacritic::None, Tone::Acute)),
        'ù' | 'Ù' => Some((BaseVowel::U, Diacritic::None, Tone::Grave)),
        'ủ' | 'Ủ' => Some((BaseVowel::U, Diacritic::None, Tone::HookAbove)),
        'ũ' | 'Ũ' => Some((BaseVowel::U, Diacritic::None, Tone::Tilde)),
        'ụ' | 'Ụ' => Some((BaseVowel::U, Diacritic::None, Tone::DotBelow)),

        'ư' | 'Ư' => Some((BaseVowel::U, Diacritic::Horn, Tone::None)),
        'ứ' | 'Ứ' => Some((BaseVowel::U, Diacritic::Horn, Tone::Acute)),
        'ừ' | 'Ừ' => Some((BaseVowel::U, Diacritic::Horn, Tone::Grave)),
        'ử' | 'Ử' => Some((BaseVowel::U, Diacritic::Horn, Tone::HookAbove)),
        'ữ' | 'Ữ' => Some((BaseVowel::U, Diacritic::Horn, Tone::Tilde)),
        'ự' | 'Ự' => Some((BaseVowel::U, Diacritic::Horn, Tone::DotBelow)),

        'ý' | 'Ý' => Some((BaseVowel::Y, Diacritic::None, Tone::Acute)),
        'ỳ' | 'Ỳ' => Some((BaseVowel::Y, Diacritic::None, Tone::Grave)),
        'ỷ' | 'Ỷ' => Some((BaseVowel::Y, Diacritic::None, Tone::HookAbove)),
        'ỹ' | 'Ỹ' => Some((BaseVowel::Y, Diacritic::None, Tone::Tilde)),
        'ỵ' | 'Ỵ' => Some((BaseVowel::Y, Diacritic::None, Tone::DotBelow)),

        _ => None,
    }
}

/// Checks if character is 'đ' or 'Đ'
pub fn is_d_stroke(c: char) -> bool {
    matches!(c, 'đ' | 'Đ')
}
