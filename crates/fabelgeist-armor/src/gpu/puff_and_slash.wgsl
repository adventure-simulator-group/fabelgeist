const COURSES: u32 = 0u;
const LENGTH: u32 = 26u;
const PROXIMAL: u32 = 27u;
const CLEARANCE: u32 = 28u;
const THICKNESS: u32 = 29u;
const CUFF_LENGTH: u32 = 30u;
const CUFF_FLARE: u32 = 31u;
const HEAD_TAPER: u32 = 32u;

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let full = 2.0 * fit.half_extents.y;
    let span = full * design[LENGTH];
    let low = -fit.half_extents.y + (full - span) * design[PROXIMAL];
    let y = low + span * coord.y + coord.z;
    var fullness = textile_fullness(coord.y, coord.z / span, coord.w, COURSES);
    if (design[HEAD_TAPER] > 0.0) { fullness *= clamp((1.0 - coord.y) / design[HEAD_TAPER], 0.0, 1.0); }
    var allowance = design[CLEARANCE] + design[THICKNESS] * select(2.0, 1.0, coord.w < 1.0) + fullness;
    // The cuff turns outward toward its free edge, below the distal band.
    if (coord.w == 3.0) { allowance += design[CUFF_FLARE] * clamp(-coord.z / design[CUFF_LENGTH], 0.0, 1.0); }
    return ShellVertex(vec3<f32>(
        (fit.half_extents.x + allowance) * sin(coord.x), y,
        (fit.half_extents.z + allowance) * cos(coord.x)), 0.0);
}
