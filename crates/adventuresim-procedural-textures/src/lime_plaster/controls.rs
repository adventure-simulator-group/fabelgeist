//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        float_center: f32 = 0.2;
        float_tracks: crate::stamps::Parameters = crate::stamps::Parameters { cells: [17,19], radius: [0.83,0.38], density:0.85, depth:0.49, angle:0.45, angle_variation:1.2, roundness:0.7, ..Default::default() };
        trowel_strokes: crate::stamps::Parameters = crate::stamps::Parameters { cells: [5, 5], radius: [0.86, 0.38], density: 0.85, depth: 0.13, angle: 0.45, angle_variation: 0.8, roundness: 0.1, edge_width: 0.15, ..Default::default() };
        lime_plaster_sample_aggregate_radius_1: f32 = 0.10;
        lime_plaster_sample_aggregate_radius_2: f32 = 0.10;
        lime_plaster_sample_aggregate: f32 = 0.085;
        lime_plaster_sample_cavity_radius_1: f32 = 0.025;
        lime_plaster_sample_cavity_radius_2: f32 = 0.035;
        lime_plaster_sample_cavity: f32 = 0.035;
        lime_plaster_sample_occlusion_1: f32 = 0.14;
        lime_plaster_sample_occlusion_2: f32 = 0.025;
        lime_plaster_sample_occlusion_3: f32 = 0.18;
        lime_plaster_sample_ao: f32 = 0.82;
        lime_plaster_sample_roughness_1: f32 = 0.805;
        lime_plaster_sample_roughness_2: f32 = 0.018;
        lime_plaster_sample_roughness_3: f32 = 0.050;
        lime_plaster_sample_roughness_4: f32 = 0.030;
        lime_plaster_sample_roughness_5: f32 = 0.018;
        lime_plaster_sample_roughness_6: f32 = 0.018;
        lime_plaster_sample_roughness_7: f32 = 0.74;
        lime_plaster_sample_roughness_8: f32 = 0.94;
        oblique_micro_variation_warp_x: f32 = 0.024;
        oblique_micro_variation_warp_y: f32 = 0.024;
        cellular_feature_site_x_1: f32 = 0.16;
        cellular_feature_site_x_2: f32 = 0.68;
        cellular_feature_site_y_1: f32 = 0.16;
        cellular_feature_site_y_2: f32 = 0.68;
        tile_metres: f32 = LIME_PLASTER_TILE_METRES;
        height_range_metres: f32 = LIME_PLASTER_HEIGHT_RANGE_METRES;
        plaster_base: Vec3 = PLASTER_BASE;
        plaster_warm_fleck: Vec3 = PLASTER_WARM_FLECK;
        plaster_cool_fleck: Vec3 = PLASTER_COOL_FLECK;
        plaster_albedo_cells_per_metre: i32 = PLASTER_ALBEDO_CELLS_PER_METRE;
        plaster_fleck_fraction: f32 = PLASTER_FLECK_FRACTION;
        trowel_body_height_metres: f32 = TROWEL_BODY_HEIGHT_METRES;
        trowel_edge_height_metres: f32 = TROWEL_EDGE_HEIGHT_METRES;
        sand_float_height_metres: f32 = SAND_FLOAT_HEIGHT_METRES;
        fine_aggregate_height_metres: f32 = FINE_AGGREGATE_HEIGHT_METRES;
        oblique_tool_mark_height_metres: f32 = OBLIQUE_TOOL_MARK_HEIGHT_METRES;
        exposed_aggregate_height_metres: f32 = EXPOSED_AGGREGATE_HEIGHT_METRES;
        rare_pull_depth_metres: f32 = RARE_PULL_DEPTH_METRES;
    }
}
