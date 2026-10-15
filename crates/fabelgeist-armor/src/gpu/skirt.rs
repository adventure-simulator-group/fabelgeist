//! One continuous cyclic skirt carrier, before shell thickening.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::{AUTHORED, CoordExtrusion, CoordKernel, CoordShell};
use super::recipe::PartRecipe;
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
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let mut coords = Vec::new();
    let mut indices = Vec::new();
    for row in 0..=SKIRT_AXIAL_SEGMENTS {
        for column in 0..SKIRT_RING_SEGMENTS {
            coords.push([column as f32, row as f32, 0.0, 0.0]);
        }
    }
    for row in 0..SKIRT_AXIAL_SEGMENTS {
        for column in 0..SKIRT_RING_SEGMENTS {
            let a = (row * SKIRT_RING_SEGMENTS + column) as u32;
            let b = (row * SKIRT_RING_SEGMENTS + (column + 1) % SKIRT_RING_SEGMENTS) as u32;
            let c = b + SKIRT_RING_SEGMENTS as u32;
            let d = a + SKIRT_RING_SEGMENTS as u32;
            // Rows descend: reverse the upward-tube convention.
            indices.extend([a, c, b, a, d, c]);
        }
    }
    let source = format!(
        "const COLUMNS: u32 = {SKIRT_RING_SEGMENTS}u;\nconst ROWS: u32 = {SKIRT_AXIAL_SEGMENTS}u;\nconst FIRST_SECTION: u32 = {SKIRT_PROFILE_FIRST_WORD}u;\nconst SECTION_WORDS: u32 = {SKIRT_SECTION_WORDS}u;\n{AUTHORED}{}",
        include_str!("skirt.wgsl"),
    );
    let kernel = CoordKernel::new(gpu, &source)?;
    let mut recipe = PartRecipe::new();
    recipe.push_coord(
        CoordShell {
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
        },
        kernel,
    )?;
    recipe.component(ArmorComponentRole::OuterFabric, None);
    let mut words = design.profile.shader_words().to_vec();
    words.push(design.fitted_fraction.unit());
    recipe.record(gpu, batch, &words, &[fit])
}
