use super::*;

#[test]
fn ground_grid_queries_discrete_centered_samples_and_rejects_invalid_data() {
    let mut samples = vec![GroundSurface::default(); 9];
    samples[4] = GroundSurface {
        substrate: GroundSubstrate::Soil,
        cover: GroundCover::LeafLitter,
        cover_density_bps: 8_500,
        cover_height_cm: 6,
    };
    let ground = SceneGround::from_samples(3, 3, 2.0, samples).unwrap();
    assert_eq!(
        ground.ground_at(ScenePlanPoint::ORIGIN).unwrap().cover,
        GroundCover::LeafLitter
    );
    assert!(ground.cover_intersects_square(
        ScenePlanPoint::from_metres(Vec2::new(1.8, 0.0)).unwrap(),
        0.3,
        GroundCover::LeafLitter,
    ));
    assert!(ground.ground_at(ScenePlanPoint::from_metres(Vec2::new(2.01, 0.0)).unwrap()).is_none());
    assert!(SceneGround::from_samples(1, 3, 1.0, vec![GroundSurface::default(); 3]).is_none());
}
