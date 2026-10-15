// Shared textile chart placed over the hanging section of a skirt carrier.
const PUFFS: u32 = 10u;
const FULLNESS: u32 = 11u;
const DISTAL: u32 = 12u;
const PUFF_ROUNDNESS: u32 = 13u;
const CONSTRICTION: u32 = 14u;
const CLOTH_GAUGE: u32 = 15u;

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let rows = f32(ROWS) - design[FIRST_SKIRT_ROW];
    let span = 2.0 * fit.half_extents.y * rows / f32(ROWS);
    let axial = coord.y + coord.z / span;
    let row = design[FIRST_SKIRT_ROW] + rows * (1.0 - axial);
    let column = coord.x / TAU * f32(COLUMNS);
    var point = skirt_point(column, row);
    let fullness = textile_fullness(coord.y, coord.z / span, coord.w,
        vec4<f32>(design[PUFFS], design[FULLNESS], design[DISTAL], design[PUFF_ROUNDNESS]), design[CONSTRICTION]);
    let radial = normalize(point.xz);
    point += vec3<f32>(radial.x, 0.0, radial.y) * (design[CLOTH_GAUGE] + fullness);
    return ShellVertex(point, 0.0);
}
