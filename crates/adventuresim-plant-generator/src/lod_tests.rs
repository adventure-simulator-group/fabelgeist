use super::*;

#[test]
fn entire_catalog_obeys_triangle_budgets_at_every_tier_and_seed() {
    for species in PlantSpecies::ALL {
        for seed in [0, 42, 99, u64::MAX] {
            let mut previous = usize::MAX;
            for lod in PlantLod::ALL {
                let mesh = species.generate(seed, lod).unwrap();
                let triangles = mesh.indices.len() / 3;
                assert!(
                    triangles <= lod.triangle_budget(),
                    "{species:?} {lod:?}: {triangles}"
                );
                assert!(triangles < previous, "{species:?} {lod:?}");
                assert_eq!(mesh, species.generate(seed, lod).unwrap());
                for triangle in mesh.indices.as_chunks::<3>().0 {
                    let a = bevy::math::Vec3::from_array(mesh.positions[triangle[0] as usize]);
                    let b = bevy::math::Vec3::from_array(mesh.positions[triangle[1] as usize]);
                    let c = bevy::math::Vec3::from_array(mesh.positions[triangle[2] as usize]);
                    assert!((b - a).cross(c - a).length_squared() > 1e-24);
                }
                previous = triangles;
            }
        }
    }
}

#[test]
fn maximum_authored_organ_counts_are_aggregated_within_budget() {
    for species in flower::FlowerSpecies::ALL {
        let mut p = species.parameters();
        p.heads = 8;
        p.petals = 40;
        p.leaves = 12;
        p.leaflets = 7;
        p.stamens = 48;
        for lod in PlantLod::ALL {
            let mesh = p.generate(42, lod).unwrap();
            assert!(
                mesh.indices.len() / 3 <= lod.triangle_budget(),
                "{species:?} {lod:?}"
            );
            let max_y = mesh.positions.iter().map(|p| p[1]).fold(0.0, f32::max);
            assert!(
                max_y >= p.height_m,
                "aggregation must retain the primary head"
            );
        }
    }
    for species in fungus::FungusSpecies::ALL {
        let mut p = species.parameters();
        p.ornament_count = 220;
        p.fold_count = 120;
        for lod in PlantLod::ALL {
            assert!(p.generate(42, lod).unwrap().indices.len() / 3 <= lod.triangle_budget());
        }
    }
}
