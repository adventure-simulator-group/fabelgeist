//! Garment kernels: WGSL recorded with generated bindings.
//!
//! Every garment kernel binds a few named storage buffers and a block of
//! named uniform words. [`dispatch`] generates both from the call, prepends
//! the shared WGSL -- fixed-order arithmetic, frame helpers, and point
//! accessors for buffers of packed points -- and records the dispatch.

use anyhow::Result;
use fabelgeist_armor::gpu::{device_error, wgsl};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use crate::device_frames::DeviceWearer;

/// Vector arithmetic in a fixed order of operations, so kernels round the same
/// way on every device, plus the fit's failure word.
pub(crate) const HOST: &str = r#"
// The word of a fit that is nonzero once the fit has failed.
const FIT_FAILED: u32 = 15u;
// Exact largest finite f32. A rounded decimal above this value is rejected
// by browser WGSL parsers even when a native backend rounds it down.
const INFINITY: f32 = MAX_FINITE;

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}

fn host_cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}

// A zero vector has no direction. Scale first so small valid vectors survive
// squaring and large finite vectors do not overflow the length calculation.
fn host_normalized(a: vec3<f32>) -> vec3<f32> {
    let scale = max(max(abs(a.x), abs(a.y)), abs(a.z));
    if (scale == 0.0) { return vec3<f32>(0.0); }
    let scaled = a / scale;
    return scaled / sqrt(host_dot(scaled, scaled));
}

// Linear interpolation from `a` to `b`.
fn mix3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a * (1.0 - t) + b * t;
}

fn host_local(f: Frame, p: vec3<f32>) -> vec3<f32> {
    let d = p - f.origin;
    return vec3<f32>(host_dot(f.x, d), host_dot(f.y, d), host_dot(f.z, d));
}

fn host_point(f: Frame, l: vec3<f32>) -> vec3<f32> {
    return f.origin + ((f.x * l.x + f.y * l.y) + f.z * l.z);
}

// `f32::rem_euclid(x, TAU)` for |x| below two turns.
fn rem_tau(x: f32) -> f32 {
    var r = x - TAU * trunc(x / TAU);
    if (r < 0.0) {
        r = r + TAU;
    }
    return r;
}
"#;

/// A storage buffer a garment kernel binds, by name.
pub(crate) struct Bound<'a> {
    pub name: &'static str,
    /// The WGSL type and access, e.g. `read_write> ... array<f32>`.
    pub kind: Access,
    pub buffer: &'a Buffer,
}

#[derive(Clone, Copy)]
pub(crate) enum Access {
    Read,
    ReadU32,
    Write,
    WriteU32,
    Atomic,
}

impl Access {
    fn declaration(self) -> &'static str {
        match self {
            Self::Read => "read> {}: array<f32>",
            Self::ReadU32 => "read> {}: array<u32>",
            Self::Write => "read_write> {}: array<f32>",
            Self::WriteU32 => "read_write> {}: array<u32>",
            Self::Atomic => "read_write> {}: array<atomic<u32>>",
        }
    }
}

pub(crate) fn read<'a>(name: &'static str, buffer: &'a Buffer) -> Bound<'a> {
    Bound {
        name,
        kind: Access::Read,
        buffer,
    }
}
pub(crate) fn read_u32<'a>(name: &'static str, buffer: &'a Buffer) -> Bound<'a> {
    Bound {
        name,
        kind: Access::ReadU32,
        buffer,
    }
}
pub(crate) fn write<'a>(name: &'static str, buffer: &'a Buffer) -> Bound<'a> {
    Bound {
        name,
        kind: Access::Write,
        buffer,
    }
}
pub(crate) fn atomic<'a>(name: &'static str, buffer: &'a Buffer) -> Bound<'a> {
    Bound {
        name,
        kind: Access::Atomic,
        buffer,
    }
}

