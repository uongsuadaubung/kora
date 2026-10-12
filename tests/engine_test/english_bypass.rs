use super::common::simulate_typing;
use kora::{EngineConfig, InputMethod, VietnameseEngine};

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

    // 5. Combined sentence with both English words and Vietnamese words
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

