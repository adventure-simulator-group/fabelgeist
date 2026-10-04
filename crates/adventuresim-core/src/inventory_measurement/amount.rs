//! Total measured inventory amount, distinct from one row's bounded fraction.

use super::ConsumableFractionMicros;

/// One-millionth stock-item units across any number of measured rows.
///
/// The native unsigned width and zero amount are intentional. Row requests are
/// capped at one whole; aggregate addition retains checked or saturating policy.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(transparent)]
pub struct MeasuredItemAmountMicros(u32);

impl MeasuredItemAmountMicros {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub const fn checked_sub_fraction(self, consumed: ConsumableFractionMicros) -> Option<Self> {
        match self.0.checked_sub(consumed.get()) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub fn row_request(self) -> ConsumableFractionMicros {
        ConsumableFractionMicros::try_new(self.0.min(ConsumableFractionMicros::MICROS_PER_WHOLE))
            .expect("a capped request cannot exceed one measured row")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_row_requests_conserve_the_exact_total_and_bound_each_row() {
        for raw in [0, 1, 999_999, 1_000_000, 1_000_001, 2_345_678, u32::MAX] {
            let requested = MeasuredItemAmountMicros::new(raw);
            let mut remaining = requested;
            let mut consumed = MeasuredItemAmountMicros::ZERO;
            while remaining != MeasuredItemAmountMicros::ZERO {
                let row = remaining.row_request();
                assert!(row <= ConsumableFractionMicros::WHOLE);
                remaining = remaining.checked_sub_fraction(row).unwrap();
                consumed = consumed
                    .checked_add(MeasuredItemAmountMicros::new(row.get()))
                    .unwrap();
            }
            assert_eq!(consumed, requested);
        }
    }

    #[test]
    fn checked_and_saturating_totals_keep_their_distinct_overflow_policy() {
        let limit = MeasuredItemAmountMicros::new(u32::MAX);
        let one = MeasuredItemAmountMicros::new(1);
        assert_eq!(limit.checked_add(one), None);
        assert_eq!(limit.saturating_add(one), limit);
        assert_eq!(
            MeasuredItemAmountMicros::ZERO
                .checked_sub_fraction(ConsumableFractionMicros::MINIMUM_NONZERO),
            None
        );
    }
}
