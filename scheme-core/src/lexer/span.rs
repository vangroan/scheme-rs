//! Location in source code.

use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Span {
    pub(crate) lo: usize, // inclusive
    pub(crate) hi: usize, // exclusive
}

impl Span {
    pub const fn new(lo: usize, size: usize) -> Self {
        Self { lo, hi: lo + size }
    }

    #[inline(always)]
    pub fn low(&self) -> usize {
        self.lo
    }

    #[inline(always)]
    pub fn high(&self) -> usize {
        self.hi
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.hi - self.lo
    }

    pub fn as_range(&self) -> Range<usize> {
        self.lo..self.hi
    }
}
