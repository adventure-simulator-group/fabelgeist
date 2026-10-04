//! Bounded positive harvest quantities and their individually issued units.

use crate::inventory_measurement::ItemQuantity;

/// A nonzero harvest count that fits the native receipt's quantity field.
///
/// Resolution saturates computed quantities at the receipt limit. Receipt
/// admission rejects zero; it never silently invents or discards units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForageYieldQuantity(ItemQuantity);

impl ForageYieldQuantity {
    pub(super) fn from_computed(quantity: u64) -> Result<Self, EmptyForageYield> {
        Self::try_from(u16::try_from(quantity).unwrap_or(u16::MAX))
    }

    /// Each harvested unit needs its own inventory row and material provenance.
    pub fn units(self) -> impl ExactSizeIterator<Item = ItemQuantity> {
        std::iter::repeat_n(ItemQuantity::ONE, self.0.get() as usize)
    }
}

impl TryFrom<u16> for ForageYieldQuantity {
    type Error = EmptyForageYield;

    fn try_from(quantity: u16) -> Result<Self, Self::Error> {
        ItemQuantity::new(u32::from(quantity))
            .map(Self)
            .ok_or(EmptyForageYield)
    }
}

impl From<ForageYieldQuantity> for u16 {
    fn from(quantity: ForageYieldQuantity) -> Self {
        u16::try_from(quantity.0.get()).expect("forage quantity is admitted within the wire range")
    }
}

impl From<ForageYieldQuantity> for ItemQuantity {
    fn from(quantity: ForageYieldQuantity) -> Self {
        quantity.0
    }
}

impl std::fmt::Display for ForageYieldQuantity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.get().fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyForageYield;

impl std::fmt::Display for EmptyForageYield {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("A forage yield must contain at least one unit")
    }
}

impl std::error::Error for EmptyForageYield {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computed_harvest_saturates_without_wrapping_or_admitting_zero() {
        assert_eq!(ForageYieldQuantity::from_computed(0), Err(EmptyForageYield));
        for quantity in [1, 2, 65_534, 65_535, 65_536, u64::MAX] {
            let admitted = ForageYieldQuantity::from_computed(quantity).unwrap();
            let expected = u16::try_from(quantity).unwrap_or(u16::MAX);
            assert_eq!(u16::from(admitted), expected);
            assert_eq!(admitted.to_string(), expected.to_string());
        }
    }

    #[test]
    fn every_native_quantity_round_trips_and_zero_is_rejected() {
        assert_eq!(ForageYieldQuantity::try_from(0), Err(EmptyForageYield));
        for native in 1..=u16::MAX {
            let quantity = ForageYieldQuantity::try_from(native).unwrap();
            assert_eq!(u16::from(quantity), native);
            assert_eq!(ItemQuantity::from(quantity).get(), u32::from(native));
        }
    }

    #[test]
    fn individual_issuance_conserves_the_full_harvest_count() {
        for native in [1, 2, 63, u16::MAX] {
            let quantity = ForageYieldQuantity::try_from(native).unwrap();
            let mut units = quantity.units();
            assert_eq!(units.len(), usize::from(native));
            let mut issued = 0;
            for unit in units.by_ref() {
                assert_eq!(unit, ItemQuantity::ONE);
                issued += unit.get();
            }
            assert_eq!(issued, u32::from(native));
            assert_eq!(units.len(), 0);
            assert!(units.next().is_none());
        }
    }
}
