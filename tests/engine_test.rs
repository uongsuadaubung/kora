use kora::{EngineAction, EngineConfig, InputMethod, VietnameseEngine};

/// Helper to simulate typing a string and collecting the rendered output on screen
fn simulate_typing(engine: &mut VietnameseEngine, text: &str) -> String {
    let mut screen = String::new();
    for ch in text.chars() {
        let is_upper = ch.is_uppercase();
        match engine.on_key(ch, is_upper, false) {
            EngineAction::Passthrough => {
                screen.push(ch);
            }
            EngineAction::Replace { backspaces, output } => {
                for _ in 0..backspaces {
                    screen.pop();
                }
                screen.push_str(&output);
            }
            EngineAction::Consume => {}
        }
    }
    screen
}

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
fn test_free_mark_and_d_stroke() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    // "dd" -> "đ"
    assert_eq!(simulate_typing(&mut engine, "ddaats "), "đất ");
    // "toans" -> "toán" (bỏ dấu sau khi gõ xong phụ âm cuối)
    assert_eq!(simulate_typing(&mut engine, "toans "), "toán ");
    // "ddangwf" -> "đằng"
    assert_eq!(simulate_typing(&mut engine, "ddangwf "), "đằng ");
    // "giowf" -> "giờ" (phụ âm gi kết hợp nguyên âm ơ)
    assert_eq!(simulate_typing(&mut engine, "giowf "), "giờ ");
    assert_eq!(simulate_typing(&mut engine, "baay giowf "), "bây giờ ");
}

#[test]
fn test_undo_tone_toggle() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    // "toans" -> "toán", gõ thêm 's' -> hủy dấu và giữ lại ký tự 's' thành "toans"
    assert_eq!(simulate_typing(&mut engine, "toanss "), "toans ");
    // Gõ phím 'z' xóa dấu -> "toan"
    assert_eq!(simulate_typing(&mut engine, "toansz "), "toan ");

    // Gõ "tesst" ra "test" (collapse -sst -> -st)
    assert_eq!(simulate_typing(&mut engine, "tesst"), "test");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Tesst"), "Test");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "TESST"), "TEST");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "besst"), "best");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "lisst"), "list");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cosst"), "cost");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "posst"), "post");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "musst"), "must");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "test "), "tét ");
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
fn test_quick_consonants() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        ..Default::default()
    });

    // f -> ph (fong -> phong)
    assert_eq!(simulate_typing(&mut engine, "fong "), "phong ");
    // j -> gi (ja -> gia)
    assert_eq!(simulate_typing(&mut engine, "ja "), "gia ");
    // w -> qu (wa -> qua)
    assert_eq!(simulate_typing(&mut engine, "wa "), "qua ");
    // g -> ng ở cuối từ (dag -> dang)
    assert_eq!(simulate_typing(&mut engine, "dag "), "dang ");
    // h -> nh ở cuối từ (tih -> tinh)
    assert_eq!(simulate_typing(&mut engine, "tih "), "tinh ");
    // k -> ch ở cuối từ (tik -> tich)
    assert_eq!(simulate_typing(&mut engine, "tik "), "tich ");
}

#[test]
fn test_bracket_w() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        bracket_w: true,
        ..Default::default()
    });

    // [ -> ư, ] -> ơ
    assert_eq!(simulate_typing(&mut engine, "t[ "), "tư ");
    assert_eq!(simulate_typing(&mut engine, "t] "), "tơ ");
}

#[test]
fn test_macro_expansion() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        ..Default::default()
    });

    engine.macro_table.insert("ko", "không");
    engine.macro_table.insert("dc", "được");

    // "ko " -> "không "
    assert_eq!(simulate_typing(&mut engine, "ko "), "không ");
    // Case matching: "Ko " -> "Không "
    assert_eq!(simulate_typing(&mut engine, "Ko "), "Không ");
    // Case matching: "KO " -> "KHÔNG "
    assert_eq!(simulate_typing(&mut engine, "KO "), "KHÔNG ");
    // "dc " -> "được "
    assert_eq!(simulate_typing(&mut engine, "dc "), "được ");
}

#[test]
fn test_macro_table_file_persistence() {
    use kora::engine::macro_table::MacroTable;

    let mut table = MacroTable::new();
    table.insert("ko", "không");
    table.insert("dc", "được");
    table.insert("vn", "Việt Nam");

    let serialized = table.save_to_str();
    assert!(serialized.contains("ko:không"));
    assert!(serialized.contains("dc:được"));
    assert!(serialized.contains("vn:Việt Nam"));

    let mut loaded = MacroTable::new();
    loaded.load_from_str(&serialized);
    assert_eq!(loaded.lookup("ko").unwrap(), "không");
    assert_eq!(loaded.lookup("dc").unwrap(), "được");
    assert_eq!(loaded.lookup("vn").unwrap(), "Việt Nam");

    // Test removing
    loaded.remove("ko");
    assert!(loaded.lookup("ko").is_none());
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
fn test_backspace_across_space_restore() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        remember_history_across_space: true,
        ..Default::default()
    });

    let mut screen = String::new();
    // Type "xin "
    for ch in "xin ".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "xin ");

    // Backspace once -> erases space and restores "xin" into active buffer
    screen.pop();
    engine.on_backspace();

    // Now type 's' -> should add acute tone to 'xin' -> "xín"
    match engine.on_key('s', false, false) {
        EngineAction::Passthrough => screen.push('s'),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                screen.pop();
            }
            screen.push_str(&output);
        }
        EngineAction::Consume => {}
    }
    assert_eq!(screen, "xín");
}

#[test]
fn test_tone_typed_immediately_after_vowel() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // Gõ dấu ngay sau nguyên âm: "tieesng" -> "tiếng" (không bao giờ ra "tíêng")
    assert_eq!(simulate_typing(&mut engine, "tieesng "), "tiếng ");

    // Gõ dấu nặng ngay sau nguyên âm: "vieejt" -> "việt" (không bao giờ ra "vịêt")
    assert_eq!(simulate_typing(&mut engine, "vieejt "), "việt ");

    // Gõ cả câu với thói quen gõ dấu ngay sau nguyên âm
    assert_eq!(
        simulate_typing(&mut engine, "Tooi gox tieesng vieejt "),
        "Tôi gõ tiếng việt "
    );

    // All-caps với tieesng vieejt
    assert_eq!(
        simulate_typing(&mut engine, "TOOI GOX TIEESNG VIEETJ "),
        "TÔI GÕ TIẾNG VIỆT "
    );
}

