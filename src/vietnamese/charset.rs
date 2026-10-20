use crate::engine::action::CompactString;

/// Bảng mã tiếng Việt (Character Encoding)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Charset {
    #[default]
    Unicode,
    Tcvn3,
}

impl Charset {
    pub const ALL: &'static [Charset] = &[Charset::Unicode, Charset::Tcvn3];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unicode => "unicode",
            Self::Tcvn3 => "tcvn3",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Unicode => "Unicode Dựng sẵn",
            Self::Tcvn3 => "TCVN3 (ABC)",
        }
    }
}

impl std::str::FromStr for Charset {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "unicode" | "utf-8" | "utf8" | "utf-16" | "utf16" => Ok(Self::Unicode),
            "tcvn3" | "tcvn" | "abc" | "vn" => Ok(Self::Tcvn3),
            _ => Ok(Self::Unicode),
        }
    }
}

/// Trait Strategy cho các bảng mã tiếng Việt
pub trait CharsetStrategy: Send + Sync {
    /// Tên định danh của bảng mã
    fn name(&self) -> &'static str;

    /// Ghép nguyên âm từ (BaseVowel, Diacritic, Tone, uppercase)
    fn compose_vowel(
        &self,
        base: BaseVowel,
        diacritic: Diacritic,
        tone: Tone,
        uppercase: bool,
    ) -> char;

    /// Ghép ký tự 'd' hoặc 'đ'
    fn compose_d(&self, has_stroke: bool, uppercase: bool) -> char;

    /// Chuyển đổi 1 ký tự Unicode sang ký tự của bảng mã đích
    fn encode_char(&self, c: char) -> char;

    /// Chuyển đổi toàn bộ chuỗi văn bản Unicode sang bảng mã đích
    fn encode_text(&self, text: &str) -> CompactString;
}

/// Strategy cho bảng mã Unicode Dựng Sẵn (Precomposed UTF-8 / UTF-16)
#[derive(Debug, Clone, Copy, Default)]
pub struct UnicodeCharset;

impl CharsetStrategy for UnicodeCharset {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "Unicode"
    }

    #[inline(always)]
    fn compose_vowel(
        &self,
        base: BaseVowel,
        diacritic: Diacritic,
        tone: Tone,
        uppercase: bool,
    ) -> char {
        compose_vowel(base, diacritic, tone, uppercase)
    }

    #[inline(always)]
    fn compose_d(&self, has_stroke: bool, uppercase: bool) -> char {
        compose_d(has_stroke, uppercase)
    }

    #[inline(always)]
    fn encode_char(&self, c: char) -> char {
        c
    }

    #[inline(always)]
    fn encode_text(&self, text: &str) -> CompactString {
        CompactString::from(text)
    }
}

/// Strategy cho bảng mã TCVN3 (ABC - tiêu chuẩn TCVN 5712:1993, font .VnTime, .VnArial)
#[derive(Debug, Clone, Copy, Default)]
pub struct Tcvn3Charset;

impl CharsetStrategy for Tcvn3Charset {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "TCVN3"
    }

    #[inline(always)]
    fn compose_vowel(
        &self,
        base: BaseVowel,
        diacritic: Diacritic,
        tone: Tone,
        uppercase: bool,
    ) -> char {
        compose_tcvn3_vowel(base, diacritic, tone, uppercase)
    }

    #[inline(always)]
    fn compose_d(&self, has_stroke: bool, uppercase: bool) -> char {
        compose_tcvn3_d(has_stroke, uppercase)
    }

    #[inline(always)]
    fn encode_char(&self, c: char) -> char {
        match c {
            'đ' => '\u{00AE}',
            'Đ' => '\u{00A7}',
            other => {
                if let Some((vowel, diacritic, tone)) = decompose_vowel(other) {
                    let uppercase = other.is_uppercase();
                    compose_tcvn3_vowel(vowel, diacritic, tone, uppercase)
                } else {
                    other
                }
            }
        }
    }

    #[inline]
    fn encode_text(&self, text: &str) -> CompactString {
        let mut out = CompactString::new();
        for ch in text.chars() {
            out.push(self.encode_char(ch));
        }
        out
    }
}

