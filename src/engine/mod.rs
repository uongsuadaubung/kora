pub mod action;
pub mod buffer;
pub mod config;
pub mod config_store;
pub mod history;
pub mod macro_table;
pub mod profile;

use crate::logger::{AsyncLogWriter, current_timestamp_str};
use crate::vietnamese::CharsetStrategy;
pub use action::{CompactString, EngineAction};
use buffer::{RawKey, TypingBuffer};
pub use config::{EngineConfig, InputMethod};
use history::{HistoryBackspace, WordHistory};
use macro_table::MacroTable;
pub use profile::ProfileManager;

/// Clean, stateful, thread-safe Vietnamese Input Engine.
#[derive(Debug, Clone)]
pub struct VietnameseEngine {
    config: EngineConfig,
    cached_effective_config: EngineConfig,
    is_active_global: bool,
    active_child_idx: Option<usize>,
    pub buffer: TypingBuffer,
    pub macro_table: MacroTable,
    pub profile_manager: ProfileManager,
    history: WordHistory,
    auto_uppercase_next: bool,
    pub debug_log: Vec<String>,
    log_writer: AsyncLogWriter,
    cached_active_macro_table: Option<MacroTable>,
    cached_macro_sig: Option<(String, Vec<String>, usize, usize)>,
}

