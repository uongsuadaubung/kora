use super::common::{backspace, type_key};
use kora::{EngineAction, EngineConfig, InputMethod, VietnameseEngine};

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
    type_str(&mut engine, " giups ", &mut screen);
    assert_eq!(screen, "tôi giúp ");

    // 4. Kiểm tra kịch bản người dùng: gõ từ, xóa đi, gõ lại không bao giờ bị mất dấu
    for _ in 0..6 {
        press_bs(&mut engine, &mut screen);
    }
    assert_eq!(screen, "tôi");

    // Gõ lại dấu cách và "giups " -> phải ra chuẩn "tôi giúp "
    type_str(&mut engine, " giups ", &mut screen);
    assert_eq!(screen, "tôi giúp ");
}

#[test]
fn test_completed_coda_preservation_and_backspace_recovery() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut screen = String::new();

    // 1. Gõ "hieejn" -> ra chuẩn "hiện"
    for ch in "hieejn".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "hiện");

    // 2. Gõ thêm phím 'e' (hieejne)
    // - Không bị undo dấu circumflex biến dạng thành "hịene"
    // - Không bị rollback thành raw keys "hieejne"
    // - Xuất ra "hiệne" (giống cơ chế bàn phím điện thoại thông minh)
    type_key(&mut engine, 'e', &mut screen);
    assert_eq!(screen, "hiệne");

    // 3. Người dùng bấm Backspace để xóa ký tự 'e' vừa gõ nhầm
    // - Phục hồi ngay lập tức thành "hiện"
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "hiện");

    // 4. Gõ tiếp phím Space -> ra chuẩn "hiện "
    type_key(&mut engine, ' ', &mut screen);
    assert_eq!(screen, "hiện ");

    // Reset cho test case tiếp theo
    engine.reset();
    screen.clear();

    // 5. Kiểm tra trường hợp khác: "tieengse" -> "tiếnge", backspace -> "tiếng"
    for ch in "tieengse".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "tiếnge");

    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "tiếng");

    type_key(&mut engine, ' ', &mut screen);
    assert_eq!(screen, "tiếng ");

    // Reset cho test case tiếp theo
    engine.reset();
    screen.clear();

    // 6. Kiểm tra lặp lại nhiều lần gõ nhầm và xóa:
    // hiện -> hiệne -> xóa -> hiện -> hiệne -> xóa -> hiện
    for ch in "hieejn".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "hiện");

    type_key(&mut engine, 'e', &mut screen);
    assert_eq!(screen, "hiệne");

    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "hiện");

    type_key(&mut engine, 'e', &mut screen);
    assert_eq!(screen, "hiệne");

    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "hiện");

    // 7. Sau khi phục hồi "hiện", gõ tiếp phụ âm 'g' để tạo "hiệng" (n + g = ng)
    type_key(&mut engine, 'g', &mut screen);
    assert_eq!(screen, "hiệng");

    type_key(&mut engine, ' ', &mut screen);
    assert_eq!(screen, "hiệng ");

    // Reset cho test case tiếp theo
    engine.reset();
    screen.clear();

    // 8. Kiểm tra cho các nguyên âm có dấu mũ khác:
    // "vaansa" -> "vấna", backspace -> "vấn"
    for ch in "vaansa".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "vấna");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "vấn");

    // "coongo" -> "côngo", backspace -> "công"
    engine.reset();
    screen.clear();
    for ch in "coongo".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "côngo");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "công");

    // 9. Dấu móc (Horn): "ddwojcw" -> "đượcw", backspace -> "được"
    engine.reset();
    screen.clear();
    for ch in "ddwojcw".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "đượcw");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "được");

    // 10. Dấu trăng (Breve): "mawstw" -> "mắtw", backspace -> "mắt"
    engine.reset();
    screen.clear();
    for ch in "mawstw".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "mắtw");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "mắt");

    // 11. Chữ Đ (D-stroke): "ddaastd" -> "đấtd", backspace -> "đất"
    engine.reset();
    screen.clear();
    for ch in "ddaastd".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "đấtd");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "đất");

    // 12. Phương thức VNI:
    let mut vni_engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Vni,
        ..Default::default()
    });
    screen.clear();

    // VNI 6 (dấu mũ): "hie65n" -> "hiện", gõ thêm "6" -> "hiện6", backspace -> "hiện"
    for ch in "hie65n".chars() {
        type_key(&mut vni_engine, ch, &mut screen);
    }
    assert_eq!(screen, "hiện");
    type_key(&mut vni_engine, '6', &mut screen);
    assert_eq!(screen, "hiện6");
    backspace(&mut vni_engine, &mut screen);
    assert_eq!(screen, "hiện");

    // VNI 7 (dấu móc): "d9uo7c5" -> "được", gõ thêm "7" -> "được7", backspace -> "được"
    vni_engine.reset();
    screen.clear();
    for ch in "d9uo7c5".chars() {
        type_key(&mut vni_engine, ch, &mut screen);
    }
    assert_eq!(screen, "được");
    type_key(&mut vni_engine, '7', &mut screen);
    assert_eq!(screen, "được7");
    backspace(&mut vni_engine, &mut screen);
    assert_eq!(screen, "được");

    // VNI 8 (dấu trăng): "ma81t" -> "mắt", gõ thêm "8" -> "mắt8", backspace -> "mắt"
    vni_engine.reset();
    screen.clear();
    for ch in "ma81t".chars() {
        type_key(&mut vni_engine, ch, &mut screen);
    }
    assert_eq!(screen, "mắt");
    type_key(&mut vni_engine, '8', &mut screen);
    assert_eq!(screen, "mắt8");
    backspace(&mut vni_engine, &mut screen);
    assert_eq!(screen, "mắt");

    // VNI 9 (chữ Đ): "d9a61t" -> "đất", gõ thêm "9" -> "đất9", backspace -> "đất"
    vni_engine.reset();
    screen.clear();
    for ch in "d9a61t".chars() {
        type_key(&mut vni_engine, ch, &mut screen);
    }
    assert_eq!(screen, "đất");
    type_key(&mut vni_engine, '9', &mut screen);
    assert_eq!(screen, "đất9");
    backspace(&mut vni_engine, &mut screen);
    assert_eq!(screen, "đất");
}

