//! The bracer generated on the device from the forearm skin.
//!
//! Seventeen rings slice the selected forearm skin at fixed axial heights;
//! each ring is one invocation, which welds its edge crossings, sorts them
//! by angle and brackets every column. The resulting sparse samples -- up to
//! four weighted surface vertices each -- are then displaced off whichever
//! body realization is bound: the wearer, or a morph sample, whose bracer is
//! the same samples on its own skin.

use fabelgeist_gpu::prelude::BufferUpload;
use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::anatomy::{DeviceSurface, STATUS_DEGENERATE};
use super::body::GpuBody;
use super::bracer_contour_wgsl;
use super::bracer_wgsl::{self, AXIS_WORDS, POINT_WORDS, SAMPLE_WORDS};
use super::{ArmorGpu, device_error};
use crate::{
    ArmorMorph, BracerDesign, GenerateError, GeneratedArmor, PlateFace, SurfaceGrid, design_hash,
    validate,
};

/// Rings along the forearm, less one.
pub(crate) const ALONG: u32 = 16;
/// Crossings one ring may weld; a ring meeting more fails.
const RING_CAPACITY: u32 = 2048;

/// Set when a ring crosses fewer than three skin edges.
pub const STATUS_EMPTY_CONTOUR: u32 = 16;

/// One realization of the body the bracer is displaced from.
pub struct BracerBody<'a> {
    pub positions: &'a Buffer,
    pub normals: &'a Buffer,
}

struct BracerMesh {
    positions: Buffer,
    normals: VertexNormals,
}

/// What every realization's mesh shares.
struct Layout {
    design: Buffer,
    surface: DeviceSurface,
    samples: Buffer,
    status: Buffer,
    around: u32,
    indices: Buffer,
    triangle_count: u32,
}

/// A mesh's positions and normals, read back.
type MeshSurface = (Vec<[f32; 3]>, Vec<[f32; 3]>);

impl Layout {
    fn sample_count(&self) -> u32 {
        self.around * (ALONG + 1)
    }

    fn vertex_count(&self) -> u32 {
        self.sample_count() * 2
    }

    /// The outer wall: its rings along the forearm, closed around it.
    fn grid(&self) -> SurfaceGrid {
        SurfaceGrid::regular(
            ALONG + 1,
            self.around,
            true,
            (0..self.sample_count()).collect(),
        )
    }

    /// The plate face of each triangle `bracer_wgsl::INDICES` writes: per
    /// ring quad two outer then two inner triangles, then two per quad of
    /// the end walls.
    fn faces(&self) -> Vec<PlateFace> {
        use PlateFace::{Edge, Inner, Outer};
        let quads = (ALONG * self.around) as usize;
        let walls = 2 * self.around as usize;
        [Outer, Outer, Inner, Inner]
            .repeat(quads)
            .into_iter()
            .chain(std::iter::repeat_n(Edge, 2 * walls))
            .collect()
    }

    /// Displace the samples off one body realization, and thicken them into
    /// a mesh with normals. The wearer's own unit skin normals at each sample
    /// go to `body_normals`, which orient the triangles.
    fn record_mesh(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        body: BracerBody,
        body_normals: Option<&Buffer>,
    ) -> Result<BracerMesh, GenerateError> {
        let vertices = self.vertex_count();
        let positions = gpu.scratch(vertices as u64 * 12, "bracer positions")?;
        let mut parameters = counted(vertices, self.around, self.surface.faces_at());
        parameters.insert("record_body_normals", u32::from(body_normals.is_some()));
        parameters.insert("body_positions", body.positions.clone());
        parameters.insert("body_normals_in", body.normals.clone());
        parameters.insert("surface", self.surface.words.clone());
        parameters.insert("design", self.design.clone());
        parameters.insert("samples", self.samples.clone());
        parameters.insert("positions", positions.clone());
        let body_normals = match body_normals {
            Some(buffer) => buffer.clone(),
            None => gpu.scratch(4, "unused body normals")?,
        };
        parameters.insert("body_normals", body_normals);
        parameters.insert("status", self.status.clone());
        batch
            .dispatch_items(
                &*kernel(gpu, &bracer_wgsl::displace(), true)?,
                &parameters,
                vertices,
            )
            .map_err(device_error)?;
        Ok(BracerMesh {
            positions,
            normals: VertexNormals::new(gpu.context(), vertices, self.triangle_count)
                .map_err(device_error)?,
        })
    }

