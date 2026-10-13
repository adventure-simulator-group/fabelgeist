//! The breastplate's plates on the device, and the stages that finish them.

use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;

use super::TorsoBody;
use super::clip_carrier::ClippedCarrier;
use super::finish_wgsl::{self, CARRIER_WORDS};
use super::kernels::{Params, dispatch};
use super::skin_wgsl::{self, MORPH_WORDS, SAMPLE_WORDS, SKIN_WORDS};
use super::topology::{MidTopology, SolidTopology, chart_columns};
use crate::gpu::{ArmorGpu, device_error};
use crate::{BreastplateConstruction, BreastplateDesign, GenerateError, Millimeters};

/// One plate's mid surface on the device.
pub(super) struct Plate {
    pub topology: MidTopology,
    outer: MidTopology,
    pub positions: Buffer,
    pub arm_distances: Buffer,
    pub rim_vertices: Vec<u32>,
}

impl Plate {
    fn new(gpu: &ArmorGpu, topology: MidTopology, label: &str) -> Result<Self, GenerateError> {
        Ok(Self {
            positions: gpu.scratch(topology.vertex_count() as u64 * 12, label)?,
            arm_distances: gpu.scratch(topology.vertex_count() as u64 * 4, "arm trim domain")?,
            outer: topology.clone(),
            topology,
            rim_vertices: Vec::new(),
        })
    }

    pub(super) fn count(&self) -> u32 {
        self.topology.vertex_count() as u32
    }

    pub(super) fn width(&self) -> u32 {
        self.topology.width() as u32
    }

