//! WGSL shared by the underlayer kernels.
//!
//! Vector arithmetic is spelled out component by component in a fixed order
//! rather than left to the device's built-in `dot` and `cross`, so threshold
//! decisions do not depend on the device's evaluation order.

use super::status::FitFailure;

/// A named `f32` constant, exact to the bit.
pub(super) fn constant(name: &str, value: f32) -> String {
    format!("const {name}: f32 = {value:?}f;\n")
}

/// The status bits as WGSL constants, and `fail` to raise one.
pub(super) fn status() -> String {
    format!(
        r#"
const STATUS_CROWDED_WELD: u32 = {}u;
const STATUS_LARGE_GROUP: u32 = {}u;
const STATUS_UNSETTLED: u32 = {}u;
const STATUS_LONG_RAY: u32 = {}u;
const STATUS_NO_INFLUENCE: u32 = {}u;

fn fail(bit: u32) {{
    atomicOr(&status[0], bit);
}}
"#,
        FitFailure::CrowdedWeld as u32,
        FitFailure::LargeGroup as u32,
        FitFailure::Unsettled as u32,
        FitFailure::LongRay as u32,
        FitFailure::NoInfluence as u32,
    )
}

/// Dot and cross products, their products and sums in a fixed order.
pub(super) const VECTOR: &str = r#"
fn dot3(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}

fn cross3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}
"#;

/// A body vertex's physical group: `links[2v]` is its lowest-index
/// coincident vertex, `links[2v + 1]` the next one above it, or `NO_VERTEX`.
pub(super) const GROUPS: &str = r#"
const NO_VERTEX: u32 = 0xffffffffu;
// The most coincident vertices one physical vertex may gather.
const GROUP_CAPACITY: u32 = 8u;
"#;
