// Shared course profile: axial runs distal to proximal; seam is axial fraction.
// A role below one is the lining; its value is the share of fullness it
// gives up, which sets it back under the outer cloth.
// `table` is the design word holding the course count, followed by the
// roundness and the axial end, fullness in metres and half band of each course.
const COURSE_WORDS: u32 = 3u;

fn textile_fullness(axial: f32, seam: f32, role: f32, table: u32) -> f32 {
    // Fitted bands and cuffs never release.
    if (role >= 2.0) { return 0.0; }
    let last = u32(design[table]) - 1u;
    var course = 0u;
    var start = 0.0;
    while (course < last && axial >= design[table + 2u + COURSE_WORDS * course]) {
        start = design[table + 2u + COURSE_WORDS * course];
        course += 1u;
    }
    let row = table + 2u + COURSE_WORDS * course;
    // Both cloth layers release from the same constricted band edges.
    // Otherwise the lining bulges through the outer pane ends.
    let edge = design[row + 2u];
    let phase = clamp(((axial + seam) - start - edge) / (design[row] - start - 2.0 * edge), 0.0, 1.0);
    let kept = 1.0 - select(0.0, role, role < 1.0);
    return kept * design[row + 1u] * pow(max(sin(PI * phase), 0.0), design[table + 1u]);
}
