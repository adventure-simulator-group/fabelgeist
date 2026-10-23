//! Final sampled body seating, normals and waist correspondence for shared cloth.
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::skirt_join::SkirtJoinPlan;
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::{SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

const FACE_SEATING_ITERATIONS: usize = 12;

/// GPU boundary: canonical metre positions, final triangle IDs and active points.
struct SeatingSurface<'a> {
    positions: &'a Buffer,
    faces: &'a Buffer,
    mask: &'a Buffer,
    face_count: u32,
    clearance_metres: f32,
}

pub(super) struct CarrierAdjacency {
    pub indices: Vec<u32>,
    pub offsets: Vec<u32>,
    pub incidents: Vec<u32>,
}

struct CarrierBuffers {
    topology: CarrierAdjacency,
    faces: Buffer,
    carrier_faces: Buffer,
    adjacency: Buffer,
    incidents: Buffer,
    shifts: Buffer,
    mask: Buffer,
}

impl CarrierBuffers {
    fn record_points(
        &self,
        device: &DeviceWearer<'_>,
        batch: &mut KernelBatch,
        fit: &Buffer,
        point_count: u32,
        support: &str,
        surface: &SeatingSurface<'_>,
    ) -> Result<()> {
        dispatch(
            device,
            batch,
            &(support.to_owned() + include_str!("device_cloth_seating_pass.wgsl")),
            &[
                write("fit", fit),
                read("positions", surface.positions),
                read_u32("seating_faces", surface.faces),
                read_u32("seating_domain", surface.mask),
            ],
            &[
                Word::U("count", point_count),
                Word::U("seating_face_count", surface.face_count),
                Word::F("clearance", surface.clearance_metres),
            ],
            Grid::Items(point_count),
        )
    }

    fn record_support(
        &self,
        device: &DeviceWearer<'_>,
        batch: &mut KernelBatch,
        fit: &Buffer,
        point_count: u32,
        header: &str,
        surface: &SeatingSurface<'_>,
    ) -> Result<()> {
        let indices = &self.topology.indices;
        let support = header.to_owned() + include_str!("device_cloth_seating.wgsl");
        self.record_points(device, batch, fit, point_count, &support, surface)?;
        for _ in 0..FACE_SEATING_ITERATIONS {
            dispatch(
                device,
                batch,
                &(support.clone() + include_str!("device_cloth_seating_faces.wgsl")),
                &[
                    read("fit", fit),
                    read("positions", surface.positions),
                    read_u32("seating_faces", surface.faces),
                    read_u32("carrier_faces", &self.carrier_faces),
                    write("face_shifts", &self.shifts),
                    read_u32("seating_domain", surface.mask),
                ],
                &[
                    Word::U("face_count", indices.len() as u32 / 3),
                    Word::U("seating_face_count", surface.face_count),
                    Word::F("clearance", surface.clearance_metres),
                ],
                Grid::Items(indices.len() as u32 / 3),
            )?;
            dispatch(
                device,
                batch,
                &(header.to_owned() + include_str!("device_cloth_seating_update.wgsl")),
                &[
                    write("fit", fit),
                    read_u32("seating_domain", surface.mask),
                    read("face_shifts", &self.shifts),
                    read_u32("adjacency", &self.adjacency),
                    read_u32("incidents", &self.incidents),
                ],
                &[Word::U("count", point_count)],
                Grid::Items(point_count),
            )?;
        }
        Ok(())
    }

    fn record_normals(
        &self,
        device: &DeviceWearer<'_>,
        batch: &mut KernelBatch,
        fit: &Buffer,
        point_count: u32,
        header: &str,
    ) -> Result<()> {
        dispatch(
            device,
            batch,
            &(header.to_owned() + include_str!("device_cloth_seating_normals.wgsl")),
            &[
                write("fit", fit),
                read_u32("seating_domain", &self.mask),
                read_u32("carrier_faces", &self.carrier_faces),
                read_u32("adjacency", &self.adjacency),
                read_u32("incidents", &self.incidents),
            ],
            &[Word::U("count", point_count)],
            Grid::Items(point_count),
        )
    }

