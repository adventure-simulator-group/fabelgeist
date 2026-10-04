//! Finish variation independent of a building’s immutable physical program.
use super::DistantBuildingPlacement;
use fabelgeist_determinism::StreamId;

const EXTERIOR_VARIATION: StreamId = StreamId::new("city.distant-exterior");

/// Finish can vary without substituting geometry at another resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistantBuildingVariant {
    Plain,
    Weathered,
    Decorated,
}

impl DistantBuildingVariant {
    const ALL: [Self; 3] = [Self::Plain, Self::Weathered, Self::Decorated];
}

impl DistantBuildingPlacement {
    pub fn exterior_variant(self) -> DistantBuildingVariant {
        let index = EXTERIOR_VARIATION
            .rng(self.seed, &[self.id])
            .index(DistantBuildingVariant::ALL.len());
        DistantBuildingVariant::ALL[index]
    }
}

#[cfg(test)]
mod tests;
