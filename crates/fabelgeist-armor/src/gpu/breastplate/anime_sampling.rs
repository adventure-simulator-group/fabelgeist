//! Course sampling and final device buffers, separate from connectivity.
use super::anime::CourseInputs;
use super::anime_layout::Layout;
use super::cut_frame::CourseFrame;
use super::kernels::{Params, dispatch};
use super::plates::Shell;
use super::skin_wgsl::SKIN_WORDS;
use super::topology::OUTER_BIT;
use crate::gpu::{ArmorGpu, device_error};
use crate::{AnimeDesign, GenerateError};
use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;

pub(super) const LINK_WORDS: u64 = 3;
pub(super) struct Sampled {
    pub shell: Shell,
    pub skin: Buffer,
    pub links: Buffer,
}
pub(super) struct CourseSampler<'a> {
    input: &'a CourseInputs<'a>,
    source: &'a Shell,
    design: Buffer,
    bounds: Buffer,
    references: Buffer,
    reference_ids: Vec<[u32; 2]>,
    params: Params,
}
impl<'a> CourseSampler<'a> {
    pub fn new(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        source: &'a Shell,
        input: &'a CourseInputs<'a>,
        design: &AnimeDesign,
    ) -> Result<Self, GenerateError> {
        let design = gpu.upload(&design_words(design))?;
        let bounds = gpu.scratch(32, "anime course bounds")?;
        let mut reference_ids =
            vec![[u32::MAX; 2]; (input.plates.front.count() + input.plates.back.count()) as usize];
        for (vertex, &mid) in source.topology.sources.iter().enumerate() {
            reference_ids[(mid & !OUTER_BIT) as usize][usize::from(mid & OUTER_BIT != 0)] =
                vertex as u32;
        }
        let references = gpu.upload(&reference_ids)?;
        let columns = gpu.upload(
            &input
                .plates
                .front
                .topology
                .columns
                .iter()
                .chain(&input.plates.back.topology.columns)
                .copied()
                .collect::<Vec<_>>(),
        )?;
        let params = Params {
            width: input.plates.front.width(),
            extra: input.plates.back.width(),
            front_count: input.plates.front.count(),
            ..Params::default()
        };
        dispatch(
            gpu,
            batch,
            include_str!("anime_bounds.wgsl"),
            &[("original", false)],
            params,
            &[
                ("plate", input.frame),
                ("original", &source.positions),
                ("references", &references),
                ("design", &design),
                ("bounds", &bounds),
                ("status", input.status),
                ("columns", &columns),
            ],
            2,
        )?;
        Ok(Self {
            input,
            source,
            design,
            bounds,
            references,
            reference_ids,
            params,
        })
    }

    /// Only connectivity and correspondence cross to the host. Shape fitting,
    /// plate extrusion, final course placement and normals remain on the GPU.
    pub async fn layout(
        &self,
        gpu: &ArmorGpu,
        design: &AnimeDesign,
    ) -> Result<Layout, GenerateError> {
        let mut staging = crate::gpu::Staging::new();
        let points = staging.stage(&self.source.positions);
        let frame = staging.stage(self.input.frame);
        let bounds = staging.stage(&self.bounds);
        let status = staging.stage(self.input.status);
        let results = gpu.read_staged_async(staging).await?;
        match results.status(status) {
            0 => {}
            bits if bits & crate::gpu::anatomy::STATUS_DEGENERATE != 0 => {
                return Err(GenerateError::Degenerate);
            }
            _ => return Err(GenerateError::InvalidSurface),
        }
        let points = results.prefix::<[f32; 3]>(points, self.source.count() as usize);
        let frame = CourseFrame::from_words(&results.prefix::<f32>(
            frame,
            (super::shape_wgsl::DESIGN_WORDS + super::shape_wgsl::WEARER_WORDS) as usize,
        ))?;
        let bounds = results.prefix::<[f32; 4]>(bounds, 2);
        Layout::new(
            self.input.plates,
            design,
            &points,
            &self.reference_ids,
            &frame,
            &bounds,
        )
    }
    pub fn record(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        layout: &Layout,
        positions: &Buffer,
        links: &Buffer,
    ) -> Result<(), GenerateError> {
        dispatch(
            gpu,
            batch,
            include_str!("anime_resample.wgsl"),
            &[("original", false), ("positions", true)],
            Params {
                count: layout.coordinates.len() as u32,
                ..self.params
            },
            &[
                ("plate", self.input.frame),
                ("original", &self.source.positions),
                ("references", &self.references),
                ("design", &self.design),
                ("bounds", &gpu.upload(&layout.bounds)?),
                ("coordinates", &gpu.upload(&layout.coordinates)?),
                ("positions", positions),
                ("links", links),
            ],
            layout.coordinates.len() as u32,
        )
    }

    pub fn finish(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        layout: &Layout,
    ) -> Result<Sampled, GenerateError> {
        let count = layout.coordinates.len() as u32;
        let triangles = (layout.topology.indices.len() / 3) as u32;
        let mut shell = Shell {
            sources: gpu.upload(&layout.topology.sources)?,
            indices: gpu.upload(&layout.topology.indices)?,
            positions: gpu.scratch(count as u64 * 12, "anime positions")?,
            normals: VertexNormals::new(gpu.context(), count, triangles).map_err(device_error)?,
            topology: layout.topology.clone(),
        };
        let links = gpu.scratch(count as u64 * LINK_WORDS * 4, "anime correspondence")?;
        let skin = gpu.scratch(count as u64 * SKIN_WORDS as u64 * 4, "anime skin")?;
        self.record(gpu, batch, layout, &shell.positions, &links)?;
        dispatch(
            gpu,
            batch,
            include_str!("anime_skin.wgsl"),
            &[],
            Params {
                count,
                ..self.params
            },
            &[
                ("links", &links),
                ("original_skin", self.input.skin),
                ("skin", &skin),
            ],
            count,
        )?;
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
        Ok(Sampled { shell, skin, links })
    }
}

pub(super) fn design_words(design: &AnimeDesign) -> Vec<f32> {
    vec![
        f32::from(design.lame_count),
        design.articulated_height.unit(),
        design.overlap.metres(),
        design.chevron_slope.unit(),
        design.rear_chevron_slope.unit(),
        design.lap_lift.metres(),
    ]
}
