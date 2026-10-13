//! Frozen road conditions at Goslar's native/browser source discrepancy.
use super::*;
use bevy::math::Vec2;

#[test]
fn goslar_road_rut_preserves_the_cross_runtime_source_vertex() {
    let terrain = SceneTerrain::from_heightmap(3, 3, 100.0, vec![7.0; 9]).unwrap();
    let mut ground = SceneGround::uniform_for_terrain(&terrain, GroundSurface::default());
    ground.urban = crate::scene::UrbanGroundSurfaces::new(
        &[CityStreetPatch::Corridor {
            start_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                -100.0, -39.0,
            ))
            .unwrap(),
            end_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(100.0, -39.0))
                .unwrap(),
            half_width_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    4.875,
                )
                .unwrap(),
            surface: crate::city_layout::CityStreetSurface::Fieldstone,
        }],
        &[],
        &[],
    )
    .unwrap();
    let seed = streams::DETAIL.seed(3_918_113_949_425_128_608.into(), &[]);
    let relief = road_surface_relief(
        Vec2::new(-43.5, -39.5),
        &ground,
        &crate::scene_input::detail_noise::DetailNoise::new(seed),
    );
    assert_eq!((7.0 + relief).to_bits(), 7.023_172_4_f32.to_bits());
}
