//! One plant grown alone, to study a species without competition.

use fabelgeist_botany::terrain::Terrain;
use fabelgeist_botany::{Climate, Ecosystem, PlantType};

/// The ground a lone plant grows on, metres along each side.
const SOLO_GROUND: f32 = 40.0;

/// A species botany knows how to grow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "viewer", derive(clap::ValueEnum))]
pub enum SoloSpecies {
    Conifer,
    Deciduous,
    Shrub,
    Grass,
    Baobab,
    Palm,
}

impl SoloSpecies {
    fn plant_type(self) -> PlantType {
        match self {
            Self::Conifer => PlantType::conifer(),
            Self::Deciduous => PlantType::deciduous(),
            Self::Shrub => PlantType::shrub(),
            Self::Grass => PlantType::grass(),
            Self::Baobab => PlantType::baobab(),
            Self::Palm => PlantType::palm(),
        }
    }

    /// One plant of this species alone on level ground, in its preferred
    /// climate. It never seeds, though a shrub still sprouts its clump.
    pub fn ecosystem(self) -> Ecosystem {
        let mut species = self.plant_type();
        species.seeding_frequency = 0.0;
        let climate = Climate::new(species.temperature_optimum, species.precipitation_optimum);
        let mut ecosystem = Ecosystem::new(
            Terrain::flat(SOLO_GROUND, SOLO_GROUND),
            climate,
            vec![species],
        );
        let _ = ecosystem.plant(
            fabelgeist_botany::identity::SpeciesId::from_word(0),
            fabelgeist_botany::units::GroundPoint::new(
                fabelgeist_math::units::Metres::from_metres(0.0),
                fabelgeist_math::units::Metres::from_metres(0.0),
            ),
        );
        ecosystem
    }
}
