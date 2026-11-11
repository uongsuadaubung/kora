use super::{
    action::CompactString,
    buffer::{RawKey, TypingBuffer},
};
use crate::vietnamese::{InlineList, SyllableState};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommittedWord {
    pub raw_keys: InlineList<RawKey, 32>,
    pub state: SyllableState,
    pub rendered: CompactString,
    pub emitted_len: u16,
    pub is_raw_restored: bool,
    pub spaces_after: u16,
}

pub const MAX_HISTORY_WORDS: usize = 50;

/// History tracker to support O(1) Backspace across spaces and word restoration
#[derive(Debug, Clone, Default)]
pub struct WordHistory {
    committed_words: VecDeque<CommittedWord>,
}

/// Result of handling a Backspace when the active word buffer is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryBackspace {
    /// History is completely empty or no trailing space to backspace across
    Empty,
    /// A trailing space was consumed (nothing to restore yet)
    ConsumedSpace,
    /// The last trailing space was consumed and the previous word snapshot was popped
    RestoredWord(CommittedWord),
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

    /// Record a newly completed word directly from the active `TypingBuffer` slice without copying unused stack capacity.
    #[inline]
    pub fn commit_buffer(&mut self, buffer: &TypingBuffer, is_raw_restored: bool) {
        let keys = buffer.raw_keys.as_slice();
        if !keys.is_empty() {
            if self.committed_words.len() >= MAX_HISTORY_WORDS {
                self.committed_words.pop_front();
            }
            self.committed_words.push_back(CommittedWord {
                raw_keys: InlineList::from_slice(keys),
                state: buffer.state,
                rendered: CompactString::from(buffer.rendered()),
                emitted_len: buffer.emitted_len as u16,
                is_raw_restored,
                spaces_after: 0,
            });
        }
    }

    /// Record space(s) typed after words
    #[inline(always)]
    pub fn add_space(&mut self) {
        if let Some(last) = self.committed_words.back_mut() {
            last.spaces_after = last.spaces_after.saturating_add(1);
        }
    }

    pub fn has_trailing_spaces(&self) -> bool {
        self.committed_words
            .back()
            .is_some_and(|w| w.spaces_after > 0)
    }

    /// Handles a backspace when the active word buffer is empty.
    #[inline]
    pub fn pop_backspace(&mut self) -> HistoryBackspace {
        if let Some(last) = self.committed_words.back_mut() {
            if last.spaces_after > 1 {
                last.spaces_after -= 1;
                return HistoryBackspace::ConsumedSpace;
            }
            if last.spaces_after == 1 {
                let word = self.committed_words.pop_back().unwrap();
                return HistoryBackspace::RestoredWord(word);
            }
        }
        HistoryBackspace::Empty
    }
}

