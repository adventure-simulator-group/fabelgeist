//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        facets: crate::stamps::Parameters = crate::stamps::Parameters { cells: [13, 11], radius: [0.85, 0.70], density: 0.9, depth: 0.21, roundness: 0.0, ..Default::default() };
        pores: crate::stamps::Parameters = crate::stamps::Parameters { cells: [91, 83], radius: [0.30, 0.26], density: 0.12, depth: 0.035, ..Default::default() };
        cavity_occlusion: f32 = 0.32;
        downsample_levels_average_ao: f32 = 0.25;
        downsample_levels_filtered_roughness_1: f32 = 0.25;
        downsample_levels_filtered_roughness_2: f32 = 0.35;
        rock_field_height_1: f32 = 0.08;
        rock_field_height_2: f32 = 0.06;
        rock_field_height_3: f32 = 0.03;
        rock_field_height_4: f32 = 0.018;
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
