//! The breastplate generated on the device.
//!
//! The wearer is measured from the selected front torso; the front and rear
//! carriers are evaluated from the authored shape, fitted to the torso's
//! sections, lapped, refined and fluted; the plates are thickened along
//! their extrusion; and every vertex takes its skin, and its morph
//! displacement, from the closest torso triangle. A morph sample of the
//! wearer only moves the finished shell by its body's displacement there.
//!
//! The kernels round every step in a fixed order with exact device arithmetic
//! (see [`fabelgeist_compute::host_float`]), so the carriers and their fit are
//! deterministic and independent of the device's fused operations: a plate
//! vertex on the symmetry plane chooses its side of the back's atlas seam by
//! those roundings.

mod carrier_wgsl;
mod finish_wgsl;
mod fit;
mod fit_wgsl;
mod kernels;
mod plates;
mod shape_wgsl;
mod skin_wgsl;
mod topology;
mod wearer_wgsl;

use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;

use super::anatomy::{DeviceSurface, STATUS_DEGENERATE};
use super::body::GpuBody;
use super::bracer::{deltas, read_prefix};
use super::{ArmorGpu, device_error};
use crate::{
    ArmorMorph, BreastplateDesign, GenerateError, GeneratedArmor, breastplate_design_hash,
    validate_breastplate,
};
use kernels::{Params, dispatch};
use plates::{Plates, Shell};
use skin_wgsl::SKIN_WORDS;

/// Floats of the rig anchors a breastplate reads: the torso's anterior
/// direction, the neck base, then the left and right shoulders.
pub const TORSO_RIG_WORDS: u32 = 12;

/// The wearer's torso on the device, as the breastplate reads it.
pub struct TorsoBody<'a> {
    pub body: &'a GpuBody,
    /// The selected front torso.
    pub surface: &'a DeviceSurface,
    /// Each body vertex's lateral and vertical torso coordinate.
    pub semantic: &'a Buffer,
    /// [`TORSO_RIG_WORDS`] floats of rig anchors.
    pub rig: &'a Buffer,
    /// The anatomical atlas, and each body face's atlas corners.
    pub atlas: &'a Buffer,
    pub atlas_faces: &'a Buffer,
    /// Faces of the torso proper, which the plates are fitted against.
    pub torso_faces: &'a [[u32; 3]],
    /// The body face index of each torso face, which skin is sampled from.
    pub eligible: &'a [u32],
}

/// A breastplate recorded on the device, with its shell on each body
/// realization.
pub struct DeviceBreastplate {
    design: BreastplateDesign,
    shell: Shell,
    skin: Buffer,
    morph_samples: Buffer,
    body_faces: Buffer,
    base_body: Buffer,
    status: Buffer,
    morphs: Vec<(Buffer, VertexNormals)>,
}

impl DeviceBreastplate {
    /// Record the breastplate fitted to `torso`. Failures raise bits in
    /// `status`, which [`DeviceBreastplate::read`] reports.
    pub fn record(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        design: &BreastplateDesign,
        torso: TorsoBody,
        status: &Buffer,
    ) -> Result<Self, GenerateError> {
        validate_breastplate(design)?;
        if torso.torso_faces.is_empty() || torso.eligible.len() != torso.torso_faces.len() {
            return Err(GenerateError::InvalidSurface);
        }
        let plates = Plates::new(gpu, design)?;
        let fitted = fit::record_fitted(gpu, batch, design, &torso, &plates, status)?;
        let extrusions = plates.record_extrusions(gpu, batch, &fitted.plate, status)?;
        let shell = plates.record_shell(gpu, batch, &fitted.plate, &extrusions)?;
        let correspondence = plates.record_correspondence(
            gpu,
            batch,
            &torso,
            (&fitted.plate, &fitted.body_local),
            &extrusions,
            &shell,
        )?;
        Ok(Self {
            design: design.clone(),
            shell,
            skin: correspondence.skin,
            morph_samples: correspondence.morph_samples,
            body_faces: torso.body.faces.clone(),
            base_body: torso.body.positions.clone(),
            status: status.clone(),
            morphs: Vec::new(),
        })
    }

