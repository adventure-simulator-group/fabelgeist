//! Long thigh plates cut from a measured upright anatomical carrier.
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use super::chart::{ChartBoundary, ChartKernel, PlateChart};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart, device_error, wgsl};
use crate::{ArmorComponentRole, GarmentArmorDesign, GarmentPlateShape, GenerateError};

/// Frame, fitting metadata and quadratic control sections precede hull nodes.
pub const HULL_START: u32 = 48;
/// Maximum corners in one convex anatomical section. Overflow is an error.
pub const HULL_LIMIT: u32 = 512;
pub const HULL_WORDS: u32 = 1 + HULL_LIMIT * 2;
pub const MEASURED_SECTIONS: u32 = 26;
pub const RADII: u32 = 64;
pub const MEASURED_WORDS: u32 = 2 + RADII;
pub const LAYER_STATIONS: u32 = 17;
pub const COURSE_ROWS: u32 = 8;

pub fn layout() -> String {
    format!(
        "const COURSE_ROWS: u32 = {COURSE_ROWS}u;\nconst HULL_START: u32 = {HULL_START}u;\nconst HULL_WORDS: u32 = {HULL_WORDS}u;\nconst HULL_LIMIT: u32 = {HULL_LIMIT}u;\nconst MEASURED_SECTIONS: u32 = {MEASURED_SECTIONS}u;\nconst RADII: u32 = {RADII}u;\nconst MEASURED_WORDS: u32 = {MEASURED_WORDS}u;\nconst LAYER_STATIONS: u32 = {LAYER_STATIONS}u;\n"
    )
}

/// Evaluate both sides after their fit buffers have been measured on the GPU.
/// The frame is identity in canonical body coordinates, Y upright, +X left.
pub fn record_wrapped_tassets(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &GarmentArmorDesign,
    fits: [&Buffer; 2],
    statuses: [&Buffer; 2],
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let GarmentPlateShape::WrappedTassets(shape) = design.plate_shape else {
        return Err(GenerateError::InvalidSurface);
    };
    let words = [
        shape.inner_wrap.unit() * std::f32::consts::PI,
        shape.outer_wrap.unit() * std::f32::consts::PI,
        // Reserve the normal extrusion so the finished medial edge retains
        // the requested gap, including on thicker plates.
        shape.inner_gap.metres()
            + design.wall_thickness.metres()
            + design.fluting.map_or(0.0, |pattern| pattern.depth.metres()),
        shape.upper_edge_slope.unit(),
        shape.inner_cutaway.metres(),
        shape.hem_rounding.metres(),
        f32::from(shape.section_break),
        shape.section_gap.metres(),
        design.wall_thickness.metres(),
    ];
    record_boundaries(gpu, batch, design.lame_count, &words, fits, statuses)?;
    let kernel = ChartKernel::new(
        gpu,
        &format!(
            "{}\n{}\n{}",
            layout(),
            include_str!("wrapped_tasset_carrier.wgsl"),
            include_str!("wrapped_tasset_shape.wgsl")
        ),
    )?;
    let mut recipe = PartRecipe::new();
    for (frame, side) in [1.0, -1.0].into_iter().enumerate() {
        for course in 0..design.lame_count {
            let top = 1.0 - f32::from(course) / f32::from(design.lame_count);
            let bottom =
                (1.0 - (f32::from(course) + 1.0 + 0.14) / f32::from(design.lame_count)).max(0.0);
            recipe.push_chart(
                PlateChart {
                    rows: COURSE_ROWS as usize,
                    around: 64,
                    boundary: ChartBoundary::Open,
                    thickness: design.wall_thickness.metres(),
                    extrusion: Extrusion::Normal,
                    origin: [0.0; 3],
                    axis: [0.0, 1.0, 0.0],
                    fluting: design.fluting,
                    span: [bottom, top],
                    values: [side, bottom, top, f32::from(course)],
                    mirrored: false,
                    frame,
                },
                kernel.clone(),
            )?;
        }
        recipe.component(ArmorComponentRole::Tassets, None);
    }
    recipe.record(gpu, batch, &words, &fits)
}

/// The inner-angle solve depends on the plate row, not its column. Reuse its
/// exact result across every vertex in that row, including fluted columns.
fn record_boundaries(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    courses: u8,
    words: &[f32],
    fits: [&Buffer; 2],
    statuses: [&Buffer; 2],
) -> Result<(), GenerateError> {
    let source = format!(
        "{}\n{}\n{}\n{}\n{}",
        wgsl::MATH,
        layout(),
        include_str!("wrapped_tasset_carrier.wgsl"),
        include_str!("wrapped_tasset_shape.wgsl"),
        BOUNDARIES
    );
    let kernel = gpu
        .cache()
        .get(gpu.context(), &source)
        .map_err(device_error)?;
    let design = gpu.upload(BufferUpload::from_elements(words))?;
    for (side, sign) in [1.0_f32, -1.0].into_iter().enumerate() {
        for course in 0..courses {
            let mut params = PassParameters::new();
            params.insert("frames", fits[side].clone());
            params.insert("status", statuses[side].clone());
            params.insert("design", design.clone());
            params.insert("value0", sign);
            params.insert(
                "value1",
                (1.0 - (f32::from(course) + 1.0 + 0.14) / f32::from(courses)).max(0.0),
            );
            params.insert("value2", 1.0 - f32::from(course) / f32::from(courses));
            params.insert("value3", f32::from(course));
            batch
                .dispatch_items(&kernel, &params, COURSE_ROWS + 1)
                .map_err(device_error)?;
        }
    }
    Ok(())
}

const BOUNDARIES: &str = r#"
@group(0) @binding(0) var<storage, read_write> frames: array<f32>;
@group(0) @binding(1) var<storage, read> design: array<f32>;
@group(0) @binding(2) var<storage, read_write> status: array<atomic<u32>>;
struct Params { value0: f32, value1: f32, value2: f32, value3: f32 };
@group(0) @binding(3) var<uniform> params: Params;
fn invalid_chart_point() -> vec3<f32> { atomicOr(&status[0],1u);return vec3<f32>(0.0); }
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let row=id.x;
    if (row>COURSE_ROWS) { return; }
    let axial=mix(params.value1,params.value2,f32(row)/f32(COURSE_ROWS));
    frames[u32(frames[45u])+u32(params.value3)*(COURSE_ROWS+1u)+row]=inner_angle(axial);
}
"#;
