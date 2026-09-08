//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        aggregate_roughness: f32 = 0.010;
        application: crate::stamps::Parameters = crate::stamps::Parameters { cells: [6, 7], radius: [0.83, 0.43], density: 0.8, depth: 0.30, angle: 0.35, angle_variation: 1.4, roundness: 0.75, edge_width: 0.55, ..Default::default() };
        aggregate: CapsuleLayer = CapsuleLayer {cells:96,salt:0x7361,enabled_threshold:0.82,half_length_range:(0.0,0.020),radius:0.130};
        fibre: CapsuleLayer = CapsuleLayer {cells:38,salt:0x19d7,enabled_threshold:0.965,half_length_range:(0.12,0.32),radius:0.028};
        shrink_crack: CapsuleLayer = CapsuleLayer {cells:7,salt:0x52b9,enabled_threshold:0.90,half_length_range:(0.040,0.105),radius:0.0035};
        generate_wattle_and_daub_textures_ao_1: f32 = 0.16;
        generate_wattle_and_daub_textures_ao_2: f32 = 0.05;
        sample_daub_warp: f32 = 0.028;
        sample_daub_smear: f32 = 0.35;
        sample_daub_surface_1: f32 = 0.61;
        sample_daub_surface_2: f32 = 0.055;
        sample_daub_surface_3: f32 = 0.025;
        sample_daub_surface_4: f32 = 0.006;
        sample_daub_surface_5: f32 = 0.030;
        sample_daub_surface_6: f32 = 0.12;
        sample_daub_surface_7: f32 = 0.014;
        sample_daub_surface_8: f32 = 0.012;
        sample_daub_surface_9: f32 = 0.075;
        sample_daub_height_1: f32 = 0.25;
        sample_daub_height_2: f32 = 0.16;
        exposed_wattle_dx: f32 = 0.22;
        exposed_wattle_dy: f32 = 0.31;
        exposed_wattle_theta_1: f32 = 0.0075;
        exposed_wattle_theta_2: f32 = 0.0105;
        exposed_wattle_irregular_radius_1: f32 = 0.7;
        exposed_wattle_irregular_radius_2: f32 = 0.15;
        exposed_wattle_irregular_radius_3: f32 = 5.0;
        exposed_wattle_irregular_radius_4: f32 = 0.4;
        exposed_wattle_irregular_radius_5: f32 = 0.07;
        exposed_wattle_edge_noise: f32 = 0.06;
        exposed_wattle_elliptical_1: f32 = 0.0105;
        exposed_wattle_elliptical_2: f32 = 0.0075;
        exposed_wattle_cavity: f32 = 0.77;
        exposed_wattle_rod_distance_1: f32 = 0.0065;
        exposed_wattle_rod_distance_2: f32 = 0.0025;
        exposed_wattle_rod_distance_3: f32 = 0.0045;
        exposed_wattle_rod_distance_4: f32 = 0.0040;
        exposed_wattle_woody_fragment_1: f32 = 0.0012;
        exposed_wattle_woody_fragment_2: f32 = 0.0025;
        exposed_wattle_chipped_occlusion_1: f32 = 0.006;
        exposed_wattle_chipped_occlusion_2: f32 = 0.003;
        exposed_wattle_chipped_occlusion_3: f32 = 0.35;
        sparse_capsules_center_1: f32 = 0.15;
        sparse_capsules_center_2: f32 = 0.70;
        sparse_capsules_center_3: f32 = 0.15;
        sparse_capsules_center_4: f32 = 0.70;
        tile_metres: f32 = WATTLE_AND_DAUB_TILE_METRES;
        height_range_metres: f32 = WATTLE_AND_DAUB_HEIGHT_RANGE_METRES;
        daub_warm: Vec3 = DAUB_WARM;
        wattle_color: Vec3 = WATTLE_COLOR;
    }
}