    /// Record the shell on a morph sample of the wearer, whose body
    /// positions are `body`.
    pub fn record_morph(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        body: &Buffer,
    ) -> Result<(), GenerateError> {
        let shell = &self.shell;
        let count = shell.count();
        let positions = gpu.scratch(count as u64 * 12, "breastplate morph positions")?;
        dispatch(
            gpu,
            batch,
            skin_wgsl::MORPH,
            &[
                ("original", false),
                ("target_body", false),
                ("base", false),
                ("positions", true),
            ],
            Params::counted(count),
            &[
                ("sources", &shell.sources),
                ("morph_samples", &self.morph_samples),
                ("body_faces", &self.body_faces),
                ("original", &self.base_body),
                ("target_body", body),
                ("base", &shell.positions),
                ("positions", &positions),
            ],
            count,
        )?;
        let mut normals =
            VertexNormals::new(gpu.context(), count, shell.triangles()).map_err(device_error)?;
        gpu.normals(NormalWeighting::Area)
            .record(
                batch,
                &positions,
                &shell.indices,
                count,
                shell.triangles(),
                &mut normals,
            )
            .map_err(device_error)?;
        self.morphs.push((positions, normals));
        Ok(())
    }

    /// Read the breastplate back after every batch has been submitted;
    /// `morphs` names the morph samples in the order they were recorded.
    pub fn read(
        &self,
        gpu: &ArmorGpu,
        domain: &str,
        morphs: &[String],
    ) -> Result<GeneratedArmor, GenerateError> {
        let status = gpu.read::<u32>(&self.status)?[0];
        if status & STATUS_DEGENERATE != 0 {
            return Err(GenerateError::Degenerate);
        }
        if status != 0 {
            return Err(GenerateError::InvalidSurface);
        }
        let count = self.shell.count() as usize;
        let read_mesh = |positions: &Buffer, normals: &VertexNormals| {
            if gpu.read::<u32>(&normals.status)?[0] != 0 {
                return Err(GenerateError::Degenerate);
            }
            Ok::<_, GenerateError>((
                read_prefix::<[f32; 3]>(gpu, positions, count)?,
                read_prefix::<[f32; 3]>(gpu, &normals.normals, count)?,
            ))
        };
        let (positions, normals) = read_mesh(&self.shell.positions, &self.shell.normals)?;
        let skin: Vec<[u32; SKIN_WORDS as usize]> = read_prefix(gpu, &self.skin, count)?;
        let morphs = self
            .morphs
            .iter()
            .zip(morphs)
            .map(|((target, target_normals), name)| {
                let (direct, target_normals) = read_mesh(target, target_normals)?;
                Ok(ArmorMorph {
                    name: name.clone(),
                    position_deltas: deltas(&positions, &direct),
                    normal_deltas: deltas(&normals, &target_normals),
                    direct_positions: direct,
                })
            })
            .collect::<Result<Vec<_>, GenerateError>>()?;
        Ok(GeneratedArmor {
            components: Vec::new(),
            design_hash: breastplate_design_hash(&self.design)?,
            surface_domain: domain.to_owned(),
            positions,
            normals,
            texcoords: skin
                .iter()
                .map(|words| [f32::from_bits(words[0]), f32::from_bits(words[1])])
                .collect(),
            joint_indices: skin
                .iter()
                .map(|words| std::array::from_fn(|k| words[2 + k]))
                .collect(),
            joint_weights: skin
                .iter()
                .map(|words| std::array::from_fn(|k| f32::from_bits(words[10 + k])))
                .collect(),
            indices: self.shell.topology.indices.clone(),
            morphs,
        })
    }
}
