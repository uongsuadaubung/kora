//! Fixed-capacity inline list allocated completely on the stack.
//! Zero heap allocations, Copy semantics when T is Copy.

use std::fmt;
use std::ops::{Deref, DerefMut};

/// An inline, fixed-capacity list of elements stored on the stack without heap allocation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InlineList<T: Copy + Default, const N: usize> {
    data: [T; N],
    len: u8,
}

impl<T: Copy + Default, const N: usize> InlineList<T, N> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_single(item: T) -> Self {
        let mut list = Self::default();
        list.push(item);
        list
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        if (self.len as usize) < N {
            self.data[self.len as usize] = item;
            self.len += 1;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.data[self.len as usize])
        } else {
            None
        }
    }

    #[inline]
    pub fn insert(&mut self, index: usize, item: T) {
        assert!(index <= self.len as usize, "index out of bounds");
        if (self.len as usize) < N {
            for i in (index..self.len as usize).rev() {
                self.data[i + 1] = self.data[i];
            }
            self.data[index] = item;
            self.len += 1;
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len as usize, "index out of bounds");
        let val = self.data[index];
        for i in index..(self.len as usize - 1) {
            self.data[i] = self.data[i + 1];
        }
        self.len -= 1;
        val
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len as usize]
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data[..self.len as usize]
    }
}

impl<T: Copy + Default, const N: usize> Default for InlineList<T, N> {
    #[inline]
    fn default() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }
}

impl<T: Copy + Default, const N: usize> Deref for InlineList<T, N> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Copy + Default, const N: usize> DerefMut for InlineList<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Copy + Default + fmt::Debug, const N: usize> fmt::Debug for InlineList<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<'a, T: Copy + Default, const N: usize> IntoIterator for &'a InlineList<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T: Copy + Default, const N: usize> IntoIterator for &'a mut InlineList<T, N> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<const N: usize> InlineList<char, N> {
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        for c in s.chars() {
            self.push(c);
        }
    }
}

impl<const N: usize> std::str::FromStr for InlineList<char, N> {
    type Err = std::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut list = Self::default();
        list.push_str(s);
        Ok(list)
    }
}

impl<const N: usize> From<&str> for InlineList<char, N> {
    #[inline]
    fn from(s: &str) -> Self {
        let mut list = Self::default();
        list.push_str(s);
        list
    }
}

impl<T: Copy + Default, const N: usize> FromIterator<T> for InlineList<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::default();
        for item in iter {
            list.push(item);
        }
        list
    }
}

