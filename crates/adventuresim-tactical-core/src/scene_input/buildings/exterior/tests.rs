use crate::scene_input::TacticalSceneInput;
use adventuresim_building_generator::{BuildingLodLevel, compile_building_lod, generate};

#[test]
fn facade_shell_and_occupied_detail_share_the_same_physical_plan() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let mut programs = Vec::new();
    for building in &input.distant_buildings {
        let program = building.occupied_program();
        if programs.contains(&program) {
            continue;
        }
        let plan = generate(&program).unwrap();
        let shell = compile_building_lod(&plan, BuildingLodLevel::Shell).unwrap();
        let facade = compile_building_lod(&plan, BuildingLodLevel::Facade).unwrap();
        assert!(!shell.meshes.is_empty());
        assert!(!facade.meshes.is_empty());
        let bounds = |meshes: &[adventuresim_building_generator::LodMesh]| {
            meshes.iter().flat_map(|m| &m.vertices).fold(
                (
                    bevy::math::Vec3::splat(f32::INFINITY),
                    bevy::math::Vec3::splat(f32::NEG_INFINITY),
                ),
                |(min, max), v| (min.min(v.position), max.max(v.position)),
            )
        };
        let (shell_min, shell_max) = bounds(&shell.meshes);
        let (facade_min, facade_max) = bounds(&facade.meshes);
        // Roof extrema must survive LOD; facade trim may extend the footprint.
        assert!(
            (shell_max.y - facade_max.y).abs() < 0.001,
            "{:?} roof changed",
            program
        );
        assert!(shell_min.is_finite() && facade_min.is_finite());
        programs.push(program);
    }
    assert!(programs.len() > 3);
    assert_eq!(
        serde_json::from_str::<TacticalSceneInput>(&serde_json::to_string(&input).unwrap())
            .unwrap(),
        input
    );
}

#[test]
fn prosperity_changes_finish_without_changing_building_mass() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    for mut building in input.distant_buildings {
        let original = building.occupied_program();
        for tier in [
            adventuresim_world_schema::ProsperityTier::Subsistence,
            adventuresim_world_schema::ProsperityTier::Wealthy,
        ] {
            building.prosperity = tier;
            assert_eq!(building.occupied_program(), original);
        }
    }
}
