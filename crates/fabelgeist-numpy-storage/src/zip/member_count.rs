//! Application count bound and capped native directory reservation.

use derive_more::From;

/// The count selected by rawzip, retained without native-width narrowing.
#[derive(Debug, Clone, Copy, From, PartialEq, Eq)]
#[from(u16, u64)]
pub(super) struct ArchiveMemberCount(u64);

impl ArchiveMemberCount {
    // The inherited allocation cap is not a directory cardinality limit.
    const INITIAL_CAPACITY_LIMIT: Self = Self(4096);

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
        capacity: usize,
    }

    #[test]
    fn ordinary_fields_preserve_initial_reservation() {
        let cases = [
            OrdinaryCase {
                wire: 0,
                capacity: 0,
            },
            OrdinaryCase {
                wire: 1,
                capacity: 1,
            },
            OrdinaryCase {
                wire: 4095,
                capacity: 4095,
            },
            OrdinaryCase {
                wire: 4096,
                capacity: 4096,
            },
            OrdinaryCase {
                wire: 4097,
                capacity: 4096,
            },
            OrdinaryCase {
                wire: 65534,
                capacity: 4096,
            },
            OrdinaryCase {
                wire: 65535,
                capacity: 4096,
            },
        ];
        for case in cases {
            let count = ArchiveMemberCount::from(case.wire);
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
