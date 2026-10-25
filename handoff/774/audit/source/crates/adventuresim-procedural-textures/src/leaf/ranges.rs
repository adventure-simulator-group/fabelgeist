//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
pub fn parameter_range(name: &str) -> Option<(f32, f32)> {
    Some(match name {
        "half_length" => (0.30, 0.78),
        "half_width" => (0.06, 0.62),
        "widest_at" => (0.10, 0.90),
        "base_exponent" | "tip_exponent" => (0.20, 3.5),
        "base_width" | "tip_width" => (0.0, 0.75),
        "asymmetry" => (-0.45, 0.45),
        "bend" => (-0.28, 0.28),
        "petiole_length" => (0.02, 0.32),
        "petiole_width" | "midrib_width" => (0.002, 0.035),
        "base_notch_depth" | "tip_notch_depth" => (0.0, 0.30),
        "base_notch_width" | "tip_notch_width" => (0.0, 0.26),
        "lobe_frequency" => (0.0, 16.0),
        "lobe_depth" => (0.0, 0.76),
        "lobe_roundness" => (0.15, 4.0),
        "lobe_stagger" => (-0.35, 0.35),
        "tooth_frequency" => (0.0, 40.0),
        "tooth_depth" => (0.0, 0.24),
        "tooth_sharpness" => (0.20, 5.0),
        "tooth_lean" => (0.10, 0.90),
        "secondary_count" => (0.0, 14.0),
        "secondary_first" => (0.06, 0.42),
        "secondary_last" => (0.58, 0.94),
        "secondary_reach" => (0.20, 1.0),
        "secondary_sweep" => (-0.20, 0.20),
        "secondary_curve" => (-0.25, 0.25),
        "secondary_width" => (0.002, 0.022),
        "secondary_tip_width" => (0.05, 0.65),
        "lobe_basal_scale" | "lobe_middle_scale" | "lobe_apical_scale" => (0.20, 1.30),
        "lobe_progressive_sweep" => (-0.20, 0.20),
        "secondary_tooth_ratio" => (1.0, 4.0),
        "margin_hierarchy"
        | "vein_alternation"
        | "vein_gradation"
        | "vein_branching"
        | "vein_loops"
        | "palmate"
        | "compound_separation"
        | "fan"
        | "parallel_venation"
        | "organ_alternation"
        | "dichotomy" => (0.0, 1.0),
        "margin_asymmetry" => (-1.0, 1.0),
        "lobe_bristle" => (0.0, 0.12),
        "organ_count" => (1.0, 11.0),
        "organ_spread" => (0.20, 1.50),
        "organ_length" => (0.08, 1.15),
        "organ_width" => (0.018, 0.32),
        "organ_attachment" => (0.0, 0.30),
        "terminal_scale" => (0.35, 1.30),
        "bipinnate" => (0.0, 1.0),
        "pinna_count" => (1.0, 6.0),
        "leaflet_count" => (1.0, 10.0),
        "pinna_length" => (0.10, 0.52),
        "basal_obliquity" => (-0.15, 0.15),
        "landmark_lobes" => (0.0, 0.65),
        "landmark_side_bias" => (-1.0, 1.0),
        "apex_truncation" => (0.0, 0.22),
        "frond" | "fan_segmentation" | "longitudinal_veins" => (0.0, 1.0),
        "peltate_depth" => (0.0, 0.42),
        _ => return None,
    })
}
