use super::*;
use crate::{BuildingArchetype, BuildingProgram, generate};

#[test]
fn missing_operable_closure_preserves_opening_and_source_identity() {
    let mut plan = generate(&BuildingProgram::fixture(
        BuildingArchetype::TownHouse,
        fabelgeist_determinism::Seed::from_u64(42),
    ))
    .unwrap();
    let window = compile_operable_windows(&plan).unwrap()[0];
    let sources = plan
        .opening_assemblies
        .iter()
        .find(|o| o.id == window.opening)
        .unwrap()
        .closure_solids
        .clone();
    plan.resolved_geometry
        .solids
        .retain(|s| !sources.contains(&s.id));
    let error = compile_operable_windows(&plan).unwrap_err();
    assert_eq!(error.opening, window.opening);
    assert_eq!(error.source_id, sources.first().copied());
    assert_eq!(error.cause, WindowErrorCause::MissingClosure { sources });
    // An ordinary ineligible opening does not require an operable closure.
    plan.opening_assemblies
        .iter_mut()
        .find(|o| o.id == window.opening)
        .unwrap()
        .closure
        .state = ClosureState::Closed;
    assert!(compile_operable_windows(&plan).is_ok());
}

#[test]
fn invalid_frame_or_leaf_reports_the_actual_window_binding() {
    let mut plan = generate(&BuildingProgram::fixture(
        BuildingArchetype::TownHouse,
        fabelgeist_determinism::Seed::from_u64(42),
    ))
    .unwrap();
    let window = compile_operable_windows(&plan).unwrap()[0];
    let opening = plan
        .opening_assemblies
        .iter_mut()
        .find(|o| o.id == window.opening)
        .unwrap();
    let tangent = opening.frame.tangent;
    opening.frame.tangent = Vec2::ZERO;
    let error = compile_operable_windows(&plan).unwrap_err();
    assert_eq!(error.opening, window.opening);
    assert_eq!(error.source_id, Some(window.source));
    assert!(matches!(error.cause, WindowErrorCause::Geometry(_)));
    plan.opening_assemblies
        .iter_mut()
        .find(|o| o.id == window.opening)
        .unwrap()
        .frame
        .tangent = tangent;
    let solid = plan
        .resolved_geometry
        .solids
        .iter_mut()
        .find(|s| s.id == window.source)
        .unwrap();
    solid.size =
        CuboidDimensions::from_metres(Vec3::new(solid.size.metres().x, 0.0, solid.size.metres().z))
            .unwrap();
    assert_eq!(
        compile_operable_windows(&plan).unwrap_err().source_id,
        Some(window.source)
    );
}

#[test]
fn window_and_bar_decoding_admit_named_roles_and_checked_geometry() {
    let plan = generate(&BuildingProgram::fixture(
        BuildingArchetype::TownHouse,
        fabelgeist_determinism::Seed::from_u64(42),
    ))
    .unwrap();
    let window = compile_operable_windows(&plan).unwrap()[0];
    let wire = serde_json::to_value(window).unwrap();
    assert_eq!(wire["bars"], serde_json::to_value(window.bars).unwrap());
    assert!(wire.get("barred").is_none());
    assert_eq!(
        wire["closed_centre"],
        serde_json::to_value(window.closed_centre.metres()).unwrap()
    );
    assert_eq!(
        wire["size_metres"],
        serde_json::to_value(window.size_metres.metres()).unwrap()
    );
    assert_eq!(
        serde_json::from_value::<WindowSpec<Architectural>>(wire.clone()).unwrap(),
        window
    );
    assert_eq!(
        postcard::from_bytes::<WindowSpec<Architectural>>(&postcard::to_allocvec(&window).unwrap())
            .unwrap(),
        window
    );
    for (field, value) in [
        ("tangent", serde_json::json!([0, 0])),
        ("outward", serde_json::json!([2, 0])),
        ("size_metres", serde_json::json!([1, 2, 0])),
        ("closed_yaw_radians", serde_json::Value::Null),
        ("bars", serde_json::json!(true)),
    ] {
        let mut invalid = wire.clone();
        invalid[field] = value;
        let error = serde_json::from_value::<WindowSpec<Architectural>>(invalid).unwrap_err();
        assert!(!error.to_string().is_empty());
    }
    for bar in compile_window_bars(&plan).unwrap() {
        let wire = postcard::to_allocvec(&bar).unwrap();
        assert_eq!(postcard::from_bytes::<WindowBarSpec>(&wire).unwrap(), bar);
        let mut invalid = serde_json::to_value(bar).unwrap();
        invalid["size_metres"] = serde_json::json!([1, -1, 1]);
        assert!(serde_json::from_value::<WindowBarSpec>(invalid).is_err());
    }
    // Fixed glazing accepts degenerate cuboid geometry, while operable leaves do not.
    assert!(
        compile_window_leaf(
            CuboidDimensions::from_metres(Vec3::new(1.0, 1.0, 0.0)).unwrap(),
            WindowLeafKind::LeadedGlass,
            ClosureState::Closed
        )
        .is_ok()
    );
    assert!(LeafDimensions::from_metres(Vec3::new(1.0, 1.0, 0.0)).is_err());
}

#[test]
fn bar_compilation_rejects_a_zero_frame_with_encoded_opening_identity() {
    let mut plan = generate(&BuildingProgram::fixture(
        BuildingArchetype::TownHouse,
        fabelgeist_determinism::Seed::from_u64(42),
    ))
    .unwrap();
    let window = plan
        .opening_assemblies
        .iter_mut()
        .find(|o| o.use_kind == OpeningUse::Window)
        .unwrap();
    window.closure.layers.push(ClosureKind::IronBars);
    window.frame.tangent = Vec2::ZERO;
    let opening = window.id;
    let error = compile_window_bars(&plan).unwrap_err();
    assert_eq!(
        error.source_id,
        ResolvedItemId((7_u64 << 60) | (opening.0 << 8))
    );
}
