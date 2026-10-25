//! Artist controls and canonical recipe defaults.
use super::*;

crate::parameters::parameter_block! {
    pub struct Parameters {
        knot_flow_strength: f32 = 1.0;
        growth_wander_grid: [i32; 2] = [5,9];
        spacing_warp_grid: [i32; 2] = [9,2];
        fiber_wander_grid: [i32; 2] = [19,17];
        fiber_interruption_grid: [i32; 2] = [53,31];
        knot_taper: f32 = KNOT_TAPER;
        knot_ring_distortion: f32 = KNOT_RING_DISTORTION;
        knot_ring_count: f32 = KNOT_RING_COUNT;
        latewood_shoulder_ratio: f32 = LATEWOOD_SHOULDER_RATIO;
        anatomical_mark_density: f32 = ANATOMICAL_MARK_DENSITY;
        vessel_radii_cells: [f32; 2] = VESSEL_RADII_CELLS;
        ray_radii_cells: [f32; 2] = RAY_RADII_CELLS;
        dark_ring_fraction: f32 = DARK_RING_FRACTION;
        ring_count: f32 = RING_COUNT;
        fiber_count: f32 = FIBER_COUNT;
        ring_spacing_warp: f32 = RING_SPACING_WARP;
        latewood_width: [f32; 2] = LATEWOOD_WIDTH;
        vessel_columns: i32 = VESSEL_COLUMNS;
        vessel_rows: i32 = VESSEL_ROWS;
        ray_columns: i32 = RAY_COLUMNS;
        ray_rows: i32 = RAY_ROWS;
        knot_influence_radii: f32 = KNOT_INFLUENCE_RADII;
        knot_core_softening: f32 = KNOT_CORE_SOFTENING;
        ring_wander: f32 = RING_WANDER;
        fiber_wander: f32 = FIBER_WANDER;
        knots: Vec<Knot> = KNOTS.to_vec();
        fiber_threshold: f32 = 0.45;
        fiber_transition: f32 = 0.55;
        fiber_interruption_threshold: f32 = 0.34;
        fiber_interruption_transition: f32 = 0.40;
        knot_core_start: f32 = 0.35;
        knot_core_transition: f32 = 0.80;
    }
}