#[test]
fn test_quick_consonant_casing() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        ..Default::default()
    });

    // Title case: Wa -> Qua (chữ u viết thường)
    assert_eq!(simulate_typing(&mut engine, "Wa "), "Qua ");

    // All caps: WA -> QUA (chữ U viết hoa)
    assert_eq!(simulate_typing(&mut engine, "WA "), "QUA ");

    // Title case: Fa -> Pha, FA -> PHA
    assert_eq!(simulate_typing(&mut engine, "Fa "), "Pha ");
    assert_eq!(simulate_typing(&mut engine, "FA "), "PHA ");

    // Title case: Ja -> Gia, JA -> GIA
    assert_eq!(simulate_typing(&mut engine, "Ja "), "Gia ");
    assert_eq!(simulate_typing(&mut engine, "JA "), "GIA ");
}

#[test]
fn test_debug_logger() {
    let test_log_file = "test_debug.log";
    if std::path::Path::new(test_log_file).exists() {
        let _ = std::fs::remove_file(test_log_file);
    }

    let mut engine = VietnameseEngine::new(EngineConfig {
        debug: true,
        debug_file_path: Some(test_log_file.to_string()),
        ..Default::default()
    });

    simulate_typing(&mut engine, "tieengs ");
    let logs = engine.debug_log();
    assert!(!logs.is_empty());
    assert!(logs.iter().any(|l| l.contains("[DBG][KEY]")));
    assert!(logs.iter().any(|l| l.contains("[DBG][BREAK]")));

    // Allow background worker thread to write and flush
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Verify file exists on disk and contains log content
    assert!(std::path::Path::new(test_log_file).exists());
    let file_content = std::fs::read_to_string(test_log_file).unwrap();
    assert!(file_content.contains("[DBG][KEY]"));
    assert!(file_content.contains("tieengs"));

    // Cleanup
    let _ = std::fs::remove_file(test_log_file);
}

#[test]
fn test_free_mark_circumflex_and_d_stroke() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // ddaua -> đâu (người dùng gõ 'a' ở cuối từ để biến 'a' thành 'â')
    assert_eq!(simulate_typing(&mut engine, "ddaua "), "đâu ");

    // maya -> mây
    assert_eq!(simulate_typing(&mut engine, "maya "), "mây ");

    // toio -> tôi
    assert_eq!(simulate_typing(&mut engine, "toio "), "tôi ");

    // dudowjc -> được (gõ 'd' sau nguyên âm 'u' kích hoạt đ-stroke)
    assert_eq!(simulate_typing(&mut engine, "dudowjc "), "được ");
}

#[test]
fn test_english_words_and_quick_end() {
    // 1. Mặc định use_macro = false: từ tiếng Anh "log", "tag", "bug" không bị biến thành "long", "tang", "bung"
    let mut engine_default = VietnameseEngine::new(EngineConfig {
        use_macro: false,
        ..Default::default()
    });
    assert_eq!(simulate_typing(&mut engine_default, "log "), "log ");
    assert_eq!(simulate_typing(&mut engine_default, "tag "), "tag ");
    assert_eq!(simulate_typing(&mut engine_default, "bug "), "bug ");

    // 2. Khi bật use_macro = true: "dag " -> "dang ", "dág " -> "dáng "
    let mut engine_quick = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        ..Default::default()
    });
    assert_eq!(simulate_typing(&mut engine_quick, "dag "), "dang ");
    assert_eq!(simulate_typing(&mut engine_quick, "dág "), "dáng ");
    // Ký tự đơn đứng một mình không bị biến đổi:
    assert_eq!(simulate_typing(&mut engine_quick, "g "), "g ");
    assert_eq!(simulate_typing(&mut engine_quick, "h "), "h ");
}

#[test]
fn test_w_horn_undo_and_coda_guard() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // chuw -> chư
    assert_eq!(simulate_typing(&mut engine, "chuw "), "chư ");

    // chuwx -> chữ
    assert_eq!(simulate_typing(&mut engine, "chuwx "), "chữ ");

    // chuww -> gõ 'w' lần thứ 2 hoàn tác sừng ư và khôi phục ký tự 'w' -> chuw
    assert_eq!(simulate_typing(&mut engine, "chuww "), "chuw ");
    // chuwz -> gõ 'z' xóa dấu sừng -> chu
    assert_eq!(simulate_typing(&mut engine, "chuwz "), "chu ");

    // 'w' không bao giờ trở thành phụ âm cuối Coda (không tạo ra chuwwx hay chưw)
    // Nếu gõ 'w' không hợp lệ sau phụ âm đầu đã có sừng, nó chuyển thành passthrough an toàn
    let mut eng2 = VietnameseEngine::new(EngineConfig::default());
    assert_eq!(simulate_typing(&mut eng2, "chuw "), "chư ");
}

#[test]
fn test_backspace_word_editing() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut screen = String::new();

    // Gõ "laafn" -> "lần"
    for ch in "laafn".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "lần");

    // Backspace 1: xóa 'n' -> màn hình còn "lầ"
    screen.pop();
    engine.on_backspace();
    assert_eq!(screen, "lầ");

    // Backspace 2: xóa 'ầ' -> màn hình còn "l"
    screen.pop();
    engine.on_backspace();
    assert_eq!(screen, "l");

    // Gõ tiếp 'a', 'j', 'i', ' ' -> màn hình phải ra đúng "lại "
    for ch in "aji ".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "lại ");
}

#[test]
fn test_backspace_typo_recovery_from_passthrough() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut screen = String::new();

    // User gõ nhầm: 't', 'j', 'e', 'e' -> "tjee"
    for ch in "tjee".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "tjee");

    // Backspace 3 lần xóa 'e', 'e', 'j' -> màn hình còn lại "t"
    for _ in 0..3 {
        screen.pop();
        engine.on_backspace();
    }
    assert_eq!(screen, "t");

    // Bây giờ người dùng gõ tiếp "hees " -> phải ra đúng "thế " (thoát khỏi Passthrough thành công!)
    for ch in "hees ".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "thế ");
}

