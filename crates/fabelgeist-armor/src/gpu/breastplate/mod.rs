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

mod anime;
mod anime_layout;
mod anime_sampling;
mod arm_trim;
mod carrier_wgsl;
mod clip_carrier;
#[cfg(test)]
mod clip_tests;
mod construction_columns;
mod course_clip;
mod course_coordinates;
mod course_flare;
mod cut_frame;
mod cut_resolution;
mod finish_wgsl;
mod fit;
#[cfg(test)]
mod fit_tests;
mod fit_wgsl;
mod kernels;
mod miter;
mod plates;
#[cfg(test)]
mod shape_tests;
mod shape_wgsl;
mod skin_wgsl;
mod solid_grid;
mod topology;
mod trimmed_carrier;
mod wearer_wgsl;

use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;

use super::anatomy::{DeviceSurface, STATUS_DEGENERATE};
use super::body::GpuBody;
use super::bracer::deltas;
use super::{ArmorGpu, device_error};
use crate::{
    ArmorMorph, BreastplateConstruction, BreastplateDesign, GenerateError, GeneratedArmor,
    breastplate_design_hash, validate_breastplate,
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
    articulation: Option<anime::Articulation>,
}

impl DeviceBreastplate {
    /// Fit and triangulate the breastplate on `torso`. Submitted stages are
    /// read before triangulation; final device failures are reported by
    /// [`DeviceBreastplate::read`].
    pub async fn record(
        gpu: &ArmorGpu,
        mut batch: KernelBatch<'_>,
        design: &BreastplateDesign,
        torso: TorsoBody<'_>,
        status: &Buffer,
    ) -> Result<Self, GenerateError> {
        validate_breastplate(design)?;
        if torso.torso_faces.is_empty() || torso.eligible.len() != torso.torso_faces.len() {
            return Err(GenerateError::InvalidSurface);
        }
        let mut plates = Plates::new(gpu, design)?;
        let fitted = fit::record_fitted(gpu, &mut batch, design, &torso, &plates, status)?;
        let mut extrusions = plates.record_extrusions(gpu, &mut batch, &fitted.plate, status)?;
        batch.submit();
        plates.remesh(gpu, status, &mut extrusions).await?;
        let mut batch = gpu.batch("triangulated breastplate");
        extrusions.record_miters(gpu, &mut batch, &plates, status)?;
        let shell = plates.record_shell(
            gpu,
            &mut batch,
            &fitted.plate,
            &extrusions,
            &design.construction,
        )?;
        let correspondence = plates.record_correspondence(
            gpu,
            &mut batch,
            &torso,
            (&fitted.plate, &fitted.body_local),
            &extrusions,
            &shell,
        )?;
        let (shell, skin, articulation) = match &design.construction {
            BreastplateConstruction::Solid => {
                batch.submit();
                (shell, correspondence.skin, None)
            }
            BreastplateConstruction::Anime(design) => {
                let articulated = anime::Articulation::record(
                    gpu,
                    batch,
                    design,
                    shell,
                    anime::CourseInputs {
                        plates: &plates,
                        frame: &fitted.plate,
                        skin: &correspondence.skin,
                        status,
                    },
                )
                .await?;
                (
                    articulated.shell,
                    articulated.skin,
                    Some(articulated.correspondence),
                )
            }
        };
        Ok(Self {
            design: design.clone(),
            shell,
            skin,
            morph_samples: correspondence.morph_samples,
            body_faces: torso.body.faces.clone(),
            base_body: torso.body.positions.clone(),
            status: status.clone(),
            morphs: Vec::new(),
            articulation,
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
        let shell = self
            .articulation
            .as_ref()
            .map_or(&self.shell, |a| &a.source);
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
            (count).into(),
        )?;
        let positions = match &self.articulation {
            Some(articulation) => articulation.record_morph(gpu, batch, &self.shell, &positions)?,
            None => positions,
        };
        let shell = &self.shell;
        let count = shell.count();
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
        pollster::block_on(self.read_async(gpu, domain, morphs))
    }

    pub async fn read_async(
        &self,
        gpu: &ArmorGpu,
        domain: &str,
        morphs: &[String],
    ) -> Result<GeneratedArmor, GenerateError> {
        let mut staging = super::Staging::new();
        let status_slot = staging.stage(&self.status);
        let meshes = std::iter::once((&self.shell.positions, &self.shell.normals)).chain(
            self.morphs
                .iter()
                .map(|(positions, normals)| (positions, normals)),
        );
        let slots = meshes
            .map(|(positions, normals)| {
                [
                    staging.stage(&normals.status),
                    staging.stage(positions),
                    staging.stage(&normals.normals),
                ]
            })
            .collect::<Vec<_>>();
        let skin_slot = staging.stage(&self.skin);
        let results = gpu.read_staged_async(staging).await?;
        let status = results.status(status_slot);
        if status & STATUS_DEGENERATE != 0 {
            return Err(GenerateError::Degenerate);
        }
        if status != 0 {
            return Err(GenerateError::InvalidSurface);
        }
        let count = self.shell.count() as usize;
        let read_mesh = |slots: [super::Staged; 3]| {
            if results.status(slots[0]) != 0 {
                return Err(GenerateError::Degenerate);
            }
            Ok::<_, GenerateError>((
                results.prefix::<[f32; 3]>(slots[1], count),
                results.prefix::<[f32; 3]>(slots[2], count),
            ))
        };
        let (positions, normals) = read_mesh(slots[0])?;
        let skin: Vec<[u32; SKIN_WORDS as usize]> = results.prefix(skin_slot, count);
        let morphs = slots
            .iter()
            .skip(1)
            .zip(morphs)
            .map(|(target, name)| {
                let (direct, target_normals) = read_mesh(*target)?;
                Ok(ArmorMorph {
                    name: name.clone(),
                    position_deltas: deltas(&positions, &direct),
                    normal_deltas: deltas(&normals, &target_normals),
                    direct_positions: direct,
                })
            })
            .collect::<Result<Vec<_>, GenerateError>>()?;
        Ok(GeneratedArmor {
            components: self
                .articulation
                .as_ref()
                .map_or_else(Vec::new, |a| a.components.clone()),
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
            faces: self.shell.topology.faces.clone(),
            trim: None,
            grids: self.shell.topology.grids.clone(),
            morphs,
        })
    }
}
