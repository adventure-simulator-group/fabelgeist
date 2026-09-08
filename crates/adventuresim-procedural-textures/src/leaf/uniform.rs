//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
#[cfg_attr(test, derive(serde::Serialize))]
pub(super) struct LeafUniform {
    // half length, maximum half width, widest point (base=0), base exponent
    pub(super) profile: [f32; 4],
    // tip exponent, base endpoint width, tip endpoint width, left/right asymmetry
    pub(super) shape: [f32; 4],
    // axis bend, petiole length, petiole width, midrib width
    pub(super) axis: [f32; 4],
    // base notch depth/width, tip notch depth/width
    pub(super) notches: [f32; 4],
    // frequency, depth, peak exponent, left/right phase stagger
    pub(super) lobes: [f32; 4],
    // frequency, depth, peak exponent, apex lean
    pub(super) teeth: [f32; 4],
    // secondary count, first/last origin, reach toward margin
    pub(super) veins: [f32; 4],
    // upward sweep, curve, base width, width at termination
    pub(super) vein_style: [f32; 4],
    // basal/middle/apical lobe scale, progressive sweep
    pub(super) lobe_grade: [f32; 4],
    // secondary-tooth multiplier, hierarchy blend, margin asymmetry, bristle length
    pub(super) margin_style: [f32; 4],
    // alternation, gradation coupling, branching, loops
    pub(super) venation: [f32; 4],
    // palmate blend, compound separation, fan blend, parallel venation
    pub(super) topology: [f32; 4],
    // organ count, angular spread, organ length, organ width
    pub(super) organs: [f32; 4],
    // attachment position, terminal/central scale, alternation, dichotomy
    pub(super) organ_style: [f32; 4],
    // bipinnate amount, pinna count, leaflet count, pinna length
    pub(super) hierarchy: [f32; 4],
    // basal obliquity, landmark lobes, side bias, apex truncation
    pub(super) landmarks: [f32; 4],
    // frond amount, fan segmentation, peltate depth, longitudinal veins
    pub(super) planar: [f32; 4],
    // canvas width/height, seed, reserved
    pub(super) render: [f32; 4],
}

macro_rules! pack { ($s:ident; $($field:ident),*) => {[$($s.$field),*]}; }
impl LeafUniform {
    pub(super) fn new(shape: &super::LeafShape, size: u32, seed: u64) -> Self {
        Self {
            profile: pack!(shape; half_length, half_width, widest_at, base_exponent),
            shape: pack!(shape; tip_exponent, base_width, tip_width, asymmetry),
            axis: pack!(shape; bend, petiole_length, petiole_width, midrib_width),
            notches: pack!(shape; base_notch_depth, base_notch_width, tip_notch_depth, tip_notch_width),
            lobes: pack!(shape; lobe_frequency, lobe_depth, lobe_roundness, lobe_stagger),
            teeth: pack!(shape; tooth_frequency, tooth_depth, tooth_sharpness, tooth_lean),
            veins: pack!(shape; secondary_count, secondary_first, secondary_last, secondary_reach),
            vein_style: pack!(shape; secondary_sweep, secondary_curve, secondary_width, secondary_tip_width),
            lobe_grade: pack!(shape; lobe_basal_scale, lobe_middle_scale, lobe_apical_scale, lobe_progressive_sweep),
            margin_style: pack!(shape; secondary_tooth_ratio, margin_hierarchy, margin_asymmetry, lobe_bristle),
            venation: pack!(shape; vein_alternation, vein_gradation, vein_branching, vein_loops),
            topology: pack!(shape; palmate, compound_separation, fan, parallel_venation),
            organs: pack!(shape; organ_count, organ_spread, organ_length, organ_width),
            organ_style: pack!(shape; organ_attachment, terminal_scale, organ_alternation, dichotomy),
            hierarchy: pack!(shape; bipinnate, pinna_count, leaflet_count, pinna_length),
            landmarks: pack!(shape; basal_obliquity, landmark_lobes, landmark_side_bias, apex_truncation),
            planar: pack!(shape; frond, fan_segmentation, peltate_depth, longitudinal_veins),
            render: [size as f32, size as f32, (seed % 65536) as f32, 0.0],
        }
        .constrained()
    }
}
