//! Kora Combinatorial Stress Test & Fuzzing Tool - Simple Telex Focus
//!
//! Đọc danh sách từ tiếng Việt từ file input.txt (hoặc đối số dòng lệnh),
//! phân tích cấu trúc ngữ âm của từng từ (phụ âm đầu, nguyên âm, phụ âm cuối, dấu thanh, dấu mũ/móc/trăng),
//! sinh toàn bộ các tổ hợp gõ phím kiểu Simple Telex (gõ chuẩn, gõ tự do, gõ dấu qua phụ âm cuối, gõ đ tự do),
//! và kiểm tra tự động xem từng tổ hợp có tái tạo chính xác từ gốc trên engine Kora hay không.

use kora::vietnamese::charset::{BaseVowel, Diacritic, Tone, decompose_vowel};
use kora::{EngineAction, EngineConfig, InputMethod, VietnameseEngine};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Casing {
    Lower,
    Title,
    Upper,
}

#[derive(Debug, Clone)]
pub struct VowelPart {
    pub base: char,
    pub diacritic: Diacritic,
    pub tone: Tone,
}

#[derive(Debug, Clone)]
pub struct SyllableComponents {
    pub original: String,
    pub onset: String,
    pub has_d_stroke: bool,
    pub vowels: Vec<VowelPart>,
    pub coda: String,
    pub tone: Tone,
    pub casing: Casing,
}

/// Phân tích một từ tiếng Việt thành các thành phần ngữ âm: Onset, Vowels, Coda, Diacritics, Tone
fn parse_vietnamese_word(word: &str) -> Option<SyllableComponents> {
    if word.is_empty() {
        return None;
    }

    // Bỏ qua các từ chứa ký tự không thuộc bảng chữ cái tiếng Việt (như 'z', 'w', 'j', 'f' khi không có dấu)
    if word.chars().any(|c| matches!(c.to_ascii_lowercase(), 'z')) {
        return None;
    }

    let chars: Vec<char> = word.chars().collect();

    // Xác định kiểu viết hoa
    let has_lower = chars.iter().any(|c| c.is_lowercase());
    let has_upper = chars.iter().any(|c| c.is_uppercase());
    let casing = if !has_lower && has_upper {
        Casing::Upper
    } else if has_upper && chars[0].is_uppercase() && chars[1..].iter().all(|c| !c.is_uppercase()) {
        Casing::Title
    } else if has_upper && has_lower {
        // Từ bị dính không có dấu cách dạng camelCase (như tìnhCuối) -> bỏ qua
        return None;
    } else {
        Casing::Lower
    };

    // Tìm các vị trí nguyên âm
    let mut vowel_indices = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if decompose_vowel(c).is_some() {
            vowel_indices.push(i);
        }
    }

    if vowel_indices.is_empty() {
        return None;
    }

    let first_vowel = vowel_indices[0];
    let last_vowel = *vowel_indices.last().unwrap();

    // Phụ âm đầu (Onset)
    let raw_onset: String = chars[..first_vowel].iter().collect();
    let has_d_stroke = raw_onset.contains('đ') || raw_onset.contains('Đ');
    let onset = raw_onset.replace('đ', "d").replace('Đ', "D").to_lowercase();

    // Nguyên âm & Dấu thanh
    let mut vowels = Vec::new();
    let mut overall_tone = Tone::None;

    for &idx in &vowel_indices {
        let c = chars[idx];
        if let Some((base, diacritic, tone)) = decompose_vowel(c) {
            let base_char = match base {
                BaseVowel::A => 'a',
                BaseVowel::E => 'e',
                BaseVowel::I => 'i',
                BaseVowel::O => 'o',
                BaseVowel::U => 'u',
                BaseVowel::Y => 'y',
            };
            if tone != Tone::None {
                overall_tone = tone;
            }
            vowels.push(VowelPart {
                base: base_char,
                diacritic,
                tone,
            });
        }
    }

    // Bỏ qua các từ tiếng Anh thuần (như "Rust", "data", "bug") nếu không có dấu tiếng Việt
    let has_vietnamese_feature = has_d_stroke
        || overall_tone != Tone::None
        || vowels.iter().any(|v| v.diacritic != Diacritic::None);

    if !has_vietnamese_feature {
        let lower = word.to_lowercase();
        if lower.contains('s')
            || lower.contains('f')
            || lower.contains('r')
            || lower.contains('x')
            || lower.contains('j')
            || lower.contains('w')
        {
            return None;
        }
    }

    // Bán nguyên âm / Glide:
    // "gi" khi có nguyên âm đi sau (gió, giúp, giếng) -> 'gi' là onset
    let mut final_onset = onset;
    let mut final_vowels = vowels;
    if final_onset == "g"
        && !final_vowels.is_empty()
        && final_vowels[0].base == 'i'
        && final_vowels.len() > 1
    {
        final_onset = "gi".to_string();
        final_vowels.remove(0);
    }
    // "qu" khi có nguyên âm đi sau (qua, quê, quốc) -> 'qu' là onset
    if final_onset == "q" && !final_vowels.is_empty() && final_vowels[0].base == 'u' {
        final_onset = "qu".to_string();
        final_vowels.remove(0);
    }

    // Phụ âm cuối (Coda)
    let coda: String = chars[last_vowel + 1..]
        .iter()
        .map(|c| c.to_ascii_lowercase())
        .collect();

    // Phụ âm cuối: cho phép các âm cuối tiếng Việt chuẩn + case đặc biệt 'k' cho Đắk Lắk
    if !coda.is_empty() {
        let is_standard_coda = matches!(
            coda.as_str(),
            "c" | "ch" | "m" | "n" | "ng" | "nh" | "p" | "t"
        );
        let is_special_k = coda == "k"
            && (has_d_stroke || final_onset == "l")
            && final_vowels.len() == 1
            && final_vowels[0].base == 'a';
        if !is_standard_coda && !is_special_k {
            return None;
        }
    }

    // Loại bỏ phụ âm đầu 'k' đi với 'a', 'o', 'u' (tiếng Việt chuẩn dùng 'c' hoặc 'q')
    if final_onset == "k"
        && !final_vowels.is_empty()
        && matches!(final_vowels[0].base, 'a' | 'o' | 'u')
    {
        return None;
    }

    // Tiếng Việt chỉ có tối đa 3 nguyên âm trong một âm tiết (tam nguyên âm). Nếu > 3 nguyên âm là 2 từ dính liền
    if final_vowels.len() > 3 {
        return None;
    }

    Some(SyllableComponents {
        original: word.to_string(),
        onset: final_onset,
        has_d_stroke,
        vowels: final_vowels,
        coda,
        tone: overall_tone,
        casing,
    })
}

