// Where an impostor's views were taken from, shared by the bake and the
// draw. A tree is seen from `frames` by `frames` directions over the upper
// hemisphere, laid out hemi-octahedrally: a view's cell in the grid is where
// its direction lands, so the three views nearest any direction are three
// neighbouring cells. Each view is orthographic, `radius` across from the
// tree's centre to a cell's edge.
#define_import_path adventuresim_environment::impostor_frames

// A unit direction at or above the horizon to the square `[-1, 1]²`.
fn hemi_encode(v: vec3<f32>) -> vec2<f32> {
    let p = v.xz / (abs(v.x) + abs(v.y) + abs(v.z));
    return vec2<f32>(p.x + p.y, p.x - p.y);
}

fn hemi_decode(e: vec2<f32>) -> vec3<f32> {
    let p = vec2<f32>(e.x + e.y, e.x - e.y) * 0.5;
    let y = 1.0 - abs(p.x) - abs(p.y);
    return normalize(vec3<f32>(p.x, y, p.y));
}

// The direction view `cell` looks back along: from the tree to the eye.
fn frame_direction(cell: vec2<u32>, frames: u32) -> vec3<f32> {
    let e = (vec2<f32>(cell) + 0.5) / f32(frames) * 2.0 - 1.0;
    return hemi_decode(e);
}

// A view's right and up, seen along `-d`.
fn frame_basis(d: vec3<f32>) -> mat2x3<f32> {
    var right = cross(vec3<f32>(0.0, 1.0, 0.0), d);
    if dot(right, right) < 1e-8 {
        right = vec3<f32>(1.0, 0.0, 0.0);
    }
    right = normalize(right);
    return mat2x3<f32>(right, cross(d, right));
}

// Where a point of view `cell`, at `local` (right and up, in radii from the
// centre), lies in a layer of the atlas, zero to one.
fn atlas_uv(cell: vec2<u32>, local: vec2<f32>, frames: u32) -> vec2<f32> {
    return (vec2<f32>(cell) + 0.5 + 0.5 * vec2<f32>(local.x, -local.y)) / f32(frames);
}

// Turn `v` about the vertical by `angle`, radians.
fn turn(v: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(c * v.x + s * v.z, v.y, -s * v.x + c * v.z);
}
