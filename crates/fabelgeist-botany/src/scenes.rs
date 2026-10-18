//! Ready-made ecosystems: a terrain, a climate and the species suited to it.

use crate::identity::SpeciesId;
use crate::identity::VineSpeciesId;
use crate::units::GroundPoint;
use crate::units::{Celsius, Millimetres};
use fabelgeist_buildings::medieval;
use fabelgeist_buildings::{FeatureKind, SolidIndex};
use fabelgeist_determinism::Seed;
use fabelgeist_math::units::Metres;

use crate::climate::Climate;
use crate::ecosystem::{Ecosystem, EcosystemSettings};
use crate::rng::Rng;
use crate::species::PlantType;
use crate::terrain::Terrain;
use crate::vine::VineType;

/// A named starting point for an ecosystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scene {
    /// Conifers, broadleaf trees and shrubs on level ground (the paper's
    /// Fig. 19 species).
    MixedForest,
    /// Cold, conifer-dominated hills with a shrub understory.
    BorealForest,
    /// Scattered baobabs over grass and shrubs.
    Savanna,
    /// Warm and wet: palms under tall broadleaf canopy.
    Rainforest,
    /// A castle on a rise and the village below it: trees growing round
    /// the towers and walls, ivy climbing them and the trees, honeysuckle
    /// twining through the shrubs by the houses.
    Castle,
    /// The same castle and village with nothing growing yet: bare ground to
    /// sow, its species and climbers ready.
    EmptyCastle,
}

impl Scene {
    pub const ALL: [Scene; 6] = [
        Scene::MixedForest,
        Scene::BorealForest,
        Scene::Savanna,
        Scene::Rainforest,
        Scene::Castle,
        Scene::EmptyCastle,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Scene::MixedForest => "Mixed forest",
            Scene::BorealForest => "Boreal forest",
            Scene::Savanna => "Savanna",
            Scene::Rainforest => "Rainforest",
            Scene::Castle => "Castle and village",
            Scene::EmptyCastle => "Castle, unplanted",
        }
    }

    /// The ecosystem, seeded and ready to step. Environment passes run on the
    /// host until [`Ecosystem::with_gpu`] is called.
    pub fn build(self, seed: Seed) -> Ecosystem {
        let (terrain, climate, species, plants, settings) = match self {
            Scene::Castle => return castle(seed, true),
            Scene::EmptyCastle => return castle(seed, false),
            Scene::MixedForest => (
                Terrain::flat(60.0, 60.0),
                Climate::new(
                    Celsius::from_degrees(12.0),
                    Millimetres::from_millimetres(672.0),
                ),
                vec![
                    PlantType::conifer(),
                    PlantType::deciduous(),
                    PlantType::shrub(),
                ],
                60,
                EcosystemSettings::default(),
            ),
            Scene::BorealForest => {
                let s = 90.0;
                let terrain = Terrain::from_fn([-s * 0.5, -s * 0.5], [s, s], [96, 96], |x, z| {
                    6.0 * ((x * 0.07).sin() * (z * 0.05).cos() + 0.5 * (x * 0.13 + z * 0.11).sin())
                });
                (
                    terrain,
                    Climate::of(crate::Biome::BorealForest),
                    vec![
                        PlantType {
                            temperature_optimum: Celsius::from_degrees(2.0),
                            ..PlantType::conifer()
                        },
                        PlantType {
                            temperature_optimum: Celsius::from_degrees(0.0),
                            ..PlantType::shrub()
                        },
                    ],
                    90,
                    EcosystemSettings::default(),
                )
            }
            Scene::Savanna => (
                Terrain::from_fn([-60.0, -60.0], [120.0, 120.0], [64, 64], |x, z| {
                    1.5 * (x * 0.04).sin() * (z * 0.03).cos()
                }),
                Climate::of(crate::Biome::Savanna),
                vec![
                    PlantType::baobab(),
                    PlantType {
                        temperature_optimum: Celsius::from_degrees(26.0),
                        precipitation_optimum: Millimetres::from_millimetres(900.0),
                        ..PlantType::grass()
                    },
                    PlantType {
                        temperature_optimum: Celsius::from_degrees(26.0),
                        precipitation_optimum: Millimetres::from_millimetres(900.0),
                        ..PlantType::shrub()
                    },
                ],
                70,
                EcosystemSettings {
                    seed_spacing: 2.0,
                    ..EcosystemSettings::default()
                },
            ),
            Scene::Rainforest => (
                Terrain::flat(60.0, 60.0),
                Climate::of(crate::Biome::TropicalRainforest),
                vec![
                    PlantType::palm(),
                    PlantType {
                        temperature_optimum: Celsius::from_degrees(26.0),
                        precipitation_optimum: Millimetres::from_millimetres(3500.0),
                        shade_tolerance: 0.7,
                        light_compensation: crate::species::compensation(0.7),
                        leaf_color: [0.16, 0.42, 0.12],
                        ..PlantType::deciduous()
                    },
                ],
                80,
                EcosystemSettings::default(),
            ),
        };
        let mut ecosystem = Ecosystem::new(terrain, climate, species)
            .with_settings(EcosystemSettings { seed, ..settings });
        ecosystem.scatter(plants);
        ecosystem
    }
}

/// The castle's ground: a gentle rise under it, rolling away.
fn castle_ground(x: f32, z: f32) -> f32 {
    3.5 * (-(x * x + (z + 12.0) * (z + 12.0)) / 900.0).exp()
        + 0.8 * (x * 0.05).sin() * (z * 0.04).cos()
}

