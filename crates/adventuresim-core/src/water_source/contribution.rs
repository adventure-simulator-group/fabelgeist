//! Exact private contaminant amounts sampled with their water volume.

use crate::material::Microliters;

/// Extensive contaminant load in water, measured in whole microunits.
///
/// Zero load represents clean water. This is a total load, not concentration;
/// sampling must follow the actual integer volume transferred.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct WaterContaminantMicrounits(u64);

impl WaterContaminantMicrounits {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

/// One private water contribution, including its conserved contaminant load.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaterMaterialContribution {
    volume: Microliters,
    contaminant_load: WaterContaminantMicrounits,
}

impl WaterMaterialContribution {
    pub const fn new(volume: Microliters, contaminant_load: WaterContaminantMicrounits) -> Self {
        Self {
            volume,
            contaminant_load,
        }
    }

    pub const fn volume(self) -> Microliters {
        self.volume
    }

    pub const fn contaminant_load(self) -> WaterContaminantMicrounits {
        self.contaminant_load
    }

    /// Sample by the public holding's transfer fraction. Integer remainders
    /// stay at the source, and load follows the rounded transferred volume.
    pub fn sample(self, public_total: Microliters, moved: Microliters) -> Option<Self> {
        if public_total.is_zero() || moved > public_total {
            return None;
        }
        if moved == public_total {
            return Some(self);
        }
        let amount = Microliters::new(
            (u128::from(self.volume.get()) * u128::from(moved.get())
                / u128::from(public_total.get())) as u64,
        );
        let load = if amount == self.volume {
            self.contaminant_load
        } else if self.volume.is_zero() {
            WaterContaminantMicrounits::ZERO
        } else {
            WaterContaminantMicrounits::new(
                (u128::from(self.contaminant_load.get()) * u128::from(amount.get())
                    / u128::from(self.volume.get())) as u64,
            )
        };
        Some(Self::new(amount, load))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tainted_and_implicit_clean_water_are_sampled_proportionally() {
        let source = WaterMaterialContribution::new(
            Microliters::new(100_000),
            WaterContaminantMicrounits::new(12_000_000),
        );
        assert_eq!(
            source.sample(Microliters::new(1_000_000), Microliters::new(100_000)),
            Some(WaterMaterialContribution::new(
                Microliters::new(10_000),
                WaterContaminantMicrounits::new(1_200_000),
            ))
        );
        assert_eq!(
            source.sample(Microliters::new(900_000), Microliters::new(900_000)),
            Some(source)
        );
    }

    #[test]
    fn sampling_retains_integer_remainders_and_rejects_invalid_public_volume() {
        let source = WaterMaterialContribution::new(
            Microliters::new(7),
            WaterContaminantMicrounits::new(13),
        );
        let selected = source
            .sample(Microliters::new(11), Microliters::new(5))
            .unwrap();
        assert_eq!(selected.volume(), Microliters::new(3));
        assert_eq!(
            selected.contaminant_load(),
            WaterContaminantMicrounits::new(5)
        );
        assert_eq!(
            source
                .contaminant_load()
                .checked_sub(selected.contaminant_load()),
            Some(WaterContaminantMicrounits::new(8))
        );
        assert_eq!(source.sample(Microliters::ZERO, Microliters::ZERO), None);
        assert_eq!(
            source.sample(Microliters::new(11), Microliters::new(12)),
            None
        );
        assert_eq!(
            source.sample(Microliters::new(11), Microliters::ZERO),
            Some(WaterMaterialContribution::new(
                Microliters::ZERO,
                WaterContaminantMicrounits::ZERO,
            ))
        );
    }

    #[test]
    fn maximum_width_sampling_conserves_load_without_intermediate_overflow() {
        let source = WaterMaterialContribution::new(
            Microliters::new(u64::MAX),
            WaterContaminantMicrounits::new(u64::MAX),
        );
        let sampled = source
            .sample(Microliters::new(u64::MAX), Microliters::new(u64::MAX - 1))
            .unwrap();
        assert_eq!(sampled.volume(), Microliters::new(u64::MAX - 1));
        assert_eq!(
            sampled.contaminant_load(),
            WaterContaminantMicrounits::new(u64::MAX - 1)
        );
        assert_eq!(
            source
                .contaminant_load()
                .checked_sub(sampled.contaminant_load()),
            Some(WaterContaminantMicrounits::new(1))
        );
    }
}
