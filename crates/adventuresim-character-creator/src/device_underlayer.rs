//! Underlayers fitted on the device.
//!
//! The cut plan is made once on the host from the wearer's body and the part
//! frames fitted to it on the device (see [`plan`]). Everything computed from
//! body positions runs on the device: the physical welding of coincident
//! vertices, the constrained offset
//! directions, the layer-stack compression frozen over the wearer, its
//! proportion samples and its morph samples, the two offset layers of every
//! realization, their normals, and the shell's surface coordinates and skin.
//! Each realization is recorded and submitted in turn, and everything is
//! read back once at the end.

mod direction;
mod gap;
pub mod plan;
mod shell;
mod standoff;
mod status;
mod sweep;
mod weld;
mod wgsl;
mod workspace;

pub use direction::{OUTWARD_MARGIN, PROJECTION_PASSES};
pub use gap::{GAP_ALLOCATION, RAY_ROUNDOFF_M};
pub use plan::CutPlan;
pub use standoff::{
    COMPRESSION_PASSES, FOCAL_DISTANCE_FRACTION, LAYER_STACK_ENVELOPE_M, PRISM_MARGIN,
};
pub use status::{FitFailure, FitStatus, UnknownFitStatus};
pub use weld::WELD_PRECISION;

use anyhow::{Context, Result, bail};
use fabelgeist_armor::gpu::device_error;
use fabelgeist_armor::{ArmorGpu, GenerateError};
use fabelgeist_compute::{NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;

use crate::armor_frames::Wearer;
use crate::device_frames::DeviceWearer;
use crate::underlayer::UnderlayerDesign;
use shell::{INFLUENCES, LayerOffsets, SKIN_FLOATS, SkinSources};
use workspace::Workspace;

/// One realization of the wearer's body over the wearer's own triangles.
#[derive(Clone, Copy, Debug)]
pub struct BodyShape<'a> {
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
}

/// Where the body's surface coordinates are: its coordinate triangles, one
/// per body triangle, and the coordinates they index.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceDomain<'a> {
    pub uv_faces: &'a [[u32; 3]],
    pub texcoords: &'a [[f32; 2]],
}

/// A shell's positions and area-weighted normals on one realization.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShellSurface {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

/// An underlayer fitted to the wearer and to each morph sample.
#[derive(Clone, Debug, PartialEq)]
pub struct FittedUnderlayer {
    pub indices: Vec<u32>,
    pub base: ShellSurface,
    /// One per morph sample, in order, sharing `indices`.
    pub endpoints: Vec<ShellSurface>,
    pub texcoords: Vec<[f32; 2]>,
    pub joint_indices: Vec<[u32; 8]>,
    pub joint_weights: Vec<[f32; 8]>,
}

/// Fit an underlayer to the wearer, its static proportion samples and its
/// morph samples, and carry the body's surface coordinates and skin onto it.
pub fn fit(
    gpu: &ArmorGpu,
    design: &UnderlayerDesign,
    placement: &str,
    body: &Wearer<'_>,
    domain: SurfaceDomain<'_>,
    proportions: &[BodyShape<'_>],
    morphs: &[BodyShape<'_>],
) -> Result<FittedUnderlayer> {
    let device_body = body.upload(gpu)?;
    let wearer = DeviceWearer {
        gpu,
        body: &device_body,
        host: body,
    };
    let plan = CutPlan::new(design, placement, body, domain.uv_faces, &|region| {
        wearer.read_frame(region)
    })?;
    Fit {
        gpu,
        design,
        body,
        plan: &plan,
        proportions,
        morphs,
        domain: Some(domain),
    }
    .run()
}

struct Fit<'a> {
    gpu: &'a ArmorGpu,
    design: &'a UnderlayerDesign,
    body: &'a Wearer<'a>,
    plan: &'a CutPlan,
    proportions: &'a [BodyShape<'a>],
    morphs: &'a [BodyShape<'a>],
    domain: Option<SurfaceDomain<'a>>,
}

/// A realization uploaded, with the directions its layers are offset along.
struct Realization {
    positions: Buffer,
    directions: Buffer,
}

