//! The bracer fitted on the device.
//!
//! One invocation per body vertex places it along the forearm and decides
//! whether the forearm supports it; the shared anatomical selection splits
//! the supported faces at atlas seams; the bracer is generated on the
//! selected skin of the wearer and displaced off each morph sample's skin.
//! Everything is read back once, at the end.

use anyhow::{Result, bail, ensure};
use fabelgeist_armor::gpu::anatomy::{
    DeviceSeams, DeviceSurface, STATUS_EMPTY_SELECTION, SeamTopology,
};
use fabelgeist_armor::gpu::body::{BodySurface, GpuBody};
use fabelgeist_armor::gpu::bracer::{BracerBody, DeviceBracer, ForearmSkin};
use fabelgeist_armor::gpu::wgsl;
use fabelgeist_armor::{ArmorGpu, BracerDesign, GeneratedArmor};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};
use fabelgeist_rig::{RigJointLookupError, RigJointMembership, RigJointName, RigJointOrdinal};

use crate::bracer::{ForearmSide, ForearmSurfaceInput};

/// Skin weight of the forearm and its neighbours that supports a vertex.
const FOREARM_WEIGHT_THRESHOLD: f32 = 0.35;
/// How far past the elbow and wrist landmarks support extends, as a
/// fraction of the forearm.
const AXIAL_SUPPORT_MARGIN: f32 = 0.1;
/// Set when the elbow and wrist landmarks coincide.
const STATUS_COINCIDENT_LANDMARKS: u32 = 32;

impl ForearmSide {
    fn joint(
        self,
        name: &RigJointName,
        joint_names: &[RigJointName],
    ) -> std::result::Result<RigJointOrdinal, RigJointLookupError> {
        name.require_in(joint_names)
    }
}

/// Fit `design` to the wearer and each morph sample of `input` on `gpu`.
pub fn generate_bracer_on_device(
    gpu: &ArmorGpu,
    design: &BracerDesign,
    input: ForearmSurfaceInput<'_>,
) -> Result<GeneratedArmor> {
    input.validate()?;
    let side = input.side;
    let lowarm = side.joint(side.lowarm(), input.joint_names)?;
    let wrist = side.joint(side.wrist(), input.joint_names)?;
    let supports = input
        .joint_names
        .iter()
        .map(|name: &RigJointName| -> RigJointMembership { side.supports_forearm_boundary(name) })
        .collect::<Vec<_>>();
    let vertex_count = input.positions.len();
    let unused_texcoords = vec![[0.0f32; 2]; vertex_count];
    let body = GpuBody::new(
        gpu,
        BodySurface {
            positions: input.positions,
            normals: input.normals,
            faces: input.faces,
            texcoords: &unused_texcoords,
            joint_indices: input.joint_indices,
            joint_weights: input.joint_weights,
            joints: input.global_joint_states,
        },
    )?;
    let seams = DeviceSeams::new(gpu, &SeamTopology::new(input.faces, input.texcoord_faces)?)?;
    let atlas = gpu.upload(BufferUpload::from_elements(input.texcoords))?;
    let status = gpu.scratch((4u64).into(), ("bracer status").into())?;

    let mut batch = gpu.batch(("bracer").into());
    let support = gpu.scratch((vertex_count as u64 * 4).into(), ("forearm support").into())?;
    let axial = gpu.scratch((vertex_count as u64 * 4).into(), ("forearm axial").into())?;
    record_support(
        gpu,
        &mut batch,
        &body,
        [lowarm, wrist],
        &gpu.upload(BufferUpload::from_elements(
            &supports.into_iter().map(u32::from).collect::<Vec<_>>(),
        ))?,
        &support,
        &axial,
        &status,
    )?;
    let surface = DeviceSurface::record(gpu, &mut batch, &seams, &body.faces, &support, &status)?;
    let mut bracer = DeviceBracer::record(
        gpu,
        &mut batch,
        design,
        ForearmSkin {
            surface: &surface,
            axial: &axial,
            atlas: &atlas,
            body: &body,
        },
        &status,
    )?;
    batch.submit();
    for morph in input.morphs {
        let positions = gpu.upload(BufferUpload::from_elements(&morph.positions))?;
        let normals = gpu.upload(BufferUpload::from_elements(&morph.normals))?;
        let mut batch = gpu.batch(("bracer morph").into());
        bracer.record_morph(
            gpu,
            &mut batch,
            BracerBody {
                positions: &positions,
                normals: &normals,
            },
        )?;
        batch.submit();
    }

    let bits = gpu.read::<u32>(&status)?[0];
    if bits & STATUS_COINCIDENT_LANDMARKS != 0 {
        bail!("MHR forearm landmarks coincide");
    }
    if bits & STATUS_EMPTY_SELECTION != 0 {
        bail!("MHR forearm skin selected no faces");
    }
    let names = input
        .morphs
        .iter()
        .map(|morph| morph.name.clone())
        .collect::<Vec<_>>();
    Ok(bracer.read(gpu, input.domain, &names)?)
}