#[test]
fn test_backspace_on_error_does_not_produce_eer() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut screen = String::new();

    let type_key = |eng: &mut VietnameseEngine, ch: char, scr: &mut String| match eng
        .on_key(ch, false, false)
    {
        EngineAction::Passthrough => scr.push(ch),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                scr.pop();
            }
            scr.push_str(&output);
        }
        EngineAction::Consume => {}
    };

    let backspace = |eng: &mut VietnameseEngine, scr: &mut String| {
        scr.pop();
        eng.on_backspace();
    };

    // 1. Gõ "errror" -> ra "error" (gõ 3 chữ 'r' để hủy dấu và lấy chữ 'rr' theo Unikey)
    for ch in "errror".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "error");

    // 2. Người dùng xóa "or" đi (2 lần backspace)
    backspace(&mut engine, &mut screen); // xóa 'r' -> còn "erro"
    assert_eq!(screen, "erro");

    backspace(&mut engine, &mut screen); // xóa 'o' -> còn "err"
    assert_eq!(screen, "err");

    // 3. Người dùng gõ tiếp "or" -> ra chuẩn "error", TUYỆT ĐỐI KHÔNG ĐƯỢC THÀNH "eer"!
    type_key(&mut engine, 'o', &mut screen);
    assert_eq!(screen, "erro");
    type_key(&mut engine, 'r', &mut screen);
    assert_eq!(screen, "error");
}

#[test]
fn test_duoc_typing_variations() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. ddwojc -> được (dd -> đ, w -> ư, o -> ươ, j -> đượ, c -> được)
    assert_eq!(simulate_typing(&mut engine, "ddwojc "), "được ");

    // 2. duowjc -> được (d -> d, u -> du, o -> duo, w -> dươ, j -> dượ, c -> dược)
    assert_eq!(simulate_typing(&mut engine, "duowjc "), "dược ");

    // 3. ddwocj -> được
    assert_eq!(simulate_typing(&mut engine, "ddwocj "), "được ");

    // 4. dduowjc -> được
    assert_eq!(simulate_typing(&mut engine, "dduowjc "), "được ");

    // 5. giari -> giải (tone on 'a', not on 'i' -> "giải", never "giaỉ")
    assert_eq!(simulate_typing(&mut engine, "giari "), "giải ");
}

#[test]
fn test_restore_on_wrong_spelling_english_words() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. Phục hồi từ tiếng Anh bắt đầu bằng phụ âm không có trong tiếng Việt (f, j, w, z)
    // "fix" gõ Telex tạo "fĩ" nhưng khi bấm Space sẽ tự động khôi phục lại thành "fix "
    assert_eq!(simulate_typing(&mut engine, "fix "), "fix ");
    assert_eq!(simulate_typing(&mut engine, "Fix "), "Fix ");
    assert_eq!(simulate_typing(&mut engine, "FIX "), "FIX ");

    assert_eq!(simulate_typing(&mut engine, "fax "), "fax ");
    assert_eq!(simulate_typing(&mut engine, "fox "), "fox ");
    assert_eq!(simulate_typing(&mut engine, "fit "), "fit ");
    assert_eq!(simulate_typing(&mut engine, "faa "), "faa ");

    // 2. Phục hồi các từ tiếng Anh nhiều âm tiết / bẫy dấu Telex phổ biến
    assert_eq!(simulate_typing(&mut engine, "first "), "first ");
    assert_eq!(simulate_typing(&mut engine, "server "), "server ");
    assert_eq!(simulate_typing(&mut engine, "user "), "user ");
    assert_eq!(simulate_typing(&mut engine, "word "), "word ");
    assert_eq!(simulate_typing(&mut engine, "win "), "win ");

    // 3. Các từ tiếng Việt hợp lệ kết thúc bằng phím 'x' (dấu ngã) KHÔNG bị khôi phục
    assert_eq!(simulate_typing(&mut engine, "max "), "mã ");
    assert_eq!(simulate_typing(&mut engine, "box "), "bõ ");
    assert_eq!(simulate_typing(&mut engine, "mix "), "mĩ ");
    assert_eq!(simulate_typing(&mut engine, "six "), "sĩ ");
    assert_eq!(simulate_typing(&mut engine, "tax "), "tã ");
    assert_eq!(simulate_typing(&mut engine, "sex "), "sẽ ");

    // 4. Các từ tiếng Việt chuẩn vẫn hoạt động hoàn hảo
    assert_eq!(simulate_typing(&mut engine, "tooi "), "tôi ");
    assert_eq!(simulate_typing(&mut engine, "vieetj "), "việt ");
    assert_eq!(simulate_typing(&mut engine, "ddwojc "), "được ");
}

#[test]
fn test_newline_clears_history_and_prevents_ghost_restoration() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        restore_on_wrong_spelling: true,
        remember_history_across_space: true,
        ..Default::default()
    });

    let mut screen = String::new();

    // 1. Gõ "khoong\r" -> ra "không\r"
    for ch in "khoong\r".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "không\r");

    // 2. Gõ "fix " -> invalid từ tiếng Việt tự phục hồi thành "fix "
    for ch in "fix ".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "không\rfix ");

    // 3. Backspace xóa dấu cách:
    screen.pop();
    engine.on_backspace();
    assert_eq!(screen, "không\rfix");

    // 4. Backspace 3 lần xóa 'x', 'i', 'f':
    for _ in 0..3 {
        screen.pop();
        engine.on_backspace();
    }
    assert_eq!(screen, "không\r");
    assert!(engine.buffer.is_empty());

    // 5. Backspace thêm lần nữa khi buffer rỗng trên dòng mới:
    // KHÔNG được phục hồi "không" từ dòng trước!
    screen.pop(); // Giả lập xóa '\r' trên editor
    engine.on_backspace();
    assert!(engine.buffer.is_empty());

    // 6. Gõ "gox " -> phải ra "gõ ", tuyệt đối không bị dính thành "khônggox "
    for ch in "gox ".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "khônggõ ");
}

#[test]
fn test_tone_placement_before_circumflex_double_vowel() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. Gõ dấu nặng 'j' trước khi gõ 'e' thứ 2: "viejet " -> "việt "
    assert_eq!(simulate_typing(&mut engine, "viejet "), "việt ");

    // 2. Gõ dấu sắc 's' trước khi gõ 'e' thứ 2: "tieseng " -> "tiếng "
    assert_eq!(simulate_typing(&mut engine, "tieseng "), "tiếng ");

    // 3. Gõ 'v' 'i' 'e' 'j' 'e' 't' tuần tự
    assert_eq!(simulate_typing(&mut engine, "viejet"), "việt");
}

