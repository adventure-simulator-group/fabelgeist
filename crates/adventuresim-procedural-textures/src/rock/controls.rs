//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        downsample_levels_average_ao: f32 = 0.25;
        downsample_levels_filtered_roughness_1: f32 = 0.25;
        downsample_levels_filtered_roughness_2: f32 = 0.35;
        rock_field_height_1: f32 = 0.13;
        rock_field_height_2: f32 = 0.10;
        rock_field_height_3: f32 = 0.055;
        rock_field_height_4: f32 = 0.025;
        rock_field_material_1: f32 = 0.62;
        rock_field_material_2: f32 = 0.38;
        tile_metres: f32 = ROCK_TILE_METRES;
        height_range_metres: f32 = ROCK_HEIGHT_RANGE_METRES;
        domain_columns: i32 = ROCK_DOMAIN_COLUMNS;
        domain_rows: i32 = ROCK_DOMAIN_ROWS;
        palette: [[u8; 3]; 4] = ROCK_PALETTE;
        roughness: [u8; 4] = ROCK_ROUGHNESS;
    }
}
