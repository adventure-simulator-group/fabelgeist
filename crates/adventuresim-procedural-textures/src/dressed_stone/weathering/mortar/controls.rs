//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        aggregate_cells: i32 = AGGREGATE_CELLS;
        stroke_cells: i32 = STROKE_CELLS;
        stroke_density: f32 = STROKE_DENSITY;
        contact_width_metres: f32 = CONTACT_WIDTH_METRES;
        binder_height: f32 = BINDER_HEIGHT;
        aggregate_relief: f32 = AGGREGATE_RELIEF;
        stroke_relief: f32 = STROKE_RELIEF;
        recess_variation: f32 = RECESS_VARIATION;
        contact_buildup: f32 = CONTACT_BUILDUP;
        pocket_depth: f32 = POCKET_DEPTH;
        aggregate_radius_cells: [f32; 2] = AGGREGATE_RADIUS_CELLS;
        stroke_angle_spread: f32 = STROKE_ANGLE_SPREAD;
        stroke_taper_cells: [f32; 2] = STROKE_TAPER_CELLS;
        stroke_curvature: f32 = STROKE_CURVATURE;
        stroke_half_width_cells: f32 = STROKE_HALF_WIDTH_CELLS;
        pocket_radii_cells: [f32; 2] = POCKET_RADII_CELLS;
        recess_noise_scale: f32 = RECESS_NOISE_SCALE;
    }
}
