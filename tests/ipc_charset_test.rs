use kora::engine::config::InputMethod;
use kora::engine::profile::{AppProfile, CharsetMode, Child};
use kora::platform::ipc::{
    handle_ipc_command, parse_http_request, parse_query, start_ipc_server_on_port, url_decode,
};
use kora::vietnamese::{Charset, detect_charset_from_font};
use kora::{EngineConfig, VietnameseEngine};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[test]
fn test_detect_charset_from_font_names() {
    // TCVN3 (starts with dot or contains vn/tc keywords)
    assert_eq!(detect_charset_from_font(".VnTime"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font(".VnTimeH"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font(".VnArial"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font(".vntime"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font("vntime"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font("VN_ARIAL.SHX"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font("vntxt.shx"), Charset::Tcvn3);
    assert_eq!(detect_charset_from_font("tc.shx"), Charset::Tcvn3);

    // Unicode (Standard true type, OpenType, and standard CAD fonts)
    assert_eq!(detect_charset_from_font("Arial"), Charset::Unicode);
    assert_eq!(detect_charset_from_font("Times New Roman"), Charset::Unicode);
    assert_eq!(detect_charset_from_font("Calibri"), Charset::Unicode);
    assert_eq!(detect_charset_from_font("Tahoma"), Charset::Unicode);
    assert_eq!(detect_charset_from_font("txt.shx"), Charset::Unicode);
    assert_eq!(detect_charset_from_font("romans.shx"), Charset::Unicode);
    assert_eq!(detect_charset_from_font(""), Charset::Unicode);
}

#[test]
fn test_update_charset_by_process_and_font_auto_create_and_isolation() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    assert_eq!(engine.effective_config().charset, Charset::Unicode);

    // Pre-create an unrelated profile for Word
    let mut word_profile = AppProfile::<Child>::new_child("word", "Word", "winword.exe", "📝");
    word_profile.charset = CharsetMode::Specific(Charset::Unicode);
    engine.profile_manager.add_profile(word_profile);

    // 1. AutoCAD sends .VnTime -> auto-creates acad profile with TCVN3
    let (is_active, charset) =
        engine.update_charset_by_process_and_font("acad.exe", ".VnTime");
    assert_eq!(charset, Charset::Tcvn3);
    assert!(!is_active); // Active was global

    // Check that acad profile was created and has TCVN3
    let acad_profile = engine
        .profile_manager
        .find_profile_by_process("acad.exe")
        .expect("acad profile should exist");
    assert_eq!(
        acad_profile.charset,
        CharsetMode::Specific(Charset::Tcvn3)
    );

    // Check isolation: Global profile and Word profile must NOT be affected
    assert_eq!(engine.config().charset, Charset::Unicode);
    let word_profile = engine
        .profile_manager
        .find_profile_by_process("winword.exe")
        .expect("word profile should exist");
    assert_eq!(
        word_profile.charset,
        CharsetMode::Specific(Charset::Unicode)
    );

    // 2. Now user focuses AutoCAD window
    let (changed, active_id) = engine.switch_profile_by_process("acad.exe");
    assert!(changed);
    assert_eq!(active_id, "acad");
    assert_eq!(engine.effective_config().charset, Charset::Tcvn3);

    // 3. While AutoCAD is active, user selects an Arial text in AutoCAD -> dynamic switch
    let (is_active, charset) = engine.update_charset_by_process_and_font("acad.exe", "Arial");
    assert_eq!(charset, Charset::Unicode);
    assert!(is_active); // Was active, so refreshed effective config
    assert_eq!(engine.effective_config().charset, Charset::Unicode);

    // 4. User selects .VnArial text in AutoCAD -> dynamic switch back to TCVN3
    let (is_active, charset) =
        engine.update_charset_by_process_and_font("acad.exe", ".VnArial");
    assert_eq!(charset, Charset::Tcvn3);
    assert!(is_active);
    assert_eq!(engine.effective_config().charset, Charset::Tcvn3);

    // 5. User switches back to Word -> Word is still Unicode
    let (changed, active_id) = engine.switch_profile_by_process("winword.exe");
    assert!(changed);
    assert_eq!(active_id, "word");
    assert_eq!(engine.effective_config().charset, Charset::Unicode);
}

#[test]
fn test_engine_apply_config_update_global_and_profile_isolation() {
    let mut engine = VietnameseEngine::new(EngineConfig::default());
    assert_eq!(engine.config().method, InputMethod::Telex);
    assert!(!engine.config().sound_enabled);
    assert!(!engine.config().use_macro);

    // 1. Global update: change method to VNI, sound to true, use_macro to true (no name)
    let mut global_params = HashMap::new();
    global_params.insert("method".to_string(), "vni".to_string());
    global_params.insert("sound".to_string(), "true".to_string());
    global_params.insert("sound_volume".to_string(), "85".to_string());
    global_params.insert("sound_profile".to_string(), "cherry_blue".to_string());
    global_params.insert("use_macro".to_string(), "true".to_string());

    let (is_active, effective) = engine.apply_config_update(None, &global_params);
    assert!(is_active);
    assert_eq!(effective.method, InputMethod::Vni);
    assert!(effective.sound_enabled);
    assert_eq!(effective.sound_volume, 85);
    assert_eq!(&*effective.sound_profile, "cherry_blue");
    assert!(effective.use_macro);

    // 2. Child profile update for VS Code (code.exe): set method to Telex specifically
    let mut code_params = HashMap::new();
    code_params.insert("method".to_string(), "telex".to_string());
    code_params.insert("sound".to_string(), "false".to_string());

    let (is_active, code_effective) = engine.apply_config_update(Some("code.exe"), &code_params);
    assert!(!is_active); // Currently global is active
    assert_eq!(code_effective.method, InputMethod::Telex);
    assert!(!code_effective.sound_enabled);

    // Verify isolation: Global still has VNI and sound enabled!
    assert_eq!(engine.config().method, InputMethod::Vni);
    assert!(engine.config().sound_enabled);

    // 3. Switch to code.exe -> effective becomes code's settings
    let (changed, active_id) = engine.switch_profile_by_process("code.exe");
    assert!(changed);
    assert_eq!(active_id, "code");
    assert_eq!(engine.effective_config().method, InputMethod::Telex);
    assert!(!engine.effective_config().sound_enabled);

    // 4. Test toggle mode
    let mut toggle_params = HashMap::new();
    toggle_params.insert("mode".to_string(), "toggle".to_string());
    let (is_active, toggled_eff) = engine.apply_config_update(Some("code.exe"), &toggle_params);
    assert!(is_active);
    assert!(!toggled_eff.enabled); // was true, now false

    let (is_active, toggled_again) = engine.apply_config_update(Some("code.exe"), &toggle_params);
    assert!(is_active);
    assert!(toggled_again.enabled); // was false, now true
}

#[test]
fn test_url_decode_and_query_parser() {
    assert_eq!(url_decode("acad.exe"), "acad.exe");
    assert_eq!(url_decode("Times%20New%20Roman"), "Times New Roman");
    assert_eq!(url_decode("%2EVnTime"), ".VnTime");
    assert_eq!(url_decode("a+b+c"), "a b c");

    let query = parse_query("name=acad.exe&font=.VnTime%20Bold&other=123");
    assert_eq!(query.get("name").unwrap(), "acad.exe");
    assert_eq!(query.get("font").unwrap(), ".VnTime Bold");
    assert_eq!(query.get("other").unwrap(), "123");
}

#[test]
fn test_parse_http_request_with_headers_and_body() {
    // GET with query string
    let raw_get = "GET /set-config?name=acad.exe&method=vni HTTP/1.1\r\nHost: 127.0.0.1:13497\r\n\r\n";
    let (method, path, params) = parse_http_request(raw_get).expect("valid get request");
    assert_eq!(method, "GET");
    assert_eq!(path, "/set-config");
    assert_eq!(params.get("name").unwrap(), "acad.exe");
    assert_eq!(params.get("method").unwrap(), "vni");

    // POST with body
    let raw_post = "POST /set-config HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/x-www-form-urlencoded\r\n\r\nname=code.exe&sound=true&sound_volume=70";
    let (method, path, params) = parse_http_request(raw_post).expect("valid post request");
    assert_eq!(method, "POST");
    assert_eq!(path, "/set-config");
    assert_eq!(params.get("name").unwrap(), "code.exe");
    assert_eq!(params.get("sound").unwrap(), "true");
    assert_eq!(params.get("sound_volume").unwrap(), "70");
}

#[test]
fn test_handle_ipc_command_all_endpoints() {
    // 1. /config with params (sets values and returns updated config)
    let mut params = HashMap::new();
    params.insert("method".to_string(), "vni".to_string());
    params.insert("font".to_string(), ".VnTime".to_string());
    let (status, body) = handle_ipc_command("GET", "/config", &params);
    assert_eq!(status, 200);
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"target\":\"global\""));
    assert!(body.contains("\"method\":\"vni\""));
    assert!(body.contains("\"charset\":\"tcvn3\""));

    // 2. /config without params (reads current config)
    let (status, body) = handle_ipc_command("GET", "/config", &HashMap::new());
    assert_eq!(status, 200);
    assert!(body.contains("\"target\":\"global\""));

    // 3. /config?mode=toggle
    let mut toggle_params = HashMap::new();
    toggle_params.insert("mode".to_string(), "toggle".to_string());
    let (status, body) = handle_ipc_command("GET", "/config", &toggle_params);
    assert_eq!(status, 200);
    assert!(body.contains("\"status\":\"ok\""));

    // 4. /profiles
    let (status, body) = handle_ipc_command("GET", "/profiles", &HashMap::new());
    assert_eq!(status, 200);
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"active_id\":\"global\""));

    // 5. Invalid path returns 404
    let (status, body) = handle_ipc_command("GET", "/invalid-endpoint", &HashMap::new());
    assert_eq!(status, 404);
    assert!(body.contains("Not Found"));

    // 6. /status
    let (status, body) = handle_ipc_command("GET", "/status", &HashMap::new());
    assert_eq!(status, 200);
    assert!(body.contains("\"service\":\"kora-ipc\""));
}

#[test]
fn test_tcp_socket_roundtrip() {
    let test_port = 14498;
    let _ = start_ipc_server_on_port(test_port);
    std::thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", test_port))
        .expect("connect to IPC server");
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();

    let request = "GET /config?name=acad.exe&font=.VnTime HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("application/json"));
    assert!(response.contains("\"charset\":\"tcvn3\""));
}
