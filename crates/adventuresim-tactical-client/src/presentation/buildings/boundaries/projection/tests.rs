use super::*;
use adventuresim_tactical_core::prelude::*;

#[test]
fn distant_enclosures_wait_for_exact_support_and_reuse_the_playable_projection_once() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap();
    let generated = input.generate().unwrap();
    let environment = input.environment_snapshot("boundary-test".into());
    let mut pending = PendingDistantBoundaries(Some(BoundaryProjection {
        scene_digest: environment.scene_digest.clone(),
        compounds: input.compounds.clone(),
    }));
    let mut other = environment.clone();
    other.scene_digest = "other-scene".into();
    assert!(
        pending
            .take_for_scene(&generated.terrain, &other)
            .unwrap()
            .is_none()
    );
    assert!(pending.0.is_some());
    let projected = pending
        .take_for_scene(&generated.terrain, &environment)
        .unwrap()
        .unwrap();
    assert_eq!(projected.len(), generated.boundaries.len());
    for (distant, playable) in projected.iter().zip(&generated.boundaries) {
        assert_eq!(distant.scene(), playable.scene());
        assert_eq!(distant.elevation_metres(), playable.elevation_metres());
    }
    assert!(pending.0.is_none());
    assert!(
        pending
            .take_for_scene(&generated.terrain, &environment)
            .unwrap()
            .is_none()
    );
}
