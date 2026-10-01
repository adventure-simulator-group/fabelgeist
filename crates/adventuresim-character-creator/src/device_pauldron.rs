//! Fit the shared shoulder saddle against body and completed lower equipment.
//! Inputs use the canonical unposed frame: Y is height, Z is anterior depth.
use crate::armor_frames::FitRegion;
use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_piece::DeviceRecording;
use anyhow::Result;
use fabelgeist_armor::{
    PauldronDesign,
    gpu::pauldron::{CARRIER_COUNT, DevicePauldronCarrier, carrier_constants},
};
use fabelgeist_compute::KernelBatch;

const CLEARANCE_SMOOTHING_PASSES: usize = 180;
const OBLIQUE_WALL_RESERVE_GAUGES: f32 = 2.5;

impl DeviceWearer<'_> {
    pub(crate) fn record_fitted_pauldron(
        &self,
        batch: &mut KernelBatch,
        design: &PauldronDesign,
        region: FitRegion,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<DeviceRecording> {
        let frame = self.record_frame(batch, region)?;
        let gpu = self.gpu;
        let carrier = DevicePauldronCarrier::record(gpu, batch, design, &frame.frame)?;
        let mut triangles = self
            .host
            .faces
            .iter()
            .map(|f| f.map(|i| self.host.positions[i as usize]))
            .collect::<Vec<_>>();
        let mut ends = vec![triangles.len() as u32];
        for layer in layers {
            triangles.extend(
                layer
                    .faces
                    .iter()
                    .map(|f| f.map(|i| layer.positions[i as usize])),
            );
            ends.push(triangles.len() as u32);
        }
        let triangles = gpu.upload(&triangles)?;
        let groups = ends.len() as u32;
        let ends = gpu.upload(&ends)?;
        let reserve = gpu.scratch(4, "shoulder sampling reserve")?;
        let delta = gpu.scratch(CARRIER_COUNT as u64 * 12, "shoulder clearance")?;
        let next = gpu.scratch(CARRIER_COUNT as u64 * 12, "shoulder clearance smoothing")?;
        let words = [
            Word::U("count", CARRIER_COUNT),
            Word::U("groups", groups),
            Word::F(
                "wall_reserve",
                design.gauge.thickness.metres() * OBLIQUE_WALL_RESERVE_GAUGES,
            ),
            Word::F(
                "body_clearance",
                (design.gauge.clearance.metres() + design.gauge.thickness.metres())
                    .max(design.gauge.thickness.metres() * OBLIQUE_WALL_RESERVE_GAUGES),
            ),
            Word::F(
                "plate_clearance",
                design.plate_clearance.metres() + design.gauge.thickness.metres(),
            ),
        ];
        let shared = format!(
            "{}\n{}",
            carrier_constants(),
            include_str!("device_pauldron.wgsl")
        );
        dispatch(
            self,
            batch,
            &format!("{shared}{RESERVE}"),
            &[
                read("fit", &carrier.frame_points),
                write("reserve", &reserve),
            ],
            &words,
            Grid::Singles(1),
        )?;
        dispatch(
            self,
            batch,
            &format!("{shared}{}", include_str!("device_pauldron_project.wgsl")),
            &[
                read("fit", &carrier.frame_points),
                read("triangles", &triangles),
                read_u32("ends", &ends),
                read("reserve", &reserve),
                write("delta", &delta),
            ],
            &words,
            Grid::Items(CARRIER_COUNT),
        )?;
        self.record_shoulder_smoothing(batch, &carrier, delta, next, &shared, &words)?;
        let part = carrier.record_plates(gpu, batch, design)?;
        dispatch(
            self,
            batch,
            COPY_STATUS,
            &[
                read_u32("carrier_status", &carrier.status),
                atomic("status", part.status()),
            ],
            &[],
            Grid::Singles(1),
        )?;
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, region)],
            checks: Vec::new(),
        })
    }
    fn record_shoulder_smoothing(
        &self,
        batch: &mut KernelBatch,
        carrier: &DevicePauldronCarrier,
        mut delta: fabelgeist_gpu::prelude::Buffer,
        mut next: fabelgeist_gpu::prelude::Buffer,
        shared: &str,
        words: &[Word],
    ) -> Result<()> {
        for _ in 0..CLEARANCE_SMOOTHING_PASSES {
            dispatch(
                self,
                batch,
                &format!("{shared}{SMOOTH}"),
                &[
                    read("fit", &carrier.frame_points),
                    read("delta", &delta),
                    write("next", &next),
                ],
                words,
                Grid::Items(CARRIER_COUNT),
            )?;
            std::mem::swap(&mut delta, &mut next);
        }
        dispatch(
            self,
            batch,
            &format!("{shared}{APPLY}"),
            &[write("fit", &carrier.frame_points), read("delta", &delta)],
            words,
            Grid::Items(CARRIER_COUNT),
        )?;
        Ok(())
    }
}

const RESERVE: &str = r#"
@compute @workgroup_size(1)
fn main() {
    var radius = params.wall_reserve;
    let frame = frame_at(0u);
    for (var i = 0u; i < params.count; i += 1u) {
        let p = frame_point(frame, formed(i));
        if (i % (CARRIER_COLUMNS + 1u) < CARRIER_COLUMNS) {
            radius = max(radius, distance(p.xy, frame_point(frame, formed(i + 1u)).xy));
        }
        if (i / (CARRIER_COLUMNS + 1u) < CARRIER_ROWS) {
            radius = max(radius, distance(p.xy, frame_point(frame, formed(i + CARRIER_COLUMNS + 1u)).xy));
        }
    }
    reserve[0] = radius;
}
"#;
const SMOOTH: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let row = i / (CARRIER_COLUMNS + 1u);
    let col = i % (CARRIER_COLUMNS + 1u);
    let neighbors = array<u32, 4>(select(i, i - 1u, col > 0u), select(i, i + 1u, col < CARRIER_COLUMNS),
        select(i, i - CARRIER_COLUMNS - 1u, row > 0u), select(i, i + CARRIER_COLUMNS + 1u, row < CARRIER_ROWS));
    for (var axis = 0u; axis < 3u; axis += 1u) {
        let d = delta[i * 3u + axis];
        var average = 0.0;
        for (var n = 0u; n < 4u; n += 1u) { average += delta[neighbors[n] * 3u + axis] * 0.25; }
        var value = d;
        if (average * d >= 0.0 && abs(average) > abs(d)) { value = mix(d, average, 0.65); }
        next[i * 3u + axis] = value;
    }
}
"#;
const APPLY: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    for (var k = 0u; k < 3u; k += 1u) { fit[FITTED_START + i * 3u + k] = fit[FORMED_START + i * 3u + k] + delta[i * 3u + k]; }
}
"#;
const COPY_STATUS: &str = r#"
@compute @workgroup_size(1)
fn main() { atomicOr(&status[0], carrier_status[0]); }
"#;
