//! Reproduction keeps compiler bindings exact while validating seated floors.
use super::*;

fn fixture() -> (TacticalSceneInput, CitySceneLayout) {
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/garden-review.json"
    )))
    .unwrap();
    let mut layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        compounds: input.compounds.clone(),
        gardens: input.gardens.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        parishes: input.parishes.clone(),
        ..Default::default()
    };
    for placement in &mut layout.playable {
        placement.base_elevation_metres =
            adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO;
    }
    for placement in &mut layout.distant {
        placement.base_elevation_metres =
            adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO;
    }
    (input, layout)
}

#[test]
fn reproduction_accepts_bound_nonzero_floors_but_rejects_stale_projection() {
    let (input, layout) = fixture();
    assert_ne!(input.buildings[0].base_elevation_metres.metres(), 0.0);
    verify(&input, &layout).unwrap();
    let mut changed = input.clone();
    changed.buildings[0].base_elevation_metres =
        adventuresim_tactical_core::city_layout::grounding::SupportElevation::from_metres(
            changed.buildings[0].base_elevation_metres.metres() + 0.01,
        )
        .unwrap();
    let error = verify(&changed, &layout).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<SceneInputError>(),
        Some(SceneInputError::GroundingProjection(error))
            if matches!(**error, CityGroundingProjectionError::PlacementMismatch { .. })
    ));
    let mut changed = input;
    changed.distant_buildings[0].base_elevation_metres =
        adventuresim_tactical_core::city_layout::grounding::SupportElevation::from_metres(
            changed.distant_buildings[0].base_elevation_metres.metres() + 0.01,
        )
        .unwrap();
    let error = verify(&changed, &layout).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<SceneInputError>(),
        Some(SceneInputError::GroundingProjection(error))
            if matches!(**error, CityGroundingProjectionError::PlacementMismatch { .. })
    ));
}

#[test]
fn reproduction_rejects_changed_programme_identity_transform_and_membership() {
    let (input, layout) = fixture();
    let mut programme = layout.clone();
    programme.playable[0].program.seed = programme.playable[0].program.seed.wrapping_offset(1);
    let mut identity = layout.clone();
    identity.playable[0].id.0 += 1;
    let mut horizontal = layout.clone();
    let displacement =
        adventuresim_tactical_core::scene_coordinates::PlanDisplacement::from_metres(
            bevy::math::Vec2::new(0.01, 0.0),
        )
        .unwrap();
    horizontal.playable[0].centre_metres = horizontal.playable[0]
        .centre_metres
        .translated(displacement)
        .unwrap();
    let mut distant = layout.clone();
    distant.distant[0].centre_metres = distant.distant[0]
        .centre_metres
        .translated(displacement)
        .unwrap();
    let mut membership = layout.clone();
    membership.gardens.clear();
    let mut roster = layout;
    roster.distant.clear();
    for changed in [programme, identity, horizontal, distant, membership, roster] {
        assert_eq!(
            verify(&input, &changed).unwrap_err().to_string(),
            "recreated programmes, membership or horizontal placement differs from the frozen input"
        );
    }
}

#[test]
#[ignore = "requires verified imported world and frozen current production exports"]
fn imported_grounded_layouts_reproduce_exact_unseated_bindings() {
    let world = std::path::PathBuf::from(std::env::var("FABELGEIST_IMPORTED_WORLD").unwrap());
    let directory =
        std::path::PathBuf::from(std::env::var("FABELGEIST_TERRAIN_SCENE_DIR").unwrap());
    for fixture in ["goslar", "kassel"] {
        let input =
            TacticalSceneInput::load(&directory.join(format!("{fixture}-input.json"))).unwrap();
        assert!(
            input
                .buildings
                .iter()
                .any(|p| p.base_elevation_metres.metres() != 0.0)
        );
        let layout = reproduce(&input, &world).unwrap();
        assert!(layout.playable.iter().all(|p| p.base_elevation_metres
            == adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO));
        assert_eq!(layout.playable.len(), input.buildings.len());
        assert_eq!(layout.distant.len(), input.distant_buildings.len());
    }
}
