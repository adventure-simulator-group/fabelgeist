//! Horizontal overlapping courses sampled from the completed fitted torso.
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::KernelBatchLabel;
use fabelgeist_gpu::prelude::Buffer;

use super::anime_sampling::CourseSampler;
use super::kernels::{Params, dispatch};
use super::plates::{Plates, Shell};
use crate::gpu::ArmorGpu;
use crate::{AnimeDesign, ArmorComponent, GenerateError};

#[cfg(test)]
#[path = "anime_tests.rs"]
mod tests;

/// Original solid vertices retained only for morph correspondence. Runtime
/// wearer fits have no morphs; they still use the same construction pipeline.
pub(super) struct Articulation {
    pub source: Shell,
    links: Buffer,
    pub components: Vec<ArmorComponent>,
}

pub(super) struct CourseInputs<'a> {
    pub plates: &'a Plates,
    pub frame: &'a Buffer,
    pub skin: &'a Buffer,
    pub status: &'a Buffer,
}

pub(super) struct Articulated {
    pub shell: Shell,
    pub skin: Buffer,
    pub correspondence: Articulation,
}

impl Articulation {
    pub(super) async fn record(
        gpu: &ArmorGpu,
        mut batch: KernelBatch<'_>,
        design: &AnimeDesign,
        source: Shell,
        input: CourseInputs<'_>,
    ) -> Result<Articulated, GenerateError> {
        let sampler = CourseSampler::new(gpu, &mut batch, &source, &input, design)?;
        batch.submit();
        let layout = sampler.layout(gpu, design).await?;
        let mut batch = gpu.batch(KernelBatchLabel::from("triangulated armor courses"));
        let sampled = sampler.finish(gpu, &mut batch, &layout)?;
        batch.submit();
        Ok(Articulated {
            shell: sampled.shell,
            skin: sampled.skin,
            correspondence: Self {
                source,
                links: sampled.links,
                components: layout.components,
            },
        })
    }

    /// Apply the original carrier's morph displacement to each new vertex;
    /// the course's lift and wall thickness remain part of its fitted shape.
    pub(super) fn record_morph(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        base: &Shell,
        target: &Buffer,
    ) -> Result<Buffer, GenerateError> {
        let positions = gpu.scratch(base.count() as u64 * 12, "anime morph")?;
        dispatch(
            gpu,
            batch,
            include_str!("anime_morph.wgsl"),
            &[
                ("original", false),
                ("target_body", false),
                ("base", false),
                ("positions", true),
            ],
            Params::counted(base.count()),
            &[
                ("original", &self.source.positions),
                ("target_body", target),
                ("base", &base.positions),
                ("links", &self.links),
                ("positions", &positions),
            ],
            base.count(),
        )?;
        Ok(positions)
    }
}
