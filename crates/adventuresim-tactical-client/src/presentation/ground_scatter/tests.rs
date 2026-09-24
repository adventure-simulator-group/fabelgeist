use super::*;
use adventuresim_tactical_core::prelude::{
    GroundCover, GroundSubstrate, GroundSurface, SceneEnvironmentFixture,
};
use bevy::{
    color::ColorToComponents,
    prelude::{App, Update},
};

#[test]
fn understory_density_caps_physical_shrubs_at_the_reduced_global_budget() {
    assert!((understory_scatter_chance(0.35, 0.03, 0.0) - 0.0573).abs() < 0.000_01);
    assert_eq!(understory_scatter_chance(0.9, 0.05, 0.0), 0.075);
    assert_eq!(understory_scatter_chance(0.1, 0.95, 0.0), 0.075);
    assert_eq!(understory_scatter_chance(0.0, 0.0, 0.0), 0.0);
    assert_eq!(understory_scatter_chance(0.0, 0.0, 1.0), 0.024);
}

#[test]
fn grass_density_favors_open_meadow_and_thins_under_closed_canopy() {
    assert_eq!(grass_scatter_density(0.0, 0.0, 0.0, 0.0), 0.98);
    assert!((grass_scatter_density(0.35, 0.0, 0.0, 0.0) - 0.6475).abs() < 0.000_01);
    assert!((grass_scatter_density(0.9, 0.0, 0.0, 0.0) - 0.125).abs() < 0.000_01);
    assert!((grass_scatter_density(0.0, 1.0, 0.0, 0.0) - 0.10).abs() < 0.000_01);
    assert!((grass_scatter_density(0.0, 0.0, 0.0, 0.65) - 0.183_75).abs() < 0.000_01);
}

#[test]
fn terminal_grass_pigment_compensates_for_foliage_optical_darkening() {
    let environment = SceneEnvironmentFixture::TemperateHills.snapshot("terminal-grass-pigment");
    let blade = grass_pigment(&environment).0.to_linear().to_f32_array();
    let terminal = grass_terminal_pigment(&environment)
        .to_linear()
        .to_f32_array();
    for (channel, expected) in [0.34, 0.38, 0.18].into_iter().enumerate() {
        assert!((terminal[channel] / blade[channel] - expected).abs() < 0.000_01);
    }
}

#[test]
fn foliage_uses_hardware_multisample_coverage() {
    assert_eq!(
        foliage_material(0.3, true).alpha_mode(),
        AlphaMode::AlphaToCoverage
    );
}

#[test]
fn volumetric_patch_footprint_suppresses_all_heightfield_scatter() {
    let ground = SceneGround::from_samples(9, 9, 1.0, vec![GroundSurface::default(); 81]).unwrap();
    let collar = TerrainTransitionCollar::irregular_ellipse(
        Vec2::ZERO,
        Vec2::X,
        2.0,
        2.0,
        0.5,
        17,
        0.2,
        1_000,
    )
    .unwrap();
    let masked = scatter_ground_without_patch(&ground, collar);

    for z in 0..masked.grid_depth() {
        for x in 0..masked.grid_width() {
            let point = Vec2::new(x as f32 - 4.0, z as f32 - 4.0);
            let sample = masked.samples()[z * masked.grid_width() + x];
            if collar.contains(point) {
                assert_eq!(sample.cover, GroundCover::Bare);
                assert_eq!(sample.substrate, GroundSubstrate::Water);
                assert_eq!(sample.cover_density_bps, 0);
            } else {
                assert_eq!(sample, GroundSurface::default());
            }
        }
    }
}
