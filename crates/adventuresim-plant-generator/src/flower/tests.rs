use super::*;
use bevy::math::Vec3;

#[test]
fn all_presets_produce_bounded_deterministic_nondegenerate_geometry() {
    for species in FlowerSpecies::ALL {
        let p = species.parameters();
        for detail in PlantLod::ALL {
            let mesh = p.generate(42, detail).unwrap();
            assert_eq!(mesh, p.generate(42, detail).unwrap());
            assert!(!mesh.indices.is_empty());
            assert!(
                mesh.indices.len() / 3 <= detail.triangle_budget(),
                "{species:?} {detail:?}"
            );
            for position in &mesh.positions {
                let v = Vec3::from_array(*position);
                assert!(v.is_finite());
                assert!(
                    v.y >= -0.001 && v.y < p.height_m + 0.15,
                    "{species:?}: {v:?}"
                );
            }
            for normal in &mesh.normals {
                assert!((Vec3::from_array(*normal).length() - 1.0).abs() < 0.001);
            }
            let palette = [p.green, p.petal, p.petal_base, p.center, p.anther].map(|c| c.linear());
            assert!(mesh.colors.iter().all(|color| palette.contains(color)));
            for triangle in mesh.indices.as_chunks::<3>().0 {
                let a = Vec3::from_array(mesh.positions[triangle[0] as usize]);
                let b = Vec3::from_array(mesh.positions[triangle[1] as usize]);
                let c = Vec3::from_array(mesh.positions[triangle[2] as usize]);
                assert!((b - a).cross(c - a).length_squared() > 1e-24);
            }
        }
    }
}

#[test]
fn invalid_documents_fail_before_generation() {
    let mut p = FlowerSpecies::Daisy.parameters();
    p.height_m = f32::NAN;
    assert!(p.generate(0, PlantLod::Medium).is_err());
    p.height_m = 0.1;
    p.heads = u8::MAX;
    assert!(p.generate(0, PlantLod::Medium).is_err());
}

#[test]
fn seed_and_shape_edits_change_geometry_and_lod_preserves_extent() {
    for species in FlowerSpecies::ALL {
        let p = species.parameters();
        let close = p.generate(42, PlantLod::High).unwrap();
        let field = p.generate(42, PlantLod::Medium).unwrap();
        assert!(field.indices.len() < close.indices.len());
        assert_ne!(
            close.positions,
            p.generate(99, PlantLod::High).unwrap().positions
        );
        let extent = |m: &PlantMesh| m.positions.iter().map(|v| v[1]).fold(0.0, f32::max);
        assert!((extent(&close) - extent(&field)).abs() < 0.005);
        let mut edited = p.clone();
        edited.petal_length_m *= 1.2;
        assert_ne!(
            close.positions,
            edited.generate(42, PlantLod::High).unwrap().positions
        );
        assert_eq!(
            p,
            serde_json::from_str::<FlowerParameters>(&serde_json::to_string(&p).unwrap()).unwrap()
        );
    }
}

#[test]
fn habitat_excludes_water_snow_wrong_season_and_dense_woodland_poppies() {
    use crate::habitat::{PlantGround, PlantHabitat};
    let mut h = PlantHabitat {
        canopy: 0.1,
        cultivation: 0.8,
        moisture: 0.5,
        snow: 0.0,
        day_of_year: 180,
        ground: PlantGround::Grass,
    };
    assert!(h.flower_weight(FlowerSpecies::CornPoppy) > 0.0);
    h.canopy = 0.9;
    assert_eq!(h.flower_weight(FlowerSpecies::CornPoppy), 0.0);
    h.ground = PlantGround::Unsuitable;
    assert!(
        FlowerSpecies::ALL
            .iter()
            .all(|s| h.flower_weight(*s) == 0.0)
    );
    h.ground = PlantGround::Grass;
    h.canopy = 0.1;
    h.snow = 0.7;
    assert!(
        FlowerSpecies::ALL
            .iter()
            .all(|s| h.flower_weight(*s) == 0.0)
    );
    h.snow = 0.0;
    h.day_of_year = 10;
    assert!(
        FlowerSpecies::ALL
            .iter()
            .all(|s| h.flower_weight(*s) == 0.0)
    );
}
