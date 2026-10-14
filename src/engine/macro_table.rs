use crate::vietnamese::charset::decompose_vowel;
use std::collections::HashMap;

/// Macro rule type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MacroType {
    /// Whole word replacement (e.g. "ko" -> "không")
    #[default]
    Normal,
    /// Quick onset consonant replacement (e.g. "f" -> "ph", "j" -> "gi", "w" -> "qu")
    StartConsonant,
    /// Quick coda consonant replacement (e.g. "g" -> "ng", "h" -> "nh", "k" -> "ch")
    EndConsonant,
}

impl MacroType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MacroType::Normal => "normal",
            MacroType::StartConsonant => "start",
            MacroType::EndConsonant => "end",
        }
    }
}

impl std::fmt::Display for MacroType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for MacroType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for MacroType {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "start" | "phụ âm đầu" | "phu am dau" => MacroType::StartConsonant,
            "end" | "phụ âm cuối" | "phu am cuoi" => MacroType::EndConsonant,
            _ => MacroType::Normal,
        }
    }
}

impl std::str::FromStr for MacroType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

/// A single macro rule entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroEntry {
    pub key: String,
    pub value: String,
    pub macro_type: MacroType,
}

/// Storage and resolution of Vietnamese shorthand / macros
#[derive(Debug, Clone, Default)]
pub struct MacroTable {
    entries: HashMap<String, MacroEntry>,
    start_consonants: Vec<MacroEntry>,
    end_consonants: Vec<MacroEntry>,
    start_first_mask: u32,
    end_last_mask: u32,
}

#[inline]
fn with_lowercase_key<R>(key: &str, f: impl FnOnce(&str) -> R) -> R {
    let mut buf = [0u8; 64];
    if key.len() <= 64 && key.is_ascii() {
        for (i, b) in key.bytes().enumerate() {
            buf[i] = b.to_ascii_lowercase();
        }
        f(std::str::from_utf8(&buf[..key.len()]).unwrap())
    } else {
        f(&key.to_lowercase())
    }
}

impl MacroTable {
    /// Create an empty macro table
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild_cache(&mut self) {
        let mut starts: Vec<MacroEntry> = self
            .entries
            .values()
            .filter(|e| e.macro_type == MacroType::StartConsonant)
            .cloned()
            .collect();
        starts.sort_by_key(|a| std::cmp::Reverse(a.key.len()));

        let mut ends: Vec<MacroEntry> = self
            .entries
            .values()
            .filter(|e| e.macro_type == MacroType::EndConsonant)
            .cloned()
            .collect();
        ends.sort_by_key(|a| std::cmp::Reverse(a.key.len()));

        let mut start_mask = 0u32;
        for s in &starts {
            if let Some(&b) = s.key.as_bytes().first()
                && b.is_ascii_lowercase()
            {
                start_mask |= 1u32 << (b - b'a');
            }
        }

        let mut end_mask = 0u32;
        for e in &ends {
            if let Some(&b) = e.key.as_bytes().last()
                && b.is_ascii_lowercase()
            {
                end_mask |= 1u32 << (b - b'a');
            }
        }

        self.start_consonants = starts;
        self.end_consonants = ends;
        self.start_first_mask = start_mask;
        self.end_last_mask = end_mask;
    }

    /// Create a macro table pre-populated with standard default rules
    pub fn with_defaults() -> Self {
        let mut table = Self::new();
        // Default Start Consonants
        table.insert_typed_no_cache("f", "ph", MacroType::StartConsonant);
        table.insert_typed_no_cache("j", "gi", MacroType::StartConsonant);
        table.insert_typed_no_cache("w", "qu", MacroType::StartConsonant);

        // Default End Consonants
        table.insert_typed_no_cache("g", "ng", MacroType::EndConsonant);
        table.insert_typed_no_cache("h", "nh", MacroType::EndConsonant);
        table.insert_typed_no_cache("k", "ch", MacroType::EndConsonant);

        table.rebuild_cache();
        table
    }