#[test]
fn test_undo_toggle_backspace_recovery() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut screen = String::new();

    // 1. Gõ 'b', 'a' -> "ba"
    for ch in "ba".chars() {
        match engine.on_key(ch, false, false) {
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
    assert_eq!(screen, "ba");

    // 2. Gõ 'j' -> "bạ"
    match engine.on_key('j', false, false) {
        EngineAction::Passthrough => screen.push('j'),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                screen.pop();
            }
            screen.push_str(&output);
        }
        EngineAction::Consume => {}
    }
    assert_eq!(screen, "bạ");

    // 3. Gõ 'j' lần nữa (Undo toggle) -> "baj"
    match engine.on_key('j', false, false) {
        EngineAction::Passthrough => screen.push('j'),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                screen.pop();
            }
            screen.push_str(&output);
        }
        EngineAction::Consume => {}
    }
    assert_eq!(screen, "baj");

    // 4. Backspace xóa chữ 'j' vừa phục hồi -> màn hình còn "ba", buffer phải quay về "ba"
    screen.pop();
    engine.on_backspace();
    assert_eq!(screen, "ba");

    // 5. Gõ lại 'j' -> phải ra "bạ", KHÔNG được lặp lại undo toggle thành "baj"
    match engine.on_key('j', false, false) {
        EngineAction::Passthrough => screen.push('j'),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                screen.pop();
            }
            screen.push_str(&output);
        }
        EngineAction::Consume => {}
    }
    assert_eq!(screen, "bạ");

    // 6. Gõ 'n' -> phải ra "bạn"
    match engine.on_key('n', false, false) {
        EngineAction::Passthrough => screen.push('n'),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                screen.pop();
            }
            screen.push_str(&output);
        }
        EngineAction::Consume => {}
    }
    assert_eq!(screen, "bạn");
}

#[test]
fn test_free_mark_circumflex_across_coda() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. "vanas " -> "vấn "
    assert_eq!(simulate_typing(&mut engine, "vanas "), "vấn ");

    // 2. "vana " -> "vân "
    assert_eq!(simulate_typing(&mut engine, "vana "), "vân ");

    // 3. "congo " -> "công "
    assert_eq!(simulate_typing(&mut engine, "congo "), "công ");

    // 4. "dene " -> "dên ", "ddene " -> "đên "
    assert_eq!(simulate_typing(&mut engine, "dene "), "dên ");
    assert_eq!(simulate_typing(&mut engine, "ddene "), "đên ");

    // 5. "liene" -> "liên" (Free mark circumflex on 'ie' + coda)
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "liene"), "liên");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "liene "), "liên ");
    assert_eq!(simulate_typing(&mut engine, "tiense "), "tiến ");
    assert_eq!(simulate_typing(&mut engine, "kiense "), "kiến ");
    assert_eq!(simulate_typing(&mut engine, "yense "), "yến ");
    assert_eq!(simulate_typing(&mut engine, "quene "), "quên ");
    assert_eq!(simulate_typing(&mut engine, "khuyene "), "khuyên ");
    assert_eq!(simulate_typing(&mut engine, "chuyense "), "chuyến ");
    assert_eq!(simulate_typing(&mut engine, "nguyenxe "), "nguyễn ");

    // 6. Free mark placement in any order for "được":
    assert_eq!(simulate_typing(&mut engine, "dduocjw "), "được ");
    assert_eq!(simulate_typing(&mut engine, "duocjwd "), "được ");
    assert_eq!(simulate_typing(&mut engine, "duocwdj "), "được ");
    assert_eq!(simulate_typing(&mut engine, "duocw "), "dươc ");
    assert_eq!(simulate_typing(&mut engine, "duocjw "), "dược ");
    assert_eq!(simulate_typing(&mut engine, "duongw "), "dương ");
    assert_eq!(simulate_typing(&mut engine, "dduongws "), "đướng ");

    // 7. Free mark circumflex across stop codas (c, ch, p, t):
    // "xepes" -> "xếp", "bepes" -> "bếp", "cotos" -> "cốt", "tetes" -> "tết"
    assert_eq!(simulate_typing(&mut engine, "xepes "), "xếp ");
    assert_eq!(simulate_typing(&mut engine, "bepes "), "bếp ");
    assert_eq!(simulate_typing(&mut engine, "cotos "), "cốt ");
    assert_eq!(simulate_typing(&mut engine, "tetes "), "tết ");
    assert_eq!(simulate_typing(&mut engine, "hetes "), "hết ");
    assert_eq!(simulate_typing(&mut engine, "metej "), "mệt ");
}

#[test]
fn test_macro_expansion_rendered_and_raw() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        ..Default::default()
    });

    // Thêm macro dạng có dấu "đc" -> "được"
    engine.macro_table.insert("đc", "được");
    // Người dùng gõ "ddc " -> "được "
    assert_eq!(simulate_typing(&mut engine, "ddc "), "được ");
    // Người dùng gõ "Đc " -> "Được "
    assert_eq!(simulate_typing(&mut engine, "DDc "), "Được ");
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
fn test_gi_and_qu_glide_coda_words() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // Case cốt lõi của người dùng: "đơn giản"
    // Gõ "dodwn gianr " -> "đơn giản "
    assert_eq!(simulate_typing(&mut engine, "dodwn gianr "), "đơn giản ");

    // Các từ ngữ với 'gi' + nguyên âm + phụ âm cuối
    assert_eq!(simulate_typing(&mut engine, "gianr "), "giản ");
    assert_eq!(simulate_typing(&mut engine, "gians "), "gián ");
    assert_eq!(simulate_typing(&mut engine, "gianf "), "giàn ");
    assert_eq!(simulate_typing(&mut engine, "gianx "), "giãn ");
    assert_eq!(simulate_typing(&mut engine, "gianj "), "giạn ");
    assert_eq!(simulate_typing(&mut engine, "giangr "), "giảng ");
    assert_eq!(simulate_typing(&mut engine, "giangs "), "giáng ");
    assert_eq!(simulate_typing(&mut engine, "gianhf "), "giành ");
    assert_eq!(simulate_typing(&mut engine, "giups "), "giúp ");
    assert_eq!(simulate_typing(&mut engine, "giumf "), "giùm ");
    assert_eq!(simulate_typing(&mut engine, "giotj "), "giọt ");
    assert_eq!(simulate_typing(&mut engine, "gieets "), "giết ");
    assert_eq!(simulate_typing(&mut engine, "gieengs "), "giếng ");
    assert_eq!(simulate_typing(&mut engine, "giuowngf "), "giường ");
    assert_eq!(simulate_typing(&mut engine, "giawcj "), "giặc ");

    // Các từ ngữ với 'qu' + nguyên âm + phụ âm cuối
    assert_eq!(simulate_typing(&mut engine, "quanr "), "quản ");
    assert_eq!(simulate_typing(&mut engine, "quans "), "quán ");
    assert_eq!(simulate_typing(&mut engine, "quangr "), "quảng ");
    assert_eq!(simulate_typing(&mut engine, "quanh "), "quanh ");
    assert_eq!(simulate_typing(&mut engine, "quen "), "quen ");
    assert_eq!(simulate_typing(&mut engine, "quetj "), "quẹt ");
    assert_eq!(simulate_typing(&mut engine, "queets "), "quết ");
    assert_eq!(simulate_typing(&mut engine, "quyeets "), "quyết ");
    assert_eq!(simulate_typing(&mut engine, "quyeenr "), "quyển ");
    assert_eq!(simulate_typing(&mut engine, "quyts "), "quýt ");
}