    /// The seated carrier's extrusion from derivatives at a uniform spacing.
    fn normals(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        status: &Buffer,
        rear: bool,
    ) -> Result<Buffer, GenerateError> {
        let normals = gpu.scratch(self.count() as u64 * 12, "plate normals")?;
        dispatch(
            gpu,
            batch,
            finish_wgsl::PLATE_NORMALS,
            &[("positions", false), ("normals", true)],
            Params {
                count: self.count(),
                width: self.width(),
                rear,
                ..Default::default()
            },
            &[
                ("positions", &self.positions),
                (
                    "columns",
                    &gpu.upload(BufferUpload::from_elements(&self.topology.columns))?,
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
    gauge: Millimeters,
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

impl Extrusions {
    pub(super) fn record_miters(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        plates: &Plates,
        status: &Buffer,
    ) -> Result<(), GenerateError> {
        self.front = plates.front.record_miter(gpu, batch, &self.front, status)?;
        self.back = plates.back.record_miter(gpu, batch, &self.back, status)?;
        Ok(())
    }
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
    /// Clip the evaluated carrier facets at the neckline. New cut samples
    /// interpolate the same facets on both walls, including their offsets.
    pub(super) async fn remesh(
        &mut self,
        gpu: &ArmorGpu,
        status: &Buffer,
        extrusions: &mut Extrusions,
    ) -> Result<(), GenerateError> {
        let mut staging = crate::gpu::Staging::new();
        let front = staging.stage(&self.front.positions);
        let back = staging.stage(&self.back.positions);
        let coarse = staging.stage(&self.coarse.positions);
        let front_extrusion = staging.stage(&extrusions.front);
        let back_extrusion = staging.stage(&extrusions.back);
        let front_arm = staging.stage(&self.front.arm_distances);
        let back_arm = staging.stage(&self.back.arm_distances);
        let status = staging.stage(status);
        let results = gpu.read_staged_async(staging).await?;
        let front = results.prefix::<[f32; 3]>(front, self.front.count() as usize);
        let back = results.prefix::<[f32; 3]>(back, self.back.count() as usize);
        let front_extrusion =
            results.prefix::<[f32; 3]>(front_extrusion, self.front.count() as usize);
        let back_extrusion = results.prefix::<[f32; 3]>(back_extrusion, self.back.count() as usize);
        match results.status(status) {
            0 => {}
            bits if bits & crate::gpu::anatomy::STATUS_DEGENERATE != 0 => {
                return Err(GenerateError::Degenerate);
            }
            _ => return Err(GenerateError::InvalidSurface),
        }
        let mut front = ClippedCarrier::new(&self.front.topology, &front, &front_extrusion)?;
        let mut back = ClippedCarrier::new(&self.back.topology, &back, &back_extrusion)?;
        front.trim_arms(&results.prefix::<f32>(front_arm, self.front.count() as usize))?;
        back.trim_arms(&results.prefix::<f32>(back_arm, self.back.count() as usize))?;
        front.validate_outer(self.gauge)?;
        back.validate_outer(self.gauge)?;
        front.install(&mut self.front.topology);
        back.install(&mut self.back.topology);
        self.front.rim_vertices = front.rim_vertices.clone();
        self.back.rim_vertices = back.rim_vertices.clone();
        self.front.outer = self.front.topology.clone();
        self.back.outer = self.back.topology.clone();
        self.front.positions = gpu.upload(BufferUpload::from_elements(&front.positions))?;
        self.back.positions = gpu.upload(BufferUpload::from_elements(&back.positions))?;
        let coarse = front
            .carrier_positions(results.prefix::<[f32; 3]>(coarse, self.coarse.count() as usize));
        self.coarse
            .topology
            .cut_columns
            .clone_from(&self.front.topology.cut_columns);
        self.coarse.positions = gpu.upload(BufferUpload::from_elements(&coarse))?;
        extrusions.front = gpu.upload(BufferUpload::from_elements(&front.directions))?;
        extrusions.back = gpu.upload(BufferUpload::from_elements(&back.directions))?;
        let carrier = (0..self.front.count())
            .map(|i| [f32::from_bits(i), f32::from_bits(i), 0.0])
            .collect::<Vec<_>>();
        extrusions.carrier = gpu.upload(BufferUpload::from_elements(&carrier))?;
        Ok(())
    }

    /// Lay out the plates: the front is fitted smooth, then refined onto the
    /// flutes' columns.
    pub(super) fn new(gpu: &ArmorGpu, design: &BreastplateDesign) -> Result<Self, GenerateError> {
        let coarse = MidTopology::new(false, chart_columns(false, design), design);
        let front = coarse.clone();
        Ok(Self {
            gauge: design.wall_thickness,
            coarse: Plate::new(gpu, coarse, "front carrier")?,
            back: Plate::new(
                gpu,
                MidTopology::new(true, chart_columns(true, design), design),
                "rear carrier",
            )?,
            front: Plate::new(gpu, front, "front plate")?,
        })
    }

    /// Continue both extrusion fields over their upper rims. Raise front
    /// flute relief radially, retaining the carrier's metal-gauge directions.
    pub(super) fn record_extrusions(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        plate: &Buffer,
        status: &Buffer,
    ) -> Result<Extrusions, GenerateError> {
        let coarse_normals = self.coarse.normals(gpu, batch, status, false)?;
        let front = &self.front;
        let extrusions = Extrusions {
            front: gpu.scratch(front.count() as u64 * 12, "front extrusion")?,
            back: self.back.normals(gpu, batch, status, true)?,
            carrier: gpu.scratch(
                front.count() as u64 * CARRIER_WORDS as u64 * 4,
                "front carrier samples",
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
        construction: &BreastplateConstruction,
    ) -> Result<Shell, GenerateError> {
        let mut topology = SolidTopology::new(&self.front.topology, &self.front.outer)?;
        topology.extend(
            SolidTopology::new(&self.back.topology, &self.back.outer)?,
            self.front.count(),
        );
        if matches!(construction, BreastplateConstruction::Solid) {
            topology.compact();
        }
        let count = topology.sources.len() as u32;
        let triangles = (topology.indices.len() / 3) as u32;
        let shell = Shell {
            sources: gpu.upload(BufferUpload::from_elements(&topology.sources))?,
            indices: gpu.upload(BufferUpload::from_elements(&topology.indices))?,
            positions: gpu.scratch(count as u64 * 12, "breastplate positions")?,
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
            queries as u64 * SAMPLE_WORDS as u64 * 4,
            "breastplate samples",
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
                shell.count() as u64 * SKIN_WORDS as u64 * 4,
                "breastplate skin",
            )?,
            morph_samples: gpu.scratch(
                (front + back) as u64 * MORPH_WORDS as u64 * 4,
                "breastplate morph samples",
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
