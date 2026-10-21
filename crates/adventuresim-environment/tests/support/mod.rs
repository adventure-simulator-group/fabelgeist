use std::sync::Arc;

use adventuresim_environment::{prepared::*, render::ForestDetail};
use fabelgeist_botany::{Biome, identity::PlantId, species::Foliage, viewer::sim::*};
use fabelgeist_math::units::Metres;

/// Different non-power-of-two spans exercise shared grids and clipped padding.
pub fn padded_fixture() -> PreparedUnique {
    let mut unique = fixture();
    let mut branches = Vec::new();
    let mut leaves = Vec::new();
    for (index, original) in unique.snapshot.branches.iter().copied().enumerate() {
        let count = if index % 2 == 0 { 3 } else { 5 };
        branches.push(original);
        for twig in 1..count {
            let mut branch = original;
            branch.a = original.b;
            branch.b = [
                original.b[0] + twig as f32 * 0.25,
                original.b[1] + twig as f32 * 0.15,
                original.b[2] + (twig % 2) as f32 * 0.3,
                0.01,
            ];
            branch.v0 = original.v1;
            branch.v1 = original.v1 + 1.0;
            branches.push(branch);
        }
        let original = unique.snapshot.leaves[index];
        for blade in 0..(if index % 2 == 0 { 5 } else { 7 }) {
            let mut leaf = original;
            leaf.center[0] += (blade as f32 - 2.0) * 0.22;
            leaf.center[2] += (blade % 2) as f32 * 0.2;
            leaf.seed += blade as f32 * 0.07;
            leaves.push(leaf);
        }
    }
    unique.snapshot.branches = branches;
    unique.snapshot.leaves = leaves;
    unique
}

pub fn fixture() -> PreparedUnique {
    let roots: Vec<_> = [
        [-8.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [8.0, 0.0, 0.0],
        [-4.0, 0.0, 5.0],
        [4.0, 0.0, 5.0],
    ]
    .into_iter()
    .enumerate()
    .map(|(index, position)| TreeRoot {
        id: PlantId::from_word(100 + index as u64),
        position: position.map(Metres::from_metres),
    })
    .collect();
    let species: Vec<_> = [Foliage::Broadleaf, Foliage::Needles, Foliage::Fronds]
        .into_iter()
        .map(|foliage| SpeciesInfo {
            name: format!("fixture {foliage:?}"),
            bark: [0.3, 0.2, 0.1],
            leaf: [0.2, 0.5, 0.1],
            foliage,
            plants: 2,
        })
        .collect();
    let mut branches = Vec::new();
    let mut leaves = Vec::new();
    let mut plants = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        let [x, y, z] = root.position.map(Metres::metres);
        let height = 4.0 + index as f32 * 0.3;
        let kind = index as u32 % 3;
        plants.push(PlantInstance {
            sphere: [x, y + height * 0.5, z, height],
            species: kind,
            ..Default::default()
        });
        branches.push(BranchInstance {
            a: [x, y, z, 0.2],
            b: [x + 0.3, y + height, z, 0.03],
            tangents: [0x7fff_0000; 2],
            plant: index as u32,
            light: 0.9,
            light_start: 0.7,
            v1: 4.0,
            ..Default::default()
        });
        leaves.push(LeafInstance {
            center: [x + 0.3, y + height, z],
            axis: [0.2, 0.9, 0.1],
            size: 0.6,
            light: 0.8,
            crown: [x, y + height - 0.5, z, 2.0],
            species: kind,
            seed: 0.13 + index as f32 * 0.1,
            plant: index as u32,
            ..Default::default()
        });
    }
    let terrain = TerrainGeometry {
        vertices: [
            [-20.0, 0.0, -20.0],
            [20.0, 0.0, -20.0],
            [-20.0, 0.0, 20.0],
            [20.0, 0.0, 20.0],
        ]
        .map(|position| TerrainVertex {
            position,
            normal: [0.0, 1.0, 0.0],
            color: [0.3, 0.4, 0.2],
        })
        .to_vec(),
        indices: vec![0, 2, 1, 1, 2, 3],
    };
    PreparedUnique {
        roots,
        label: "prepared parity fixture".into(),
        detail: ForestDetail::default(),
        focus: Some(TreeFocus {
            root: [-8.0, 0.0, 0.0].map(Metres::from_metres),
            height: Metres::from_metres(4.0),
        }),
        snapshot: Snapshot {
            scene_generation: 1,
            time: 120.0,
            plants: 6,
            modules: 6,
            step_ms: 0.0,
            species,
            plant_bounds: plants,
            branches,
            leaves,
            blend: BlendShapes::default(),
            fillets: false,
            fillet_size: 0.0,
            litter: Litter::default(),
            hierarchy: Default::default(),
            terrain: Some(Arc::new(terrain)),
            structures: None,
            bounds: [[-20.0, 0.0, -20.0], [20.0, 10.0, 20.0]],
            biome: Biome::TemperateSeasonalForest,
        },
    }
}
