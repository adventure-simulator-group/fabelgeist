//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        check_field_center_u_1: f32 = 0.2;
        check_field_center_u_2: f32 = 0.6;
        check_field_center_v_1: f32 = 0.2;
        check_field_center_v_2: f32 = 0.6;
        check_field_bend: f32 = 6.0;
        tile_metres: f32 = HEWN_OAK_TILE_METRES;
        height_range_metres: f32 = HEWN_OAK_HEIGHT_RANGE_METRES;
        adze_columns: i32 = ADZE_COLUMNS;
        adze_rows: i32 = ADZE_ROWS;
        check_columns: i32 = CHECK_COLUMNS;
        check_rows: i32 = CHECK_ROWS;
        latewood_relief: f32 = LATEWOOD_RELIEF;
        fiber_relief: f32 = FIBER_RELIEF;
        knot_ring_relief: f32 = KNOT_RING_RELIEF;
        vessel_relief: f32 = VESSEL_RELIEF;
        ray_relief: f32 = RAY_RELIEF;
        adze_relief_gain: f32 = ADZE_RELIEF_GAIN;
        adze_blend_sharpness: f32 = ADZE_BLEND_SHARPNESS;
        adze_edge_relief: f32 = ADZE_EDGE_RELIEF;
        colors: HewnOakColors = HEWN_OAK_COLORS;
        oak_roughness: u8 = OAK_ROUGHNESS;
        relief_ao_strength: f32 = RELIEF_AO_STRENGTH;
        check_occlusion: f32 = 0.20;
        knot_occlusion: f32 = 0.08;
        tool_occlusion: f32 = 0.035;
        ambient_floor: f32 = 0.72;
        minimum_ambient_visibility: f32 = 0.70;
        check_strength: f32 = 0.18;
        knot_strength: f32 = 0.10;
        tool_recess_threshold: f32 = 0.038;
        tool_recess_range: f32 = 0.075;
        base_height: f32 = 0.57;
        broad_growth_relief: f32 = 0.030;
        timber_relief: f32 = 0.045;
        check_depth: f32 = 0.28;
        knot_depth: f32 = 0.055;
        check_absence_probability: f32 = 0.73;
        check_bend_frequency: f32 = 13.0;
        check_bend_amplitude: f32 = 0.0035;
        check_half_width: f32 = 0.0015;
        check_half_length: f32 = 0.028;
        check_length_variation: f32 = 0.050;
        angle_spread: f32 = 0.95;
        cross_slope: f32 = 0.10;
        longitudinal_slope: f32 = 0.14;
        facet_height_variation: f32 = 0.030;
        center_u_margin: f32 = 0.16;
        center_u_jitter: f32 = 0.68;
        center_v_margin: f32 = 0.13;
        center_v_jitter: f32 = 0.74;
        cross_facet_weight: f32 = 0.72;
        cross_facet_variation: f32 = 0.36;
        long_facet_weight: f32 = 0.76;
        long_facet_variation: f32 = 0.32;
    }
}
