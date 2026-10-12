use super::buffer::RawKey;
use crate::vietnamese::InlineList;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommittedWord {
    pub raw_keys: InlineList<RawKey, 32>,
    pub is_raw_restored: bool,
    pub spaces_after: usize,
}

pub const MAX_HISTORY_WORDS: usize = 50;

/// History tracker to support Backspace across spaces and word restoration
#[derive(Debug, Clone, Default)]
pub struct WordHistory {
    committed_words: VecDeque<CommittedWord>,
}

impl WordHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.committed_words.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.committed_words.is_empty()
    }

    /// Record a newly completed word
    pub fn commit_word(&mut self, raw_keys: InlineList<RawKey, 32>, is_raw_restored: bool) {
        if !raw_keys.is_empty() {
            if self.committed_words.len() >= MAX_HISTORY_WORDS {
                self.committed_words.pop_front();
            }
            self.committed_words.push_back(CommittedWord {
                raw_keys,
                is_raw_restored,
                spaces_after: 0,
            });
        }
    }

    /// Record space(s) typed after words
    pub fn add_space(&mut self) {
        if let Some(last) = self.committed_words.back_mut() {
            last.spaces_after += 1;
        }
    }

    pub fn has_trailing_spaces(&self) -> bool {
        self.committed_words
            .back()
            .is_some_and(|w| w.spaces_after > 0)
    }

    /// Handles a backspace when the active word buffer is empty.
    /// Returns:
    /// - `Some(None)` if a trailing space was consumed (nothing to restore yet)
    /// - `Some(Some((raw_keys, is_raw_restored)))` if the trailing space was the last one and the previous word was restored
    /// - `None` if history is completely empty or no space to backspace across
    pub fn pop_backspace(&mut self) -> Option<Option<(InlineList<RawKey, 32>, bool)>> {
        if let Some(last) = self.committed_words.back_mut() {
            if last.spaces_after > 1 {
                last.spaces_after -= 1;
                return Some(None);
            }
            if last.spaces_after == 1 {
                let word = self.committed_words.pop_back().unwrap();
                return Some(Some((word.raw_keys, word.is_raw_restored)));
            }
        }
        None
    }
}
