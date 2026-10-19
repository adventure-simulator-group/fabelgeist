// Exact grown tube evaluation, shared by drawing and native authoring.
#define_import_path adventuresim_environment::branch_geometry
#import adventuresim_environment::forest::{TAU, Branch, basis, octahedral}

struct BranchVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    light: f32,
};

struct AxisPoint {
    center: vec3<f32>,
    tangent: vec3<f32>,
};

// A point of a segment's axis at `s` in `[0, 1]`: its centre and tangent.
fn axis_at(branch: Branch, s: f32) -> AxisPoint {
    let p0 = branch.a.xyz;
    let p1 = branch.b.xyz;
    let t0 = octahedral(branch.ta);
    let t1 = octahedral(branch.tb);
    let len = max(distance(p0, p1), 1e-6);
    let s2 = s * s;
    let s3 = s2 * s;
    var out: AxisPoint;
    out.center = (2.0 * s3 - 3.0 * s2 + 1.0) * p0 + (s3 - 2.0 * s2 + s) * len * t0
        + (3.0 * s2 - 2.0 * s3) * p1 + (s3 - s2) * len * t1;
    out.tangent = normalize((6.0 * s2 - 6.0 * s) * p0 + (3.0 * s2 - 4.0 * s + 1.0) * len * t0
        + (6.0 * s - 6.0 * s2) * p1 + (3.0 * s2 - 2.0 * s) * len * t1);
    return out;
}

fn branch_vertex(branch: Branch, k: u32, j: u32, sides: u32, spans: u32) -> BranchVertex {
    let s = f32(j) / f32(spans);
    // The end rings are taken as stored, so a neighbour sharing one builds
    // the very same vertices.
    var point: AxisPoint;
    if (j == 0u) {
        point.center = branch.a.xyz;
        point.tangent = octahedral(branch.ta);
    } else if (j == spans) {
        point.center = branch.b.xyz;
        point.tangent = octahedral(branch.tb);
    } else {
        point = axis_at(branch, s);
    }
    let radius = mix(branch.a.w, branch.b.w, s);
    let frame = basis(point.tangent);
    let theta = TAU * f32(k) / f32(sides);
    let n = cos(theta) * frame[0] + sin(theta) * frame[1];
    let surface = point.center + n * radius;
    var out: BranchVertex;
    out.position = surface;
    out.normal = n;
    out.light = mix(branch.light_start, branch.light, s);
    return out;
}