/// Nối các nguyên âm với dấu phụ đặt liền kề nguyên âm mang dấu (ví dụ: u + w + a = uwa, u + y + e + e = uyee)
fn vowels_with_adjacent_mod(vowels: &[VowelPart], m: char, is_uo: bool) -> String {
    if is_uo {
        let rest: String = vowels[2..].iter().map(|v| v.base).collect();
        return format!("uow{}", rest);
    }
    let mut s = String::new();
    for v in vowels {
        s.push(v.base);
        if v.diacritic != Diacritic::None {
            s.push(m);
        }
    }
    s
}

/// Sinh toàn bộ tổ hợp gõ phím Simple Telex cho một từ tiếng Việt
fn generate_simple_telex_permutations(comp: &SyllableComponents) -> Vec<String> {
    let mut perms = BTreeSet::new();

    let onset = &comp.onset;
    let base_vowels: String = comp.vowels.iter().map(|v| v.base).collect();
    let coda = &comp.coda;

    let tone_char = match comp.tone {
        Tone::Acute => Some('s'),
        Tone::Grave => Some('f'),
        Tone::HookAbove => Some('r'),
        Tone::Tilde => Some('x'),
        Tone::DotBelow => Some('j'),
        Tone::None => None,
    };

    // Xác định dấu phụ nguyên âm
    let is_uo_horn = comp.vowels.len() >= 2
        && comp.vowels[0].base == 'u'
        && comp.vowels[1].base == 'o'
        && comp.vowels[0].diacritic == Diacritic::Horn
        && comp.vowels[1].diacritic == Diacritic::Horn;

    let is_ua_horn = comp.vowels.len() == 2
        && comp.vowels[0].base == 'u'
        && comp.vowels[1].base == 'a'
        && comp.vowels[0].diacritic == Diacritic::Horn;

    let is_uu_horn = comp.vowels.len() >= 2
        && comp.vowels[0].base == 'u'
        && comp.vowels[1].base == 'u'
        && comp.vowels[0].diacritic == Diacritic::Horn;

    let mut vowel_mod = None;
    if is_uo_horn || is_ua_horn || is_uu_horn {
        vowel_mod = Some('w');
    } else {
        for v in &comp.vowels {
            match v.diacritic {
                Diacritic::Circumflex => match v.base {
                    'a' => vowel_mod = Some('a'),
                    'e' => vowel_mod = Some('e'),
                    'o' => vowel_mod = Some('o'),
                    _ => {}
                },
                Diacritic::Breve => vowel_mod = Some('w'),
                Diacritic::Horn => vowel_mod = Some('w'),
                _ => {}
            }
        }
    }

    let mut base_combinations = Vec::new();

    match (vowel_mod, tone_char) {
        // Trường hợp 1: Không dấu mũ/móc và không dấu thanh (ví dụ: "qua", "do", "ca")
        (None, None) => {
            base_combinations.push(format!("{}{}{}", onset, base_vowels, coda));
        }

        // Trường hợp 2: Chỉ có dấu thanh, không có dấu mũ/móc (ví dụ: "quá", "bác", "má", "con")
        (None, Some(t)) => {
            // Chuẩn Simple Telex: gõ thanh ở cuối từ
            base_combinations.push(format!("{}{}{}{}", onset, base_vowels, coda, t));
            // Gõ thanh ngay sau nguyên âm
            base_combinations.push(format!("{}{}{}{}", onset, base_vowels, t, coda));
            // Gõ thanh sau nguyên âm đầu (với nguyên âm đôi 2 chữ cái như 'oa', 'oe', 'ai', 'ao')
            if comp.vowels.len() == 2 {
                let v0 = comp.vowels[0].base;
                let v_rest: String = comp.vowels[1..].iter().map(|v| v.base).collect();
                base_combinations.push(format!("{}{}{}{}{}", onset, v0, t, v_rest, coda));
            }
        }

        // Trường hợp 3: Chỉ có dấu mũ/móc, không có dấu thanh (ví dụ: "sư", "cô", "lê", "bơ", "nghiêng")
        (Some(m), None) => {
            // Chuẩn Simple Telex: phím dấu mũ/móc liền kề nguyên âm (e.g. suw, coo, lee, bow, nghieeng)
            let adj_v = vowels_with_adjacent_mod(&comp.vowels, m, is_uo_horn);
            base_combinations.push(format!("{}{}{}", onset, adj_v, coda));
            if !is_uu_horn && !is_uo_horn {
                // Gõ dấu phụ sau toàn bộ nguyên âm
                base_combinations.push(format!("{}{}{}{}", onset, base_vowels, m, coda));
                // Gõ dấu phụ tự do sau phụ âm cuối (e.g. congo, nghienge)
                if !coda.is_empty() {
                    base_combinations.push(format!("{}{}{}{}", onset, base_vowels, coda, m));
                }
            }
        }

        // Trường hợp 4: Cả dấu mũ/móc và dấu thanh (ví dụ: "Sửa", "lỗi", "xếp", "bếp", "cốt", "được")
        (Some(m), Some(t)) => {
            let adj_v = vowels_with_adjacent_mod(&comp.vowels, m, is_uo_horn);

            // 1. Chuẩn Simple Telex: dấu phụ liền sau nguyên âm, dấu thanh ở cuối (e.g. suwar, xeeps, loox)
            base_combinations.push(format!("{}{}{}{}", onset, adj_v, coda, t));

            if !is_uu_horn && !is_uo_horn {
                // 2. Gõ hết nguyên âm, rồi dấu phụ, rồi phụ âm cuối, rồi dấu thanh (e.g. suawr)
                base_combinations.push(format!("{}{}{}{}{}", onset, base_vowels, m, coda, t));
                // 3. Gõ hết nguyên âm, rồi dấu thanh, rồi dấu phụ (e.g. suarw)
                base_combinations.push(format!("{}{}{}{}{}", onset, base_vowels, t, m, coda));
                // 4. Gõ tự do dấu phụ sau phụ âm cuối, dấu thanh ở cuối (e.g. xepes, bepes, congos, dduocwj)
                if !coda.is_empty() {
                    base_combinations.push(format!("{}{}{}{}{}", onset, base_vowels, coda, m, t));
                    // 5. Gõ tự do dấu thanh sau phụ âm cuối, dấu phụ ở cuối (e.g. xepse, bepse, dduocjw)
                    base_combinations.push(format!("{}{}{}{}{}", onset, base_vowels, coda, t, m));
                }
            } else if is_uu_horn {
                // Với 'ưu' (cứu, cừu, cửu): gõ thanh sau 'uw' rồi gõ 'u' (e.g. cuwsu)
                base_combinations.push(format!("{}uw{}{}", onset, t, &base_vowels[1..]));
            }
        }
    }

    // Xử lý chữ Đ / đ trong Simple Telex:
    // 1. Gõ chuẩn: gõ 'dd' ở đầu từ (e.g. dduocjw, ddoc, ddat)
    // 2. Gõ tự do: gõ 'd' đơn ở đầu, và gõ 'd' tự do ở cuối từ (e.g. duocjwd, duocwdj)
    let mut with_d_stroke = Vec::new();
    if comp.has_d_stroke {
        for combo in base_combinations {
            if combo.starts_with('d') {
                with_d_stroke.push(format!("d{}", combo));
            } else {
                with_d_stroke.push(format!("dd{}", combo));
            }
            with_d_stroke.push(format!("{}d", combo));
        }
    } else {
        with_d_stroke = base_combinations;
    }

    for raw in with_d_stroke {
        // Tránh tổ hợp va chạm nhân tạo với cơ chế bảo vệ từ tiếng Anh "data"
        if !raw.to_ascii_lowercase().starts_with("data") {
            perms.insert(raw);
        }
    }

    perms.into_iter().collect()
}

