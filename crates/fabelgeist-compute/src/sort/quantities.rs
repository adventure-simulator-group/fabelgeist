//! Sorting cardinality, tile layout, and digit schedule own their operations.
use super::{BITS_PER_PASS, RADIX, TILE};
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameters, WorkgroupGrid};

/// Number of key/payload pairs; zero and one are valid no-op requests.
///
/// ```compile_fail
/// use fabelgeist_compute::{SortItemCount, SortKeyWidth};
/// fn items(_: SortItemCount) {}
/// items(SortKeyWidth::from(32));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortItemCount(u32);
impl From<u32> for SortItemCount {
    fn from(items: u32) -> Self {
        Self(items)
    }
}
impl std::fmt::Display for SortItemCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SortWork {
    NoOp,
    Required,
}
impl SortItemCount {
    pub(super) fn work(self) -> SortWork {
        if self.0 <= 1 {
            SortWork::NoOp
        } else {
            SortWork::Required
        }
    }
    pub(super) fn with_sentinel(self) -> Self {
        Self(self.0.max(1))
    }
    pub fn word_bytes(self) -> BufferByteLength {
        (self.0 as u64 * 4).into()
    }
    pub(super) fn tiles(self) -> SortTileCount {
        SortTileCount(self.0.div_ceil(TILE))
    }
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("count".into(), self.0.into());
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SortTileCount(u32);
impl SortTileCount {
    pub(super) fn histogram_bytes(self) -> BufferByteLength {
        (RADIX as u64 * self.0 as u64 * 4).into()
    }
    pub(super) fn grid(self) -> WorkgroupGrid {
        [self.0, 1, 1].into()
    }
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("tiles".into(), self.0.into());
    }
}

/// Declared low-bit width, retaining the original clamp to one through 32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortKeyWidth(u32);
impl From<u32> for SortKeyWidth {
    fn from(bits: u32) -> Self {
        Self(bits.clamp(1, 32))
    }
}
impl SortKeyWidth {
    pub fn pass_count(self) -> SortPassCount {
        SortPassCount(self.0.div_ceil(BITS_PER_PASS))
    }
    pub fn digits(self) -> SortDigits {
        SortDigits {
            next: 0,
            end: self.pass_count(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortPassCount(u32);
impl std::fmt::Display for SortPassCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SortCopyBack {
    Required,
    Unneeded,
}
impl SortPassCount {
    pub(super) fn copy_back(self) -> SortCopyBack {
        if self.0 % 2 == 1 {
            SortCopyBack::Required
        } else {
            SortCopyBack::Unneeded
        }
    }
}
/// One digit's low-bit shift, derived from the admitted width's schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortDigit(u32);
impl SortDigit {
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("shift".into(), self.0.into());
    }
}
pub struct SortDigits {
    next: u32,
    end: SortPassCount,
}
impl Iterator for SortDigits {
    type Item = SortDigit;
    fn next(&mut self) -> Option<SortDigit> {
        if self.next >= self.end.0 {
            return None;
        }
        let digit = SortDigit(self.next * BITS_PER_PASS);
        self.next += 1;
        Some(digit)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchGrowth {
    Unchanged,
    Grown,
}
