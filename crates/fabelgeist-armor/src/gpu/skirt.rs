//! One continuous cyclic skirt carrier, before shell thickening.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::{AUTHORED, CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use super::skirt_attachment::{SKIRT_ATTACHMENT_POINT_WORDS, SkirtAttachment};
use super::{ArmorGpu, DevicePart};
use crate::{
    ArmorComponentRole, BoundaryNormals, GenerateError, SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS,
    SkirtDesign,
};

/// The standard fitted frame and its failure word precede the radial sections.
pub const SKIRT_PROFILE_FIRST_WORD: u32 = 16;
/// Angular radii about the common pelvis-frame axis, one per column.
pub const SKIRT_SECTION_WORDS: u32 = SKIRT_RING_SEGMENTS as u32;
pub const SKIRT_FIT_WORDS: u32 =
    SKIRT_PROFILE_FIRST_WORD + (SKIRT_AXIAL_SEGMENTS as u32 + 1) * SKIRT_SECTION_WORDS;

/// Record an authored skirt against a fitted radial profile. `fit` starts with
/// a standard upright frame and then holds waist-to-hem sections in row order.
/// Section radii include the clearance and inward wall gauge already.
/// Coordinates and connectivity depend on the design alone across morphs.
pub fn record_skirt(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &SkirtDesign,
    fit: &Buffer,
    attachment: Option<&SkirtAttachment>,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    if design.bodice.is_some() != attachment.is_some() {
        return Err(GenerateError::InvalidSurface);
    }
    let first_row = design
        .bodice
        .as_ref()
        .map_or(0, |bodice| bodice.first_skirt_row());
    let kernel = coordinate_kernel(gpu, include_str!("skirt.wgsl"))?;
    let mut recipe = PartRecipe::new();
    recipe.textile_chart();
    recipe.push_coord(carrier(design, attachment)?, kernel.clone())?;
    recipe.component(
        if design.puff_and_slash.is_some() {
            ArmorComponentRole::Undercloth
        } else {
            ArmorComponentRole::OuterFabric
        },
        None,
    );
    super::skirt_structure::append(&mut recipe, &kernel, design)?;
    if let Some(pattern) = &design.puff_and_slash {
        let puff_kernel = coordinate_kernel(gpu, include_str!("skirt_puff.wgsl"))?;
        super::textile_pattern::append_outer(&mut recipe, &puff_kernel, pattern, design.thickness)?;
    }
    let mut words = design.profile.shader_words().to_vec();
    words.push(design.fitted_fraction.unit());
    words.push(first_row as f32);
    words.extend(
        design
            .pleats
            .as_ref()
            .map_or([0.0; 4], |pleats| pleats.shader_words()),
    );
    words.extend(
        design
            .puff_and_slash
            .as_ref()
            .map_or([0.0; 5], |pattern| pattern.shader_words()),
    );
    words.push(design.thickness.metres());
    words.push(
        design
            .pleats
            .as_ref()
            .map_or(1.0, |pleats| pleats.front_depth.unit()),
    );
    words.extend(design.pleats.as_ref().map_or([0.0; 2], |pleats| {
        [pleats.waist_depth.metres(), pleats.front_panel.unit()]
    }));
    recipe.record(gpu, batch, &words, &[fit])
}

fn coordinate_kernel(gpu: &ArmorGpu, shape: &str) -> Result<CoordKernel, GenerateError> {
    let source = format!(
        "const COLUMNS: u32 = {SKIRT_RING_SEGMENTS}u;\nconst ROWS: u32 = {SKIRT_AXIAL_SEGMENTS}u;\nconst FIRST_SECTION: u32 = {SKIRT_PROFILE_FIRST_WORD}u;\nconst SECTION_WORDS: u32 = {SKIRT_SECTION_WORDS}u;\nconst ATTACHMENT: u32 = {SKIRT_FIT_WORDS}u;\nconst POINT_WORDS: u32 = {SKIRT_ATTACHMENT_POINT_WORDS}u;\n{AUTHORED}{}{}{}{}",
        include_str!("skirt_folds.wgsl"),
        include_str!("skirt_surface.wgsl"),
        include_str!("textile_fullness.wgsl"),
        shape,
    );
    CoordKernel::new(gpu, &source)
}

/// Native topology adapter; all positions remain GPU evaluated.
fn carrier(
    design: &SkirtDesign,
    attachment: Option<&SkirtAttachment>,
) -> Result<CoordShell, GenerateError> {
    let mut coords = Vec::new();
    let mut indices = Vec::new();
    let first_row = design
        .bodice
        .as_ref()
        .map_or(0, |bodice| bodice.first_skirt_row());
    for row in first_row..=SKIRT_AXIAL_SEGMENTS {
        for column in 0..SKIRT_RING_SEGMENTS {
            coords.push([column as f32, row as f32, 0.0, 0.0]);
        }
    }
    for row in 0..SKIRT_AXIAL_SEGMENTS - first_row {
        for column in 0..SKIRT_RING_SEGMENTS {
            let a = (row * SKIRT_RING_SEGMENTS + column) as u32;
            let b = (row * SKIRT_RING_SEGMENTS + (column + 1) % SKIRT_RING_SEGMENTS) as u32;
            let c = b + SKIRT_RING_SEGMENTS as u32;
            let d = a + SKIRT_RING_SEGMENTS as u32;
            // Rows descend: reverse the upward-tube convention.
            indices.extend([a, c, b, a, d, c]);
        }
    }
    if let Some(attachment) = attachment {
        attachment.append(&mut coords, &mut indices)?;
    }
    Ok(CoordShell {
        coords,
        indices,
        boundary_normals: BoundaryNormals::Smooth,
        thickness: design.thickness.metres(),
        extrusion: CoordExtrusion::Normal,
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    })
}
