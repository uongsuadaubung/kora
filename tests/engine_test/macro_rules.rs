use super::common::simulate_typing;
use kora::{EngineConfig, VietnameseEngine};

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

