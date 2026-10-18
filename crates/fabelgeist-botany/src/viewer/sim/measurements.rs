//! Snapshot statistics and native world-space render bounds.
use super::*;
use fabelgeist_math::units::Metres;

pub(super) struct SceneMeasurements {
    pub(super) species: Vec<SpeciesInfo>,
    pub(super) bounds: [[f32; 3]; 2],
    pub(super) biome: Biome,
}
impl SceneMeasurements {
    pub(super) fn new(ecosystem: &Ecosystem) -> Self {
        let mut species: Vec<SpeciesInfo> = ecosystem
            .species
            .iter()
            .map(|s| SpeciesInfo {
                name: s.name.clone(),
                bark: s.bark_color,
                leaf: s.leaf_color,
                foliage: s.foliage,
                plants: 0,
            })
            .collect();
        species.extend(ecosystem.vine_species.iter().map(|s| SpeciesInfo {
            name: s.name.clone(),
            bark: s.bark_color,
            leaf: s.leaf_color,
            foliage: Foliage::Broadleaf,
            plants: 0,
        }));
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for plant in &ecosystem.plants {
            species[plant.species.index()].plants += 1;
            for node in &plant.nodes {
                let p = node.position.to_array();
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
        for vine in &ecosystem.vines {
            species[ecosystem.species.len() + vine.species.index()].plants += 1;
        }
        if let Some(obstacles) = &ecosystem.obstacles {
            let [a, b] = obstacles.structure.bounds();
            for k in 0..3 {
                lo[k] = lo[k].min(a[k]);
                hi[k] = hi[k].max(b[k]);
            }
        }
        let t = &ecosystem.terrain;
        let (low, high) = t.height_range();
        let bounds = [
            [t.origin[0], low.min(lo[1]), t.origin[1]],
            [
                t.origin[0] + t.size[0],
                high.max(hi[1]).max(low + 1.0),
                t.origin[1] + t.size[1],
            ],
        ];
        Self {
            species,
            bounds,
            biome: ecosystem.climate.biome_at(Metres::from_metres(low)),
        }
    }
}

/// Pack a plant's native world-space bounding sphere and species word.
pub(super) fn plant_bounds(plant: &Plant, species: &PlantType) -> PlantInstance {
    let crowned = species.foliage == Foliage::Fronds;
    let mut lo = plant.position;
    let mut hi = plant.position;
    for node in &plant.nodes {
        lo = Vec3::new(
            lo.x.min(node.position.x),
            lo.y.min(node.position.y),
            lo.z.min(node.position.z),
        );
        hi = Vec3::new(
            hi.x.max(node.position.x),
            hi.y.max(node.position.y),
            hi.z.max(node.position.z),
        );
    }
    let reach = species.leaf_size * if crowned { 4.5 } else { 1.5 };
    let center = (lo + hi) * 0.5;
    PlantInstance {
        sphere: [
            center.x,
            center.y,
            center.z,
            (hi - lo).length() * 0.5 + reach,
        ],
        species: plant.species.word() as u32,
        pad: [0; 3],
    }
}
