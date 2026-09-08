//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        generate_lead_sheet_textures_base_1: f32 = 91.0;
        generate_lead_sheet_textures_base_2: f32 = 8.0;
        generate_lead_sheet_textures_base_3: f32 = 1.3;
        generate_lead_sheet_textures_roughness_1: f32 = 178.0;
        generate_lead_sheet_textures_roughness_2: f32 = 56.0;
        generate_lead_sheet_textures_roughness_3: f32 = 180.0;
        generate_lead_sheet_textures_roughness_4: f32 = 224.0;
        generate_lead_sheet_textures_metallic_1: f32 = 240.0;
        generate_lead_sheet_textures_metallic_2: f32 = 8.0;
        generate_lead_sheet_textures_metallic_3: f32 = 228.0;
        generate_lead_sheet_textures_metallic_4: f32 = 244.0;
        field_long_warp_1: f32 = 0.065;
        field_long_warp_2: f32 = 5.0;
        field_long_warp_3: f32 = 0.018;
        field_roll_secondary_1: f32 = 7.0;
        field_roll_secondary_2: f32 = 0.55;
        field_rolling_1: f32 = 0.72;
        field_rolling_2: f32 = 0.28;
        field_patina_1: f32 = 0.11;
        field_patina_2: f32 = 0.035;
        field_patina_3: f32 = 0.16;
        field_height_1: f32 = 0.010;
        field_height_2: f32 = 0.010;
        field_height_3: f32 = 0.005;
        field_height_4: f32 = 0.018;
        field_height_5: f32 = 0.006;
        tile_metres: f32 = LEAD_SHEET_TILE_METRES;
        height_range_metres: f32 = LEAD_SHEET_HEIGHT_RANGE_METRES;
    }
}