    fn record_normals(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        mesh: &mut BracerMesh,
    ) -> Result<(), GenerateError> {
        gpu.normals(NormalWeighting::Area)
            .record(
                batch,
                &mesh.positions,
                &self.indices,
                self.vertex_count(),
                self.triangle_count,
                &mut mesh.normals,
            )
            .map_err(device_error)
    }
}

/// A bracer recorded on the device, with a mesh per body realization.
pub struct DeviceBracer {
    design: BracerDesign,
    layout: Layout,
    texcoords: Buffer,
    joint_indices: Buffer,
    joint_weights: Buffer,
    base: BracerMesh,
    morphs: Vec<BracerMesh>,
}

/// The design as the kernels read it; see `bracer_wgsl::DESIGN`.
fn design_words(design: &BracerDesign) -> Vec<f32> {
    let (start, end) = design.axial_interval();
    let flute = design.fluting.as_ref().map_or([0.0; 8], |f| {
        [
            f32::from(f.count.0),
            f.width.unit(),
            f.depth.metres(),
            f.spread.unit(),
            f.lower_spread.unit(),
            f.start.unit(),
            f.end.unit(),
            f.fade.unit(),
        ]
    });
    let mut words = vec![
        design.clearance.metres(),
        design.wall_thickness.metres(),
        design.elbow_flare.metres(),
        design.wrist_flare.metres(),
        design.center_ridge.metres(),
        start,
        end,
        f32::from(u8::from(design.fluting.is_some())),
    ];
    words.extend(flute);
    words.extend(design.columns());
    words
}

fn kernel(gpu: &ArmorGpu, entry: &str, binds_status: bool) -> Result<Arc<Kernel>, GenerateError> {
    gpu.cache()
        .get(
            gpu.context(),
            &bracer_wgsl::source(entry, binds_status).into(),
        )
        .map_err(device_error)
}

fn counted(count: u32, around: u32, faces_at: u32) -> PassParameters {
    let mut parameters = PassParameters::new();
    parameters.insert("count", count);
    parameters.insert("around", around);
    parameters.insert("faces_at", faces_at);
    parameters.insert("record_body_normals", 0u32);
    parameters
}

/// What the bracer is fitted to: the selected forearm skin of the wearer.
pub struct ForearmSkin<'a> {
    pub surface: &'a DeviceSurface,
    /// Each body vertex's clamped elbow-to-wrist coordinate.
    pub axial: &'a Buffer,
    /// The anatomical coordinates the surface's atlas indices name.
    pub atlas: &'a Buffer,
    pub body: &'a GpuBody,
}

impl Layout {
    /// Measure the forearm's axis, then slice its rings into samples.
    /// Returns the axis, which orients the end walls.
    fn record_samples(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        skin: &ForearmSkin,
    ) -> Result<Buffer, GenerateError> {
        let axis = gpu.scratch(AXIS_WORDS as u64 * 4, "bracer axis")?;
        let faces_at = self.surface.faces_at();
        let mut parameters = counted(1, self.around, faces_at);
        parameters.insert("positions", skin.body.positions.clone());
        parameters.insert("surface", self.surface.words.clone());
        parameters.insert("axial", skin.axial.clone());
        parameters.insert("joint_weights", skin.body.joint_weights.clone());
        parameters.insert("axis", axis.clone());
        parameters.insert("status", self.status.clone());
        batch
            .dispatch(
                &*kernel(gpu, bracer_wgsl::AXIS, true)?,
                &parameters,
                [1, 1, 1],
            )
            .map_err(device_error)?;

        let mut parameters = counted(RING_CAPACITY, self.around, faces_at);
        parameters.insert("positions", skin.body.positions.clone());
        parameters.insert("surface", self.surface.words.clone());
        parameters.insert("axial", skin.axial.clone());
        parameters.insert("design", self.design.clone());
        parameters.insert("axis", axis.clone());
        parameters.insert(
            "points",
            gpu.scratch(
                (ALONG + 1) as u64 * RING_CAPACITY as u64 * POINT_WORDS as u64 * 4,
                "bracer ring crossings",
            )?,
        );
        parameters.insert("samples", self.samples.clone());
        parameters.insert("status", self.status.clone());
        let contour = kernel(gpu, &bracer_contour_wgsl::contour(), true)?;
        batch
            .dispatch(&contour, &parameters, [ALONG + 1, 1, 1])
            .map_err(device_error)?;
        Ok(axis)
    }

