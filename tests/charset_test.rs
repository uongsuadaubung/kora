use kora::{
    BaseVowel, Charset, CharsetStrategy, Diacritic, EngineConfig, InputMethod, Tcvn3Charset, Tone,
    UnicodeCharset, VietnameseEngine,
};

fn simulate_typing(engine: &mut VietnameseEngine, text: &str) -> String {
    let mut output = String::new();
    for ch in text.chars() {
        let is_upper = ch.is_uppercase();
        let action = engine.on_key(ch, is_upper, false);
        match action {
            kora::EngineAction::Passthrough => output.push(ch),
            kora::EngineAction::Replace {
                backspaces,
                output: replace_str,
            } => {
                for _ in 0..backspaces {
                    output.pop();
                }
                output.push_str(replace_str.as_str());
            }
            kora::EngineAction::Consume => {}
        }
    }
    output
}

#[test]
fn test_charset_strategy_unicode() {
    let strategy = UnicodeCharset;
    assert_eq!(strategy.name(), "Unicode");

    // Vowel compose
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::None, Tone::Acute, false),
        'á'
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::Circumflex, Tone::None, true),
        'Â'
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::O, Diacritic::Horn, Tone::HookAbove, false),
        'ở'
    );

    // D compose
    assert_eq!(strategy.compose_d(true, false), 'đ');
    assert_eq!(strategy.compose_d(true, true), 'Đ');
    assert_eq!(strategy.compose_d(false, false), 'd');
    assert_eq!(strategy.compose_d(false, true), 'D');

    // Encode text
    assert_eq!(strategy.encode_text("Việt Nam"), "Việt Nam");
}

#[test]
fn test_charset_strategy_tcvn3() {
    let strategy = Tcvn3Charset;
    assert_eq!(strategy.name(), "TCVN3");

    // Độc thù chữ hoa không dấu thanh trong TCVN3
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::Circumflex, Tone::None, true),
        '\u{00A2}' // Â
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::Circumflex, Tone::None, false),
        '\u{00A9}' // â
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::Breve, Tone::None, true),
        '\u{00A1}' // Ă
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::Breve, Tone::None, false),
        '\u{00A8}' // ă
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::E, Diacritic::Circumflex, Tone::None, true),
        '\u{00A3}' // Ê
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::O, Diacritic::Circumflex, Tone::None, true),
        '\u{00A4}' // Ô
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::O, Diacritic::Horn, Tone::None, true),
        '\u{00A5}' // Ơ
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::U, Diacritic::Horn, Tone::None, true),
        '\u{00A6}' // Ư
    );

    // Ký tự đ / Đ
    assert_eq!(strategy.compose_d(true, false), '\u{00AE}'); // đ
    assert_eq!(strategy.compose_d(true, true), '\u{00A7}'); // Đ

    // Chữ hoa có dấu thanh dùng chung mã thường trong font TCVN3 (.VnTime)
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::None, Tone::Acute, false),
        '\u{00B8}' // á
    );
    assert_eq!(
        strategy.compose_vowel(BaseVowel::A, Diacritic::None, Tone::Acute, true),
        '\u{00B8}' // Á -> fallback 0xB8
    );

    // Encode text
    // "Việt" -> 'V' + 'i' + 'ệ' (0xD6) + 't'
    let encoded = strategy.encode_text("Việt");
    assert_eq!(encoded.as_str(), "Vi\u{00D6}t");

    // "đoán" -> 'đ' (0xAE) + 'o' + 'á' (0xB8) + 'n'
    let encoded_doan = strategy.encode_text("đoán");
    assert_eq!(encoded_doan.as_str(), "\u{00AE}o\u{00B8}n");
}

#[test]
fn test_engine_tcvn3_typing() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Telex,
        charset: Charset::Tcvn3,
        ..Default::default()
    });

    // "tooi " -> "tôi " -> 't' + '\u{00AB}' (ô) + 'i' + ' '
    let res = simulate_typing(&mut engine, "tooi ");
    assert_eq!(res, "t\u{00AB}i ");

    // "gox " -> "gõ " -> 'g' + '\u{00E2}' (õ) + ' '
    let res = simulate_typing(&mut engine, "gox ");
    assert_eq!(res, "g\u{00E2} ");

    // "ddaay " -> "đây " -> '\u{00AE}' (đ) + '\u{00A9}' (â) + 'y' + ' '
    let res = simulate_typing(&mut engine, "ddaay ");
    assert_eq!(res, "\u{00AE}\u{00A9}y ");

    // Chữ hoa: "DD " -> 'Đ ' -> '\u{00A7} '
    let res = simulate_typing(&mut engine, "DD ");
    assert_eq!(res, "\u{00A7} ");
}