#[test]
fn test_linguistic_edge_cases_normalized() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        restore_on_wrong_spelling: true,
        ..Default::default()
    });

    // 1. Phím 'w' trên nguyên âm đôi: "ua" + 'w' -> "ưa" (chỉ 'u' nhận móc, 'a' KHÔNG bị biến thành 'ă')
    assert_eq!(simulate_typing(&mut engine, "muaw "), "mưa ");
    assert_eq!(simulate_typing(&mut engine, "chuaw "), "chưa ");
    assert_eq!(simulate_typing(&mut engine, "xuaw "), "xưa ");
    assert_eq!(simulate_typing(&mut engine, "cuaw "), "cưa ");

    // 2. Phím 'w' trên "uo" -> "ươ" (cả 2 cùng nhận móc)
    assert_eq!(simulate_typing(&mut engine, "muonw "), "mươn ");
    assert_eq!(simulate_typing(&mut engine, "luonw "), "lươn ");
    assert_eq!(simulate_typing(&mut engine, "muonwj "), "mượn ");
    assert_eq!(simulate_typing(&mut engine, "dduocwj "), "được ");
    assert_eq!(simulate_typing(&mut engine, "muown "), "mươn ");
    assert_eq!(simulate_typing(&mut engine, "dduowjc "), "được ");

    // 3. Phím 'w' trên "oa" -> "oă" (chỉ 'a' nhận trăng)
    assert_eq!(simulate_typing(&mut engine, "hoawcj "), "hoặc ");
    assert_eq!(simulate_typing(&mut engine, "khoanw "), "khoăn ");

    // 4. Nguyên âm đôi 'oo' (cái xoong, boong tàu) gõ bằng phím undo 'o' thứ 3 trong Telex:
    assert_eq!(simulate_typing(&mut engine, "xoong "), "xông ");
    assert_eq!(simulate_typing(&mut engine, "boong "), "bông ");
    assert_eq!(simulate_typing(&mut engine, "xooong "), "xoong ");
    assert_eq!(simulate_typing(&mut engine, "booong "), "boong ");

    // 5. Chống biến âm sai từ tiếng Anh kết thúc bằng stop coda (c, ch, p, t):
    assert_eq!(simulate_typing(&mut engine, "data "), "data ");
    assert_eq!(simulate_typing(&mut engine, "vote "), "vote ");
    assert_eq!(simulate_typing(&mut engine, "gate "), "gate ");

    // 6. Gõ tự do dấu mũ sau phụ âm cuối (Free-mark circumflex on diphthongs):
    assert_eq!(simulate_typing(&mut engine, "tiepes "), "tiếp ");
    assert_eq!(simulate_typing(&mut engine, "bietes "), "biết ");
    assert_eq!(simulate_typing(&mut engine, "viecej "), "việc ");
    assert_eq!(simulate_typing(&mut engine, "cuocoj "), "cuộc ");
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
fn test_taskbar_theme_detection() {
    use kora::ui::tray::is_windows_dark_taskbar;

    // Checks that registry query runs smoothly and returns a bool
    let is_dark = is_windows_dark_taskbar();
    println!("Windows taskbar is dark theme: {}", is_dark);
}

#[test]
fn test_optimized_png_icon_loading() {
    use kora::ui::tray::{load_icon_from_memory, safe_destroy_icon};

    static VIET_BYTES: &[u8] = include_bytes!("../assets/vi.ico");
    static ENG_BYTES: &[u8] = include_bytes!("../assets/en.ico");
    static APP_BYTES: &[u8] = include_bytes!("../assets/icon.ico");

    assert!(VIET_BYTES.len() < 3000, "vi.ico should be tiny (< 3KB)");
    assert!(ENG_BYTES.len() < 3000, "en.ico should be tiny (< 3KB)");
    assert!(APP_BYTES.len() < 10000, "icon.ico should be tiny (< 10KB)");

    for sz in [16, 20, 24, 32] {
        let h_viet = load_icon_from_memory(VIET_BYTES, sz, sz);
        assert!(
            h_viet.is_some() && h_viet.unwrap() != 0,
            "Failed to load vi.ico at size {}",
            sz
        );
        safe_destroy_icon(h_viet.unwrap());

        let h_eng = load_icon_from_memory(ENG_BYTES, sz, sz);
        assert!(
            h_eng.is_some() && h_eng.unwrap() != 0,
            "Failed to load en.ico at size {}",
            sz
        );
        safe_destroy_icon(h_eng.unwrap());
    }

    for sz in [16, 24, 32, 48, 256] {
        let h_app = load_icon_from_memory(APP_BYTES, sz, sz);
        assert!(
            h_app.is_some() && h_app.unwrap() != 0,
            "Failed to load icon.ico at size {}",
            sz
        );
        safe_destroy_icon(h_app.unwrap());
    }
}

#[test]
fn test_dynamic_debug_toggle() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        debug: false,
        ..Default::default()
    });

    simulate_typing(&mut engine, "tieengs ");
    assert!(
        engine.debug_log.is_empty(),
        "Logs should be empty when debug is disabled"
    );

    // Dynamically enable debug
    engine.config_mut().debug = true;
    simulate_typing(&mut engine, "vieetj ");
    assert!(
        !engine.debug_log.is_empty(),
        "Logs should be recorded after dynamically enabling debug"
    );

    // Dynamically disable debug
    let log_count = engine.debug_log.len();
    engine.config_mut().debug = false;
    simulate_typing(&mut engine, "nam ");
    assert_eq!(
        engine.debug_log.len(),
        log_count,
        "No new logs should be recorded when disabled again"
    );
}

