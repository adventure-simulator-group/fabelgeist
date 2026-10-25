const PUFFS: u32 = 0u;
const FULLNESS: u32 = 1u;
const DISTAL: u32 = 2u;
const ROUNDNESS: u32 = 3u;
const CONSTRICTION: u32 = 4u;
const LENGTH: u32 = 5u;
const PROXIMAL: u32 = 6u;
const CLEARANCE: u32 = 7u;
const THICKNESS: u32 = 8u;

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let full = 2.0 * fit.half_extents.y;
    let span = full * design[LENGTH];
    let low = -fit.half_extents.y + (full - span) * design[PROXIMAL];
    let y = low + span * coord.y + coord.z;
    let course = 1.0 / design[PUFFS];
    let puff = min(floor(coord.y / course), design[PUFFS] - 1.0);
    let proximal = puff / max(design[PUFFS] - 1.0, 1.0);
    let scale = select(mix(design[DISTAL], 1.0, proximal), 1.0, design[PUFFS] == 1.0);
    var edge = 0.0;
    if (coord.w == 1.0) { edge = course * design[CONSTRICTION] * 0.5; }
    let phase = clamp(((coord.y + coord.z / span) - puff * course - edge) / (course - 2.0 * edge), 0.0, 1.0);
    var fullness = design[FULLNESS] * scale * pow(max(sin(PI * phase), 0.0), design[ROUNDNESS]);
    if (coord.w == 2.0) { fullness = 0.0; }
    let allowance = design[CLEARANCE] + design[THICKNESS] * select(2.0, 1.0, coord.w == 0.0) + fullness;
    return ShellVertex(vec3<f32>(
        (fit.half_extents.x + allowance) * sin(coord.x), y,
        (fit.half_extents.z + allowance) * cos(coord.x)), 0.0);
}
