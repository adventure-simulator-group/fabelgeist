//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        dents: crate::stamps::Parameters = crate::stamps::Parameters { cells: [13, 9], radius: [0.65, 0.45], density: 0.3, depth: 0.08, ..Default::default() };
        patina_srgb: [u8; 3] = [84, 89, 94];
        patina_roughness: f32 = 0.72;
        field_long_warp_1: f32 = 0.065;
        field_long_warp_2: f32 = 5.0;
        field_long_warp_3: f32 = 0.018;
        field_roll_secondary_1: f32 = 7.0;
        field_roll_secondary_2: f32 = 0.55;
        field_height_1: f32 = 0.010;
        field_height_2: f32 = 0.010;
        field_height_3: f32 = 0.005;
        field_height_4: f32 = 0.018;
        field_height_5: f32 = 0.006;
        tile_metres: f32 = LEAD_SHEET_TILE_METRES;
        height_range_metres: f32 = LEAD_SHEET_HEIGHT_RANGE_METRES;
    }
}