impl Default for VietnameseEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl VietnameseEngine {
    pub fn new(config: EngineConfig) -> Self {
        let log_writer = if config.debug {
            AsyncLogWriter::new(config.debug_file_path.clone())
        } else {
            AsyncLogWriter::disabled()
        };
        let cached_effective_config = config.clone();
        Self {
            config,
            cached_effective_config,
            is_active_global: true,
            active_child_idx: None,
            buffer: TypingBuffer::new(),
            macro_table: MacroTable::with_defaults(),
            profile_manager: ProfileManager::new(),
            history: WordHistory::new(),
            auto_uppercase_next: false,
            debug_log: Vec::new(),
            log_writer,
            cached_active_macro_table: None,
            cached_macro_sig: None,
        }
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut EngineConfig {
        &mut self.config
    }

    pub fn active_profile_id(&self) -> &str {
        &self.profile_manager.active_id
    }

    pub fn refresh_effective_config(&mut self) {
        self.is_active_global = self.profile_manager.active_id == "global";
        if self.is_active_global {
            self.active_child_idx = None;
            self.cached_effective_config = self.config.clone();
        } else {
            self.active_child_idx = self
                .profile_manager
                .profiles
                .iter()
                .position(|p| p.id == self.profile_manager.active_id);
            self.cached_effective_config = self
                .profile_manager
                .resolve_effective_config(&self.profile_manager.active_id, &self.config);
        }
    }

    #[inline(always)]
    pub fn effective_config(&self) -> &EngineConfig {
        if self.is_active_global {
            &self.config
        } else {
            &self.cached_effective_config
        }
    }

    #[inline]
    pub fn is_key_disabled(&self, vk: u32) -> bool {
        self.profile_manager.is_vk_disabled(
            &self.profile_manager.active_id,
            self.is_active_global,
            self.active_child_idx,
            vk,
        )
    }

    #[inline]
    pub fn lookup_key_remap(&self, vk: u32) -> Option<u32> {
        self.profile_manager.lookup_remap_vk(
            &self.profile_manager.active_id,
            self.is_active_global,
            self.active_child_idx,
            vk,
        )
    }

    #[inline]
    pub fn lookup_key_macro(&self, vk: u32) -> Option<&[profile::MacroAction]> {
        self.profile_manager.lookup_key_macro_actions(
            &self.profile_manager.active_id,
            self.is_active_global,
            self.active_child_idx,
            vk,
        )
    }

    pub fn effective_remaps(&self) -> Vec<profile::KeyRemapRule> {
        self.profile_manager
            .resolve_effective_remaps(&self.profile_manager.active_id)
    }

    pub fn effective_disabled(&self) -> Vec<u32> {
        self.profile_manager
            .resolve_effective_disabled(&self.profile_manager.active_id)
    }

    pub fn effective_key_macros(&self) -> Vec<profile::KeyMacroRule> {
        self.profile_manager
            .resolve_effective_key_macros(&self.profile_manager.active_id)
    }

    pub fn effective_macros(&self) -> Vec<crate::engine::macro_table::MacroEntry> {
        self.profile_manager.resolve_effective_macros(
            &self.profile_manager.active_id,
            &self.macro_table.get_sorted_entries(),
        )
    }

    pub fn switch_profile_by_process(&mut self, exe_name: &str) -> (bool, String) {
        let (changed, new_id) = self.profile_manager.set_active_by_process(exe_name);
        if changed {
            self.refresh_effective_config();
            self.reset();
            if self.config.debug {
                self.log_debug(format!(
                    "[PROFILE] Switched active profile to '{}' (process: {})",
                    new_id, exe_name
                ));
            }
        }
        (changed, new_id)
    }

    /// Dynamically sets the character set for an application based on process name and font name.
    /// If the profile is currently active in the foreground, immediately refreshes effective config
    /// and resets the typing buffer to prevent cross-charset state corruption.
    pub fn update_charset_by_process_and_font(
        &mut self,
        process_name: &str,
        font_name: &str,
    ) -> (bool, crate::vietnamese::Charset) {
        let (is_active, charset) = self
            .profile_manager
            .update_charset_by_process_and_font(process_name, font_name);
        if is_active {
            self.refresh_effective_config();
            self.reset();
            if self.config.debug {
                self.log_debug(format!(
                    "[CHARSET] Auto-updated charset for '{}' (font: '{}') -> {:?}",
                    process_name, font_name, charset
                ));
            }
        }
        (is_active, charset)
    }

    /// Dynamically applies configuration updates from external callers (IPC / GUI).
    /// If `target_process` is Some (and not "global"), updates that child profile (creating it if needed).
    /// If `target_process` is None or Some("global"), updates global EngineConfig.
    /// Automatically saves configuration to disk (debounced).
    /// Returns (is_active_profile_updated, effective_config_for_target).
    pub fn apply_config_update(
        &mut self,
        target_process: Option<&str>,
        params: &std::collections::HashMap<String, String>,
    ) -> (bool, EngineConfig) {
        let is_global = match target_process {
            None => true,
            Some(t) => {
                let trimmed = t.trim();
                trimmed.is_empty() || trimmed.eq_ignore_ascii_case("global")
            }
        };

        if is_global {
            if let Some(mode) = params.get("mode") {
                let m = mode.trim().to_ascii_lowercase();
                if m == "toggle" {
                    self.config.enabled = !self.config.enabled;
                } else if let Ok(b) = m.parse::<bool>() {
                    self.config.enabled = b;
                } else if m == "vietnamese" || m == "viet" || m == "vn" || m == "vi" || m == "1" || m == "on" {
                    self.config.enabled = true;
                } else if m == "english" || m == "eng" || m == "en" || m == "0" || m == "off" {
                    self.config.enabled = false;
                }
            }
            if let Some(method) = params.get("method") {
                let m: InputMethod = method.trim().parse().unwrap_or_default();
                self.config.method = m;
            }
            if let Some(charset) = params.get("charset") {
                if let Ok(c) = charset.trim().parse() {
                    self.config.charset = c;
                }
            }
            if let Some(font) = params.get("font") {
                self.config.charset = crate::vietnamese::charset::detect_charset_from_font(font);
            }
            if let Some(sound) = params.get("sound") {
                let s = sound.trim().to_ascii_lowercase();
                if let Ok(b) = s.parse::<bool>() {
                    self.config.sound_enabled = b;
                } else if s == "on" || s == "1" || s == "enable" || s == "enabled" {
                    self.config.sound_enabled = true;
                } else if s == "off" || s == "0" || s == "disable" || s == "disabled" {
                    self.config.sound_enabled = false;
                }
            }
            if let Some(sp) = params.get("sound_profile") {
                let trimmed = sp.trim();
                if !trimmed.is_empty() {
                    self.config.sound_profile = trimmed.into();
                }
            }
            if let Some(vol) = params.get("sound_volume") {
                if let Ok(v) = vol.trim().parse::<u8>() {
                    self.config.sound_volume = v.min(100);
                }
            }
            if let Some(use_macro) = params.get("use_macro") {
                let m = use_macro.trim().to_ascii_lowercase();
                if let Ok(b) = m.parse::<bool>() {
                    self.config.use_macro = b;
                } else if m == "on" || m == "1" || m == "enable" || m == "enabled" {
                    self.config.use_macro = true;
                } else if m == "off" || m == "0" || m == "disable" || m == "disabled" {
                    self.config.use_macro = false;
                }
            }
            if let Some(caps) = params.get("auto_caps") {
                let c = caps.trim().to_ascii_lowercase();
                if let Ok(b) = c.parse::<bool>() {
                    self.config.auto_uppercase_first_char = b;
                } else if c == "on" || c == "1" || c == "enable" || c == "enabled" {
                    self.config.auto_uppercase_first_char = true;
                } else if c == "off" || c == "0" || c == "disable" || c == "disabled" {
                    self.config.auto_uppercase_first_char = false;
                }
            }

            self.refresh_effective_config();
            self.reset();
            let _ = crate::engine::config_store::save_config_all_debounced(
                &self.config,
                &self.macro_table,
                &self.profile_manager,
            );
            (self.profile_manager.active_id == "global", self.config.clone())
        } else {
            let proc_name = target_process.unwrap().trim();
            let target_id = {
                let child = self.profile_manager.get_or_create_profile_mut(proc_name);
                if let Some(mode) = params.get("mode") {
                    let m = mode.trim().to_ascii_lowercase();
                    if m == "inherit" {
                        child.mode = profile::LanguageMode::Inherit;
                    } else if m == "toggle" {
                        let cur_effective = if child.mode == profile::LanguageMode::Vietnamese {
                            true
                        } else if child.mode == profile::LanguageMode::English {
                            false
                        } else {
                            self.config.enabled
                        };
                        child.mode = if cur_effective {
                            profile::LanguageMode::English
                        } else {
                            profile::LanguageMode::Vietnamese
                        };
                    } else if let Ok(b) = m.parse::<bool>() {
                        child.mode = if b { profile::LanguageMode::Vietnamese } else { profile::LanguageMode::English };
                    } else if m == "vietnamese" || m == "viet" || m == "vn" || m == "vi" || m == "1" || m == "on" {
                        child.mode = profile::LanguageMode::Vietnamese;
                    } else if m == "english" || m == "eng" || m == "en" || m == "0" || m == "off" {
                        child.mode = profile::LanguageMode::English;
                    }
                }
                if let Some(method) = params.get("method") {
                    let m = method.trim().to_ascii_lowercase();
                    if m == "inherit" {
                        child.method = profile::MethodMode::Inherit;
                    } else {
                        let im: InputMethod = m.parse().unwrap_or_default();
                        child.method = profile::MethodMode::Specific(im);
                    }
                }
                if let Some(charset) = params.get("charset") {
                    let c = charset.trim().to_ascii_lowercase();
                    if c == "inherit" {
                        child.charset = profile::CharsetMode::Inherit;
                    } else if let Ok(cs) = c.parse() {
                        child.charset = profile::CharsetMode::Specific(cs);
                    }
                }
                if let Some(font) = params.get("font") {
                    child.charset = profile::CharsetMode::Specific(crate::vietnamese::charset::detect_charset_from_font(font));
                }
                if let Some(sound) = params.get("sound") {
                    let s = sound.trim().to_ascii_lowercase();
                    if s == "inherit" {
                        child.sound_enabled = None;
                    } else if let Ok(b) = s.parse::<bool>() {
                        child.sound_enabled = Some(b);
                    } else if s == "on" || s == "1" || s == "enable" || s == "enabled" {
                        child.sound_enabled = Some(true);
                    } else if s == "off" || s == "0" || s == "disable" || s == "disabled" {
                        child.sound_enabled = Some(false);
                    }
                }
                if let Some(sp) = params.get("sound_profile") {
                    let trimmed = sp.trim();
                    if trimmed.eq_ignore_ascii_case("inherit") {
                        child.sound_profile = None;
                    } else if !trimmed.is_empty() {
                        child.sound_profile = Some(trimmed.into());
                    }
                }
                if let Some(vol) = params.get("sound_volume") {
                    let trimmed = vol.trim();
                    if trimmed.eq_ignore_ascii_case("inherit") {
                        child.sound_volume = None;
                    } else if let Ok(v) = trimmed.parse::<u8>() {
                        child.sound_volume = Some(v.min(100));
                    }
                }
                if let Some(use_macro) = params.get("use_macro") {
                    let m = use_macro.trim().to_ascii_lowercase();
                    if m == "inherit" {
                        child.use_macro = None;
                    } else if let Ok(b) = m.parse::<bool>() {
                        child.use_macro = Some(b);
                    } else if m == "on" || m == "1" || m == "enable" || m == "enabled" {
                        child.use_macro = Some(true);
                    } else if m == "off" || m == "0" || m == "disable" || m == "disabled" {
                        child.use_macro = Some(false);
                    }
                }
                if let Some(caps) = params.get("auto_caps") {
                    let c = caps.trim().to_ascii_lowercase();
                    if c == "inherit" {
                        child.auto_uppercase_first_char = None;
                    } else if let Ok(b) = c.parse::<bool>() {
                        child.auto_uppercase_first_char = Some(b);
                    } else if c == "on" || c == "1" || c == "enable" || c == "enabled" {
                        child.auto_uppercase_first_char = Some(true);
                    } else if c == "off" || c == "0" || c == "disable" || c == "disabled" {
                        child.auto_uppercase_first_char = Some(false);
                    }
                }
                child.id.clone()
            };

            let is_active = self.profile_manager.active_id == target_id;
            if is_active {
                self.refresh_effective_config();
                self.reset();
            }
            let effective = self.profile_manager.resolve_effective_config(&target_id, &self.config);
            let _ = crate::engine::config_store::save_config_all_debounced(
                &self.config,
                &self.macro_table,
                &self.profile_manager,
            );
            (is_active, effective)
        }
    }

    /// Chuyển đổi qua lại giữa chế độ gõ Tiếng Việt và Tiếng Anh (trả về trạng thái mới: true là Tiếng Việt, false là Tiếng Anh)
    /// If the active child profile pins a mode, the toggle flips that profile's mode; otherwise the global one.
    pub fn toggle_enabled(&mut self) -> bool {
        let new_state = !self.effective_config().enabled;
        let active_id = self.profile_manager.active_id.clone();
        match self.profile_manager.get_profile_mut(&active_id) {
            Some(p) if p.mode != profile::LanguageMode::Inherit => {
                p.mode = if new_state {
                    profile::LanguageMode::Vietnamese
                } else {
                    profile::LanguageMode::English
                };
            }
            _ => self.config.enabled = new_state,
        }
        self.refresh_effective_config();
        self.reset();
        new_state
    }

    #[inline]
    fn ensure_active_macro_table(&mut self) {
        let global_mutes_len = self.profile_manager.global_profile.muted_macros.len();
        let current_child_sum = if self.is_active_global {
            0
        } else {
            match self
                .profile_manager
                .resolve_child(&self.profile_manager.active_id, self.active_child_idx)
            {
                Some(p) => p.muted_macros.len() + p.overridden_macros.len() + p.app_macros.len(),
                None => 0,
            }
        };

        if global_mutes_len == 0 && current_child_sum == 0 {
            if self.cached_active_macro_table.is_some() {
                self.cached_active_macro_table = None;
                self.cached_macro_sig = None;
            }
            return;
        }

        let is_valid =
            if let Some((ref cached_id, ref cached_global_mutes, cached_global_len, child_sum)) =
                self.cached_macro_sig
            {
                cached_global_len == self.macro_table.len()
                    && child_sum == current_child_sum
                    && cached_global_mutes.len() == global_mutes_len
                    && (self.is_active_global || *cached_id == self.profile_manager.active_id)
                    && cached_global_mutes.as_slice()
                        == self.profile_manager.global_profile.muted_macros.as_slice()
            } else {
                false
            };

        if !is_valid {
            let mut table = MacroTable::new();
            for e in self.effective_macros() {
                table.insert_typed_no_cache(e.key, e.value, e.macro_type);
            }
            table.rebuild_cache();
            self.cached_macro_sig = Some((
                self.profile_manager.active_id.clone(),
                self.profile_manager.global_profile.muted_macros.clone(),
                self.macro_table.len(),
                current_child_sum,
            ));
            self.cached_active_macro_table = Some(table);
        }
    }

    /// Macro table of the active profile: borrowed reference without heap allocations
    fn active_macro_table(&self) -> &MacroTable {
        if let Some(ref cached) = self.cached_active_macro_table {
            cached
        } else {
            &self.macro_table
        }
    }

    pub fn log_debug(&mut self, msg: impl Into<String>) {
        if self.config.debug {
            let msg = msg.into();
            if !self.log_writer.is_active() && self.config.debug_file_path.is_some() {
                self.log_writer = AsyncLogWriter::new(self.config.debug_file_path.clone());
            }
            let timestamp = current_timestamp_str();
            let formatted = format!("[{}]{}", timestamp, msg);
            eprintln!("{}", formatted);

            // Ghi nhật ký bất đồng bộ ra background thread để triệt tiêu mọi I/O latency trên hook
            self.log_writer.write_line(formatted.clone());

            if self.debug_log.len() >= 500 {
                self.debug_log.remove(0);
            }
            self.debug_log.push(formatted);
        }
    }

    /// Lấy danh sách các bản ghi log gần nhất
    pub fn debug_log(&self) -> &[String] {
        &self.debug_log
    }

    /// Xuất toàn bộ log vết phím gần đây thành chuỗi
    pub fn dump_debug_log(&self) -> String {
        self.debug_log.join("\n")
    }

    /// Xóa sạch lịch sử debug log
    pub fn clear_debug_log(&mut self) {
        self.debug_log.clear();
    }

    /// Reset current word session and history (e.g. on mouse click or navigation keys)
    pub fn reset(&mut self) {
        if self.config.debug && !self.buffer.is_empty() {
            let prev_word: String = self.buffer.raw_keys.iter().map(|k| k.ch).collect();
            self.log_debug(format!(
                "[DBG][RESET] Session reset. Word in buffer was: {:?}",
                prev_word
            ));
        }
        self.buffer.clear();
        self.history.clear();
    }

    /// Feed a character into the engine with hardware Shift & CapsLock state.
    pub fn on_key(&mut self, ch: char, is_shift: bool, is_caps: bool) -> EngineAction {
        let is_letter = if ch.is_ascii() {
            ch.is_ascii_alphabetic()
        } else {
            ch.is_alphabetic()
        };
        let mut is_upper = (is_shift ^ is_caps) && is_letter;

        // Auto-uppercase first letter after period or newline (only check if triggered)
        if self.auto_uppercase_next
            && is_letter
            && self.buffer.is_empty()
            && if self.is_active_global {
                self.config.auto_uppercase_first_char
            } else {
                self.cached_effective_config.auto_uppercase_first_char
            }
        {
            is_upper = true;
            self.auto_uppercase_next = false;
        }

        // Letter Fast-Path: Hơn 95% thao tác gõ phím là chữ cái thông thường ('a'..'z', 'A'..'Z').
        // Nhờ điều kiện `!is_letter`, toàn bộ các chữ cái bỏ qua kiểm tra whitespace và bitmask ngắt từ
        // ngay trong đúng 1 chu kỳ CPU mà không tốn lệnh rẽ nhánh phụ hay đọc cấu hình bracket_w.
        if !is_letter {
            let bracket_w = if self.is_active_global {
                self.config.bracket_w
            } else {
                self.cached_effective_config.bracket_w
            };
            if ch.is_whitespace() || TypingBuffer::is_word_break(ch, bracket_w) {
                return self.handle_word_break(ch);
            }
        }

        // Settings of the active (foreground app) profile, resolved over the global config
        let cfg = if self.is_active_global {
            &self.config
        } else {
            &self.cached_effective_config
        };

        let prev_state_summary = if self.config.debug {
            self.buffer.state.summary()
        } else {
            String::new()
        };

        if !cfg.enabled {
            if cfg.use_macro && cfg.use_macro_in_english_mode {
                if ch.is_alphanumeric() || ch == '_' {
                    self.buffer.raw_keys.push(RawKey { ch, is_upper });
                    self.buffer.emitted_len += 1;
                } else {
                    self.buffer.clear();
                }
            } else {
                self.buffer.clear();
            }
            return EngineAction::Passthrough;
        }

        let raw_key = RawKey { ch, is_upper };

        // If the incoming key begins a new CamelCase boundary, save preceding word into history
        if self.buffer.is_boundary(&raw_key) {
            let is_same_as_raw = self.buffer.is_rendered_identical_to_raw();
            let should_restore = cfg.restore_on_wrong_spelling
                && !is_same_as_raw
                && !self.buffer.state.is_valid_spelling();

            if should_restore {
                let backspaces = self.buffer.emitted_len;
                let mut output: CompactString = self.buffer.raw_keys.iter().map(|k| k.ch).collect();
                output.push(ch);
                let current_raw = self.buffer.raw_keys;
                self.history.commit_word(current_raw, false);
                self.buffer.clear();
                self.buffer.raw_keys.push(raw_key);
                let (next_state, _) =
                    crate::vietnamese::SyllableState::Empty.feed_key(raw_key, 0, &cfg);
                self.buffer.state = next_state;
                self.buffer.last_rendered.push(ch);
                self.buffer.emitted_len = ch.len_utf16();
                return EngineAction::Replace { backspaces, output };
            }

            let current_raw = self.buffer.raw_keys;
            self.history.commit_word(current_raw, false);
            self.buffer.clear();
        }

        // If this buffer was restored from history across a space, check if the incoming key
        // is starting a new word instead of modifying the restored word.
        if self.buffer.is_restored_across_space {
            self.buffer.is_restored_across_space = false;
            let (test_state, _) =
                self.buffer
                    .state
                    .feed_key(raw_key, self.buffer.emitted_len, &cfg);
            if matches!(test_state, crate::vietnamese::SyllableState::Passthrough(_))
                || self.buffer.is_passthrough()
            {
                let current_raw = self.buffer.raw_keys;
                self.history.commit_word(current_raw, false);
                self.history.add_space();
                self.buffer.clear();
            }
        }

        let action = self.buffer.feed_key(raw_key, &cfg);

        if self.config.debug {
            let next_state_summary = self.buffer.state.summary();
            let current_word: String = self.buffer.raw_keys.iter().map(|k| k.ch).collect();
            self.log_debug(format!(
                "[DBG][KEY] {:?} (shift={}, caps={}, upper={}) | State: {} -> {} | Word: {:?} (screen_len={}) | Action: {:?}",
                ch,
                is_shift as u8,
                is_caps as u8,
                is_upper as u8,
                prev_state_summary,
                next_state_summary,
                current_word,
                self.buffer.emitted_len,
                action
            ));
        }

        action
    }

    fn handle_word_break(&mut self, ch: char) -> EngineAction {
        // Fast path: if buffer is empty, no macro expansion or spelling restoration needed
        if self.buffer.is_empty() {
            let auto_uppercase = if self.is_active_global {
                self.config.auto_uppercase_first_char
            } else {
                self.cached_effective_config.auto_uppercase_first_char
            };
            if auto_uppercase && (ch == '.' || ch == '\n' || ch == '\r') {
                self.auto_uppercase_next = true;
            }
            if ch == ' ' {
                self.history.add_space();
            } else {
                self.history.clear();
            }
            return EngineAction::Passthrough;
        }

        let (auto_uppercase, use_macro, restore_on_wrong_spelling) = if self.is_active_global {
            (
                self.config.auto_uppercase_first_char,
                self.config.use_macro,
                self.config.restore_on_wrong_spelling,
            )
        } else {
            (
                self.cached_effective_config.auto_uppercase_first_char,
                self.cached_effective_config.use_macro,
                self.cached_effective_config.restore_on_wrong_spelling,
            )
        };

        // Update auto-uppercase trigger
        if auto_uppercase && (ch == '.' || ch == '\n' || ch == '\r') {
            self.auto_uppercase_next = true;
        }

        // O(1) Length Fast-Path: Với mọi từ tiếng Việt có dấu phụ hoặc dấu thanh (tiếng, việt, được, rất),
        // số phím thô (raw_keys: "t-i-e-e-n-g-s") luôn khác số ký tự phát ra màn hình (emitted_len: 5 ký tự "tiếng").
        // Ta kết luận ngay `is_same = false` chỉ trong 1 lệnh so sánh số nguyên O(1), tránh việc duyệt UTF-8 lặp.
        let is_same = self.buffer.is_rendered_identical_to_raw();

        // Check Macro expansion before clearing buffer
        if use_macro {
            self.ensure_active_macro_table();
            let macro_table = self.active_macro_table();
            if !macro_table.is_empty() {
                let rendered = self.buffer.rendered();
                let is_space = ch == ' ';
                let allow_start = is_space;
                let allow_end = is_space;

                let expanded = macro_table
                    .expand_word(rendered, allow_start, allow_end)
                    .or_else(|| {
                        if !is_same {
                            let current_word: CompactString =
                                self.buffer.raw_keys.iter().map(|k| k.ch).collect();
                            macro_table.expand_word(&current_word, allow_start, allow_end)
                        } else {
                            None
                        }
                    });

                if let Some(mut output) = expanded {
                    let backspaces = self.buffer.emitted_len;
                    output.push(ch);

                    if self.config.debug {
                        let current_word: String =
                            self.buffer.raw_keys.iter().map(|k| k.ch).collect();
                        self.log_debug(format!(
                            "[DBG][MACRO] Key: {:?} | Expanded {:?} -> {:?} | Replace(bs={}, output={:?})",
                            ch, current_word, output, backspaces, output
                        ));
                    }

                    self.buffer.clear();
                    if ch == ' ' {
                        self.history
                            .commit_word(crate::vietnamese::InlineList::new(), false);
                        self.history.add_space();
                    } else {
                        self.history.clear();
                    }

                    let final_output = if self.config.charset != crate::vietnamese::Charset::Unicode
                    {
                        self.config.charset.encode_text(&output)
                    } else {
                        output
                    };
                    return EngineAction::Replace {
                        backspaces,
                        output: final_output,
                    };
                }
            }
        }

        // Restore raw keys if the word is an invalid Vietnamese syllable
        if restore_on_wrong_spelling {
            let is_invalid = if !is_same {
                !self.buffer.state.is_valid_spelling()
            } else {
                false
            };

            if is_invalid {
                let backspaces = self.buffer.emitted_len;
                let mut output: CompactString = self.buffer.raw_keys.iter().map(|k| k.ch).collect();
                output.push(ch);

                if self.config.debug {
                    self.log_debug(format!(
                        "[DBG][RESTORE] Invalid word {:?} (screen: {:?}) -> Restoring to {:?}",
                        self.buffer
                            .raw_keys
                            .iter()
                            .map(|k| k.ch)
                            .collect::<String>(),
                        self.buffer.rendered(),
                        output
                    ));
                }

                if ch == ' ' {
                    self.history.commit_word(self.buffer.raw_keys, true);
                    self.history.add_space();
                } else {
                    self.history.clear();
                }
                self.buffer.clear();

                return EngineAction::Replace { backspaces, output };
            }
        }

        // Commit word to history
        if !self.buffer.is_empty() {
            if self.config.debug {
                let current_raw: String = self.buffer.raw_keys.iter().map(|k| k.ch).collect();
                self.log_debug(format!(
                    "[DBG][BREAK] Key: {:?} | Committed word: {:?} | Screen: {:?}",
                    ch,
                    current_raw,
                    self.buffer.rendered()
                ));
            }
            if ch == ' ' {
                let current_raw = self.buffer.raw_keys;
                self.history.commit_word(current_raw, false);
                self.history.add_space();
            } else {
                self.history.clear();
            }
        } else if ch == ' ' {
            self.history.add_space();
        } else {
            self.history.clear();
        }

        self.buffer.clear();
        EngineAction::Passthrough
    }

    /// Feed a backspace event
    pub fn on_backspace(&mut self) -> EngineAction {
        let cfg = if self.is_active_global {
            &self.config
        } else {
            &self.cached_effective_config
        };
        if !cfg.enabled {
            if cfg.use_macro && cfg.use_macro_in_english_mode && !self.buffer.is_empty() {
                self.buffer.raw_keys.pop();
                if self.buffer.emitted_len > 0 {
                    self.buffer.emitted_len -= 1;
                }
            } else {
                self.buffer.clear();
                self.history.clear();
            }
            return EngineAction::Passthrough;
        }

        let prev_word = if self.config.debug {
            self.buffer
                .raw_keys
                .iter()
                .map(|k| k.ch)
                .collect::<String>()
        } else {
            String::new()
        };

        if self.buffer.is_empty() {
            let mut restored_word = None;
            if cfg.remember_history_across_space
                && let HistoryBackspace::RestoredWord {
                    raw_keys,
                    is_raw_restored,
                } = self.history.pop_backspace()
            {
                if self.config.debug {
                    restored_word = Some(raw_keys.iter().map(|k| k.ch).collect::<String>());
                }
                if is_raw_restored {
                    self.buffer.restore_as_passthrough(raw_keys);
                } else {
                    // Replay previous word into buffer
                    for &k in &raw_keys {
                        self.buffer.feed_key(k, cfg);
                    }
                    self.buffer.is_restored_across_space = true;
                }
            }
            if self.config.debug {
                self.log_debug(format!(
                    "[DBG][BS] Buffer was empty. History restored across space: {:?}",
                    restored_word
                ));
            }
            return EngineAction::Passthrough;
        }

        self.buffer.handle_backspace(&cfg);
        if self.config.debug {
            let now_word: String = self.buffer.raw_keys.iter().map(|k| k.ch).collect();
            self.log_debug(format!(
                "[DBG][BS] Backspaced. Word: {:?} -> {:?} | State: {} (screen_len={})",
                prev_word,
                now_word,
                self.buffer.state.summary(),
                self.buffer.emitted_len
            ));
        }

        EngineAction::Passthrough
    }
}