#[test]
fn test_completed_syllable_preservation_and_backspace_recovery() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut screen = String::new();

    // 1. Nucleus với nguyên âm đôi có dấu: "naaus" -> "nấu", gõ thêm 'c' -> "nấuc" (KHÔNG bị bung thô thành "naasuc")
    for ch in "naaus".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "nấu");
    type_key(&mut engine, 'c', &mut screen);
    assert_eq!(screen, "nấuc");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "nấu");

    // 2. Tiếp tục backspace từng bước trong từ "nấu" sau khi phục hồi:
    // "nấu" gồm ['n', 'ấ', 'u']:
    // Xóa 'u' -> màn hình còn "nấ"
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "nấ");
    // Xóa 'ấ' -> màn hình còn "n"
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "n");
    // Xóa 'n' -> màn hình rỗng
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "");

    // 3. Coda với dấu nặng: "bajn" -> "bạn", gõ thêm 'k' -> "bạnk", backspace -> "bạn"
    engine.reset();
    screen.clear();
    for ch in "bajn".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "bạn");
    type_key(&mut engine, 'k', &mut screen);
    assert_eq!(screen, "bạnk");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "bạn");

    // 4. "vieetj" -> "việt", gõ thêm 'm' -> "việtm", backspace -> "việt"
    engine.reset();
    screen.clear();
    for ch in "vieetj".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "việt");
    type_key(&mut engine, 'm', &mut screen);
    assert_eq!(screen, "việtm");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "việt");

    // 5. "dduowjc" -> "được", gõ thêm 'k' -> "đượck", backspace -> "được"
    engine.reset();
    screen.clear();
    for ch in "dduowjc".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "được");
    type_key(&mut engine, 'k', &mut screen);
    assert_eq!(screen, "đượck");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "được");

    // 6. Nguyên âm đôi plain có dấu: "mais" -> "mái", gõ thêm 'k' -> "máik", backspace -> "mái"
    engine.reset();
    screen.clear();
    for ch in "mais".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "mái");
    type_key(&mut engine, 'k', &mut screen);
    assert_eq!(screen, "máik");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "mái");

    // 7. Gõ 2 ký tự thừa liên tiếp: "naaus" -> "nấu" + 'c' + 'k' -> "nấuck", xóa 2 lần -> "nấu"
    engine.reset();
    screen.clear();
    for ch in "naaus".chars() {
        type_key(&mut engine, ch, &mut screen);
    }
    assert_eq!(screen, "nấu");
    type_key(&mut engine, 'c', &mut screen);
    type_key(&mut engine, 'k', &mut screen);
    assert_eq!(screen, "nấuck");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "nấuc");
    backspace(&mut engine, &mut screen);
    assert_eq!(screen, "nấu");
}

