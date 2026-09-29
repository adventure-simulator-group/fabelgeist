//! The gorget bib's directional seating cage on the device: in each direction
//! around the neck, the bib moves along a line that lifts over the trapezius
//! at the sides and turns into depth at the front and back, until it clears
//! the highest body surface under it.

/// One invocation per measured sample below the collar seam: the deepest
/// body crossing of the sample's ray, and the offset that clears it.
pub(crate) const MEASURE: &str = r#"
const DETERMINANT_EPSILON: f32 = 1e-10;

// Where a depth ray crosses a triangle rotated into the ray's directional
// section; `y` is one when it does.
fn ray_depth(p: vec2<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec2<f32> {
    let determinant = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if (abs(determinant) < DETERMINANT_EPSILON) {
        return vec2<f32>(0.0, 0.0);
    }
    let u = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / determinant;
    let v = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / determinant;
    if (u >= 0.0 && v >= 0.0 && u + v <= 1.0) {
        return vec2<f32>(u * a.z + v * b.z + (1.0 - u - v) * c.z, 1.0);
    }
    return vec2<f32>(0.0, 0.0);
}

fn rotated(vertex: u32, direction: vec3<f32>) -> vec3<f32> {
    let b = points_at(vertex);
    return vec3<f32>(
        b.x,
        b.y * direction.z - b.z * direction.y,
        b.y * direction.y + b.z * direction.z,
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= (BIB_ROWS - 1u) * BIB_COLUMNS) {
        return;
    }
    let row = index / BIB_COLUMNS + 1u;
    let column = index % BIB_COLUMNS;
    let t = f32(row) / f32(BIB_ROWS - 1u);
    let angle = TAU * f32(column) / f32(BIB_COLUMNS - 1u);
    let control = control_angle(angle);
    let p = bib_point(t, control);
    let direction = bib_direction(angle);
    let query = vec2<f32>(p.x, p.y * direction.z - p.z * direction.y);
    let upper_height = bib_point(0.0, control).y;
    let origin = p.y * direction.y + p.z * direction.z;
    var deepest = -INFINITY;
    var hit = false;
    for (var f = 0u; f < params.faces_count; f = f + 1u) {
        let crossing = ray_depth(
            query,
            rotated(faces[f * 3u], direction),
            rotated(faces[f * 3u + 1u], direction),
            rotated(faces[f * 3u + 2u], direction),
        );
        if (crossing.y == 0.0) {
            continue;
        }
        // The head above the collar is not shoulder.
        let hit_height = query.y * direction.z + crossing.x * direction.y;
        if (hit_height <= upper_height) {
            deepest = max(deepest, crossing.x);
            hit = true;
        }
    }
    var offset = 0.0;
    if (hit) {
        offset = deepest + params.padding - origin;
        if (offset < 0.0) {
            offset = offset * t;
        }
    }
    fit[BIB_RAW + row * BIB_COLUMNS + column] = offset;
}
"#;

/// The bib offsets smoothed by a 3x3 binomial filter, periodic around the neck,
/// keeping the collar seam's row at zero.
pub(crate) const SMOOTH: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= BIB_ROWS * BIB_COLUMNS) {
        return;
    }
    let row = i32(index / BIB_COLUMNS);
    let column = i32(index % BIB_COLUMNS);
    var value = 0.0;
    if (row != 0) {
        let weights = array<f32, 3>(0.25, 0.5, 0.25);
        let period = i32(BIB_COLUMNS) - 1;
        for (var dr = 0; dr < 3; dr = dr + 1) {
            let r = clamp(row + dr - 1, 0, i32(BIB_ROWS) - 1);
            for (var dc = 0; dc < 3; dc = dc + 1) {
                let c = ((column + dc - 1) % period + period) % period;
                value = value + fit[BIB_RAW + u32(r) * BIB_COLUMNS + u32(c)] * weights[dr] * weights[dc];
            }
        }
    }
    fit[BIB + index] = value;
}
"#;