#[test]
fn test_macro_master_switch_and_sub_options() {
    // 1. Tắt gõ tắt (use_macro = false), dù bật các option còn lại:
    // KHÔNG được phép hoạt động bất kỳ tính năng gõ tắt nào.
    let mut engine_disabled = VietnameseEngine::new(EngineConfig {
        use_macro: false,
        use_macro_in_english_mode: true,
        enabled: true,
        ..Default::default()
    });
    engine_disabled.macro_table.insert("ko", "không");
    engine_disabled.macro_table.insert("dc", "được");

    // Phụ âm đầu bị chặn (f không thành ph, j không thành gi):
    assert_eq!(simulate_typing(&mut engine_disabled, "fong "), "fong ");
    assert_eq!(simulate_typing(&mut engine_disabled, "ja "), "ja ");

    // Phụ âm cuối bị chặn (g không thành ng, h không thành nh, k không thành ch):
    assert_eq!(simulate_typing(&mut engine_disabled, "dag "), "dag ");
    assert_eq!(simulate_typing(&mut engine_disabled, "tih "), "tih ");
    assert_eq!(simulate_typing(&mut engine_disabled, "tik "), "tik ");

    // Bảng từ macro trong tiếng Việt bị chặn:
    assert_eq!(simulate_typing(&mut engine_disabled, "ko "), "ko ");
    assert_eq!(simulate_typing(&mut engine_disabled, "dc "), "dc ");

    // Bảng từ macro trong tiếng Anh bị chặn:
    engine_disabled.config_mut().enabled = false;
    assert_eq!(simulate_typing(&mut engine_disabled, "ko "), "ko ");
    assert_eq!(simulate_typing(&mut engine_disabled, "dc "), "dc ");

    // 2. Bật gõ tắt (use_macro = true), tắt gõ tắt trong tiếng Anh:
    let mut engine_active = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        use_macro_in_english_mode: false,
        enabled: true,
        ..Default::default()
    });
    engine_active.macro_table.insert("ko", "không");
    engine_active.macro_table.insert("dc", "được");

    // Hoạt động với danh sách từ và các loại gõ tắt trong bảng macro:
    assert_eq!(simulate_typing(&mut engine_active, "ko "), "không ");
    assert_eq!(simulate_typing(&mut engine_active, "Ko "), "Không ");
    assert_eq!(simulate_typing(&mut engine_active, "dc "), "được ");
    assert_eq!(simulate_typing(&mut engine_active, "fong "), "phong ");
    assert_eq!(simulate_typing(&mut engine_active, "dag "), "dang ");

    // Khi người dùng xóa quy tắc phụ âm khỏi bảng: quy tắc đó không hoạt động nữa!
    engine_active.macro_table.remove("f");
    assert_eq!(simulate_typing(&mut engine_active, "fong "), "fong ");
    engine_active.macro_table.remove("g");
    assert_eq!(simulate_typing(&mut engine_active, "dag "), "dag ");

    // Gõ tắt trong tiếng Anh bị chặn khi use_macro_in_english_mode = false:
    engine_active.config_mut().enabled = false;
    assert_eq!(simulate_typing(&mut engine_active, "ko "), "ko ");

    // 3. Khi bật use_macro_in_english_mode = true:
    engine_active.config_mut().use_macro_in_english_mode = true;
    assert_eq!(simulate_typing(&mut engine_active, "ko "), "không ");
}

#[test]
fn test_macro_type_customization_and_single_character_guard() {
    use kora::engine::macro_table::MacroType;

    let mut engine = VietnameseEngine::new(EngineConfig {
        use_macro: true,
        enabled: true,
        ..Default::default()
    });

    // 1. Single character guard: 'f', 'j', 'g', 'h', 'k' đứng một mình khi gõ Space KHÔNG bị biến đổi!
    assert_eq!(simulate_typing(&mut engine, "f "), "f ");
    assert_eq!(simulate_typing(&mut engine, "j "), "j ");
    assert_eq!(simulate_typing(&mut engine, "g "), "g ");
    assert_eq!(simulate_typing(&mut engine, "h "), "h ");
    assert_eq!(simulate_typing(&mut engine, "k "), "k ");

    // Trong Telex, 'w' đứng một mình ra 'ư' (quy tắc Telex chuẩn):
    assert_eq!(simulate_typing(&mut engine, "w "), "ư ");
    // Nhưng 'wa ' có nguyên âm đi sau -> thành 'qua ':
    assert_eq!(simulate_typing(&mut engine, "wa "), "qua ");

    // Với bộ gõ VNI: 'w' đứng một mình là 'w ', 'wa ' là 'qua ':
    let mut engine_vni = VietnameseEngine::new(EngineConfig {
        method: kora::engine::config::InputMethod::Vni,
        use_macro: true,
        ..Default::default()
    });
    assert_eq!(simulate_typing(&mut engine_vni, "w "), "w ");
    assert_eq!(simulate_typing(&mut engine_vni, "wa "), "qua ");

    // Cùng với biểu thức toán học hoặc code: "f(x) "
    assert_eq!(simulate_typing(&mut engine, "f(x) "), "f(x) ");

    // 2. Phụ âm đầu khi là một từ có nguyên âm theo sau:
    assert_eq!(simulate_typing(&mut engine, "fong "), "phong ");
    assert_eq!(simulate_typing(&mut engine, "fa "), "pha ");
    assert_eq!(simulate_typing(&mut engine, "Fong "), "Phong ");
    assert_eq!(simulate_typing(&mut engine, "FONG "), "PHONG ");

    // 3. Từ tiếng Anh không có nguyên âm ngay sau phụ âm đầu: 'flash', 'ftp' không bị biến đổi
    assert_eq!(simulate_typing(&mut engine, "flash "), "flash ");
    assert_eq!(simulate_typing(&mut engine, "ftp "), "ftp ");

    // 4. Phụ âm cuối:
    assert_eq!(simulate_typing(&mut engine, "dag "), "dang ");
    assert_eq!(simulate_typing(&mut engine, "Dag "), "Dang ");
    assert_eq!(simulate_typing(&mut engine, "DAG "), "DANG ");
    assert_eq!(simulate_typing(&mut engine, "dág "), "dáng ");

    // Khi trước 'g' là phụ âm ('dang'): không bị nhân đôi thành 'danng'
    assert_eq!(simulate_typing(&mut engine, "dang "), "dang ");

    // 5. Kết hợp cả phụ âm đầu và phụ âm cuối trong một từ: 'fag' -> 'phang'
    assert_eq!(simulate_typing(&mut engine, "fag "), "phang ");
    assert_eq!(simulate_typing(&mut engine, "Fag "), "Phang ");

    // 6. Xóa quy tắc gõ tắt mặc định: Người dùng xóa quy tắc 'g' -> 'ng'
    engine.macro_table.remove("g");
    // Khi này 'dag ' giữ nguyên là 'dag '
    assert_eq!(simulate_typing(&mut engine, "dag "), "dag ");
    // Nhưng quy tắc 'f' -> 'ph' vẫn hoạt động bình thường
    assert_eq!(simulate_typing(&mut engine, "fong "), "phong ");

    // 7. Thêm quy tắc tùy biến: Thêm 'z' -> 'd' loại Phụ âm đầu (StartConsonant)
    engine
        .macro_table
        .insert_typed("z", "d", MacroType::StartConsonant);
    assert_eq!(simulate_typing(&mut engine, "za "), "da ");
    assert_eq!(simulate_typing(&mut engine, "Za "), "Da ");
    assert_eq!(simulate_typing(&mut engine, "z "), "z "); // ký tự đơn đứng một mình vẫn giữ nguyên!

    // 8. Thêm quy tắc tùy biến: Thêm 'x' -> 'ch' loại Phụ âm cuối (EndConsonant)
    engine
        .macro_table
        .insert_typed("x", "ch", MacroType::EndConsonant);
    assert_eq!(simulate_typing(&mut engine, "tax "), "tach ");
    assert_eq!(simulate_typing(&mut engine, "x "), "x "); // ký tự đơn giữ nguyên!
}