/// A uniform word a garment kernel reads as `params.<name>`.
#[derive(Clone, Copy)]
pub(crate) enum Word {
    U(&'static str, u32),
    F(&'static str, f32),
}

/// How many invocations a garment kernel runs.
#[derive(Clone, Copy)]
pub(crate) enum Grid {
    /// One per item, in workgroups of 64.
    Items(u32),
    /// One single-invocation workgroup per item.
    Singles(u32),
}

/// Compile and record one garment kernel: its bindings and uniform block
/// are generated from `buffers` and `words`, then `body`, which defines
/// `main`, follows the shared WGSL.
pub(crate) fn dispatch(
    wearer: &DeviceWearer,
    batch: &mut KernelBatch,
    body: &str,
    buffers: &[Bound],
    words: &[Word],
    grid: Grid,
) -> Result<()> {
    let gpu = wearer.gpu;
    let mut source = String::new();
    let mut parameters = PassParameters::new();
    for (binding, bound) in buffers.iter().enumerate() {
        source.push_str(&format!(
            "@group(0) @binding({binding}) var<storage, {};\n",
            bound.kind.declaration().replace("{}", bound.name)
        ));
        parameters.insert(bound.name, bound.buffer.clone());
    }
    source.push_str("struct Params {\n");
    let mut members = words.to_vec();
    let mut pad = 0;
    while members.is_empty() || !members.len().is_multiple_of(4) {
        members.push(Word::U(PADS[pad], 0));
        pad += 1;
    }
    for word in &members {
        match *word {
            Word::U(name, value) => {
                source.push_str(&format!("    {name}: u32,\n"));
                parameters.insert(name, value);
            }
            Word::F(name, value) => {
                source.push_str(&format!("    {name}: f32,\n"));
                parameters.insert(name, value);
            }
        }
    }
    source.push_str(&format!(
        "}};\n@group(0) @binding({}) var<uniform> params: Params;\n",
        buffers.len()
    ));
    source.push_str(wgsl::MATH);
    source.push_str(wgsl::FRAME.replace("frames[", "FRAME_SOURCE[").as_str());
    source.push_str(HOST);
    for bound in buffers {
        if matches!(bound.kind, Access::Read | Access::Write) && POINT_BUFFERS.contains(&bound.name)
        {
            source.push_str(&match bound.kind {
                Access::Write => wgsl::points(bound.name),
                _ => wgsl::read_points(bound.name),
            });
        }
    }
    source.push_str(body);
    // A kernel without a fit still shares the frame helpers.
    let source = if buffers.iter().any(|b| b.name == FIT_BUFFER) {
        source.replace("FRAME_SOURCE[", "fit[")
    } else {
        format!("var<private> no_fit: array<f32, 16>;\n{source}")
            .replace("FRAME_SOURCE[", "no_fit[")
    };
    let kernel = gpu
        .cache()
        .get(gpu.context(), &source)
        .map_err(device_error)?;
    match grid {
        Grid::Items(count) => batch.dispatch_items(&kernel, &parameters, count),
        Grid::Singles(count) => batch.dispatch(&kernel, &parameters, [count, 1, 1]),
    }
    .map_err(device_error)?;
    Ok(())
}

const PADS: [&str; 4] = ["pad0", "pad1", "pad2", "pad3"];
/// The buffer a garment's fit is bound as; kernels read their frame from it.
const FIT_BUFFER: &str = "fit";
/// Buffers of packed points, which get `<name>_at` accessors.
const POINT_BUFFERS: [&str; 5] = ["positions", "normals", "carriers", "points", "extra"];

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_armor::ArmorGpu;
    use fabelgeist_gpu::prelude::BufferUpload;

    #[test]
    fn shader_normalization_preserves_direction_across_finite_scales() {
        let gpu = ArmorGpu::open().unwrap();
        let vectors: [[f32; 4]; 4] = [
            [0.0; 4],
            [3.0e-30, 0.0, 4.0e-30, 0.0],
            [3.0, 0.0, 4.0, 0.0],
            [3.0e30, 0.0, 4.0e30, 0.0],
        ];
        let input = gpu.upload(BufferUpload::from_elements(&vectors)).unwrap();
        let output = gpu
            .scratch(size_of_val(&vectors) as u64, "normalized directions")
            .unwrap();
        let source = format!(
            "{}\nvar<private> frames: array<f32, 16>;\n{}\n{}\n{}",
            wgsl::MATH,
            wgsl::FRAME,
            HOST,
            r#"
            @group(0) @binding(0) var<storage, read> vectors: array<vec4<f32>>;
            @group(0) @binding(1) var<storage, read_write> directions: array<vec4<f32>>;
            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                if (id.x >= 4u) { return; }
                directions[id.x] = vec4<f32>(host_normalized(vectors[id.x].xyz), 0.0);
            }
        "#
        );
        let kernel = gpu.cache().get(gpu.context(), &source).unwrap();
        let mut parameters = PassParameters::new();
        parameters.insert("vectors", input);
        parameters.insert("directions", output.clone());
        let mut batch = gpu.batch("normalization scales");
        batch
            .dispatch_items(&kernel, &parameters, vectors.len() as u32)
            .unwrap();
        batch.submit();
        let directions = gpu.read::<[f32; 4]>(&output).unwrap();
        assert_eq!(directions[0], [0.0; 4]);
        // Division and square root may round each unit-vector component twice.
        let component_roundoff = 2.0 * (0.8_f32.next_up() - 0.8);
        for direction in &directions[1..] {
            for (actual, expected) in direction.iter().zip([0.6, 0.0, 0.8, 0.0]) {
                assert!(
                    (actual - expected).abs() <= component_roundoff,
                    "{direction:?}"
                );
            }
        }
    }
}