/// Record each body vertex's clamped axial coordinate, and whether the
/// forearm supports it.
#[expect(clippy::too_many_arguments, reason = "the device buffers of one pass")]
fn record_support(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    body: &GpuBody,
    [lowarm, wrist]: [RigJointOrdinal; 2],
    supports: &Buffer,
    support: &Buffer,
    axial: &Buffer,
    status: &Buffer,
) -> Result<()> {
    ensure!(
        body.vertex_count > 0,
        "forearm surface inputs are inconsistent"
    );
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (body.vertex_count).into());
    parameters.insert("lowarm".into(), (usize::from(lowarm) as u32).into());
    parameters.insert("wrist".into(), (usize::from(wrist) as u32).into());
    parameters.insert("pad0".into(), (0u32).into());
    parameters.insert("positions".into(), (body.positions.clone()).into());
    parameters.insert("joints".into(), (body.joints.clone()).into());
    parameters.insert("joint_indices".into(), (body.joint_indices.clone()).into());
    parameters.insert("joint_weights".into(), (body.joint_weights.clone()).into());
    parameters.insert("supports".into(), (supports.clone()).into());
    parameters.insert("support".into(), (support.clone()).into());
    parameters.insert("axial".into(), (axial.clone()).into());
    parameters.insert("status".into(), (status.clone()).into());
    let kernel = gpu
        .cache()
        .get(gpu.context(), &support_source())
        .map_err(fabelgeist_armor::GenerateError::from)?;
    batch
        .dispatch_items(&kernel, &parameters, (body.vertex_count).into())
        .map_err(fabelgeist_armor::GenerateError::from)?;
    Ok(())
}

fn support_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> joints: array<f32>;
@group(0) @binding(2) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(3) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(4) var<storage, read> supports: array<u32>;
@group(0) @binding(5) var<storage, read_write> support: array<u32>;
@group(0) @binding(6) var<storage, read_write> axial: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    lowarm: u32,
    wrist: u32,
    pad0: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{status_code}
{positions}

const WEIGHT_THRESHOLD: f32 = {threshold};
const MARGIN: f32 = {margin};
const COINCIDENT_LANDMARKS: u32 = {coincident}u;
// Rust's `f32::EPSILON`.
const EPSILON: f32 = 1.1920929e-7;

fn joint_position(joint: u32) -> vec3<f32> {{
    return vec3<f32>(joints[joint * 8u], joints[joint * 8u + 1u], joints[joint * 8u + 2u]);
}}

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {{
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let v = id.x;
    if (v >= params.count) {{
        return;
    }}
    let proximal = joint_position(params.lowarm);
    let axis = joint_position(params.wrist) - proximal;
    let length_squared = host_dot(axis, axis);
    if (length_squared <= EPSILON) {{
        fail(COINCIDENT_LANDMARKS);
        return;
    }}
    let along = host_dot(positions_at(v) - proximal, axis) / length_squared;
    var weight = 0.0;
    for (var k = 0u; k < 8u; k = k + 1u) {{
        if (supports[joint_indices[v * 8u + k]] != 0u) {{
            weight = weight + joint_weights[v * 8u + k];
        }}
    }}
    let supported = along >= -MARGIN && along <= 1.0 + MARGIN && weight >= WEIGHT_THRESHOLD;
    support[v] = select(0u, 1u, supported);
    axial[v] = clamp(along, 0.0, 1.0);
}}
"#,
        status_code = wgsl::STATUS,
        positions = wgsl::read_points("positions"),
        threshold = FOREARM_WEIGHT_THRESHOLD,
        margin = AXIAL_SUPPORT_MARGIN,
        coincident = STATUS_COINCIDENT_LANDMARKS,
    ))
}