impl Fit<'_> {
    fn run(&self) -> Result<FittedUnderlayer> {
        let gpu = self.gpu;
        let body = self.body;
        let vertex_count = body.positions.len();
        if body.normals.len() != vertex_count
            || self
                .proportions
                .iter()
                .chain(self.morphs)
                .any(|s| s.positions.len() != vertex_count || s.normals.len() != vertex_count)
        {
            bail!(GenerateError::InvalidSurface);
        }
        let sets = 1 + self.proportions.len() as u32;
        let mut ws = Workspace::new(gpu, self.plan, body.faces, vertex_count, sets)?;
        let vector_bytes = vertex_count as u64 * 12;
        let links = gpu.scratch(
            (vertex_count as u64 * 8).into(),
            ("underlayer weld links").into(),
        )?;

        // The wearer: its neutral compression, then the proportion samples'
        // face constraints and the directions they leave the wearer.
        let base_positions = gpu.upload(BufferUpload::from_elements(body.positions))?;
        let base_normals = gpu.upload(BufferUpload::from_elements(body.normals))?;
        let neutral = gpu.scratch((vector_bytes).into(), ("underlayer directions").into())?;
        let mut batch = gpu.batch(("underlayer wearer").into());
        ws.record_weld(&mut batch, &base_positions, &links)?;
        ws.record_constraints(&mut batch, &base_positions, 0)?;
        ws.record_directions(&mut batch, &base_normals, &links, 1, &neutral)?;
        ws.record_standoff(&mut batch, &base_positions, &neutral, &links)?;
        let mut samples = Vec::with_capacity(self.proportions.len());
        for (set, sample) in (1..).zip(self.proportions) {
            let positions = gpu.upload(BufferUpload::from_elements(sample.positions))?;
            ws.record_constraints(&mut batch, &positions, set)?;
            samples.push(positions);
        }
        let directions = if sets > 1 {
            let constrained =
                gpu.scratch((vector_bytes).into(), ("underlayer directions").into())?;
            ws.record_directions(&mut batch, &base_normals, &links, sets, &constrained)?;
            constrained
        } else {
            neutral
        };
        ws.record_standoff(&mut batch, &base_positions, &directions, &links)?;
        batch.submit();

        // Bone proportion translations keep the wearer's offset vectors.
        for positions in &samples {
            let mut batch = gpu.batch(("underlayer proportion sample").into());
            ws.record_weld(&mut batch, positions, &links)?;
            ws.record_standoff(&mut batch, positions, &directions, &links)?;
            batch.submit();
        }

        let mut realizations = vec![Realization {
            positions: base_positions,
            directions,
        }];
        for morph in self.morphs {
            let positions = gpu.upload(BufferUpload::from_elements(morph.positions))?;
            let normals = gpu.upload(BufferUpload::from_elements(morph.normals))?;
            let directions =
                gpu.scratch((vector_bytes).into(), ("underlayer directions").into())?;
            let mut batch = gpu.batch(("underlayer morph sample").into());
            ws.record_weld(&mut batch, &positions, &links)?;
            ws.record_constraints(&mut batch, &positions, 0)?;
            ws.record_directions(&mut batch, &normals, &links, sets, &directions)?;
            ws.record_standoff(&mut batch, &positions, &directions, &links)?;
            batch.submit();
            realizations.push(Realization {
                positions,
                directions,
            });
        }
        self.shells(&ws, &realizations)
    }

    /// Evaluate the frozen shell on every realization and read it all back.
    fn shells(&self, ws: &Workspace, realizations: &[Realization]) -> Result<FittedUnderlayer> {
        let gpu = self.gpu;
        let plan = self.plan;
        let count = plan.vertex_count();
        let shell_bytes = count as u64 * 12;
        let offsets = LayerOffsets {
            outer: self.design.clearance.metres() + self.design.thickness.metres(),
            inner: self.design.clearance.metres(),
        };
        let indices = gpu.upload(BufferUpload::from_elements(&plan.indices))?;
        let shell = gpu.scratch((shell_bytes).into(), ("underlayer shell").into())?;
        let mut normals = VertexNormals::new(gpu.context(), count, plan.triangle_count())
            .map_err(device_error)?;
        let total = realizations.len() as u64;
        let all_positions = gpu.scratch(
            (shell_bytes * total).into(),
            ("underlayer shell positions").into(),
        )?;
        let all_normals = gpu.scratch(
            (shell_bytes * total).into(),
            ("underlayer shell normals").into(),
        )?;
        let all_status = gpu.scratch((4 * total).into(), ("underlayer normal status").into())?;
        for (index, realization) in (0u64..).zip(realizations) {
            let mut batch = gpu.batch(("underlayer shell").into());
            ws.record_layers(
                &mut batch,
                &realization.positions,
                &realization.directions,
                offsets,
                &shell,
            )?;
            gpu.normals(NormalWeighting::Area)
                .record(
                    &mut batch,
                    &shell,
                    &indices,
                    count,
                    plan.triangle_count(),
                    &mut normals,
                )
                .map_err(device_error)?;
            let encoder = batch.encoder();
            let offset = index * shell_bytes;
            encoder.copy_buffer_to_buffer(
                &shell.buffer,
                0,
                &all_positions.buffer,
                offset,
                shell_bytes,
            );
            encoder.copy_buffer_to_buffer(
                &normals.normals.buffer,
                0,
                &all_normals.buffer,
                offset,
                shell_bytes,
            );
            encoder.copy_buffer_to_buffer(
                &normals.status.buffer,
                0,
                &all_status.buffer,
                index * 4,
                4,
            );
            batch.submit();
        }
        let skin = match self.domain {
            Some(domain) => Some(self.record_skin(ws, domain)?),
            None => None,
        };
        FitStatus::try_from(gpu.read::<u32>(&ws.status)?[0])?.into_result()?;
        let status: Vec<u32> = gpu.read(&all_status)?;
        let positions: Vec<[f32; 3]> = gpu.read(&all_positions)?;
        let vectors: Vec<[f32; 3]> = gpu.read(&all_normals)?;
        let mut surfaces =
            shell_surfaces(&positions, &vectors, &status, count as usize)?.into_iter();
        let base = surfaces.next().expect("the wearer is always fitted");
        let (texcoords, joint_indices, joint_weights) = match skin {
            Some((joints, floats)) => read_skin(gpu, &joints, &floats, count as usize)?,
            None => Default::default(),
        };
        Ok(FittedUnderlayer {
            indices: plan.indices.clone(),
            base,
            endpoints: surfaces.collect(),
            texcoords,
            joint_indices,
            joint_weights,
        })
    }

    fn record_skin(&self, ws: &Workspace, domain: SurfaceDomain) -> Result<(Buffer, Buffer)> {
        let gpu = self.gpu;
        let count = self.plan.vertex_count() as u64;
        let sources = SkinSources {
            texcoords: gpu.upload(BufferUpload::from_elements(domain.texcoords))?,
            joint_indices: gpu.upload(BufferUpload::from_elements(self.body.joint_indices))?,
            joint_weights: gpu.upload(BufferUpload::from_elements(self.body.joint_weights))?,
        };
        let joints = gpu.scratch(
            (count * INFLUENCES as u64 * 4).into(),
            ("underlayer joints").into(),
        )?;
        let floats = gpu.scratch(
            (count * SKIN_FLOATS as u64 * 4).into(),
            ("underlayer skin").into(),
        )?;
        let mut batch = gpu.batch(("underlayer skin").into());
        ws.record_skin(&mut batch, &sources, &joints, &floats)?;
        batch.submit();
        Ok((joints, floats))
    }
}

