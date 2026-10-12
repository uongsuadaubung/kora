use super::common::simulate_typing;
use kora::{EngineConfig, InputMethod, VietnameseEngine};

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

    // 8. Undo toggle circumflex across coda when typing modifier key again:
    // "themee" -> "theme", "memee" -> "meme", "vanaa" -> "vana", "denee" -> "dene", "congoo" -> "congo"
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "theme"), "thêm");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "themee"), "theme");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "themee "), "theme ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "Themee"), "Theme");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "THEMEE"), "THEME");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "memee"), "meme");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "vanaa"), "vana");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "denee"), "dene");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "congoo"), "congo");
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
fn test_luuw_uu_horn_typing() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());

    // 1. Gõ "luuw" -> "lưu"
    assert_eq!(simulate_typing(&mut engine, "luuw "), "lưu ");
    engine.reset();

    // 2. Gõ các thanh điệu sau "luuw":
    assert_eq!(simulate_typing(&mut engine, "luuws "), "lứu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "luuwf "), "lừu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "luuwr "), "lửu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "luuwx "), "lữu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "luuwj "), "lựu ");
    engine.reset();

    // 3. Hoàn tác Telex khi gõ 'w' lần thứ 2: luuww -> luuw
    assert_eq!(simulate_typing(&mut engine, "luuww "), "luuw ");
    engine.reset();

    // 4. Các từ chứa "ưu" thông dụng khác:
    assert_eq!(simulate_typing(&mut engine, "muuw "), "mưu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "huuw "), "hưu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cuuws "), "cứu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "cuuwf "), "cừu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "suuw "), "sưu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "tuuwr "), "tửu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "uuw "), "ưu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "uuws "), "ứu ");
    engine.reset();

    // 5. Casing preservation:
    assert_eq!(simulate_typing(&mut engine, "Luuw "), "Lưu ");
    engine.reset();
    assert_eq!(simulate_typing(&mut engine, "LUUW "), "LƯU ");
    engine.reset();

    // 6. Gõ "luwu" (cách gõ thứ 2) vẫn hoạt động chính xác:
    assert_eq!(simulate_typing(&mut engine, "luwu "), "lưu ");
    engine.reset();

    // 7. Gõ theo phương thức VNI:
    let mut vni_engine = VietnameseEngine::new(EngineConfig {
        method: InputMethod::Vni,
        ..Default::default()
    });
    assert_eq!(simulate_typing(&mut vni_engine, "luu7 "), "lưu ");
    vni_engine.reset();
    assert_eq!(simulate_typing(&mut vni_engine, "lu7u "), "lưu ");
}

