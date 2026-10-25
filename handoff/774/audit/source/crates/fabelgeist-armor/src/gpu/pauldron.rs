//! A shared shoulder saddle fitted before it is divided into overlapping plates.
use super::chart::{ChartBoundary, ChartKernel, PlateChart};
use super::part::Extrusion;
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart, device_error, wgsl};
use crate::{
    ArmorComponentRole, GenerateError, LimbArmorDesign, PauldronDesign, Permille, PlateCourse,
    PlateGridEnd, PlateJointMotion, PlateMount, PlateParent,
};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

pub const CARRIER_COLUMNS: u32 = 96;
pub const CARRIER_ROWS: u32 = 64;
pub const CARRIER_COUNT: u32 = (CARRIER_COLUMNS + 1) * (CARRIER_ROWS + 1);
pub const FORMED_START: u32 = 16;
pub const FITTED_START: u32 = FORMED_START + CARRIER_COUNT * 3;
const LAME_ROWS: usize = 8;
const MAIN_ROWS: usize = 32;
const NECK_OVERLAP: f32 = 0.02;

pub struct DevicePauldronCarrier {
    /// Frame header, then formed and fitted points in that frame's local axes.
    pub frame_points: Buffer,
    pub status: Buffer,
}

pub fn carrier_constants() -> String {
    format!(
        "const CARRIER_COLUMNS: u32 = {CARRIER_COLUMNS}u;\nconst CARRIER_ROWS: u32 = {CARRIER_ROWS}u;\nconst FORMED_START: u32 = {FORMED_START}u;\nconst FITTED_START: u32 = {FITTED_START}u;\n"
    )
}

fn design_words(d: &PauldronDesign) -> Vec<f32> {
    let o = &d.outline;
    vec![
        d.gauge.thickness.metres(),
        d.gauge.clearance.metres(),
        d.arm_allowance.metres(),
        d.front_reach.metres(),
        d.rear_reach.metres(),
        d.front_drop.metres(),
        d.rear_drop.metres(),
        d.neck_reach.metres(),
        d.crown_height.unit(),
        d.arm_length.metres(),
        f32::from(d.lower_lames),
        d.fluting.map_or(0.0, |f| f.depth.metres()),
        o.front_return.radians(),
        o.rear_return.radians(),
        o.front_extension.metres(),
        o.rear_extension.metres(),
        o.wing_start.radians(),
        o.corner_rounding.unit(),
        o.front_wing_position.unit(),
        o.rear_wing_position.unit(),
        o.front_wing_rounding.unit(),
        o.rear_wing_rounding.unit(),
        o.upper_span.unit(),
        o.arm_wrap.radians(),
    ]
}

