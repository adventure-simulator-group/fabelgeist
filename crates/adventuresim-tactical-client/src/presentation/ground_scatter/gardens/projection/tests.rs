use super::*;
use adventuresim_tactical_core::prelude::*;

fn inputs() -> (CityGarden, SceneEnvironment) {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../../assets/tactical-scenes/garden-review.json"
    ))
    .unwrap();
    (
        input.gardens[1].clone(),
        input.environment_snapshot("garden-root-test".into()),
    )
}

#[test]
fn distant_gardens_wait_for_the_exact_terrain_and_project_only_once() {
    let (garden, environment) = inputs();
    let mut app = App::new();
    app.init_resource::<PendingDistantGardens>()
        .add_systems(Update, project_pending);
    app.world_mut().resource_mut::<PendingDistantGardens>().0 = Some(GardenProjection {
        scene_digest: environment.scene_digest.clone(),
        gardens: vec![garden.clone()],
    });
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&SceneGarden>()
            .iter(app.world())
            .count(),
        0
    );
    let mut wrong = environment.clone();
    wrong.scene_digest = "another-scene".into();
    app.world_mut().spawn((
        wrong,
        SceneTerrain::from_heightmap(201, 201, 1.0, vec![3.0; 201 * 201]).unwrap(),
    ));
    app.update();
    assert!(app.world().resource::<PendingDistantGardens>().0.is_some());
    app.world_mut().spawn((
        environment,
        SceneTerrain::from_heightmap(201, 201, 1.0, vec![4.0; 201 * 201]).unwrap(),
    ));
    app.update();
    let projected = app
        .world_mut()
        .query::<&SceneGarden>()
        .single(app.world())
        .unwrap();
    assert_eq!(projected.garden, garden);
    assert!(
        projected
            .plant_support
            .iter()
            .all(|support| support.elevation.metres() == 4.0)
    );
    assert!(app.world().resource::<PendingDistantGardens>().0.is_none());
    app.update();
    assert_eq!(
        app.world_mut()
            .query::<&SceneGarden>()
            .iter(app.world())
            .count(),
        1
    );
}
