//! The breastplate's plates on the device, and the stages that finish them.

use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;

use super::TorsoBody;
use super::finish_wgsl::{self, CARRIER_WORDS};
use super::kernels::{Params, dispatch};
use super::skin_wgsl::{self, MORPH_WORDS, SAMPLE_WORDS, SKIN_WORDS};
use super::topology::{MidTopology, SolidTopology, chart_columns};
use crate::gpu::{ArmorGpu, device_error};
use crate::{BreastplateDesign, GenerateError};

/// One plate's mid surface on the device.
pub(super) struct Plate {
    pub topology: MidTopology,
    pub positions: Buffer,
    faces: Buffer,
}

impl Plate {
    fn new(gpu: &ArmorGpu, topology: MidTopology, label: &str) -> Result<Self, GenerateError> {
        Ok(Self {
            positions: gpu.scratch((topology.vertex_count() as u64 * 12).into(), (label).into())?,
            faces: gpu.upload(BufferUpload::from_elements(&topology.faces))?,
            topology,
        })
    }

    pub(super) fn count(&self) -> u32 {
        self.topology.vertex_count() as u32
    }

    pub(super) fn width(&self) -> u32 {
        self.topology.width() as u32
    }

    /// The plate's area-weighted vertex normals, summed in face order.
    fn normals(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        status: &Buffer,
    ) -> Result<Buffer, GenerateError> {
        let normals = gpu.scratch((self.count() as u64 * 12).into(), ("plate normals").into())?;
        let (offsets, incident) = self.topology.incident_faces();
        dispatch(
            gpu,
            batch,
            finish_wgsl::PLATE_NORMALS,
            &[("positions", false), ("normals", true)],
            Params::counted(self.count()),
            &[
                ("positions", &self.positions),
                ("faces", &self.faces),
                (
                    "offsets",
                    &gpu.upload(BufferUpload::from_elements(&offsets))?,
                ),
                (
                    "incident",
                    &gpu.upload(BufferUpload::from_elements(&incident))?,
                ),
                ("normals", &normals),
                ("status", status),
            ],
            (self.count()).into(),
        )?;
        Ok(normals)
    }
}

/// The front carrier as fitted, the rear plate, and the front plate it
/// refines into.
pub(super) struct Plates {
    pub coarse: Plate,
    pub back: Plate,
    pub front: Plate,
}

/// The finished mid surfaces: where each plate's outer wall goes, and where
/// each refined front vertex lies on the fitted front carrier.
pub(super) struct Extrusions {
    front: Buffer,
    back: Buffer,
    carrier: Buffer,
}

/// The thickened shell on the device.
pub(super) struct Shell {
    pub topology: SolidTopology,
    pub sources: Buffer,
    pub indices: Buffer,
    pub positions: Buffer,
    pub normals: VertexNormals,
}

impl Shell {
    pub(super) fn count(&self) -> u32 {
        self.topology.sources.len() as u32
    }

    pub(super) fn triangles(&self) -> u32 {
        (self.topology.indices.len() / 3) as u32
    }
}

/// Each solid vertex's skin, and each mid vertex's morph samples.
pub(super) struct Correspondence {
    pub skin: Buffer,
    pub morph_samples: Buffer,
}

impl Plates {
    /// Lay out the plates: the front is fitted smooth, then refined onto the
    /// flutes' columns.
    pub(super) fn new(gpu: &ArmorGpu, design: &BreastplateDesign) -> Result<Self, GenerateError> {
        let mut smooth = design.clone();
        smooth.fluting = None;
        let coarse = MidTopology::new(false, chart_columns(false, &smooth), design);
        let front = if design.fluting.is_some() {
            MidTopology::new(false, chart_columns(false, design), design)
        } else {
            coarse.clone()
        };
        Ok(Self {
            coarse: Plate::new(gpu, coarse, "front carrier")?,
            back: Plate::new(
                gpu,
                MidTopology::new(true, chart_columns(true, design), design),
                "rear carrier",
            )?,
            front: Plate::new(gpu, front, "front plate")?,
        })
    }

    /// Continue the front's extrusion over its rim, refine it onto the
    /// flutes and raise them; take the rear's extrusion from its normals.
    pub(super) fn record_extrusions(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        plate: &Buffer,
        status: &Buffer,
    ) -> Result<Extrusions, GenerateError> {
        let coarse_normals = self.coarse.normals(gpu, batch, status)?;
        let rim = Params {
            width: self.coarse.width(),
            ..Params::default()
        };
        dispatch(
            gpu,
            batch,
            finish_wgsl::RIM,
            &[("normals", true)],
            rim,
            &[("normals", &coarse_normals), ("status", status)],
            (finish_wgsl::UPPER_RIM_ROWS * self.coarse.width()).into(),
        )?;
        let front = &self.front;
        let extrusions = Extrusions {
            front: gpu.scratch(
                (front.count() as u64 * 12).into(),
                ("front extrusion").into(),
            )?,
            back: self.back.normals(gpu, batch, status)?,
            carrier: gpu.scratch(
                (front.count() as u64 * CARRIER_WORDS as u64 * 4).into(),
                ("front carrier samples").into(),
            )?,
        };
        dispatch(
            gpu,
            batch,
            finish_wgsl::REFINE,
            &[
                ("coarse", false),
                ("coarse_normals", false),
                ("positions", true),
                ("extrusion", true),
            ],
            Params {
                width: front.width(),
                ..Params::default()
            },
            &[
                ("plate", plate),
                ("coarse", &self.coarse.positions),
                ("coarse_normals", &coarse_normals),
                (
                    "columns",
                    &gpu.upload(BufferUpload::from_elements(&front.topology.columns))?,
                ),
                ("positions", &front.positions),
                ("extrusion", &extrusions.front),
                ("carrier", &extrusions.carrier),
            ],
            (front.count()).into(),
        )?;
        Ok(extrusions)
    }

