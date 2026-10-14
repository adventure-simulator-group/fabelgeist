//! Declared ZIP directory cardinality and its native allocation/iteration ports.

/// A declared member count, preserving every ordinary or ZIP64 wire value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ArchiveMemberCount(u64);

impl From<u16> for ArchiveMemberCount {
    fn from(value: u16) -> Self {
        Self(u64::from(value))
    }
}

impl From<u64> for ArchiveMemberCount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl ArchiveMemberCount {
    // The inherited allocation cap is not a directory cardinality limit.
    const INITIAL_CAPACITY_LIMIT: Self = Self(4096);

    /// Whether the ordinary field requests ZIP64 count/offset substitution.
    pub(super) fn is_saturated_ordinary(self) -> bool {
        self == Self::from(u16::MAX)
    }

    /// Projects only the capped count into native `Vec` allocation capacity.
    pub(super) fn initial_capacity(self) -> usize {
        self.0.min(Self::INITIAL_CAPACITY_LIMIT.0) as usize
    }

    /// Projects into the original native u64 range; callers ignore its ordinal.
    ///
    /// The count is never narrowed to usize for directory iteration.
    pub(super) fn native_iteration_range(self) -> std::ops::Range<u64> {
        0..self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct OrdinaryCase {
        wire: u16,
        requests_zip64: bool,
        capacity: usize,
    }

    #[test]
    fn ordinary_fields_preserve_saturation_and_initial_reservation() {
        let cases = [
            OrdinaryCase {
                wire: 0,
                requests_zip64: false,
                capacity: 0,
            },
            OrdinaryCase {
                wire: 1,
                requests_zip64: false,
                capacity: 1,
            },
            OrdinaryCase {
                wire: 4095,
                requests_zip64: false,
                capacity: 4095,
            },
            OrdinaryCase {
                wire: 4096,
                requests_zip64: false,
                capacity: 4096,
            },
            OrdinaryCase {
                wire: 4097,
                requests_zip64: false,
                capacity: 4096,
            },
            OrdinaryCase {
                wire: 65534,
                requests_zip64: false,
                capacity: 4096,
            },
            OrdinaryCase {
                wire: 65535,
                requests_zip64: true,
                capacity: 4096,
            },
        ];
        for case in cases {
            let count = ArchiveMemberCount::from(case.wire);
            assert_eq!(count.is_saturated_ordinary(), case.requests_zip64);
            assert_eq!(count.initial_capacity(), case.capacity);
        }
    }

    #[test]
    fn zip64_range_retains_high_bits_and_exclusive_end_despite_capacity_cap() {
        for wire in [4096u64, 4097, 1 << 32, (1 << 32) + 1, 1 << 63, u64::MAX] {
            let count = ArchiveMemberCount::from(wire);
            let mut range = count.native_iteration_range();
            assert_eq!(range.next(), Some(0));
            assert_eq!(range.end, wire);
            assert_eq!(range.next_back(), Some(wire - 1));
            assert_eq!(count.initial_capacity(), 4096);
        }
        let empty = ArchiveMemberCount::from(0u64);
        assert_eq!(empty.native_iteration_range().next(), None);
        let single = ArchiveMemberCount::from(1u64);
        assert_eq!(single.native_iteration_range().collect::<Vec<_>>(), [0]);
        assert_eq!(single.initial_capacity(), 1);
    }
}
