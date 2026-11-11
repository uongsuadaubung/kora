use super::{
    action::{CompactString, EngineAction},
    config::EngineConfig,
};
use crate::vietnamese::{CharsetStrategy, InlineList, state::SyllableState};

/// Packed 4-byte raw key representation (`8B -> 4B`, shrinking `InlineList<RawKey, 32>` from 260B to 132B):
/// - Bits `0..21`: Unicode scalar value (`char`)
/// - Bit `31`: `is_upper` flag
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct RawKey(u32);

impl RawKey {
    #[inline(always)]
    pub const fn new(ch: char, is_upper: bool) -> Self {
        Self((ch as u32) | ((is_upper as u32) << 31))
    }

    #[inline(always)]
    pub const fn ch(self) -> char {
        // SAFETY: Bits 0..31 are always constructed from a valid `char` (or 0 for `Default`).
        unsafe { char::from_u32_unchecked(self.0 & 0x7FFF_FFFF) }
    }

    #[inline(always)]
    pub const fn is_upper(self) -> bool {
        (self.0 >> 31) != 0
    }
}

impl std::fmt::Debug for RawKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawKey")
            .field("ch", &self.ch())
            .field("is_upper", &self.is_upper())
            .finish()
    }
}

impl From<char> for RawKey {
    #[inline(always)]
    fn from(ch: char) -> Self {
        let is_upper = if ch.is_ascii() {
            ch.is_ascii_uppercase()
        } else {
            ch.is_uppercase()
        };
        Self::new(ch, is_upper)
    }
}

/// Branchless UTF-16 code unit count directly over valid UTF-8 bytes.
/// Leading bytes satisfy `(b as i8) >= -0x40`; 4-byte supplementary leads (`b >= 0xF0`) add 1 extra unit.
#[inline(always)]
fn utf16_len_utf8(bytes: &[u8]) -> usize {
    let mut count = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        count += ((b as i8) >= -0x40) as usize + (b >= 0xF0) as usize;
        i += 1;
    }
    count
}

/// Result of evaluating a sequence of raw keys in the typing buffer
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub state: SyllableState,
    pub rendered: CompactString,
    pub emitted_len: usize,
    pub effective_raw_keys: InlineList<RawKey, 32>,
    pub last_action: EngineAction,
}

/// TypingBuffer managing word state with raw_keys as the Single Source of Truth.
/// SyllableState is deterministically projected from raw_keys.
#[derive(Debug, Clone)]
pub struct TypingBuffer {
    /// History of raw keys entered in this word session (Single Source of Truth)
    pub raw_keys: InlineList<RawKey, 32>,
    /// Projected state in the state machine (Empty, Onset, Nucleus, Coda, Passthrough)
    pub state: SyllableState,
    /// Length in UTF-16 code units of the rendered text currently emitted to the screen
    pub emitted_len: usize,
    /// Cached rendered string of the current state on the stack, avoiding re-renders and heap allocations
    pub last_rendered: CompactString,
    /// Whether this buffer was restored from history across a space via backspace
    pub is_restored_across_space: bool,
    /// Incremental flag tracking whether `raw_keys` contains `'_'` (for O(1) `is_boundary`)
    has_underscore: bool,
    /// Incremental flag tracking whether `raw_keys` contains any lowercase alphabetic key (for O(1) `is_boundary`)
    has_lower_alpha: bool,
}

impl Default for TypingBuffer {
    fn default() -> Self {
        Self {
            raw_keys: InlineList::new(),
            state: SyllableState::Empty,
            emitted_len: 0,
            last_rendered: CompactString::new(),
            is_restored_across_space: false,
            has_underscore: false,
            has_lower_alpha: false,
        }
    }
}

