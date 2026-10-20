use super::common::{simulate_typing, type_key};
use kora::{EngineConfig, InputMethod, VietnameseEngine};

#[test]
fn test_lowercase_vowel_case_preservation() {
    // Test case cốt lõi: "tôi gõ tiếng việt" KHÔNG BAO GIỜ bị biến thành "tÔi gÕ tiẾng viỆt"
    let mut engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Telex,
        ..Default::default()
    });

    // "tooi" -> "tôi"
    assert_eq!(simulate_typing(&mut engine, "tooi "), "tôi ");

    // "gox" -> "gõ"
    assert_eq!(simulate_typing(&mut engine, "gox "), "gõ ");

    // "tieengs" -> "tiếng"
    assert_eq!(simulate_typing(&mut engine, "tieengs "), "tiếng ");

    // "vieetj" -> "việt"
    assert_eq!(simulate_typing(&mut engine, "vieetj "), "việt ");

    // Cả câu liền mạch
    assert_eq!(
        simulate_typing(&mut engine, "tooi gox tieengs vieetj "),
        "tôi gõ tiếng việt "
    );
}

#[test]
fn test_capitalized_first_letter() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    assert_eq!(simulate_typing(&mut engine, "Tooi "), "Tôi ");
    assert_eq!(simulate_typing(&mut engine, "Vieetj "), "Việt ");
    assert_eq!(
        simulate_typing(&mut engine, "Tieengs Vieetj "),
        "Tiếng Việt "
    );
}

#[test]
fn test_all_caps() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    assert_eq!(simulate_typing(&mut engine, "TOOI "), "TÔI ");
    assert_eq!(simulate_typing(&mut engine, "VIEETJ "), "VIỆT ");
}

#[test]
fn test_vni_input_method() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Vni,
        ..Default::default()
    });

    assert_eq!(simulate_typing(&mut engine, "to6i "), "tôi ");
    assert_eq!(simulate_typing(&mut engine, "go4 "), "gõ ");
    assert_eq!(simulate_typing(&mut engine, "tie6ng1 "), "tiếng ");
    assert_eq!(simulate_typing(&mut engine, "vie6t5 "), "việt ");
    assert_eq!(simulate_typing(&mut engine, "d9a6t1 "), "đất ");
    assert_eq!(simulate_typing(&mut engine, "d9uo7ng2 "), "đường ");
    assert_eq!(simulate_typing(&mut engine, "a8n "), "ăn ");
    // Test remove tone with '0'
    assert_eq!(simulate_typing(&mut engine, "to10 "), "to ");
    // Test tone toggle with same number
    assert_eq!(simulate_typing(&mut engine, "to11 "), "to ");
}

#[test]
fn test_auto_uppercase_first_char() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        auto_uppercase_first_char: true,
        ..Default::default()
    });

    assert_eq!(simulate_typing(&mut engine, "chao. toi "), "chao. Toi ");
}

#[test]
fn test_toggle_enabled_mode_switch() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    assert!(engine.config().enabled);

    // Ở chế độ Tiếng Việt: gõ "tieengs vieetj " -> "tiếng việt "
    assert_eq!(
        simulate_typing(&mut engine, "tieengs vieetj "),
        "tiếng việt "
    );

    // Chuyển sang chế độ Tiếng Anh bằng toggle_enabled()
    let enabled = engine.toggle_enabled();
    assert!(!enabled);
    assert!(!engine.config().enabled);

    // Ở chế độ Tiếng Anh: gõ các phím Telex không bị biến âm
    assert_eq!(
        simulate_typing(&mut engine, "tieengs vieetj "),
        "tieengs vieetj "
    );
    assert_eq!(simulate_typing(&mut engine, "ddas banhs "), "ddas banhs ");

    // Chuyển lại sang Tiếng Việt
    let enabled2 = engine.toggle_enabled();
    assert!(enabled2);
    assert!(engine.config().enabled);

    // Gõ tiếng Việt lại bình thường
    assert_eq!(
        simulate_typing(&mut engine, "tieengs vieetj "),
        "tiếng việt "
    );
}

#[test]
fn test_simple_telex_vs_standard_telex() {
    // 1. Standard Telex
    let mut telex_engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Telex,
        bracket_w: true,
        restore_on_wrong_spelling: false,
        ..Default::default()
    });
    // Trong Telex chuẩn: 'w' đứng một mình biến thành 'ư', '[' và ']' biến thành 'ư' và 'ơ'
    assert_eq!(simulate_typing(&mut telex_engine, "w"), "ư");
    telex_engine.reset();
    assert_eq!(simulate_typing(&mut telex_engine, "win"), "ưin");
    telex_engine.reset();
    assert_eq!(simulate_typing(&mut telex_engine, "web"), "ưeb");
    telex_engine.reset();
    assert_eq!(simulate_typing(&mut telex_engine, "["), "ư");
    telex_engine.reset();
    assert_eq!(simulate_typing(&mut telex_engine, "]"), "ơ");
    telex_engine.reset();
    assert_eq!(simulate_typing(&mut telex_engine, "sw"), "sư");

    // 2. Simple Telex
    let mut simple_engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::SimpleTelex1,
        bracket_w: true, // Thậm chí bật bracket_w thì Simple Telex vẫn không chiếm phím ngoặc
        restore_on_wrong_spelling: true,
        ..Default::default()
    });
    // Trong Simple Telex: 'w' đầu từ giữ nguyên là 'w', các phím ngoặc '[' và ']' giữ nguyên
    assert_eq!(simulate_typing(&mut simple_engine, "w"), "w");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "win"), "win");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "web"), "web");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "wiki"), "wiki");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "world "), "world ");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "["), "[");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "]"), "]");

    // Trong Simple Telex: 'sw' không bị biến thành 'sư' mà giữ nguyên onset tiếng Anh
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "sw"), "sw");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "switch"), "switch");
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "two"), "two");

    // Tuy nhiên Simple Telex VẪN gõ tiếng Việt hoàn hảo khi 'w' đi sau nguyên âm:
    simple_engine.reset();
    assert_eq!(simulate_typing(&mut simple_engine, "muaw "), "mưa ");
    assert_eq!(simulate_typing(&mut simple_engine, "tow "), "tơ ");
    assert_eq!(simulate_typing(&mut simple_engine, "tuw "), "tư ");
    assert_eq!(simulate_typing(&mut simple_engine, "tieengs "), "tiếng ");
    assert_eq!(simulate_typing(&mut simple_engine, "vieetj "), "việt ");
    assert_eq!(simulate_typing(&mut simple_engine, "ddangw "), "đăng ");
}