#[test]
fn test_macro_type_serialization_and_roundtrip() {
    use kora::engine::macro_table::{MacroTable, MacroType};

    let mut table = MacroTable::new();
    table.insert_typed("f", "ph", MacroType::StartConsonant);
    table.insert_typed("g", "ng", MacroType::EndConsonant);
    table.insert_typed("ko", "không", MacroType::Normal);

    let saved = table.save_to_str();
    assert!(saved.contains("f:ph:start"));
    assert!(saved.contains("g:ng:end"));
    assert!(saved.contains("ko:không:normal"));

    let mut loaded = MacroTable::new();
    loaded.load_from_str(&saved);

    assert_eq!(loaded.len(), 3);
    let entry_f = loaded.get("f").unwrap();
    assert_eq!(entry_f.value, "ph");
    assert_eq!(entry_f.macro_type, MacroType::StartConsonant);

    let entry_g = loaded.get("g").unwrap();
    assert_eq!(entry_g.value, "ng");
    assert_eq!(entry_g.macro_type, MacroType::EndConsonant);

    let entry_ko = loaded.get("ko").unwrap();
    assert_eq!(entry_ko.value, "không");
    assert_eq!(entry_ko.macro_type, MacroType::Normal);
}

#[test]
fn test_special_coda_dak_lak_and_english_guard() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Telex,
        restore_on_wrong_spelling: true,
        ..Default::default()
    });

    // 1. Gõ dấu sau: DDawks Lawks -> Đắk Lắk
    assert_eq!(simulate_typing(&mut engine, "DDawks Lawks "), "Đắk Lắk ");

    // 2. Gõ dấu trước: DDawsk Lawsk -> Đắk Lắk
    assert_eq!(simulate_typing(&mut engine, "DDawsk Lawsk "), "Đắk Lắk ");

    // 3. Chữ thường: ddawsk lawsk -> đắk lắk
    assert_eq!(simulate_typing(&mut engine, "ddawsk lawsk "), "đắk lắk ");

    // 4. Không dấu: ddawk lawk -> đăk lăk
    assert_eq!(simulate_typing(&mut engine, "ddawk lawk "), "đăk lăk ");

    // 5. Viết HOA: DDAWSK LAWSK -> ĐẮK LẮK
    assert_eq!(simulate_typing(&mut engine, "DDAWSK LAWSK "), "ĐẮK LẮK ");

    // 6. Đak trong Đak Bla, Đak Đoa
    assert_eq!(simulate_typing(&mut engine, "DDak "), "Đak ");

    // 7. Bảo vệ các từ tiếng Anh kết thúc bằng -sk không bị biến thành tiếng Việt
    assert_eq!(simulate_typing(&mut engine, "task "), "task ");
    assert_eq!(simulate_typing(&mut engine, "desk "), "desk ");
    assert_eq!(simulate_typing(&mut engine, "risk "), "risk ");
    assert_eq!(simulate_typing(&mut engine, "mask "), "mask ");
    assert_eq!(simulate_typing(&mut engine, "ask "), "ask ");
    assert_eq!(simulate_typing(&mut engine, "disk "), "disk ");
}

#[test]
fn test_backspace_across_space_does_not_trap_new_words() {
    let mut engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Telex,
        remember_history_across_space: true,
        ..Default::default()
    });

    let mut screen = String::new();
    let type_str = |eng: &mut VietnameseEngine, s: &str, scr: &mut String| {
        for ch in s.chars() {
            let is_upper = ch.is_uppercase();
            match eng.on_key(ch, is_upper, false) {
                EngineAction::Passthrough => scr.push(ch),
                EngineAction::Replace { backspaces, output } => {
                    for _ in 0..backspaces {
                        scr.pop();
                    }
                    scr.push_str(&output);
                }
                EngineAction::Consume => {}
            }
        }
    };
    let press_bs = |eng: &mut VietnameseEngine, scr: &mut String| {
        scr.pop();
        eng.on_backspace();
    };

    // 1. Gõ "tooi " -> "tôi "
    type_str(&mut engine, "tooi ", &mut screen);
    assert_eq!(screen, "tôi ");

    // 2. Nhấn Backspace xóa dấu cách -> buffer phục hồi "tôi" qua dấu cách
    press_bs(&mut engine, &mut screen);
    assert_eq!(screen, "tôi");

    // 3. Người dùng gõ dấu cách rồi gõ từ mới "giups " -> phải ra chuẩn "tôi giúp "
    // Thay vì bị dính vào "tôi" thành Passthrough "tôigiups",
    // engine phải tự động tách từ mới và gõ ra chuẩn "giúp "!
    type_str(&mut engine, " giups ", &mut screen);
    assert_eq!(screen, "tôi giúp ");

    // 4. Kiểm tra kịch bản người dùng: gõ từ, xóa đi, gõ lại không bao giờ bị mất dấu
    // Xóa từ " giúp ": 6 lần Backspace (gồm dấu cách cuối, g, i, ú, p, dấu cách đầu)
    for _ in 0..6 {
        press_bs(&mut engine, &mut screen);
    }
    assert_eq!(screen, "tôi");

    // Gõ lại dấu cách và "giups " -> phải ra chuẩn "tôi giúp "
    type_str(&mut engine, " giups ", &mut screen);
    assert_eq!(screen, "tôi giúp ");
}

