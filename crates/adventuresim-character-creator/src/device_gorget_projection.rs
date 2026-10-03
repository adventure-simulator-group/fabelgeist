//! Directional triangle sections shared by collar and bib fitting.

pub(super) const PROJECTION: &str = r#"
const DETERMINANT_EPSILON: f32 = 1e-10;
const BARYCENTRIC_TOLERANCE: f32 = 16.0 * FLT_EPSILON;
const SECTION_HEIGHT_TOLERANCE_M: f32 = 1e-6;

// Where a depth ray crosses a triangle rotated into the ray's directional
// section; `y` is one when it does.
fn ray_depth(p: vec2<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec2<f32> {
    let determinant = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if (abs(determinant) < DETERMINANT_EPSILON) {
        return vec2<f32>(0.0, 0.0);
    }
    let u = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / determinant;
    let v = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / determinant;
    if (u >= -BARYCENTRIC_TOLERANCE && v >= -BARYCENTRIC_TOLERANCE &&
        u + v <= 1.0 + BARYCENTRIC_TOLERANCE) {
        return vec2<f32>(u * a.z + v * b.z + (1.0 - u - v) * c.z, 1.0);
    }
    return vec2<f32>(0.0, 0.0);
}

fn rotated(b: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        b.x,
        b.y * direction.z - b.z * direction.y,
        b.y * direction.y + b.z * direction.z,
    );
}

"#;