    /// Thicken both plates into one shell and compute its normals.
    pub(super) fn record_shell(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        plate: &Buffer,
        extrusions: &Extrusions,
    ) -> Result<Shell, GenerateError> {
        let mut topology = SolidTopology::new(&self.front.topology)?;
        topology.extend(SolidTopology::new(&self.back.topology)?, self.front.count());
        let count = topology.sources.len() as u32;
        let triangles = (topology.indices.len() / 3) as u32;
        let shell = Shell {
            sources: gpu.upload(BufferUpload::from_elements(&topology.sources))?,
            indices: gpu.upload(BufferUpload::from_elements(&topology.indices))?,
            positions: gpu.scratch((count as u64 * 12).into(), ("breastplate positions").into())?,
            normals: VertexNormals::new(gpu.context(), count, triangles).map_err(device_error)?,
            topology,
        };
        dispatch(
            gpu,
            batch,
            finish_wgsl::SOLID,
            &[
                ("front", false),
                ("front_extrusion", false),
                ("back", false),
                ("back_extrusion", false),
                ("positions", true),
            ],
            Params {
                count,
                front_count: self.front.count(),
                ..Params::default()
            },
            &[
                ("plate", plate),
                ("sources", &shell.sources),
                ("front", &self.front.positions),
                ("front_extrusion", &extrusions.front),
                ("back", &self.back.positions),
                ("back_extrusion", &extrusions.back),
                ("positions", &shell.positions),
            ],
            (count).into(),
        )?;
        let mut shell = shell;
        gpu.normals(NormalWeighting::Area)
            .record(
                batch,
                &shell.positions,
                &shell.indices,
                count,
                triangles,
                &mut shell.normals,
            )
            .map_err(device_error)?;
        Ok(shell)
    }

    /// Sample the torso under every mid vertex -- and under the fitted front
    /// carrier, which the refined front's morphs follow -- then give each
    /// solid vertex its skin.
    pub(super) fn record_correspondence(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        torso: &TorsoBody,
        frame: (&Buffer, &Buffer),
        extrusions: &Extrusions,
        shell: &Shell,
    ) -> Result<Correspondence, GenerateError> {
        let (plate, body_local) = frame;
        let (front, back) = (self.front.count(), self.back.count());
        let queries = front + back + self.coarse.count();
        let samples = gpu.scratch(
            (queries as u64 * SAMPLE_WORDS as u64 * 4).into(),
            ("breastplate samples").into(),
        )?;
        let params = Params {
            count: queries,
            torso_count: torso.torso_faces.len() as u32,
            front_count: front,
            extra: back,
            ..Params::default()
        };
        dispatch(
            gpu,
            batch,
            skin_wgsl::SAMPLE,
            &[
                ("front", false),
                ("back", false),
                ("coarse", false),
                ("body_local", false),
            ],
            params,
            &[
                ("plate", plate),
                ("front", &self.front.positions),
                ("back", &self.back.positions),
                ("coarse", &self.coarse.positions),
                (
                    "eligible",
                    &gpu.upload(BufferUpload::from_elements(torso.eligible))?,
                ),
                ("body_faces", &torso.body.faces),
                ("body_local", body_local),
                ("samples", &samples),
            ],
            (queries).into(),
        )?;
        let correspondence = Correspondence {
            skin: gpu.scratch(
                (shell.count() as u64 * SKIN_WORDS as u64 * 4).into(),
                ("breastplate skin").into(),
            )?,
            morph_samples: gpu.scratch(
                ((front + back) as u64 * MORPH_WORDS as u64 * 4).into(),
                ("breastplate morph samples").into(),
            )?,
        };
        dispatch(
            gpu,
            batch,
            skin_wgsl::MORPH_SAMPLES,
            &[],
            Params {
                count: front + back,
                ..params
            },
            &[
                ("samples", &samples),
                ("carrier", &extrusions.carrier),
                ("morph_samples", &correspondence.morph_samples),
            ],
            (front + back).into(),
        )?;
        dispatch(
            gpu,
            batch,
            skin_wgsl::SKIN,
            &[],
            Params::counted(shell.count()),
            &[
                ("sources", &shell.sources),
                ("samples", &samples),
                ("body_faces", &torso.body.faces),
                ("atlas_faces", torso.atlas_faces),
                ("atlas", torso.atlas),
                ("body_joint_indices", &torso.body.joint_indices),
                ("body_joint_weights", &torso.body.joint_weights),
                ("skin", &correspondence.skin),
            ],
            (shell.count()).into(),
        )?;
        Ok(correspondence)
    }
}
