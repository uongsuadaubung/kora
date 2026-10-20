use std::fmt;
use std::ops::{Deref, RangeBounds};

/// Compact inline-or-heap string representation for zero-allocation engine actions.
/// When length <= 24 bytes (which covers 100% of Vietnamese syllables and words),
/// it is stored completely on the Stack with zero heap allocation.
///
/// Automatically upgrades to Heap String when length exceeds 24 bytes (e.g. long macros).
#[derive(Clone, Eq)]
pub enum CompactString {
    Inline { buf: [u8; 24], len: u8 },
    Heap(String),
}

impl CompactString {
    #[inline]
    pub const fn new() -> Self {
        Self::Inline {
            buf: [0u8; 24],
            len: 0,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Inline { buf, len } => unsafe {
                std::str::from_utf8_unchecked(&buf[..*len as usize])
            },
            Self::Heap(s) => s.as_str(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len as usize,
            Self::Heap(s) => s.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn clear(&mut self) {
        match self {
            Self::Inline { len, .. } => *len = 0,
            Self::Heap(s) => s.clear(),
        }
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        match self {
            Self::Inline { buf, len } => {
                let cur = *len as usize;
                if new_len < cur && (new_len == 0 || (buf[new_len] as i8) >= -0x40) {
                    *len = new_len as u8;
                }
            }
            Self::Heap(s) => s.truncate(new_len),
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        match self {
            Self::Inline { buf, len } => {
                let s = unsafe { std::str::from_utf8_unchecked(&buf[..*len as usize]) };
                let ch = s.chars().next_back()?;
                *len -= ch.len_utf8() as u8;
                Some(ch)
            }
            Self::Heap(s) => s.pop(),
        }
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        let ch_len = ch.len_utf8();
        match self {
            Self::Inline { buf, len } => {
                let cur = *len as usize;
                if cur + ch_len <= 24 {
                    ch.encode_utf8(&mut buf[cur..]);
                    *len += ch_len as u8;
                } else {
                    let mut s = String::with_capacity(cur + ch_len + 8);
                    s.push_str(unsafe { std::str::from_utf8_unchecked(&buf[..cur]) });
                    s.push(ch);
                    *self = Self::Heap(s);
                }
            }
            Self::Heap(s) => s.push(ch),
        }
    }

    #[inline]
    pub fn push_str(&mut self, text: &str) {
        let text_len = text.len();
        match self {
            Self::Inline { buf, len } => {
                let cur = *len as usize;
                if cur + text_len <= 24 {
                    buf[cur..cur + text_len].copy_from_slice(text.as_bytes());
                    *len += text_len as u8;
                } else {
                    let mut s = String::with_capacity(cur + text_len + 8);
                    s.push_str(unsafe { std::str::from_utf8_unchecked(&buf[..cur]) });
                    s.push_str(text);
                    *self = Self::Heap(s);
                }
            }
            Self::Heap(s) => s.push_str(text),
        }
    }

    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) {
        match self {
            Self::Inline { buf, len } => {
                use std::ops::Bound;
                let cur = *len as usize;
                let start = match range.start_bound() {
                    Bound::Included(&n) => n,
                    Bound::Excluded(&n) => n + 1,
                    Bound::Unbounded => 0,
                };
                let end = match range.end_bound() {
                    Bound::Included(&n) => n + 1,
                    Bound::Excluded(&n) => n,
                    Bound::Unbounded => cur,
                };
                if start <= end
                    && end <= cur
                    && (start == 0 || start == cur || (buf[start] as i8) >= -0x40)
                    && (end == 0 || end == cur || (buf[end] as i8) >= -0x40)
                {
                    let count = end - start;
                    buf.copy_within(end..cur, start);
                    *len -= count as u8;
                }
            }
            Self::Heap(s) => {
                s.drain(range);
            }
        }
    }
}

impl FromIterator<char> for CompactString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut s = Self::new();
        for ch in iter {
            s.push(ch);
        }
        s
    }
}

impl Extend<char> for CompactString {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for ch in iter {
            self.push(ch);
        }
    }
}

impl Default for CompactString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CompactString {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for CompactString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for CompactString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for CompactString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for CompactString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq for CompactString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for CompactString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for CompactString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<CompactString> for &str {
    #[inline]
    fn eq(&self, other: &CompactString) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<String> for CompactString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<CompactString> for String {
    #[inline]
    fn eq(&self, other: &CompactString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&str> for CompactString {
    #[inline]
    fn from(s: &str) -> Self {
        if s.len() <= 24 {
            let mut buf = [0u8; 24];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            Self::Inline {
                buf,
                len: s.len() as u8,
            }
        } else {
            Self::Heap(s.to_string())
        }
    }
}

impl From<String> for CompactString {
    #[inline]
    fn from(s: String) -> Self {
        if s.len() <= 24 {
            Self::from(s.as_str())
        } else {
            Self::Heap(s)
        }
    }
}

impl From<char> for CompactString {
    #[inline]
    fn from(c: char) -> Self {
        let mut s = Self::new();
        s.push(c);
        s
    }
}

/// Action returned by the engine to tell the platform layer how to handle a keystroke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    /// Let the OS handle the keystroke normally without interference.
    Passthrough,

    /// Intercept the keystroke, send `backspaces` count to delete previous chars,
    /// then emit `output` string into the active input field.
    ///
    /// `output` is stored as `CompactString` (stack-allocated up to 24 bytes, 0 heap churn for 100% of Vietnamese words).
    Replace {
        backspaces: usize,
        output: CompactString,
    },

    /// Consume (drop) the keystroke entirely. Do not let OS process it.
    Consume,
}