#[test]
fn test_diacritic_switching() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. Vòng lặp hoán đổi liên tục 20 lần: luôn <-> lươn (phím 'w' và 'o')
    let mut scr = String::new();
    for ch in "luoon".chars() {
        type_key(&mut engine, ch, &mut scr);
    }
    assert_eq!(scr, "luôn");

    for i in 1..=20 {
        type_key(&mut engine, 'w', &mut scr);
        assert_eq!(scr, "lươn", "Lỗi vòng lặp {} khi chuyển luôn -> lươn", i);
        type_key(&mut engine, 'o', &mut scr);
        assert_eq!(scr, "luôn", "Lỗi vòng lặp {} khi chuyển lươn -> luôn", i);
    }

    // 2. Vòng lặp hoán đổi liên tục 20 lần có dấu thanh: muộn <-> mượn
    engine.reset();
    scr.clear();
    for ch in "muoojn".chars() {
        type_key(&mut engine, ch, &mut scr);
    }
    assert_eq!(scr, "muộn");

    for i in 1..=20 {
        type_key(&mut engine, 'w', &mut scr);
        assert_eq!(scr, "mượn", "Lỗi vòng lặp {} khi chuyển muộn -> mượn", i);
        type_key(&mut engine, 'o', &mut scr);
        assert_eq!(scr, "muộn", "Lỗi vòng lặp {} khi chuyển mượn -> muộn", i);
    }

    // 3. Vòng lặp hoán đổi liên tục 20 lần có dấu thanh và âm cuối khác: cuộc <-> cược
    engine.reset();
    scr.clear();
    for ch in "cuoojc".chars() {
        type_key(&mut engine, ch, &mut scr);
    }
    assert_eq!(scr, "cuộc");

    for i in 1..=20 {
        type_key(&mut engine, 'w', &mut scr);
        assert_eq!(scr, "cược", "Lỗi vòng lặp {} khi chuyển cuộc -> cược", i);
        type_key(&mut engine, 'o', &mut scr);
        assert_eq!(scr, "cuộc", "Lỗi vòng lặp {} khi chuyển cược -> cuộc", i);
    }

    // 4. Vòng lặp hoán đổi liên tục 20 lần: cân <-> căn (phím 'w' và 'a')
    engine.reset();
    scr.clear();
    for ch in "caan".chars() {
        type_key(&mut engine, ch, &mut scr);
    }
    assert_eq!(scr, "cân");

    for i in 1..=20 {
        type_key(&mut engine, 'w', &mut scr);
        assert_eq!(scr, "căn", "Lỗi vòng lặp {} khi chuyển cân -> căn", i);
        type_key(&mut engine, 'a', &mut scr);
        assert_eq!(scr, "cân", "Lỗi vòng lặp {} khi chuyển căn -> cân", i);
    }

    // 5. Vòng lặp hoán đổi liên tục 20 lần: côn <-> cơn (phím 'w' và 'o')
    engine.reset();
    scr.clear();
    for ch in "coon".chars() {
        type_key(&mut engine, ch, &mut scr);
    }
    assert_eq!(scr, "côn");

    for i in 1..=20 {
        type_key(&mut engine, 'w', &mut scr);
        assert_eq!(scr, "cơn", "Lỗi vòng lặp {} khi chuyển côn -> cơn", i);
        type_key(&mut engine, 'o', &mut scr);
        assert_eq!(scr, "côn", "Lỗi vòng lặp {} khi chuyển cơn -> côn", i);
    }

    // 6. Vòng lặp hoán đổi liên tục 20 lần kiểu gõ VNI: luôn <-> lươn (phím '7' và '6')
    let mut vni_engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Vni,
        ..Default::default()
    });
    let mut vni_scr = String::new();
    for ch in "luon6".chars() {
        type_key(&mut vni_engine, ch, &mut vni_scr);
    }
    assert_eq!(vni_scr, "luôn");

    for i in 1..=20 {
        type_key(&mut vni_engine, '7', &mut vni_scr);
        assert_eq!(
            vni_scr, "lươn",
            "Lỗi VNI vòng lặp {} khi chuyển luôn -> lươn",
            i
        );
        type_key(&mut vni_engine, '6', &mut vni_scr);
        assert_eq!(
            vni_scr, "luôn",
            "Lỗi VNI vòng lặp {} khi chuyển lươn -> luôn",
            i
        );
    }
}
