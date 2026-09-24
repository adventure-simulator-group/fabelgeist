use super::{GroundScatterLayer, PLANT_SEED};
use adventuresim_plant_generator::{PlantLod, PlantSpecies};
use bevy::{
    camera::{primitives::Aabb, visibility::VisibilityRange},
    prelude::*,
};

const HIGH_TRANSITION: std::ops::Range<f32> = 2.25..2.75;
const LOW_TRANSITION: std::ops::Range<f32> = 6.5..7.5;
const DISTANCE_FADE: std::ops::Range<f32> = 22.0..27.0;

/// Three co-located tiers use identical distance anchors and complementary
/// per-view dithering. Shared mesh/material handles allow automatic instancing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct PlantLodInstance {
    pub level: PlantLod,
    pub species: PlantSpecies,
}
impl PlantLodInstance {
    pub(crate) fn range(self) -> VisibilityRange {
        let (start_margin, end_margin) = match self.level {
            PlantLod::High => (0.0..0.0, HIGH_TRANSITION),
            PlantLod::Medium => (HIGH_TRANSITION, LOW_TRANSITION),
            PlantLod::Low => (LOW_TRANSITION, DISTANCE_FADE),
        };
        VisibilityRange {
            start_margin,
            end_margin,
            use_aabb: false,
        }
    }
}

struct Specimen {
    meshes: [Handle<Mesh>; 3],
    bounds: Aabb,
}
#[derive(Resource, Default)]
pub(super) struct SpecimenCache {
    specimens: Vec<Specimen>,
    material: Option<Handle<StandardMaterial>>,
}
impl SpecimenCache {
    pub(super) fn prepare(
        &mut self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        if !self.specimens.is_empty() {
            return;
        }
        self.material = Some(materials.add(StandardMaterial {
            perceptual_roughness: 0.72,
            double_sided: true,
            cull_mode: None,
            ..default()
        }));
        self.specimens = PlantSpecies::ALL
            .iter()
            .map(|species| {
                let generated = PlantLod::ALL.map(|lod| {
                    species
                        .generate(PLANT_SEED, lod)
                        .expect("valid botanical preset")
                });
                let mut min = Vec3::splat(f32::INFINITY);
                let mut max = Vec3::splat(f32::NEG_INFINITY);
                for point in generated.iter().flat_map(|m| &m.positions) {
                    min = min.min(Vec3::from_array(*point));
                    max = max.max(Vec3::from_array(*point));
                }
                Specimen {
                    meshes: generated.map(|m| meshes.add(m.into_bevy())),
                    bounds: Aabb::from_min_max(min, max),
                }
            })
            .collect();
    }

    pub(super) fn spawn(
        &self,
        commands: &mut Commands,
        species: PlantSpecies,
        transform: Transform,
    ) {
        let specimen = &self.specimens[species.index()];
        for level in PlantLod::ALL {
            let instance = PlantLodInstance { level, species };
            commands.spawn((
                Name::new(format!("{species:?} {level:?}")),
                GroundScatterLayer::BotanicalPlants,
                Mesh3d(specimen.meshes[level.index()].clone()),
                MeshMaterial3d(
                    self.material
                        .as_ref()
                        .expect("prepared plant material")
                        .clone(),
                ),
                transform,
                specimen.bounds,
                instance.range(),
                instance,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjacent_tiers_cover_distance_without_gaps_or_unmatched_fades() {
        let ranges = PlantLod::ALL.map(|level| {
            PlantLodInstance {
                level,
                species: PlantSpecies::ALL[0],
            }
            .range()
        });
        assert_eq!(ranges[0].end_margin, ranges[1].start_margin);
        assert_eq!(ranges[1].end_margin, ranges[2].start_margin);
        assert!(ranges.iter().all(|r| !r.use_aabb));
        for distance in [0.0, 1.0, 2.5, 3.0, 7.0, 8.0, 22.0, 25.0, 27.0, 40.0] {
            let visible = ranges
                .iter()
                .filter(|r| distance >= r.start_margin.start && distance < r.end_margin.end)
                .count();
            assert_eq!(
                visible,
                if distance >= 27.0 {
                    0
                } else if distance == 2.5 || distance == 7.0 {
                    2
                } else {
                    1
                }
            );
        }
    }
}
