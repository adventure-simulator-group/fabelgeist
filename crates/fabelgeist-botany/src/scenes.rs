//! Ready-made ecosystems: a terrain, a climate and the species suited to it.

use fabelgeist_buildings::medieval;
use fabelgeist_buildings::{FeatureKind, SolidIndex};

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
}

impl Scene {
    pub const ALL: [Scene; 5] = [
        Scene::MixedForest,
        Scene::BorealForest,
        Scene::Savanna,
        Scene::Rainforest,
        Scene::Castle,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Scene::MixedForest => "Mixed forest",
            Scene::BorealForest => "Boreal forest",
            Scene::Savanna => "Savanna",
            Scene::Rainforest => "Rainforest",
            Scene::Castle => "Castle and village",
        }
    }

    /// The ecosystem, seeded and ready to step. Environment passes run on the
    /// host until [`Ecosystem::with_gpu`] is called.
    pub fn build(self, seed: u64) -> Ecosystem {
        let (terrain, climate, species, plants, settings) = match self {
            Scene::Castle => return castle(seed),
            Scene::MixedForest => (
                Terrain::flat(60.0, 60.0),
                Climate::new(12.0, 672.0),
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
                            temperature_optimum: 2.0,
                            ..PlantType::conifer()
                        },
                        PlantType {
                            temperature_optimum: 0.0,
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
                        temperature_optimum: 26.0,
                        precipitation_optimum: 900.0,
                        ..PlantType::grass()
                    },
                    PlantType {
                        temperature_optimum: 26.0,
                        precipitation_optimum: 900.0,
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
                        temperature_optimum: 26.0,
                        precipitation_optimum: 3500.0,
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

/// [`Scene::Castle`].
fn castle(seed: u64) -> Ecosystem {
    let structure = medieval::settlement(&castle_ground, 7);
    // Nothing roots under a building, nor in the foundations at its foot.
    let index = SolidIndex::new(structure.solids.clone(), 2.0);
    let terrain = Terrain::from_fn([-55.0, -55.0], [110.0, 110.0], [221, 221], castle_ground)
        .with_soil(|x, z, h| index.distance([x, h + 0.3, z]) < 0.5);
    let features = structure.features.clone();
    let mut ecosystem = Ecosystem::new(
        terrain,
        Climate::new(11.0, 800.0),
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

    let mut rng = Rng::new(seed ^ 0x00ca_571e);
    let mut pick = |points: &[[f32; 3]]| {
        points[(rng.next_f32() * points.len() as f32) as usize % points.len()]
    };
    for feature in &features {
        match feature.kind {
            FeatureKind::Tower | FeatureKind::Keep => {
                // A broadleaf close enough that its crown has to grow round
                // the tower, and ivy at the tower's foot.
                let p = pick(&feature.around(2.0, 3.0));
                ecosystem.plant(0, p[0], p[2]);
                for p in feature.around(0.8, 6.0).iter().step_by(3) {
                    ecosystem.plant_vine(0, p[0], p[2]);
                }
            }
            FeatureKind::Wall => {
                let points = feature.around(0.8, 6.0);
                let p = pick(&points);
                ecosystem.plant_vine(0, p[0], p[2]);
            }
            FeatureKind::House => {
                // A shrub by the house, and honeysuckle at its foot to twine
                // through it.
                let p = pick(&feature.around(1.6, 2.0));
                if ecosystem.plant(2, p[0], p[2]).is_some() {
                    ecosystem.plant_vine(1, p[0] + 0.5, p[2] + 0.3);
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
        .filter(|p| p.species == 0)
        .map(|p| [p.position.x, p.position.z])
        .collect();
    for t in trees.iter().step_by(3) {
        ecosystem.plant_vine(0, t[0] + 0.6, t[1]);
    }
    ecosystem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_builds_with_plants() {
        for scene in Scene::ALL {
            let ecosystem = scene.build(1);
            assert!(!ecosystem.plants.is_empty(), "{}", scene.name());
        }
    }
}