    /// Insert or update a macro entry with Normal type (backward-compatible)
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.insert_typed(key, value, MacroType::Normal);
    }

    /// Insert entry without immediately rebuilding search cache (for bulk insertion)
    pub fn insert_typed_no_cache(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        macro_type: MacroType,
    ) {
        let key_str = key.into().to_lowercase();
        let value_str = value.into();
        self.entries.insert(
            key_str.clone(),
            MacroEntry {
                key: key_str,
                value: value_str,
                macro_type,
            },
        );
    }

    /// Insert or update a macro entry with a specific type
    pub fn insert_typed(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        macro_type: MacroType,
    ) {
        self.insert_typed_no_cache(key, value, macro_type);
        self.rebuild_cache();
    }

    /// Remove a macro entry
    pub fn remove(&mut self, key: &str) -> Option<MacroEntry> {
        let res = with_lowercase_key(key, |k| self.entries.remove(k));
        if res.is_some() {
            self.rebuild_cache();
        }
        res
    }

    /// Check if a macro shortcut key exists
    pub fn contains_key(&self, key: &str) -> bool {
        with_lowercase_key(key, |k| self.entries.contains_key(k))
    }

    /// Clear all macro entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.start_consonants.clear();
        self.end_consonants.clear();
        self.start_first_mask = 0;
        self.end_last_mask = 0;
    }

    /// Number of macro entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if macro table is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Access all entries
    pub fn entries(&self) -> &HashMap<String, MacroEntry> {
        &self.entries
    }

    /// Get a specific macro entry
    pub fn get(&self, key: &str) -> Option<&MacroEntry> {
        with_lowercase_key(key, |k| self.entries.get(k))
    }

    /// Get all macro entries sorted by shortcut key
    pub fn get_sorted_entries(&self) -> Vec<MacroEntry> {
        let mut list: Vec<MacroEntry> = self.entries.values().cloned().collect();
        list.sort_by(|a, b| a.key.cmp(&b.key));
        list
    }

    /// Load macros from a multiline text configuration (e.g. "key:expansion:type" or "key = expansion:type")
    pub fn load_from_str(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((k, rest)) = line.split_once('=') {
                let k = k.trim();
                let rest = rest.trim();
                if let Some((v, t)) = rest.split_once(':') {
                    self.insert_typed_no_cache(k, v.trim(), MacroType::from(t));
                } else {
                    self.insert_typed_no_cache(k, rest, MacroType::Normal);
                }
            } else if line.contains('\t') {
                let parts: Vec<&str> = line
                    .split('\t')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.len() >= 3 {
                    self.insert_typed_no_cache(parts[0], parts[1], MacroType::from(parts[2]));
                } else if parts.len() == 2 {
                    self.insert_typed_no_cache(parts[0], parts[1], MacroType::Normal);
                }
            } else if line.contains(':') {
                let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
                if parts.len() >= 3 {
                    self.insert_typed_no_cache(parts[0], parts[1], MacroType::from(parts[2]));
                } else if parts.len() == 2 {
                    self.insert_typed_no_cache(parts[0], parts[1], MacroType::Normal);
                }
            }
        }
        self.rebuild_cache();
    }

    /// Serialize all macros to configuration format
    pub fn save_to_str(&self) -> String {
        let mut s = String::from(
            "# Bảng gõ tắt Kora\n# Cú pháp: <từ viết tắt>:<cụm từ thay thế>:<loại (normal/start/end)>\n",
        );
        for entry in self.get_sorted_entries() {
            s.push_str(&format!(
                "{}:{}:{}\n",
                entry.key,
                entry.value,
                entry.macro_type.as_str()
            ));
        }
        s
    }

    /// Load macros from a file
    pub fn load_from_file(&mut self, path: &str) -> std::io::Result<()> {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.load_from_str(&content);
        }
        Ok(())
    }

    /// Save macros to a file
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, self.save_to_str())
    }

    /// Lookup macro expansion for Normal type, matching the case convention of the input word
    pub fn lookup(&self, word: &str) -> Option<String> {
        if word.is_empty() {
            return None;
        }

        with_lowercase_key(word, |key| {
            let entry = self.entries.get(key)?;
            if entry.macro_type == MacroType::Normal {
                Some(apply_case_style(word, &entry.value))
            } else {
                None
            }
        })
    }

    /// Expand word on Space: checks Normal, StartConsonant, and EndConsonant macros.
    ///
    /// - Normal macros: full match (e.g. "ko" -> "không")
    /// - StartConsonant macros: onset prefix when followed by a vowel (e.g. "fong" -> "phong")
    /// - EndConsonant macros: coda suffix when preceded by a vowel (e.g. "dag" -> "dang")
    ///
    /// Single-character inputs (e.g. "f", "j", "w", "g") are strictly preserved as literal characters.
    pub fn expand_word(
        &self,
        word: &str,
        allow_start_consonant: bool,
        allow_end_consonant: bool,
    ) -> Option<String> {
        if word.is_empty() || self.entries.is_empty() {
            return None;
        }

        let mut buf = [0u8; 64];
        let lower_cow;
        let lower: &str = if word.len() <= 64 && word.is_ascii() {
            for (i, b) in word.bytes().enumerate() {
                buf[i] = b.to_ascii_lowercase();
            }
            std::str::from_utf8(&buf[..word.len()]).unwrap()
        } else {
            lower_cow = word.to_lowercase();
            &lower_cow
        };

        // 1. Exact Normal macro match takes highest priority
        if let Some(entry) = self.entries.get(lower)
            && entry.macro_type == MacroType::Normal
        {
            return Some(apply_case_style(word, &entry.value));
        }

        // 2. Single-character guard: single letters like 'f', 'j', 'w' MUST NOT be modified
        if word.len() <= 1 || word.chars().nth(1).is_none() {
            return None;
        }

        let mut matched_start: Option<(&str, &str)> = None;
        let mut matched_end: Option<(&str, &str)> = None;
        let lower_bytes = lower.as_bytes();

        // 3. Quick start consonant match (onset) — O(1) bitmask pre-filter skips loop when first char cannot match
        if allow_start_consonant && !self.start_consonants.is_empty() {
            let b0 = lower_bytes[0];
            if !b0.is_ascii_lowercase() || (self.start_first_mask & (1u32 << (b0 - b'a'))) != 0 {
                for entry in &self.start_consonants {
                    if lower.starts_with(&entry.key) {
                        let rem = &lower[entry.key.len()..];
                        if let Some(next_char) = rem.chars().next()
                            && is_vowel(next_char)
                        {
                            matched_start = Some((entry.key.as_str(), entry.value.as_str()));
                            break;
                        }
                    }
                }
            }
        }

        // 4. Quick end consonant match (coda) — O(1) bitmask pre-filter skips loop when last char cannot match
        if allow_end_consonant && !self.end_consonants.is_empty() {
            let bl = lower_bytes[lower_bytes.len() - 1];
            if !bl.is_ascii_lowercase() || (self.end_last_mask & (1u32 << (bl - b'a'))) != 0 {
                for entry in &self.end_consonants {
                    if lower.ends_with(&entry.key) {
                        let prefix_len = lower.len() - entry.key.len();
                        let prefix = &lower[..prefix_len];
                        if let Some(prev_char) = prefix.chars().last()
                            && is_vowel(prev_char)
                        {
                            matched_end = Some((entry.key.as_str(), entry.value.as_str()));
                            break;
                        }
                    }
                }
            }
        }

        // 5. Construct replaced word
        match (matched_start, matched_end) {
            (Some((sk, sv)), Some((ek, ev))) => {
                if word.len() >= sk.len() + ek.len() {
                    let middle = &word[sk.len()..word.len() - ek.len()];
                    let mut res = String::with_capacity(sv.len() + middle.len() + ev.len());
                    res.push_str(sv);
                    res.push_str(middle);
                    res.push_str(ev);
                    Some(apply_case_style(word, &res))
                } else {
                    None
                }
            }
            (Some((sk, sv)), None) => {
                let rem = &word[sk.len()..];
                let mut res = String::with_capacity(sv.len() + rem.len());
                res.push_str(sv);
                res.push_str(rem);
                Some(apply_case_style(word, &res))
            }
            (None, Some((ek, ev))) => {
                let prefix = &word[..word.len() - ek.len()];
                let mut res = String::with_capacity(prefix.len() + ev.len());
                res.push_str(prefix);
                res.push_str(ev);
                Some(apply_case_style(word, &res))
            }
            (None, None) => None,
        }
    }
}

/// Checks whether a character is a Vietnamese vowel (including all tonal/diacritic variants)
#[inline]
pub fn is_vowel(c: char) -> bool {
    decompose_vowel(c).is_some()
}

/// Applies the casing style of `source` onto `target`
/// - "ko" -> "không"
/// - "Ko" -> "Không"
/// - "KO" -> "KHÔNG"
pub fn apply_case_style(source: &str, target: &str) -> String {
    if source.is_empty() || target.is_empty() {
        return target.to_string();
    }

    let is_all_upper = source
        .chars()
        .all(|c| !c.is_alphabetic() || c.is_uppercase());
    if is_all_upper {
        return target.to_uppercase();
    }

    if let (Some(first_source), Some(first_target)) = (source.chars().next(), target.chars().next())
        && first_source.is_uppercase()
    {
        let mut res = String::with_capacity(target.len());
        for u in first_target.to_uppercase() {
            res.push(u);
        }
        res.push_str(&target[first_target.len_utf8()..]);
        return res;
    }

    target.to_string()
}