impl CharsetStrategy for Charset {
    #[inline(always)]
    fn name(&self) -> &'static str {
        match self {
            Self::Unicode => UnicodeCharset.name(),
            Self::Tcvn3 => Tcvn3Charset.name(),
        }
    }

    #[inline(always)]
    fn compose_vowel(
        &self,
        base: BaseVowel,
        diacritic: Diacritic,
        tone: Tone,
        uppercase: bool,
    ) -> char {
        match self {
            Self::Unicode => UnicodeCharset.compose_vowel(base, diacritic, tone, uppercase),
            Self::Tcvn3 => Tcvn3Charset.compose_vowel(base, diacritic, tone, uppercase),
        }
    }

    #[inline(always)]
    fn compose_d(&self, has_stroke: bool, uppercase: bool) -> char {
        match self {
            Self::Unicode => UnicodeCharset.compose_d(has_stroke, uppercase),
            Self::Tcvn3 => Tcvn3Charset.compose_d(has_stroke, uppercase),
        }
    }

    #[inline(always)]
    fn encode_char(&self, c: char) -> char {
        match self {
            Self::Unicode => UnicodeCharset.encode_char(c),
            Self::Tcvn3 => Tcvn3Charset.encode_char(c),
        }
    }

    #[inline(always)]
    fn encode_text(&self, text: &str) -> CompactString {
        match self {
            Self::Unicode => UnicodeCharset.encode_text(text),
            Self::Tcvn3 => Tcvn3Charset.encode_text(text),
        }
    }
}

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
/// Compose the letter 'd' / 'đ' in TCVN3 (ABC) encoding
pub fn compose_tcvn3_d(has_stroke: bool, uppercase: bool) -> char {
    match (has_stroke, uppercase) {
        (true, false) => '\u{00AE}', // đ
        (true, true) => '\u{00A7}',  // Đ
        (false, false) => 'd',
        (false, true) => 'D',
    }
}

