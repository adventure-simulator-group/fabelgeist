//! Sorting cardinality, tile layout, and digit schedule own their operations.
use super::{BITS_PER_PASS, RADIX, TILE};
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameters};

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
        (self.0 as u64 * size_of::<u32>() as u64).into()
    }
    pub(super) fn tiles(self) -> SortTileCount {
        SortTileCount(self.0.div_ceil(TILE))
    }
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("count", self.0);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SortTileCount(u32);
impl SortTileCount {
    pub(super) fn histogram_bytes(self) -> BufferByteLength {
        (RADIX as u64 * self.0 as u64 * size_of::<u32>() as u64).into()
    }
    pub(super) fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("tiles", self.0);
    }
}

// The existing compute provider admits a native grid at recording.
impl From<SortTileCount> for [u32; 3] {
    fn from(tiles: SortTileCount) -> Self {
        [tiles.0, 1, 1]
    }
}

/// Declared low-bit width, retaining the original clamp to one through 32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortKeyWidth(u32);
impl From<u32> for SortKeyWidth {
    fn from(bits: u32) -> Self {
        Self(bits.clamp(1, u32::BITS))
    }
}
impl SortKeyWidth {
    pub fn pass_count(self) -> SortPassCount {
        SortPassCount(self.0.div_ceil(BITS_PER_PASS))
    }
    pub fn digits(self) -> SortDigits {
        SortDigits {
            next: SortPassCount(0),
            end: self.pass_count(),
        }
    }
}
/// Number of whole eight-bit digits in an admitted key-width schedule.
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
        parameters.insert("shift", self.0);
    }
}
/// The ordered low-bit shifts selected by an admitted key width.
pub struct SortDigits {
    next: SortPassCount,
    end: SortPassCount,
}
impl Iterator for SortDigits {
    type Item = SortDigit;
    fn next(&mut self) -> Option<SortDigit> {
        if self.next.0 >= self.end.0 {
            return None;
        }
        let digit = SortDigit(self.next.0 * BITS_PER_PASS);
        self.next.0 += 1;
        Some(digit)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchGrowth {
    Unchanged,
    Grown,
}