/// Each realization's shell, read back in order, checked finite and with
/// valid normals.
fn shell_surfaces(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    status: &[u32],
    count: usize,
) -> Result<Vec<ShellSurface>> {
    positions
        .chunks(count)
        .zip(normals.chunks(count))
        .enumerate()
        .map(|(index, (positions, normals))| {
            let context = || match index {
                0 => "fitting underlayer".to_string(),
                i => format!("fitting underlayer morph {i}"),
            };
            if positions.iter().flatten().any(|v| !v.is_finite()) {
                return Err(GenerateError::InvalidSurface).with_context(context);
            }
            if status[index] != 0 {
                return Err(GenerateError::Degenerate).with_context(context);
            }
            Ok(ShellSurface {
                positions: positions.to_vec(),
                normals: normals.to_vec(),
            })
        })
        .collect()
}

type Skin = (Vec<[f32; 2]>, Vec<[u32; 8]>, Vec<[f32; 8]>);

fn read_skin(gpu: &ArmorGpu, joints: &Buffer, floats: &Buffer, count: usize) -> Result<Skin> {
    let mut joint_indices: Vec<[u32; 8]> = gpu.read(joints)?;
    joint_indices.truncate(count);
    let floats: Vec<[f32; SKIN_FLOATS]> = gpu.read(floats)?;
    let mut texcoords = Vec::with_capacity(count);
    let mut joint_weights = Vec::with_capacity(count);
    for vertex in floats.iter().take(count) {
        texcoords.push([vertex[0], vertex[1]]);
        joint_weights.push(std::array::from_fn(|i| vertex[2 + i]));
    }
    Ok((texcoords, joint_indices, joint_weights))
}

#[cfg(test)]
mod tests;