#[test]
fn test_engine_tcvn3_macro_expansion() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Telex,
        charset: Charset::Tcvn3,
        use_macro: true,
        ..Default::default()
    });

    engine.macro_table.insert_typed_no_cache(
        "vn",
        "Việt Nam",
        kora::engine::macro_table::MacroType::Normal,
    );

    // Gõ tắt: "vn " -> nở ra "Việt Nam" được mã hóa theo TCVN3
    let res = simulate_typing(&mut engine, "vn ");
    assert_eq!(res, "Vi\u{00D6}t Nam ");
}

#[test]
fn test_config_codec_charset() {
    let config = EngineConfig {
        charset: Charset::Tcvn3,
        ..Default::default()
    };
    let macros = kora::engine::macro_table::MacroTable::new();
    let ini_str =
        kora::engine::config_store::codec::encoder::serialize_config_and_macros(&config, &macros);
    assert!(ini_str.contains("charset = tcvn3"));

    let (loaded_config, _) =
        kora::engine::config_store::codec::decoder::parse_config_and_macros(&ini_str);
    assert_eq!(loaded_config.charset, Charset::Tcvn3);
}

#[test]
fn test_child_profile_charset_inheritance_and_override() {
    use kora::{AppProfile, CharsetMode, ProfileManager};

    let mut profiles = ProfileManager::new();
    let mut child_word = AppProfile::new_child("word", "Microsoft Word", "winword.exe", "📝");
    child_word.charset = CharsetMode::Inherit;
    profiles.add_profile(child_word);

    let mut child_excel = AppProfile::new_child("excel", "Microsoft Excel", "excel.exe", "📊");
    child_excel.charset = CharsetMode::Specific(Charset::Tcvn3);
    profiles.add_profile(child_excel);

    let mut global_cfg = EngineConfig {
        charset: Charset::Unicode,
        ..Default::default()
    };

    // 1. Child 'word' kế thừa Global (Unicode)
    let eff_word = profiles.resolve_effective_config("word", &global_cfg);
    assert_eq!(eff_word.charset, Charset::Unicode);

    // 2. Khi Global đổi sang TCVN3 -> Child 'word' tự động kế thừa thành TCVN3
    global_cfg.charset = Charset::Tcvn3;
    let eff_word_tcvn3 = profiles.resolve_effective_config("word", &global_cfg);
    assert_eq!(eff_word_tcvn3.charset, Charset::Tcvn3);

    // 3. Child 'excel' ghi đè riêng sang TCVN3, bất kể Global là Unicode
    global_cfg.charset = Charset::Unicode;
    let eff_excel = profiles.resolve_effective_config("excel", &global_cfg);
    assert_eq!(eff_excel.charset, Charset::Tcvn3);
}

#[test]
fn test_child_profile_charset_codec_roundtrip() {
    use kora::{AppProfile, CharsetMode, ProfileManager};

    let global_cfg = EngineConfig {
        charset: Charset::Unicode,
        ..Default::default()
    };
    let macros = kora::engine::macro_table::MacroTable::new();
    let mut profiles = ProfileManager::new();

    let mut child = AppProfile::new_child("office", "Office App", "office.exe", "📝");
    child.charset = CharsetMode::Specific(Charset::Tcvn3);
    profiles.add_profile(child);

    let ini_str = kora::engine::config_store::codec::encoder::serialize_config_all(
        &global_cfg,
        &macros,
        &profiles,
    );
    assert!(ini_str.contains("[profile:office]"));
    assert!(ini_str.contains("charset = tcvn3"));

    let (_, _, loaded_profiles) =
        kora::engine::config_store::codec::decoder::parse_config_all(&ini_str);
    let loaded_office = loaded_profiles
        .get_profile("office")
        .expect("profile office exists");
    assert_eq!(loaded_office.charset, CharsetMode::Specific(Charset::Tcvn3));
}