#[test]
fn test_smart_english_bypass_on_the_fly() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. Instant bypass for non-Vietnamese onsets (f, j, z) WITHOUT Space
    assert_eq!(simulate_typing(&mut engine, "format"), "format");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "fix"), "fix");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "fox"), "fox");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "fax"), "fax");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "first"), "first");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "file"), "file");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "json"), "json");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "zoom"), "zoom");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "zero"), "zero");
    engine.reset();

    // 2. Instant bypass for English initial consonant clusters (pr, pl, cl, cr, br, bl, fl, gl, dr, sk, sm, sn, str, spr...) WITHOUT Space
    assert_eq!(simulate_typing(&mut engine, "project"), "project");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "class"), "class");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "clear"), "clear");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "print"), "print");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "client"), "client");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "black"), "black");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "great"), "great");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "drive"), "drive");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "smart"), "smart");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "string"), "string");
    engine.reset();

    // 3. Instant rollback for English words where Telex tone/diacritic was temporarily placed on vowel
    assert_eq!(simulate_typing(&mut engine, "server"), "server");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "service"), "service");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "card"), "card");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "word"), "word");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "win"), "win");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "user"), "user");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "case"), "case");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "base"), "base");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "team"), "team");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "read"), "read");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "sound"), "sound");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "port"), "port");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "part"), "part");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "after"), "after");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "often"), "often");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "left"), "left");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "text"), "text");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "next"), "next");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "soft"), "soft");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "chart"), "chart");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "gift"), "gift");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "what"), "what");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "when"), "when");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "where"), "where");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "which"), "which");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "white"), "white");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "write"), "write");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "wrong"), "wrong");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "web"), "web");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "week"), "week");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "work"), "work");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "good"), "good");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "look"), "look");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "book"), "book");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cool"), "cool");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "tool"), "tool");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "action"), "action");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "open"), "open");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "input"), "input");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "order"), "order");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "other"), "other");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "testing"), "testing");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "listing"), "listing");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "posting"), "posting");
    engine.reset();

    // 4. TitleCase and ALL CAPS English word bypass
    assert_eq!(simulate_typing(&mut engine, "Project"), "Project");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Clear"), "Clear");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Format"), "Format");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Port"), "Port");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Text"), "Text");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Next"), "Next");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Wrong"), "Wrong");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "PROJECT"), "PROJECT");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "SERVER"), "SERVER");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "FORMAT"), "FORMAT");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "TEXT"), "TEXT");
    engine.reset();

    // 4. Combined sentence with both English words and Vietnamese words
    assert_eq!(
        simulate_typing(
            &mut engine,
            "chungs tooi ddang fix bug trong file project mowsi "
        ),
        "chúng tôi đang fix bug trong file project mới "
    );
    engine.reset();
    assert_eq!(
        simulate_typing(&mut engine, "server mowsi cos user vaf format ddepj "),
        "server mới có user và format đẹp "
    );
}

#[test]
fn test_hieuer_to_hieu() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    assert_eq!(simulate_typing(&mut engine, "hieuer"), "hiểu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "hieure"), "hiểu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "tieuer"), "tiểu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "kieues"), "kiếu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "yeuer"), "yểu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "yeue"), "yêu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "muoios"), "muối");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "nguoiwf"), "người");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "keue"), "kêu");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "passs"), "pass");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "passs "), "pass ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "passsword"), "password");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "passsword "), "password ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "errror"), "error");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "errror "), "error ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "offf "), "off ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cofffee "), "coffee ");
}

#[test]
fn test_orthography_instant_passthrough_english_words() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. Invalid Onset + Vowel combinations in Vietnamese (ce, ci, cy, ge, gy, qa, ka, ko, by, dy, ry, phy, khy):
    // "center": 'c' followed by 'e' immediately escapes to Passthrough!
    assert_eq!(simulate_typing(&mut engine, "center "), "center ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cinema "), "cinema ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "city "), "city ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cycle "), "cycle ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cell "), "cell ");
    engine.reset();
    // "get": 'g' followed by 'e' immediately escapes to Passthrough (no temporary 'gét'):
    assert_eq!(simulate_typing(&mut engine, "get "), "get ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "general "), "general ");
    engine.reset();
    // "gym": 'g' followed by 'y' immediately escapes to Passthrough (Vietnamese has no gy):
    assert_eq!(simulate_typing(&mut engine, "gym "), "gym ");
    engine.reset();
    // "by", "byte": 'b' followed by 'y' immediately escapes to Passthrough (Vietnamese has no by):
    assert_eq!(simulate_typing(&mut engine, "by "), "by ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "byte "), "byte ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "dynamic "), "dynamic ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "rhythm "), "rhythm ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "physics "), "physics ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "python "), "python ");
    engine.reset();
    // "qatar": 'q' followed by 'a' immediately escapes to Passthrough (q must precede u in Vietnamese):
    assert_eq!(simulate_typing(&mut engine, "qatar "), "qatar ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "karaoke "), "karaoke ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "korea "), "korea ");
    engine.reset();

    // 2. Vowel-Coda Constraints (oo + m/t/p, oe + m, y + coda without qu):
    assert_eq!(simulate_typing(&mut engine, "sync "), "sync ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "system "), "system ");
    engine.reset();
    // "poem": 'oe' does not accept 'm' -> immediately escapes to Passthrough:
    assert_eq!(simulate_typing(&mut engine, "poem "), "poem ");
    engine.reset();
    // "boot": trong Telex 'oo' ra 'ô', gõ 'ooo' để hoàn tác 'oo', 'oo' không nhận 't'/'m' -> Passthrough:
    assert_eq!(simulate_typing(&mut engine, "booot "), "boot ");
    engine.reset();
    // "foot": từ bắt đầu bằng phụ âm 'f' vào thẳng Passthrough nên gõ "foot" ra "foot" trực tiếp:
    assert_eq!(simulate_typing(&mut engine, "foot "), "foot ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "rooot "), "root ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "rooom "), "room ");
    engine.reset();

    // 3. Valid Vietnamese words with these consonants are completely unaffected:
    // 'k' with e, i, y:
    assert_eq!(simulate_typing(&mut engine, "kes "), "ké ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "kim "), "kim ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "kyf "), "kỳ ");
    engine.reset();
    // 'c' with a, o, u:
    assert_eq!(simulate_typing(&mut engine, "cas "), "cá ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "con "), "con ");
    engine.reset();
    // 'gh' with e, i:
    assert_eq!(simulate_typing(&mut engine, "ghes "), "ghé ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "ghi "), "ghi ");
    engine.reset();
    // 'g' with a, o, u, i (gi glide):
    assert_eq!(simulate_typing(&mut engine, "gas "), "gá ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "giof "), "giò ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "giowf "), "giờ ");
    engine.reset();
    // 'oo' with 'ng' (xoong, boong) gõ 3 chữ 'o' trong Telex để ra 'oo':
    assert_eq!(simulate_typing(&mut engine, "xooong "), "xoong ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "booong "), "boong ");
}

