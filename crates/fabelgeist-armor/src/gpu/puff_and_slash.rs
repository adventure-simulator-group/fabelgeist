//! Limb placement of the shared puff-and-slash textile construction.
use super::coord::{AUTHORED, CoordKernel};
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::{ArmorComponentRole, GenerateError, PuffAndSlashDesign};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

pub fn record_puff_and_slash(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &PuffAndSlashDesign,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let kernel = CoordKernel::new(
        gpu,
        &format!(
            "{AUTHORED}{}{}",
            include_str!("textile_fullness.wgsl"),
            include_str!("puff_and_slash.wgsl")
        ),
    )?;
    let mut recipe = PartRecipe::new();
    recipe.textile_chart();
    recipe.push_coord(
        super::textile_pattern::lining(design.thickness),
        kernel.clone(),
    )?;
    recipe.component(ArmorComponentRole::Undercloth, None);
    super::textile_pattern::append_outer(&mut recipe, &kernel, &design.pattern, design.thickness)?;
    let mut words = design.pattern.shader_words().to_vec();
    words.extend([
        design.length.unit(),
        design.proximal_position.unit(),
        design.clearance.metres(),
        design.thickness.metres(),
    ]);
    recipe.record(gpu, batch, &words, &[frame])
}
