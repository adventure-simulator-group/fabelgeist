use super::*;
use bevy::math::Vec3;

#[test]
fn presets_have_bounded_finite_deterministic_geometry_at_both_details() {
    for species in FungusSpecies::ALL {
        let p = species.parameters();
        let close = p.generate(42, PlantLod::High).unwrap();
        let field = p.generate(42, PlantLod::Medium).unwrap();
        assert!(field.indices.len() < close.indices.len());
        assert_ne!(
            close.positions,
            p.generate(99, PlantLod::High).unwrap().positions
        );
        for mesh in [close, field] {
            assert!(
                mesh.positions.len() < 40_000,
                "{species:?}: {}",
                mesh.positions.len()
            );
            assert_eq!(mesh.positions.len(), mesh.normals.len());
            assert_eq!(mesh.positions.len(), mesh.colors.len());
            for v in &mesh.positions {
                assert!(Vec3::from_array(*v).is_finite(), "{species:?}: {v:?}");
                assert!(
                    v[1] >= -0.001 && v[1] < p.height_m() + 0.015,
                    "{species:?}: {v:?}"
                );
            }
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.001)
            );
            for [a, b, c] in mesh.indices.as_chunks::<3>().0 {
                let [a, b, c] = [*a, *b, *c].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                assert!((b - a).cross(c - a).length_squared() > 1e-24);
            }
        }
        assert_eq!(
            p.generate(42, PlantLod::High).unwrap(),
            p.generate(42, PlantLod::High).unwrap()
        );
        assert_eq!(
            p,
            serde_json::from_str::<FungusParameters>(&serde_json::to_string(&p).unwrap()).unwrap()
        );
    }
}

#[test]
fn invalid_profiles_reject_before_allocation_and_topologies_change_surfaces() {
    let mut p = FungusSpecies::FlyAgaric.parameters();
    p.cap_radius_m = f32::NAN;
    assert!(p.generate(0, PlantLod::High).is_err());
    p.cap_radius_m = 0.012;
    p.stipe_radius_m = 0.04;
    assert!(p.validate().is_err());
    p = FungusSpecies::FlyAgaric.parameters();
    p.cap_rise_ratio = 0.0;
    p.cap_depression_ratio = 0.3;
    assert!(
        p.validate().is_err(),
        "cap top cannot pass through its underside"
    );
    p = FungusSpecies::FlyAgaric.parameters();
    let gills = p.generate(0, PlantLod::High).unwrap();
    p.fertile_surface = FertileSurface::Enclosed;
    assert!(p.generate(0, PlantLod::High).unwrap().positions != gills.positions);
    p.decurrent_m = p.cap_elevation_m;
    assert!(p.validate().is_err());
}

#[test]
fn fungi_respect_season_moisture_and_host_habitat() {
    use crate::habitat::{PlantGround, PlantHabitat};
    let h = PlantHabitat {
        canopy: 0.65,
        cultivation: 0.0,
        moisture: 0.7,
        snow: 0.0,
        day_of_year: 270,
        ground: PlantGround::WoodlandLitter,
    };
    for species in FungusSpecies::ALL {
        assert!(h.fungus_weight(species) > 0.0);
        for invalid in [
            PlantHabitat { snow: 0.2, ..h },
            PlantHabitat { moisture: 0.1, ..h },
            PlantHabitat {
                day_of_year: 20,
                ..h
            },
            PlantHabitat {
                ground: PlantGround::Unsuitable,
                ..h
            },
        ] {
            assert_eq!(invalid.fungus_weight(species), 0.0);
        }
    }
    assert_eq!(
        PlantHabitat {
            canopy: 0.05,
            ground: PlantGround::Grass,
            ..h
        }
        .fungus_weight(FungusSpecies::Porcini),
        0.0
    );
}