/// [`Scene::Castle`], or with nothing planted, [`Scene::EmptyCastle`].
fn castle(seed: Seed, planted: bool) -> Ecosystem {
    let structure = medieval::settlement(&castle_ground, 7);
    // Nothing roots under a building, nor in the foundations at its foot.
    let index = SolidIndex::new(structure.solids.clone(), 2.0);
    let terrain = Terrain::from_fn([-55.0, -55.0], [110.0, 110.0], [221, 221], castle_ground)
        .with_soil(|x, z, h| index.distance([x, h + 0.3, z]) < 0.5);
    let features = structure.features.clone();
    let mut ecosystem = Ecosystem::new(
        terrain,
        Climate::new(
            Celsius::from_degrees(11.0),
            Millimetres::from_millimetres(800.0),
        ),
        vec![
            PlantType::deciduous(),
            PlantType::conifer(),
            // Clumps by the houses, renewing themselves from the root
            // rather than seeding over the whole hill.
            PlantType {
                seeding_frequency: 0.1,
                ..PlantType::shrub()
            },
        ],
    )
    .with_settings(EcosystemSettings {
        seed,
        ..EcosystemSettings::default()
    })
    .with_obstacles(structure)
    .with_vines(vec![VineType::ivy(), VineType::honeysuckle()]);
    if !planted {
        return ecosystem;
    }

    let mut rng = Rng::new(seed.xor_word(0x00ca_571e));
    let mut pick = |points: &[[f32; 3]]| {
        points[(rng.next_f32() * points.len() as f32) as usize % points.len()]
    };
    for feature in &features {
        match feature.kind {
            FeatureKind::Tower | FeatureKind::Keep => {
                // A broadleaf close enough that its crown has to grow round
                // the tower, and ivy at the tower's foot.
                let p = pick(&feature.around(2.0, 3.0));
                let _ = ecosystem.plant(
                    SpeciesId::from_word(0),
                    GroundPoint::new(Metres::from_metres(p[0]), Metres::from_metres(p[2])),
                );
                for p in feature.around(0.8, 6.0).iter().step_by(3) {
                    let _ = ecosystem.plant_vine(
                        VineSpeciesId::from_word(0),
                        GroundPoint::new(Metres::from_metres(p[0]), Metres::from_metres(p[2])),
                    );
                }
            }
            FeatureKind::Wall => {
                let points = feature.around(0.8, 6.0);
                let p = pick(&points);
                let _ = ecosystem.plant_vine(
                    VineSpeciesId::from_word(0),
                    GroundPoint::new(Metres::from_metres(p[0]), Metres::from_metres(p[2])),
                );
            }
            FeatureKind::House => {
                // A shrub by the house, and honeysuckle at its foot to twine
                // through it.
                let p = pick(&feature.around(1.6, 2.0));
                if ecosystem
                    .plant(
                        SpeciesId::from_word(2),
                        GroundPoint::new(Metres::from_metres(p[0]), Metres::from_metres(p[2])),
                    )
                    .is_ok()
                {
                    let _ = ecosystem.plant_vine(
                        VineSpeciesId::from_word(1),
                        GroundPoint::new(
                            Metres::from_metres(p[0] + 0.5),
                            Metres::from_metres(p[2] + 0.3),
                        ),
                    );
                }
            }
            _ => {}
        }
    }
    ecosystem.scatter(70);
    // Ivy at the foot of some of the broadleaves, to climb them.
    let trees: Vec<[f32; 2]> = ecosystem
        .plants
        .iter()
        .filter(|p| p.species == SpeciesId::from_word(0))
        .map(|p| [p.position.x, p.position.z])
        .collect();
    for t in trees.iter().step_by(3) {
        let _ = ecosystem.plant_vine(
            VineSpeciesId::from_word(0),
            GroundPoint::new(Metres::from_metres(t[0] + 0.6), Metres::from_metres(t[1])),
        );
    }
    ecosystem
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_math::Vec3;

    #[test]
    fn every_scene_builds_with_plants() {
        for scene in Scene::ALL.into_iter().filter(|s| *s != Scene::EmptyCastle) {
            let ecosystem = scene.build(Seed::from_u64(1));
            assert!(!ecosystem.plants.is_empty(), "{}", scene.name());
        }
    }

    #[test]
    fn seeds_thrown_at_the_empty_castle_land_on_open_soil() {
        let mut ecosystem = Scene::EmptyCastle.build(Seed::from_u64(1));
        assert!(ecosystem.plants.is_empty() && ecosystem.vines.is_empty());
        let down = Vec3::new(0.0, -1.0, 0.0);
        // Straight down onto open ground, outside the walls.
        let ground = ecosystem
            .landing(Vec3::new(-20.0, 60.0, 30.0), down)
            .expect("open ground");
        let (x, z) = (ground.x.metres(), ground.z.metres());
        assert!((x + 20.0).abs() < 1e-3 && (z - 30.0).abs() < 1e-3);
        assert!(ecosystem.plant(SpeciesId::from_word(0), ground).is_ok());
        // Thrown level from the west at the south-west corner tower: it
        // falls to the drum's foot on that side.
        let wall = ecosystem.landing(Vec3::new(-30.0, 6.0, 3.0), Vec3::new(1.0, 0.0, 0.0));
        let ground = wall.expect("the foot of the tower");
        let (x, z) = (ground.x.metres(), ground.z.metres());
        assert!(ecosystem.terrain.is_open(x, z));
        assert!(x < -15.0, "on the tower's west side, not behind it");
        let from_tower = (x + 15.0).hypot(z - 3.0);
        assert!(
            (3.4..5.0).contains(&from_tower),
            "{from_tower} m from the tower's axis"
        );
        assert!(
            ecosystem
                .plant_vine(VineSpeciesId::from_word(0), ground)
                .is_ok()
        );
    }
}
