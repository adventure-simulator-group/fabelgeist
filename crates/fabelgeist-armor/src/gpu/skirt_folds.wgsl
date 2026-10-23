// Native authoring words. Depths are metres; layout shares are unit fractions.
const PLEAT_COUNT: u32 = 6u;
const PLEAT_DEPTH: u32 = 7u;
const PLEAT_STYLE: u32 = 8u;
const PLEAT_ROTATION: u32 = 9u;
const PLEAT_FRONT_DEPTH: u32 = 16u;
const PLEAT_WAIST_DEPTH: u32 = 17u;
const PLEAT_FRONT_PANEL: u32 = 18u;
const PLEAT_WAIST_COUNT: u32 = 19u;
const PLEAT_GATHER_FALLOFF: u32 = 20u;
const PLEAT_GATHER_RETENTION: u32 = 22u;

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
    let rounded = pow(0.5 - 0.5 * cos(TAU * phase), 0.7);
    let accordion = 1.0 - abs(2.0 * phase - 1.0);
    // Canonical +Z is anterior. Front depth and panel width are independent:
    // the former softens folds, the latter changes their angular arrangement.
    let front = pow(max(cos(TAU * around), 0.0), 2.0);
    let distribution = mix(1.0, design[PLEAT_FRONT_DEPTH], front);
    let depth = design[PLEAT_DEPTH] * sqrt(descent);
    let fine_phase = chart * design[PLEAT_WAIST_COUNT] + design[PLEAT_ROTATION];
    let gathering = (0.5 - 0.5 * cos(TAU * fine_phase)) * design[PLEAT_WAIST_DEPTH]
        * mix(1.0, design[PLEAT_GATHER_RETENTION], smoothstep(0.0, design[PLEAT_GATHER_FALLOFF], descent));
    return (mix(rounded, accordion, design[PLEAT_STYLE]) * depth + gathering)
        * release * distribution * edge;
}
