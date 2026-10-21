//! Request-driven CPU source growth, without a renderer, files or scene state.
//!
//! This finite stand is generic deciduous content, not calibrated oak or beech.
//! Normal mortality, competition, pruning and global shadowing stay enabled.
//! Its complete growth, capture and encoding belong inside a foreground miss.

use anyhow::{Result, ensure};
use fabelgeist_botany::{
    Climate, Ecosystem, EcosystemSettings, PlantType,
    identity::{PlantId, SpeciesId},
    policy::{GlobalShadowing, Mortality, Seeding},
    terrain::Terrain,
    units::{GroundPoint, StepDuration, Years},
    viewer::{profile::Profile, sim},
};
use fabelgeist_determinism::Seed;
use fabelgeist_math::units::Metres;

use super::{LibraryId, PreparedUnique, TreeRoot, VariantBudget, encode_library};
use crate::render::ForestDetail;

const MATURE_SOURCE_AGE: Years = Years::from_years(120.0);
// The tactical pilot selects two distinct representatives from natural survivors.
const MIN_LIVING_SOURCE_COUNT: usize = 2;
const REGULATION_STEPS: usize = 240;
const SOURCE_TERRAIN_WIDTH: Metres = Metres::from_metres(128.0);
const SOURCE_TERRAIN_DEPTH: Metres = Metres::from_metres(64.0);

/// Eight initial candidates at two-metre spacing, with explicit generic biology.
/// Mortality may reduce the catalogue; at least two natural survivors are required.
/// Construction selects this content; it never substitutes another stand on failure.
#[derive(Clone, Copy, Debug)]
pub struct FiniteStand {
    seed: Seed,
}

/// Encoded source models, with their actual surviving botanical identities.
/// Tactical positions and physical roots are intentionally absent.
pub struct FiniteBank {
    pub bytes: Box<[u8]>,
    pub id: LibraryId,
    pub sources: Box<[PlantId]>,
    pub age: Years,
}

impl FiniteStand {
    pub fn dense_deciduous(seed: Seed) -> Self {
        Self { seed }
    }

    /// Runs the genuine CPU ecosystem and produces one shared immutable bank.
    /// No device is created and no source is preserved, respawned or simplified.
    pub async fn grow(self) -> Result<FiniteBank> {
        let positions = planting_positions();
        let mut kind = PlantType::deciduous();
        kind.seeding_frequency = 0.0;
        let climate = Climate::new(kind.temperature_optimum, kind.precipitation_optimum);
        // Botany's native terrain adapter consumes dimensions in metres.
        let mut ecosystem = Ecosystem::new(
            Terrain::flat(SOURCE_TERRAIN_WIDTH.metres(), SOURCE_TERRAIN_DEPTH.metres()),
            climate,
            vec![kind],
        )
        .with_settings(EcosystemSettings {
            seed: self.seed,
            dt: StepDuration::HALF_YEAR,
            max_plants: positions.len(),
            seeding: Seeding::Disabled,
            mortality: Mortality::Enabled,
            global_shadowing: GlobalShadowing::Enabled,
            ..Default::default()
        });
        for point in &positions {
            ecosystem.plant(SpeciesId::from_word(0), *point)?;
        }
        for _ in 0..REGULATION_STEPS {
            ecosystem.step().await?;
        }
        ensure!(
            ecosystem.time == MATURE_SOURCE_AGE,
            "finite stand did not reach its declared age"
        );
        let source_count = ecosystem.plants.len();
        ensure!(
            source_count >= MIN_LIVING_SOURCE_COUNT
                && ecosystem
                    .plants
                    .iter()
                    .all(|plant| plant.alive && !plant.preserved),
            "finite stand lacks two natural living sources: planted={}, retained={}, alive={}, preserved={}",
            positions.len(),
            ecosystem.plants.len(),
            ecosystem.plants.iter().filter(|plant| plant.alive).count(),
            ecosystem
                .plants
                .iter()
                .filter(|plant| plant.preserved)
                .count()
        );
        let unique = freeze(ecosystem);
        unique.validate()?;
        let library = unique.bake(VariantBudget::AllPlants)?;
        ensure!(
            library.models.len() == source_count,
            "finite source catalogue is incomplete"
        );
        ensure!(
            library
                .models
                .iter()
                .all(|model| model.branches.count > 0 && model.leaves.count > 0),
            "finite source model has empty wood or foliage"
        );
        let sources = library.models.iter().map(|model| model.source).collect();
        let (bytes, id) = encode_library(&library)?;
        Ok(FiniteBank {
            bytes: bytes.into_boxed_slice(),
            id,
            sources,
            age: MATURE_SOURCE_AGE,
        })
    }
}

fn freeze(ecosystem: Ecosystem) -> PreparedUnique {
    let profile = Profile::FAST;
    let snapshot = sim::capture(&ecosystem, &profile, 1, true, 0.0);
    let roots = ecosystem
        .plants
        .iter()
        .map(|plant| TreeRoot {
            id: plant.id,
            position: [plant.position.x, plant.position.y, plant.position.z]
                .map(Metres::from_metres),
        })
        .collect();
    drop(ecosystem);
    PreparedUnique {
        snapshot,
        roots,
        detail: ForestDetail {
            tube_sides: profile.tube_sides,
            tube_spans: profile.tube_spans,
            leaf_blades: profile.leaf_blades,
            full_pixels: profile.full_pixels,
            middle_pixels: profile.middle_pixels,
            ..Default::default()
        },
        label: "Grown generic dense deciduous stand".into(),
        focus: None,
    }
}

fn planting_positions() -> Vec<GroundPoint> {
    // Source-stand catalogue: east/north metres, two rows and four columns.
    [-1.0, 1.0]
        .into_iter()
        .flat_map(|north| {
            [-3.0, -1.0, 1.0, 3.0].into_iter().map(move |east| {
                GroundPoint::new(Metres::from_metres(east), Metres::from_metres(north))
            })
        })
        .collect()
}