    fn new(device: &DeviceWearer<'_>, plan: &SkirtJoinPlan) -> Result<Self> {
        Self::from_topology(
            device,
            CarrierAdjacency::new(plan)?,
            &crate::cloth_seating_domain::mask(plan),
        )
    }

    fn from_topology(
        device: &DeviceWearer<'_>,
        topology: CarrierAdjacency,
        mask: &[u32],
    ) -> Result<Self> {
        Ok(Self {
            faces: device
                .gpu
                .upload(BufferUpload::from_elements(device.host.faces))?,
            carrier_faces: device
                .gpu
                .upload(BufferUpload::from_elements(&topology.indices))?,
            adjacency: device
                .gpu
                .upload(BufferUpload::from_elements(&topology.offsets))?,
            incidents: device
                .gpu
                .upload(BufferUpload::from_elements(&topology.incidents))?,
            shifts: device.gpu.scratch(
                topology.indices.len() as u64 * 4 * 4,
                "cloth face body corrections",
            )?,
            mask: device.gpu.upload(BufferUpload::from_elements(mask))?,
            topology,
        })
    }
}

impl CarrierAdjacency {
    fn new(plan: &SkirtJoinPlan) -> Result<Self> {
        let indices = plan.patch.indices.clone();
        Self::from_indices(plan.patch.point_count, indices)
    }

    /// Native point ordinals and triangle-corner ordinals. Keep this topology
    /// shared by seating normals and backing-plane clearance queries.
    pub(super) fn from_indices(point_count: u32, indices: Vec<u32>) -> Result<Self> {
        ensure!(
            indices.len().is_multiple_of(3),
            "cloth adjacency has an incomplete triangle"
        );
        let mut incident = vec![Vec::new(); point_count as usize];
        for (ordinal, &vertex) in indices.iter().enumerate() {
            incident
                .get_mut(vertex as usize)
                .context("cloth adjacency vertex is outside its carrier")?
                .push(u32::try_from(ordinal)?);
        }
        let mut offsets = Vec::new();
        let mut incidents = Vec::new();
        for entries in incident {
            offsets.extend([
                u32::try_from(incidents.len())?,
                u32::try_from(entries.len())?,
            ]);
            incidents.extend(entries);
        }
        Ok(Self {
            indices,
            offsets,
            incidents,
        })
    }
}

impl DeviceWearer<'_> {
    pub(crate) fn record_cloth_seating(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SkirtJoinPlan,
        gauge_metres: f32,
    ) -> Result<()> {
        let buffers = CarrierBuffers::new(self, plan)?;
        let header = format!(
            "const FIRST_POINT:u32={SKIRT_FIT_WORDS}u;\nconst POINT_WORDS:u32={SKIRT_ATTACHMENT_POINT_WORDS}u;\n"
        );
        buffers.record_support(
            self,
            batch,
            fit,
            plan.patch.point_count,
            &header,
            &SeatingSurface {
                positions: &self.body.positions,
                faces: &buffers.faces,
                mask: &buffers.mask,
                face_count: self.host.faces.len() as u32,
                clearance_metres: gauge_metres + crate::cloth_panel::MINIMUM_CLOTH_CLEARANCE_M,
            },
        )?;
        buffers.record_normals(self, batch, fit, plan.patch.point_count, &header)?;
        self.record_seated_waist(batch, fit, plan, &header)
    }

    fn record_seated_waist(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SkirtJoinPlan,
        header: &str,
    ) -> Result<()> {
        let samples = self
            .gpu
            .upload(BufferUpload::from_elements(&plan.samples))?;
        let count = plan.samples.len() as u32 / 3;
        dispatch(
            self,
            batch,
            &(header.to_owned() + include_str!("device_cloth_seating_waist.wgsl")),
            &[write("fit", fit), read_u32("samples", &samples)],
            &[
                Word::U("count", count),
                Word::U("points", plan.patch.point_count),
            ],
            Grid::Items(count),
        )
    }
}