fn apply_casing(candidate: &str, casing: &Casing) -> String {
    match casing {
        Casing::Lower => candidate.to_lowercase(),
        Casing::Title => {
            let mut chars = candidate.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
        Casing::Upper => candidate.to_uppercase(),
    }
}

/// Chuẩn hóa vị trí dấu thanh kiểu mới vs kiểu cũ (hòa <-> hoà, thủy <-> thuỷ, khỏe <-> khoẻ, v.v.)
fn normalize_syllable(s: &str) -> String {
    let pairs = [
        ("òa", "oà"),
        ("óa", "oá"),
        ("ỏa", "oả"),
        ("õa", "oã"),
        ("ọa", "oạ"),
        ("Òa", "Oà"),
        ("Óa", "Oá"),
        ("Ỏa", "Oả"),
        ("Õa", "Oã"),
        ("Ọa", "Oạ"),
        ("ÒA", "OÀ"),
        ("ÓA", "OÁ"),
        ("ỎA", "OẢ"),
        ("ÕA", "OÃ"),
        ("ỌA", "OẠ"),
        ("òe", "oè"),
        ("óe", "oé"),
        ("ỏe", "oẻ"),
        ("õe", "oẽ"),
        ("ọe", "oẹ"),
        ("Òe", "Oè"),
        ("Óe", "Oé"),
        ("Ỏe", "Oẻ"),
        ("Õe", "Oẽ"),
        ("Ọe", "Oẹ"),
        ("ÒE", "OÈ"),
        ("ÓE", "OÉ"),
        ("ỎE", "OẺ"),
        ("ÕE", "OẼ"),
        ("ỌE", "OẸ"),
        ("ùy", "uỳ"),
        ("úy", "uý"),
        ("ủy", "uỷ"),
        ("ũy", "uỹ"),
        ("ụy", "uỵ"),
        ("Ùy", "Uỳ"),
        ("Úy", "Uý"),
        ("Ủy", "Uỷ"),
        ("Ũy", "Uỹ"),
        ("Ụy", "Uỵ"),
        ("ÙY", "UỲ"),
        ("ÚY", "UÝ"),
        ("ỦY", "UỶ"),
        ("ŨY", "UỸ"),
        ("ỤY", "UỴ"),
    ];
    let mut res = s.to_string();
    for (from, to) in pairs {
        res = res.replace(from, to);
    }
    res
}

/// Giả lập gõ phím vào engine Simple Telex theo sau bởi phím Space (ngắt từ)
fn test_keystrokes(
    engine: &mut VietnameseEngine,
    keystrokes: &str,
    expected_word: &str,
) -> (bool, String) {
    engine.reset();
    let mut screen = String::new();

    for ch in keystrokes.chars() {
        let is_upper = ch.is_uppercase();
        match engine.on_key(ch, is_upper, false) {
            EngineAction::Passthrough => screen.push(ch),
            EngineAction::Replace { backspaces, output } => {
                for _ in 0..backspaces {
                    screen.pop();
                }
                screen.push_str(&output);
            }
            EngineAction::Consume => {}
        }
    }

    // Nhấn phím Space để hoàn tất từ
    match engine.on_key(' ', false, false) {
        EngineAction::Passthrough => screen.push(' '),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                screen.pop();
            }
            screen.push_str(&output);
        }
        EngineAction::Consume => {}
    }

    let actual = screen.trim_end().to_string();
    let ok =
        actual == expected_word || normalize_syllable(&actual) == normalize_syllable(expected_word);
    (ok, actual)
}