/// Helper computing (lowercase, uppercase) for a given vowel combination in TCVN3
const fn compute_tcvn3_vowel_pair(
    base: BaseVowel,
    diacritic: Diacritic,
    tone: Tone,
) -> (char, char) {
    match (base, diacritic, tone) {
        // --- A ---
        (BaseVowel::A, Diacritic::None, Tone::None) => ('a', 'A'),
        (BaseVowel::A, Diacritic::None, Tone::Acute) => ('\u{00B8}', '\u{00B8}'), // á, Á
        (BaseVowel::A, Diacritic::None, Tone::Grave) => ('\u{00B5}', '\u{00B5}'), // à, À
        (BaseVowel::A, Diacritic::None, Tone::HookAbove) => ('\u{00B6}', '\u{00B6}'), // ả, Ả
        (BaseVowel::A, Diacritic::None, Tone::Tilde) => ('\u{00B7}', '\u{00B7}'), // ã, Ã
        (BaseVowel::A, Diacritic::None, Tone::DotBelow) => ('\u{00B9}', '\u{00B9}'), // ạ, Ạ

        (BaseVowel::A, Diacritic::Circumflex, Tone::None) => ('\u{00A9}', '\u{00A2}'), // â, Â
        (BaseVowel::A, Diacritic::Circumflex, Tone::Acute) => ('\u{00CA}', '\u{00CA}'), // ấ, Ấ
        (BaseVowel::A, Diacritic::Circumflex, Tone::Grave) => ('\u{00C7}', '\u{00C7}'), // ầ, Ầ
        (BaseVowel::A, Diacritic::Circumflex, Tone::HookAbove) => ('\u{00C8}', '\u{00C8}'), // ẩ, Ẩ
        (BaseVowel::A, Diacritic::Circumflex, Tone::Tilde) => ('\u{00C9}', '\u{00C9}'), // ẫ, Ẫ
        (BaseVowel::A, Diacritic::Circumflex, Tone::DotBelow) => ('\u{00CB}', '\u{00CB}'), // ậ, Ậ

        (BaseVowel::A, Diacritic::Breve, Tone::None) => ('\u{00A8}', '\u{00A1}'), // ă, Ă
        (BaseVowel::A, Diacritic::Breve, Tone::Acute) => ('\u{00BE}', '\u{00BE}'), // ắ, Ắ
        (BaseVowel::A, Diacritic::Breve, Tone::Grave) => ('\u{00BB}', '\u{00BB}'), // ằ, Ằ
        (BaseVowel::A, Diacritic::Breve, Tone::HookAbove) => ('\u{00BC}', '\u{00BC}'), // ẳ, Ẳ
        (BaseVowel::A, Diacritic::Breve, Tone::Tilde) => ('\u{00BD}', '\u{00BD}'), // ẵ, Ẵ
        (BaseVowel::A, Diacritic::Breve, Tone::DotBelow) => ('\u{00C6}', '\u{00C6}'), // ặ, Ặ

        // --- E ---
        (BaseVowel::E, Diacritic::None, Tone::None) => ('e', 'E'),
        (BaseVowel::E, Diacritic::None, Tone::Acute) => ('\u{00D0}', '\u{00D0}'), // é, É
        (BaseVowel::E, Diacritic::None, Tone::Grave) => ('\u{00CC}', '\u{00CC}'), // è, È
        (BaseVowel::E, Diacritic::None, Tone::HookAbove) => ('\u{00CE}', '\u{00CE}'), // ẻ, Ẻ
        (BaseVowel::E, Diacritic::None, Tone::Tilde) => ('\u{00CF}', '\u{00CF}'), // ẽ, Ẽ
        (BaseVowel::E, Diacritic::None, Tone::DotBelow) => ('\u{00D1}', '\u{00D1}'), // ẹ, Ẹ

        (BaseVowel::E, Diacritic::Circumflex, Tone::None) => ('\u{00AA}', '\u{00A3}'), // ê, Ê
        (BaseVowel::E, Diacritic::Circumflex, Tone::Acute) => ('\u{00D5}', '\u{00D5}'), // ế, Ế
        (BaseVowel::E, Diacritic::Circumflex, Tone::Grave) => ('\u{00D2}', '\u{00D2}'), // ề, Ề
        (BaseVowel::E, Diacritic::Circumflex, Tone::HookAbove) => ('\u{00D3}', '\u{00D3}'), // ể, Ể
        (BaseVowel::E, Diacritic::Circumflex, Tone::Tilde) => ('\u{00D4}', '\u{00D4}'), // ễ, Ễ
        (BaseVowel::E, Diacritic::Circumflex, Tone::DotBelow) => ('\u{00D6}', '\u{00D6}'), // ệ, Ệ

        // --- I ---
        (BaseVowel::I, _, Tone::None) => ('i', 'I'),
        (BaseVowel::I, _, Tone::Acute) => ('\u{00DD}', '\u{00DD}'), // í, Í
        (BaseVowel::I, _, Tone::Grave) => ('\u{00D7}', '\u{00D7}'), // ì, Ì
        (BaseVowel::I, _, Tone::HookAbove) => ('\u{00D8}', '\u{00D8}'), // ỉ, Ỉ
        (BaseVowel::I, _, Tone::Tilde) => ('\u{00DC}', '\u{00DC}'), // ĩ, Ĩ
        (BaseVowel::I, _, Tone::DotBelow) => ('\u{00DE}', '\u{00DE}'), // ị, Ị

        // --- O ---
        (BaseVowel::O, Diacritic::None, Tone::None) => ('o', 'O'),
        (BaseVowel::O, Diacritic::None, Tone::Acute) => ('\u{00E3}', '\u{00E3}'), // ó, Ó
        (BaseVowel::O, Diacritic::None, Tone::Grave) => ('\u{00DF}', '\u{00DF}'), // ò, Ò
        (BaseVowel::O, Diacritic::None, Tone::HookAbove) => ('\u{00E1}', '\u{00E1}'), // ỏ, Ỏ
        (BaseVowel::O, Diacritic::None, Tone::Tilde) => ('\u{00E2}', '\u{00E2}'), // õ, Õ
        (BaseVowel::O, Diacritic::None, Tone::DotBelow) => ('\u{00E4}', '\u{00E4}'), // ọ, Ọ

        (BaseVowel::O, Diacritic::Circumflex, Tone::None) => ('\u{00AB}', '\u{00A4}'), // ô, Ô
        (BaseVowel::O, Diacritic::Circumflex, Tone::Acute) => ('\u{00E8}', '\u{00E8}'), // ố, Ố
        (BaseVowel::O, Diacritic::Circumflex, Tone::Grave) => ('\u{00E5}', '\u{00E5}'), // ồ, Ồ
        (BaseVowel::O, Diacritic::Circumflex, Tone::HookAbove) => ('\u{00E6}', '\u{00E6}'), // ổ, Ổ
        (BaseVowel::O, Diacritic::Circumflex, Tone::Tilde) => ('\u{00E7}', '\u{00E7}'), // ỗ, Ỗ
        (BaseVowel::O, Diacritic::Circumflex, Tone::DotBelow) => ('\u{00E9}', '\u{00E9}'), // ộ, Ộ

        // --- Ơ ---
        (BaseVowel::O, Diacritic::Horn, Tone::None) => ('\u{00AC}', '\u{00A5}'), // ơ, Ơ
        (BaseVowel::O, Diacritic::Horn, Tone::Acute) => ('\u{00ED}', '\u{00ED}'), // ớ, Ớ
        (BaseVowel::O, Diacritic::Horn, Tone::Grave) => ('\u{00EA}', '\u{00EA}'), // ờ, Ờ
        (BaseVowel::O, Diacritic::Horn, Tone::HookAbove) => ('\u{00EB}', '\u{00EB}'), // ở, Ở
        (BaseVowel::O, Diacritic::Horn, Tone::Tilde) => ('\u{00EC}', '\u{00EC}'), // ỡ, Ỡ
        (BaseVowel::O, Diacritic::Horn, Tone::DotBelow) => ('\u{00EE}', '\u{00EE}'), // ợ, Ợ

        // --- U ---
        (BaseVowel::U, Diacritic::None, Tone::None) => ('u', 'U'),
        (BaseVowel::U, Diacritic::None, Tone::Acute) => ('\u{00F3}', '\u{00F3}'), // ú, Ú
        (BaseVowel::U, Diacritic::None, Tone::Grave) => ('\u{00EF}', '\u{00EF}'), // ù, Ù
        (BaseVowel::U, Diacritic::None, Tone::HookAbove) => ('\u{00F1}', '\u{00F1}'), // ủ, Ủ
        (BaseVowel::U, Diacritic::None, Tone::Tilde) => ('\u{00F2}', '\u{00F2}'), // ũ, Ũ
        (BaseVowel::U, Diacritic::None, Tone::DotBelow) => ('\u{00F4}', '\u{00F4}'), // ụ, Ụ

        // --- Ư ---
        (BaseVowel::U, Diacritic::Horn, Tone::None) => ('\u{00AD}', '\u{00A6}'), // ư, Ư
        (BaseVowel::U, Diacritic::Horn, Tone::Acute) => ('\u{00F8}', '\u{00F8}'), // ứ, Ứ
        (BaseVowel::U, Diacritic::Horn, Tone::Grave) => ('\u{00F5}', '\u{00F5}'), // ừ, Ừ
        (BaseVowel::U, Diacritic::Horn, Tone::HookAbove) => ('\u{00F6}', '\u{00F6}'), // ử, Ử
        (BaseVowel::U, Diacritic::Horn, Tone::Tilde) => ('\u{00F7}', '\u{00F7}'), // ữ, Ữ
        (BaseVowel::U, Diacritic::Horn, Tone::DotBelow) => ('\u{00F9}', '\u{00F9}'), // ự, Ự

        // --- Y ---
        (BaseVowel::Y, _, Tone::None) => ('y', 'Y'),
        (BaseVowel::Y, _, Tone::Acute) => ('\u{00FD}', '\u{00FD}'), // ý, Ý
        (BaseVowel::Y, _, Tone::Grave) => ('\u{00FA}', '\u{00FA}'), // ỳ, Ỳ
        (BaseVowel::Y, _, Tone::HookAbove) => ('\u{00FB}', '\u{00FB}'), // ỷ, Ỷ
        (BaseVowel::Y, _, Tone::Tilde) => ('\u{00FC}', '\u{00FC}'), // ỹ, Ỹ
        (BaseVowel::Y, _, Tone::DotBelow) => ('\u{00FE}', '\u{00FE}'), // ỵ, Ỵ

        // Fallbacks for invalid combinations
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

/// Compile-time lookup table mapping (BaseVowel, Diacritic, Tone, uppercase) -> TCVN3 codepoint.
const TCVN3_COMPOSE_LUT: [u16; 360] = {
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
                let (lower, upper) = compute_tcvn3_vowel_pair(bases[b], diacritics[d], tones[t]);
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

/// Builds a TCVN3 character from base vowel, diacritic, tone, and uppercase flag.
#[inline(always)]
pub const fn compose_tcvn3_vowel(
    base: BaseVowel,
    diacritic: Diacritic,
    tone: Tone,
    uppercase: bool,
) -> char {
    let idx = (((base as usize * 5 + diacritic as usize) * 6 + tone as usize) << 1)
        | (uppercase as usize);
    unsafe { char::from_u32_unchecked(TCVN3_COMPOSE_LUT[idx] as u32) }
}

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