    /// Wind the rings' and end walls' triangles to face along the skin.
    fn record_indices(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        positions: &Buffer,
        body_normals: &Buffer,
        axis: &Buffer,
    ) -> Result<(), GenerateError> {
        let quads = ALONG * self.around + 2 * self.around;
        let mut parameters = counted(quads, self.around, self.surface.faces_at());
        parameters.insert("positions", positions.clone());
        parameters.insert("body_normals", body_normals.clone());
        parameters.insert("axis", axis.clone());
        parameters.insert("indices", self.indices.clone());
        batch
            .dispatch_items(
                &*kernel(gpu, bracer_wgsl::INDICES, false)?,
                &parameters,
                quads,
            )
            .map_err(device_error)?;
        Ok(())
    }
}

impl DeviceBracer {
    /// Record the bracer's samples on the forearm and its mesh on the
    /// wearer. Failures raise bits in `status`, which
    /// [`DeviceBracer::read`] reports.
    pub fn record(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        design: &BracerDesign,
        skin: ForearmSkin,
        status: &Buffer,
    ) -> Result<Self, GenerateError> {
        validate(design)?;
        let around = design.columns().len() as u32;
        let sample_count = around * (ALONG + 1);
        let triangle_count = ALONG * around * 4 + 2 * around * 2;
        let layout = Layout {
            design: gpu.upload(BufferUpload::from_elements(&design_words(design)))?,
            surface: skin.surface.clone(),
            samples: gpu.scratch(
                sample_count as u64 * SAMPLE_WORDS as u64 * 4,
                "bracer samples",
            )?,
            status: status.clone(),
            around,
            indices: gpu.scratch(triangle_count as u64 * 12, "bracer indices")?,
            triangle_count,
        };
        let axis = layout.record_samples(gpu, batch, &skin)?;
        let body_normals = gpu.scratch(sample_count as u64 * 12, "bracer body normals")?;
        let body = BracerBody {
            positions: &skin.body.positions,
            normals: &skin.body.normals,
        };
        let mut base = layout.record_mesh(gpu, batch, body, Some(&body_normals))?;
        layout.record_indices(gpu, batch, &base.positions, &body_normals, &axis)?;
        layout.record_normals(gpu, batch, &mut base)?;

        let vertex_count = layout.vertex_count() as u64;
        let bracer = Self {
            design: design.clone(),
            texcoords: gpu.scratch(vertex_count * 8, "bracer texcoords")?,
            joint_indices: gpu.scratch(vertex_count * 32, "bracer joint indices")?,
            joint_weights: gpu.scratch(vertex_count * 32, "bracer joint weights")?,
            base,
            morphs: Vec::new(),
            layout,
        };
        bracer.record_skin(gpu, batch, &skin)?;
        Ok(bracer)
    }

    /// Carry the skin's atlas coordinates and joints onto both walls.
    fn record_skin(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        skin: &ForearmSkin,
    ) -> Result<(), GenerateError> {
        let layout = &self.layout;
        let samples = layout.sample_count();
        let mut parameters = counted(samples, layout.around, layout.surface.faces_at());
        parameters.insert("surface", layout.surface.words.clone());
        parameters.insert("samples", layout.samples.clone());
        parameters.insert("atlas", skin.atlas.clone());
        parameters.insert("body_joint_indices", skin.body.joint_indices.clone());
        parameters.insert("body_joint_weights", skin.body.joint_weights.clone());
        parameters.insert("texcoords", self.texcoords.clone());
        parameters.insert("joint_indices", self.joint_indices.clone());
        parameters.insert("joint_weights", self.joint_weights.clone());
        batch
            .dispatch_items(
                &*kernel(gpu, bracer_wgsl::SKIN, false)?,
                &parameters,
                samples,
            )
            .map_err(device_error)?;
        Ok(())
    }

