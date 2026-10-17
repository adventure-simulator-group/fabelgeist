//! One continuous cyclic skirt carrier, before shell thickening.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::{AUTHORED, CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
use super::skirt_attachment::{SKIRT_ATTACHMENT_POINT_WORDS, SkirtAttachment};
use super::{ArmorGpu, DevicePart};
use crate::skirt::{SKIRT_MESH_AXIAL_SEGMENTS, SKIRT_MESH_RING_SEGMENTS};
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
    if attachment.is_some_and(|patch| patch.seam_layout == super::SkirtSeamLayout::StraightFall)
        && (!matches!(design.profile, crate::SkirtProfile::Straight)
            || design.pleats.is_some()
            || design.puff_and_slash.is_some())
    {
        return Err(GenerateError::InvalidSurface);
    }
    if design.bodice.is_some() && attachment.is_none() {
        return Err(GenerateError::InvalidSurface);
    }
    let first_row = attachment.map_or(0, |patch| patch.first_row);
    let kernel = coordinate_kernel(gpu, include_str!("skirt.wgsl"))?;
    let mut recipe = PartRecipe::new();
    recipe.textile_chart();
    let cloth = carrier(design, attachment)?;
    let first_finish = cloth.coords.len() as u32;
    recipe.push_coord(cloth, kernel.clone())?;
    recipe.component(
        if design.puff_and_slash.is_some() {
            ArmorComponentRole::Undercloth
        } else {
            ArmorComponentRole::OuterFabric
        },
        None,
    );
    if let Some(patch) = attachment {
        super::skirt_attachment_layers::append(&mut recipe, &kernel, patch, first_finish)?;
    }
    super::skirt_structure::append(&mut recipe, &kernel, design, first_row)?;
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
    words.push(
        design
            .pleats
            .as_ref()
            .map_or(0., |pleats| pleats.front_panel.unit()),
    );
    words.push(attachment.map_or(0, |patch| patch.point_count) as f32);
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
    let first_row = attachment.map_or(0, |patch| patch.first_row);
    let hanging_intervals =
        attachment.map_or(SKIRT_MESH_AXIAL_SEGMENTS, |patch| patch.hanging_intervals);
    let columns = attachment.map_or(SKIRT_MESH_RING_SEGMENTS, |patch| patch.hanging_columns());
    for row in 0..=hanging_intervals {
        let descent = first_row as f32
            + (SKIRT_AXIAL_SEGMENTS - first_row) as f32 * row as f32 / hanging_intervals as f32;
        for column in 0..columns {
            if let Some(patch) =
                attachment.filter(|patch| patch.seam_layout == super::SkirtSeamLayout::StraightFall)
            {
                // Native point ordinal, descent share, and straight-fall tag.
                const STRAIGHT_FALL_COORDINATE: f32 = 5.;
                coords.push([
                    patch.waist[column].vertex as f32,
                    descent / SKIRT_AXIAL_SEGMENTS as f32,
                    0.,
                    STRAIGHT_FALL_COORDINATE,
                ]);
            } else {
                coords.push([
                    column as f32 * SKIRT_RING_SEGMENTS as f32 / SKIRT_MESH_RING_SEGMENTS as f32,
                    descent,
                    0.0,
                    0.0,
                ]);
            }
        }
    }
    for row in 0..hanging_intervals {
        for column in 0..columns {
            let a = (row * columns + column) as u32;
            let b = (row * columns + (column + 1) % columns) as u32;
            let c = b + columns as u32;
            let d = a + columns as u32;
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
        extrusion: match attachment.map_or(fabelgeist_compute::NormalWeighting::Area, |a| {
            a.normal_weighting()
        }) {
            fabelgeist_compute::NormalWeighting::Angle => CoordExtrusion::AngleWeightedNormal,
            fabelgeist_compute::NormalWeighting::Area => CoordExtrusion::Normal,
        },
        values: [0.0; 4],
        mirrored: false,
        frame: 0,
        passes: 1,
        hinge: None,
    })
}