const fn make_word_break_mask(include_brackets: bool) -> u128 {
    let chars: &[u8] = if include_brackets {
        b".,;:!?\"'()[]{}/\\+=*<>@#$%^&`~|-"
    } else {
        b".,;:!?\"'(){}/\\+=*<>@#$%^&`~|-"
    };
    let mut mask = 0u128;
    let mut i = 0;
    while i < chars.len() {
        mask |= 1u128 << chars[i];
        i += 1;
    }
    mask
}

const WORD_BREAK_MASK: u128 = make_word_break_mask(true);
const WORD_BREAK_NO_BRACKETS_MASK: u128 = make_word_break_mask(false);

impl TypingBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks whether `c` is a word break character (punctuation, symbols) in O(1) time
    /// via a zero-branching 128-bit bitmask lookup.
    #[inline(always)]
    pub fn is_word_break(c: char, bracket_w: bool) -> bool {
        let cp = c as u32;
        if cp < 128 {
            let mask = if bracket_w {
                WORD_BREAK_NO_BRACKETS_MASK
            } else {
                WORD_BREAK_MASK
            };
            (mask & (1u128 << cp)) != 0
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn rendered(&self) -> &str {
        self.last_rendered.as_str()
    }

    #[inline(always)]
    pub fn is_passthrough(&self) -> bool {
        matches!(self.state, SyllableState::Passthrough(_))
    }

    /// Checks whether the current buffer represents a valid Vietnamese syllable (or valid shorthand/ASCII passthrough).
    #[inline]
    pub fn is_valid_spelling(&self) -> bool {
        if self.is_passthrough() {
            let s = self.last_rendered.as_str();
            if s.is_ascii() || crate::vietnamese::spelling::is_valid_vietnamese_shorthand_str(s) {
                return true;
            }
            // TCVN3 encoded `đ`/`Đ` followed by one ASCII letter.
            let b = s.as_bytes();
            b.len() == 3
                && b[0] == 0xC2
                && matches!(b[1], 0xAE | 0xA7)
                && b[2].is_ascii_alphabetic()
        } else {
            self.state.is_valid_spelling()
        }
    }

    #[inline(always)]
    pub fn push_raw_key(&mut self, key: RawKey) {
        let ch = key.ch();
        if ch == '_' {
            self.has_underscore = true;
        } else if !key.is_upper() {
            let is_alpha = if ch.is_ascii() {
                ch.is_ascii_lowercase()
            } else {
                ch.is_alphabetic()
            };
            if is_alpha {
                self.has_lower_alpha = true;
            }
        }
        self.raw_keys.push(key);
    }

    #[inline]
    pub(crate) fn rebuild_key_flags(&mut self) {
        let mut has_u = false;
        let mut has_l = false;
        for &k in &self.raw_keys {
            let ch = k.ch();
            if ch == '_' {
                has_u = true;
            } else if !k.is_upper() {
                let is_alpha = if ch.is_ascii() {
                    ch.is_ascii_lowercase()
                } else {
                    ch.is_alphabetic()
                };
                if is_alpha {
                    has_l = true;
                }
            }
        }
        self.has_underscore = has_u;
        self.has_lower_alpha = has_l;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.raw_keys.clear();
        self.state = SyllableState::Empty;
        self.emitted_len = 0;
        self.last_rendered.clear();
        self.is_restored_across_space = false;
        self.has_underscore = false;
        self.has_lower_alpha = false;
    }

    pub fn restore_as_passthrough(&mut self, keys: &[RawKey]) {
        self.last_rendered.clear();
        let mut emitted_len = 0;
        for &k in keys {
            let ch = k.ch();
            self.last_rendered.push(ch);
            emitted_len += ch.len_utf16();
        }
        self.emitted_len = emitted_len;
        self.state = SyllableState::passthrough();
        self.raw_keys = InlineList::from_slice(keys);
        self.rebuild_key_flags();
        self.is_restored_across_space = true;
    }

    #[inline]
    pub fn restore_snapshot(
        &mut self,
        keys: &[RawKey],
        state: SyllableState,
        rendered: &str,
        emitted_len: usize,
    ) {
        self.raw_keys = InlineList::from_slice(keys);
        self.state = state;
        self.last_rendered.clear();
        self.last_rendered.push_str(rendered);
        self.emitted_len = emitted_len;
        self.rebuild_key_flags();
        self.is_restored_across_space = true;
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.raw_keys.is_empty()
    }

    /// Returns true if the rendered string on screen matches the sequence of raw keys exactly.
    ///
    /// Evaluated in O(1) via length check fast-path when lengths differ, avoiding repeated UTF-8 iteration.
    #[inline]
    pub fn is_rendered_identical_to_raw(&self) -> bool {
        if self.raw_keys.len() != self.emitted_len {
            false
        } else {
            let rendered_bytes = self.last_rendered.as_bytes();
            if rendered_bytes.len() != self.raw_keys.len() {
                return false;
            }
            rendered_bytes
                .iter()
                .zip(self.raw_keys.iter())
                .all(|(&b, k)| b as char == k.ch())
        }
    }

    /// Checks if a transition into Passthrough should trigger an instant raw-key rollback
    /// for English words (Smart English Word Bypass), and returns the rolled back state and action if so.
    #[inline]
    fn try_smart_english_rollback(
        config: &EngineConfig,
        was_nucleus_or_coda: bool,
        current_state: &SyllableState,
        next_state: &SyllableState,
        action: &EngineAction,
        last_rendered: &str,
        emitted_len: usize,
        raw_keys_prefix: &[RawKey],
        all_raw_keys: &[RawKey],
    ) -> Option<(SyllableState, EngineAction)> {
        if config.restore_on_wrong_spelling
            && was_nucleus_or_coda
            && matches!(next_state, SyllableState::Passthrough(_))
            && matches!(action, EngineAction::Passthrough)
            && !current_state.is_unambiguous_completed_vietnamese()
        {
            let is_same_as_raw = if emitted_len != raw_keys_prefix.len()
                || last_rendered.len() != raw_keys_prefix.len()
            {
                false
            } else {
                last_rendered
                    .as_bytes()
                    .iter()
                    .zip(raw_keys_prefix.iter())
                    .all(|(&b, k)| b as char == k.ch())
            };
            if !is_same_as_raw {
                let mut output = CompactString::new();
                for &k in all_raw_keys {
                    output.push(k.ch());
                }
                return Some((
                    SyllableState::passthrough(),
                    EngineAction::Replace {
                        backspaces: emitted_len,
                        output,
                    },
                ));
            }
        }
        None
    }

    /// Evaluates a sequence of RawKeys deterministically and returns an `EvaluationResult`.
    pub fn evaluate_keys(keys: &[RawKey], config: &EngineConfig) -> EvaluationResult {
        let mut state = SyllableState::Empty;
        let mut current_len = 0;
        let mut effective = InlineList::<RawKey, 32>::new();
        let mut last_action = EngineAction::Passthrough;
        let mut last_rendered = CompactString::new();

        for (idx, &key) in keys.iter().enumerate() {
            effective.push(key);
            let was_nucleus_or_coda =
                matches!(state, SyllableState::Nucleus(_) | SyllableState::Coda(_));

            let (mut next_state, mut action) = state.feed_key(key, current_len, config);

            // Smart English Word Bypass / Instant Restore on Wrong Spelling:
            if let Some((rollback_state, rollback_action)) = Self::try_smart_english_rollback(
                config,
                was_nucleus_or_coda,
                &state,
                &next_state,
                &action,
                &last_rendered,
                current_len,
                &keys[..idx],
                &keys[..=idx],
            ) {
                next_state = rollback_state;
                action = rollback_action;
            }

            // Synchronize effective Unicode keys BEFORE TCVN3 encoding mutates `output`.
            if let EngineAction::Replace { ref output, .. } = action {
                if matches!(next_state, SyllableState::Passthrough(_)) {
                    effective.clear();
                    for c in output.chars() {
                        effective.push(RawKey::from(c));
                    }
                } else if matches!(key.ch(), 'z' | 'Z' | '0') && output.is_ascii() {
                    effective.clear();
                    for &b in output.as_bytes() {
                        effective.push(RawKey::from(b as char));
                    }
                }
            }

            if config.charset != crate::vietnamese::Charset::Unicode {
                if let EngineAction::Replace { ref mut output, .. } = action {
                    *output = config.charset.encode_text(output.as_str());
                }
            }

            state = next_state;
            match &action {
                EngineAction::Replace { output, .. } => {
                    current_len = utf16_len_utf8(output.as_bytes());
                    last_rendered.clear();
                    last_rendered.push_str(output);
                }
                EngineAction::Passthrough => {
                    let ch = key.ch();
                    current_len += ch.len_utf16();
                    last_rendered.push(ch);
                }
                EngineAction::Consume => {}
            }
            last_action = action;
        }

        let rendered =
            if matches!(state, SyllableState::Passthrough(_)) && !last_rendered.is_empty() {
                last_rendered
            } else {
                let s = state.render_compact();
                if config.charset != crate::vietnamese::Charset::Unicode {
                    config.charset.encode_text(s.as_str())
                } else {
                    s
                }
            };

        EvaluationResult {
            state,
            rendered,
            emitted_len: current_len,
            effective_raw_keys: effective,
            last_action,
        }
    }

    /// Checks whether `key` marks a word boundary (e.g. CamelCase / PascalCase boundary) in O(1) time.
    #[inline(always)]
    pub fn is_boundary(&self, key: &RawKey) -> bool {
        key.is_upper() && !self.has_underscore && self.has_lower_alpha
    }

    /// Feeds a new key into the state machine and returns the resulting EngineAction.
    #[inline]
    pub fn feed_key(&mut self, key: RawKey, config: &EngineConfig) -> EngineAction {
        self.push_raw_key(key);

        let current_state = self.state;
        let was_nucleus_or_coda = matches!(
            current_state,
            SyllableState::Nucleus(_) | SyllableState::Coda(_)
        );

        let (mut next_state, mut action) = current_state.feed_key(key, self.emitted_len, config);

        // Smart English Word Bypass / Instant Restore on Wrong Spelling:
        if let Some((rollback_state, rollback_action)) = Self::try_smart_english_rollback(
            config,
            was_nucleus_or_coda,
            &current_state,
            &next_state,
            &action,
            &self.last_rendered,
            self.emitted_len,
            &self.raw_keys[..self.raw_keys.len() - 1],
            &self.raw_keys,
        ) {
            next_state = rollback_state;
            action = rollback_action;
        }

        // Synchronize `raw_keys` from full Unicode `output` BEFORE TCVN3 encoding or Common Prefix truncation
        // when an undo toggle / English rollback enters Passthrough or mark removal strips back to ASCII.
        if let EngineAction::Replace { ref output, .. } = action {
            if matches!(next_state, SyllableState::Passthrough(_)) {
                self.raw_keys.clear();
                for c in output.chars() {
                    self.raw_keys.push(RawKey::from(c));
                }
                self.rebuild_key_flags();
            } else if matches!(key.ch(), 'z' | 'Z' | '0') && output.is_ascii() {
                self.raw_keys.clear();
                for &b in output.as_bytes() {
                    self.raw_keys.push(RawKey::from(b as char));
                }
                self.rebuild_key_flags();
            }
        }

        if config.charset != crate::vietnamese::Charset::Unicode {
            if let EngineAction::Replace { ref mut output, .. } = action {
                *output = config.charset.encode_text(output.as_str());
            }
        }

        self.state = next_state;

        // Raw-Byte Single-Pass Common Prefix Optimization:
        // Compares raw UTF-8 bytes directly, rewinds at most 1-3 continuation bytes if split inside
        // a multi-byte scalar (e.g. 'ấ' vs 'ầ'), and counts UTF-16 length ONLY on the changed suffix!
        let (action, new_emitted_len) = match action {
            EngineAction::Replace {
                backspaces,
                mut output,
            } => {
                let prev_bytes = self.last_rendered.as_bytes();
                let new_bytes = output.as_bytes();
                let min_len = prev_bytes.len().min(new_bytes.len());
                let mut common_bytes = 0usize;
                while common_bytes < min_len && prev_bytes[common_bytes] == new_bytes[common_bytes]
                {
                    common_bytes += 1;
                }

                if common_bytes > 0
                    && ((common_bytes < prev_bytes.len()
                        && (prev_bytes[common_bytes] as i8) < -0x40)
                        || (common_bytes < new_bytes.len()
                            && (new_bytes[common_bytes] as i8) < -0x40))
                {
                    while common_bytes > 0 && (prev_bytes[common_bytes - 1] as i8) < -0x40 {
                        common_bytes -= 1;
                    }
                    common_bytes = common_bytes.saturating_sub(1);
                }

                if common_bytes > 0 {
                    let opt_backspaces = utf16_len_utf8(&prev_bytes[common_bytes..]);
                    let added_utf16 = utf16_len_utf8(&new_bytes[common_bytes..]);
                    let updated_len = self.emitted_len - opt_backspaces + added_utf16;
                    output.drain(..common_bytes);
                    self.last_rendered.truncate(common_bytes);
                    self.last_rendered.push_str(&output);
                    (
                        EngineAction::Replace {
                            backspaces: opt_backspaces,
                            output,
                        },
                        updated_len,
                    )
                } else {
                    let full_output_utf16 = utf16_len_utf8(new_bytes);
                    self.last_rendered.clear();
                    self.last_rendered.push_str(&output);
                    (
                        EngineAction::Replace { backspaces, output },
                        full_output_utf16,
                    )
                }
            }
            EngineAction::Passthrough => {
                let ch = key.ch();
                self.last_rendered.push(ch);
                (EngineAction::Passthrough, self.emitted_len + ch.len_utf16())
            }
            EngineAction::Consume => (EngineAction::Consume, self.emitted_len),
        };

        self.emitted_len = new_emitted_len;
        action
    }

    /// Handles a backspace event: pops raw_keys until rendered length drops by 1,
    /// re-projecting the state deterministically from raw_keys.
    pub fn handle_backspace(&mut self, config: &EngineConfig) -> bool {
        if self.is_empty() {
            return false;
        }

        if self.is_passthrough() {
            self.raw_keys.pop();
            self.last_rendered.pop();
            self.emitted_len = utf16_len_utf8(self.last_rendered.as_bytes());

            if self.raw_keys.is_empty() {
                self.clear();
            } else {
                // Check if the remaining raw keys can legitimately recover a structured Vietnamese syllable
                // that EXACTLY matches the text on screen (e.g. "tjee" backspaced to "t" matches Onset('t')).
                let eval = Self::evaluate_keys(&self.raw_keys, config);
                if eval.rendered == self.last_rendered {
                    self.raw_keys = eval.effective_raw_keys;
                    self.state = eval.state;
                    self.emitted_len = eval.emitted_len;
                    self.last_rendered = eval.rendered;
                } else {
                    self.state = SyllableState::passthrough();
                }
                self.rebuild_key_flags();
            }
            return true;
        }

        let target_len = self.emitted_len.saturating_sub(1);
        while self.emitted_len > target_len && !self.raw_keys.is_empty() {
            self.raw_keys.pop();
            let eval = Self::evaluate_keys(&self.raw_keys, config);
            self.raw_keys = eval.effective_raw_keys;
            self.state = eval.state;
            self.emitted_len = eval.emitted_len;
            self.last_rendered = eval.rendered;
        }

        if self.raw_keys.is_empty() {
            self.clear();
        } else {
            self.rebuild_key_flags();
        }

        true
    }
}

