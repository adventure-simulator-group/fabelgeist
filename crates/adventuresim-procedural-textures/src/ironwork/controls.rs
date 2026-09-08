//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        generate_ironwork_textures_oxide_1: f32 = 0.48;
        generate_ironwork_textures_oxide_2: f32 = 0.34;
        generate_ironwork_textures_base_1: f32 = 43.0;
        generate_ironwork_textures_base_2: f32 = 4.0;
        generate_ironwork_textures_base_3: f32 = 1.2;
        generate_ironwork_textures_ao_1: f32 = 0.995;
        generate_ironwork_textures_ao_2: f32 = 0.075;
        generate_ironwork_textures_roughness_1: f32 = 192.0;
        generate_ironwork_textures_roughness_2: f32 = 27.0;
        generate_ironwork_textures_roughness_3: f32 = 22.0;
        generate_ironwork_textures_roughness_4: f32 = 145.0;
        generate_ironwork_textures_roughness_5: f32 = 222.0;
        field_draw_1: f32 = 7.0;
        field_draw_2: f32 = 0.34;
        field_scale_1: f32 = 0.62;
        field_scale_2: f32 = 0.38;
        field_scale_recess_1: f32 = 0.79;
        field_scale_recess_2: f32 = 0.13;
        field_height_1: f32 = 0.045;
        field_height_2: f32 = 0.0045;
        field_height_3: f32 = 0.012;
        field_height_4: f32 = 0.035;
        hammer_facets_angle: f32 = 0.62;
        hammer_facets_half_length_1: f32 = 0.052;
        hammer_facets_half_length_2: f32 = 0.068;
        hammer_facets_half_width_1: f32 = 0.018;
        hammer_facets_half_width_2: f32 = 0.028;
        hammer_facets_tilt: f32 = 0.55;
        tile_metres: f32 = IRONWORK_TILE_METRES;
        height_range_metres: f32 = IRONWORK_HEIGHT_RANGE_METRES;
    }
}
