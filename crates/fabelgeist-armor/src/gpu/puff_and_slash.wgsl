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
    let fullness = textile_fullness(coord.y, coord.z / span, coord.w, vec4<f32>(design[PUFFS], design[FULLNESS], design[DISTAL], design[ROUNDNESS]), design[CONSTRICTION]);
    let allowance = design[CLEARANCE] + design[THICKNESS] * select(2.0, 1.0, coord.w == 0.0) + fullness;
    return ShellVertex(vec3<f32>(
        (fit.half_extents.x + allowance) * sin(coord.x), y,
        (fit.half_extents.z + allowance) * cos(coord.x)), 0.0);
}