    /// Record the bracer on a morph sample of the wearer.
    pub fn record_morph(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        body: BracerBody,
    ) -> Result<(), GenerateError> {
        let mut mesh = self.layout.record_mesh(gpu, batch, body, None)?;
        self.layout.record_normals(gpu, batch, &mut mesh)?;
        self.morphs.push(mesh);
        Ok(())
    }

    /// Read the bracer back after every batch has been submitted; `morphs`
    /// names the morph samples in the order they were recorded.
    pub fn read(
        &self,
        gpu: &ArmorGpu,
        domain: &str,
        morphs: &[String],
    ) -> Result<GeneratedArmor, GenerateError> {
        pollster::block_on(self.read_async(gpu, domain, morphs))
    }

    pub async fn read_async(
        &self,
        gpu: &ArmorGpu,
        domain: &str,
        morphs: &[String],
    ) -> Result<GeneratedArmor, GenerateError> {
        let mut staging = super::Staging::new();
        let status_slot = staging.stage(&self.layout.status);
        let meshes = std::iter::once(&self.base)
            .chain(&self.morphs)
            .collect::<Vec<_>>();
        let slots = meshes
            .iter()
            .map(|mesh| {
                [
                    staging.stage(&mesh.normals.status),
                    staging.stage(&mesh.positions),
                    staging.stage(&mesh.normals.normals),
                ]
            })
            .collect::<Vec<_>>();
        let attributes = [
            &self.texcoords,
            &self.joint_indices,
            &self.joint_weights,
            &self.layout.indices,
        ]
        .map(|buffer| staging.stage(buffer));
        let results = gpu.read_staged_async(staging).await?;
        let status = results.status(status_slot);
        if status & STATUS_EMPTY_CONTOUR != 0 {
            return Err(GenerateError::EmptySelection);
        }
        if status & STATUS_DEGENERATE != 0 {
            return Err(GenerateError::Degenerate);
        }
        if status != 0 {
            return Err(GenerateError::InvalidSurface);
        }
        let vertex_count = self.layout.vertex_count() as usize;
        let read_mesh = |slots: [super::Staged; 3]| -> Result<MeshSurface, GenerateError> {
            if results.status(slots[0]) != 0 {
                return Err(GenerateError::Degenerate);
            }
            Ok((
                results.prefix(slots[1], vertex_count),
                results.prefix(slots[2], vertex_count),
            ))
        };
        let (positions, normals) = read_mesh(slots[0])?;
        let morphs = slots
            .iter()
            .skip(1)
            .zip(morphs)
            .map(|(target, name)| {
                let (target_positions, target_normals) = read_mesh(*target)?;
                Ok(ArmorMorph {
                    name: name.clone(),
                    position_deltas: deltas(&positions, &target_positions),
                    normal_deltas: deltas(&normals, &target_normals),
                    direct_positions: target_positions,
                })
            })
            .collect::<Result<Vec<_>, GenerateError>>()?;
        Ok(GeneratedArmor {
            components: Vec::new(),
            design_hash: design_hash(&self.design)?,
            surface_domain: domain.to_owned(),
            positions,
            normals,
            texcoords: results.prefix(attributes[0], vertex_count),
            joint_indices: results.prefix(attributes[1], vertex_count),
            joint_weights: results.prefix(attributes[2], vertex_count),
            indices: results.prefix(attributes[3], self.layout.triangle_count as usize * 3),
            faces: self.layout.faces(),
            trim: None,
            grids: vec![self.layout.grid()],
            morphs,
        })
    }
}

pub(crate) fn deltas(base: &[[f32; 3]], target: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(target)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}