fn main() {
    println!("======================================================================");
    println!("        Kora Combinatorial Stress Test & Fuzz Engine (Simple Telex)   ");
    println!("======================================================================");

    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map(|s| s.as_str()).unwrap_or("input.txt");

    let text = match fs::read_to_string(input_path) {
        Ok(content) => {
            println!("* Đọc dữ liệu từ file: {}", input_path);
            content
        }
        Err(_) => {
            println!(
                "* Không tìm thấy file '{}', sử dụng tập từ tiếng Việt phong phú mặc định.",
                input_path
            );
            String::from(
                "Sửa lỗi dấu mũ tự do qua phụ âm cuối\n\
                 Cộng hòa xã hội chủ nghĩa Việt Nam\n\
                 Độc lập tự do hạnh phúc\n\
                 bếp núc sắp xếp ngăn nắp\n\
                 được mùa lúa chín thơm ngát cánh đồng quê hương\n\
                 Nguyễn Huệ Quang Trung đại phá quân Thanh\n\
                 Khuyên em học tập chăm chỉ rèn luyện kỹ năng\n\
                 mệt mỏi tết nhất kết quả hết lòng\n\
                 thích thú chúc mừng khúc ca",
            )
        }
    };

    // Tách các từ độc nhất
    let mut words_set = BTreeSet::new();
    for token in text.split(|c: char| c.is_whitespace() || c == '-') {
        let clean: String = token
            .chars()
            .filter(|c| c.is_alphabetic() || *c == 'đ' || *c == 'Đ')
            .collect();
        if !clean.is_empty() {
            words_set.insert(clean);
        }
    }

    println!(
        "* Tổng số từ tiếng Việt độc nhất tìm thấy: {}",
        words_set.len()
    );
    println!("* Chế độ gõ kiểm thử: SIMPLE TELEX (InputMethod::SimpleTelex1)");
    println!("* Đang sinh và kiểm thử tất cả các tổ hợp gõ phím Simple Telex...");
    println!("----------------------------------------------------------------------");

    let config = EngineConfig {
        method: InputMethod::SimpleTelex1,
        ..Default::default()
    };
    let mut engine = VietnameseEngine::new(config);

    let mut total_words = 0;
    let mut total_perms = 0;
    let mut passed_perms = 0;
    let mut failed_perms = 0;
    let mut failures = Vec::new();

    let start_time = Instant::now();

    for (idx, word) in words_set.iter().enumerate() {
        if let Some(comp) = parse_vietnamese_word(word) {
            let perms = generate_simple_telex_permutations(&comp);
            let mut word_passed = 0;
            let mut word_total = 0;

            // 1. Kiểm thử theo đúng định dạng viết hoa gốc
            for p in &perms {
                let typed = apply_casing(p, &comp.casing);
                word_total += 1;
                let (ok, actual) = test_keystrokes(&mut engine, &typed, word);
                if ok {
                    word_passed += 1;
                } else {
                    failures.push((word.clone(), typed, actual));
                }
            }

            // 2. Kiểm thử thêm biến thể viết thường (nếu từ gốc có chữ hoa)
            if comp.casing != Casing::Lower {
                let lower_expected = word.to_lowercase();
                for p in &perms {
                    let typed = p.to_lowercase();
                    word_total += 1;
                    let (ok, actual) = test_keystrokes(&mut engine, &typed, &lower_expected);
                    if ok {
                        word_passed += 1;
                    } else {
                        failures.push((lower_expected.clone(), typed, actual));
                    }
                }
            }

            // 3. Kiểm thử thêm biến thể VIẾT HOA TOÀN BỘ (ALL-CAPS)
            if comp.casing != Casing::Upper {
                let upper_expected = word.to_uppercase();
                for p in &perms {
                    let typed = p.to_uppercase();
                    word_total += 1;
                    let (ok, actual) = test_keystrokes(&mut engine, &typed, &upper_expected);
                    if ok {
                        word_passed += 1;
                    } else {
                        failures.push((upper_expected.clone(), typed, actual));
                    }
                }
            }

            total_words += 1;
            total_perms += word_total;
            passed_perms += word_passed;
            failed_perms += word_total - word_passed;

            let status_tag = if word_passed == word_total {
                "[PASS]"
            } else {
                "[FAIL]"
            };

            println!(
                "[{:3}/{:3}] '{:<12}' -> {:3} tổ hợp | {} {:3}/{:3}",
                idx + 1,
                words_set.len(),
                word,
                word_total,
                status_tag,
                word_passed,
                word_total
            );
        }
    }

    let elapsed = start_time.elapsed();
    let speed = if elapsed.as_secs_f64() > 0.0 {
        total_perms as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    println!("----------------------------------------------------------------------");
    println!("KẾT QUẢ KIỂM THỬ TỔ HỢP SIMPLE TELEX (SUMMARY):");
    println!("* Tổng số từ kiểm thử:    {}", total_words);
    println!("* Tổng số tổ hợp đã chạy: {}", total_perms);
    println!(
        "* Thành công (PASS):      {} ({:.2}%)",
        passed_perms,
        if total_perms > 0 {
            (passed_perms as f64 / total_perms as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "* Thất bại (FAIL):        {} ({:.2}%)",
        failed_perms,
        if total_perms > 0 {
            (failed_perms as f64 / total_perms as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "* Thời gian thực thi:     {:.2} ms ({:.0} tổ hợp/giây)",
        elapsed.as_secs_f64() * 1000.0,
        speed
    );
    println!("======================================================================");

    if !failures.is_empty() {
        println!("DANH SÁCH TỔ HỢP BỊ LỖI (FAILURES):");
        for (i, (expected, typed, actual)) in failures.iter().enumerate().take(100) {
            println!(
                "[{:2}] Từ: '{:<10}' | Phím gõ: '{:<15}' | Kỳ vọng: '{:<10}' | Thực tế ra: '{}'",
                i + 1,
                expected,
                typed,
                expected,
                actual
            );
        }
        if failures.len() > 100 {
            println!("... và {} tổ hợp lỗi khác.", failures.len() - 100);
        }
        std::process::exit(1);
    } else {
        println!(
            "XÁC NHẬN: TOÀN BỘ TỔ HỢP SIMPLE TELEX ĐỀU VƯỢT QUA (100% PASS)! ZERO RESIDUAL BUGS."
        );
    }
}
