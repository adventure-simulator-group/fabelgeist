// Native authoring words. Depths are metres; layout shares are unit fractions.
const PLEAT_COUNT: u32 = 6u;
const PLEAT_DEPTH: u32 = 7u;
const PLEAT_STYLE: u32 = 8u;
const PLEAT_ROTATION: u32 = 9u;
const PLEAT_FRONT_DEPTH: u32 = 16u;
const PLEAT_FRONT_PANEL: u32 = 17u;

const PLEAT_FRONT_STYLE_WIDTH: u32 = 24u;
const PLEAT_FRONT_STYLE: u32 = 25u;

fn fold_profile(phase: f32, style: f32) -> f32 {
    if (style > 1.5) {
        if (phase < KNIFE_FACE) { return phase / KNIFE_FACE; }
        return (1.0 - phase) / (1.0 - KNIFE_FACE);
    }
    if (style > 0.5) { return 1.0 - abs(2.0 * phase - 1.0); }
    return pow(0.5 - 0.5 * cos(TAU * phase), 0.7);
}

fn skirt_fold(column: f32, descent: f32, release: f32) -> f32 {
    let around = fract(column / f32(COLUMNS));
    let front_half = design[PLEAT_FRONT_PANEL] * 0.5;
    var chart = around;
    var edge = 1.0;
    if (front_half > 0.0) {
        // Reserve a quiet panel and distribute the gathers over the remaining
        // cloth. Feather its seams over one angular cell, including rotations
        // whose fold crest would otherwise jump abruptly at the panel edge.
        if (around <= front_half || around >= 1.0 - front_half) { return 0.0; }
        chart = (around - front_half) / (1.0 - 2.0 * front_half);
        edge = smoothstep(0.0, 1.0 / f32(COLUMNS), min(around - front_half, 1.0 - front_half - around));
    }
    let phase = fract(chart * design[PLEAT_COUNT] + design[PLEAT_ROTATION]);
    var profile = fold_profile(phase, design[PLEAT_STYLE]);
    let half_width = design[PLEAT_FRONT_STYLE_WIDTH] * 0.5;
    if (half_width > 0.0) {
        let front_distance = min(around, 1.0 - around);
        let zone = smoothstep(0.0, 1.0 / f32(COLUMNS), half_width - front_distance);
        profile = mix(profile, fold_profile(phase, design[PLEAT_FRONT_STYLE]), zone);
    }
    // Canonical +Z is anterior. Front depth and panel width are independent:
    // the former softens folds, the latter changes their angular arrangement.
    let front = pow(max(cos(TAU * around), 0.0), 2.0);
    let distribution = mix(1.0, design[PLEAT_FRONT_DEPTH], front);
    let depth = design[PLEAT_DEPTH] * sqrt(descent);
    // Fine waist gathering is a physical height field in the material chart.
    // Only the broad hanging falls contribute to the carrier silhouette.
    return profile * depth
        * release * distribution * edge;
}