impl DevicePauldronCarrier {
    pub fn record(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        design: &PauldronDesign,
        frame: &Buffer,
    ) -> Result<Self, GenerateError> {
        LimbArmorDesign::Pauldron(design.clone()).validate()?;
        let carrier = Self {
            frame_points: gpu.scratch(
                (FITTED_START + CARRIER_COUNT * 3) as u64 * 4,
                "shoulder carrier",
            )?,
            status: gpu.scratch(4, "shoulder status")?,
        };
        let source = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            wgsl::MATH,
            wgsl::FRAME,
            wgsl::STATUS,
            carrier_constants(),
            include_str!("pauldron_shape.wgsl"),
            FORM
        );
        let kernel = gpu
            .cache()
            .get(gpu.context(), &source)
            .map_err(device_error)?;
        let mut params = PassParameters::new();
        params.insert("frames", frame.clone());
        params.insert("arena", carrier.frame_points.clone());
        params.insert("status", carrier.status.clone());
        params.insert(
            "design",
            gpu.upload(BufferUpload::from_elements(&design_words(design)))?,
        );
        batch
            .dispatch_items(&kernel, &params, CARRIER_COUNT)
            .map_err(device_error)?;
        Ok(carrier)
    }

    pub fn record_plates(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        d: &PauldronDesign,
    ) -> Result<DevicePart, GenerateError> {
        let source = format!(
            "{}\n{}\n{}",
            carrier_constants(),
            include_str!("pauldron_shape.wgsl"),
            include_str!("pauldron_chart.wgsl")
        );
        let kernel = ChartKernel::new(gpu, &source)?;
        let mut recipe = PartRecipe::new();
        let neck = 1.0 - d.outline.upper_span.unit();
        recipe.push_chart(
            chart(d, MAIN_ROWS, [0.0, neck + NECK_OVERLAP], 0.0, None),
            kernel.clone(),
        )?;
        recipe.mounted_component(
            ArmorComponentRole::Plate,
            PlateMount {
                course: PlateCourse(0),
                parent: None,
                incoming: PlateGridEnd::Last,
                motion: PlateJointMotion::Hinge,
                follow: Permille(0),
            },
        );
        for i in 0..d.upper_lames {
            let step = d.outline.upper_span.unit() / f32::from(d.upper_lames);
            let start = neck + f32::from(i) * step;
            recipe.push_chart(
                chart(
                    d,
                    LAME_ROWS,
                    [start, (start + step + NECK_OVERLAP).min(1.0)],
                    d.gauge.thickness.metres() * 2.0 * f32::from(i + 1),
                    None,
                ),
                kernel.clone(),
            )?;
            recipe.mounted_component(
                ArmorComponentRole::Plate,
                PlateMount {
                    course: PlateCourse(u16::from(i) + 1),
                    parent: Some(PlateParent {
                        course: PlateCourse(u16::from(i)),
                        edge: PlateGridEnd::Last,
                    }),
                    incoming: PlateGridEnd::First,
                    motion: PlateJointMotion::Hinge,
                    follow: Permille(0),
                },
            );
        }
        for i in 0..d.lower_lames {
            recipe.push_chart(
                chart(
                    d,
                    LAME_ROWS,
                    [
                        1.0 - f32::from(i + 1) / f32::from(d.lower_lames),
                        1.0 - f32::from(i) / f32::from(d.lower_lames)
                            + 0.004 / d.arm_length.metres(),
                    ],
                    0.0,
                    Some(i),
                ),
                kernel.clone(),
            )?;
            recipe.mounted_component(
                ArmorComponentRole::JointExtension,
                PlateMount {
                    course: PlateCourse(u16::from(d.upper_lames) + u16::from(i) + 1),
                    parent: Some(PlateParent {
                        course: PlateCourse(if i == 0 {
                            0
                        } else {
                            u16::from(d.upper_lames) + u16::from(i)
                        }),
                        edge: PlateGridEnd::First,
                    }),
                    incoming: PlateGridEnd::Last,
                    motion: PlateJointMotion::Flexible,
                    follow: Permille(Permille::ONE.0 * u16::from(i + 1) / u16::from(d.lower_lames)),
                },
            );
        }
        recipe.record(gpu, batch, &design_words(d), &[&self.frame_points])
    }
}

fn chart(
    d: &PauldronDesign,
    rows: usize,
    span: [f32; 2],
    offset: f32,
    arm: Option<u8>,
) -> PlateChart {
    PlateChart {
        rows,
        around: CARRIER_COLUMNS as usize,
        boundary: ChartBoundary::Open,
        thickness: d.gauge.thickness.metres(),
        extrusion: Extrusion::Normal,
        origin: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        fluting: d.fluting,
        span,
        values: [span[0], span[1], offset, arm.map_or(-1.0, f32::from)],
        mirrored: false,
        frame: 0,
    }
}

const FORM: &str = r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> design: array<f32>;
@group(0) @binding(2) var<storage, read_write> arena: array<f32>;
@group(0) @binding(3) var<storage, read_write> status: array<atomic<u32>>;
var<private> fit: Frame;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= (CARRIER_COLUMNS + 1u) * (CARRIER_ROWS + 1u)) { return; }
    fit = frame_at(0u);
    if (length(fit.y.xz) < 0.01) { fail(STATUS_INVALID_SURFACE); return; }
    if (i < 15u) { arena[i] = frames[i]; }
    let u = f32(i % (CARRIER_COLUMNS + 1u)) / f32(CARRIER_COLUMNS);
    let v = f32(i / (CARRIER_COLUMNS + 1u)) / f32(CARRIER_ROWS);
    let p = saddle_point(u, v);
    for (var k = 0u; k < 3u; k += 1u) {
        arena[FORMED_START + i * 3u + k] = p[k];
        arena[FITTED_START + i * 3u + k] = p[k];
    }
}
"#;
