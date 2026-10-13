//! Measure neck support throughout the collar, including intermediate bulges.
use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
#[cfg(test)]
use fabelgeist_gpu::prelude::BufferUpload;

use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_gorget_bib::ENVELOPE;
use crate::device_gorget_cage::{BIB_COLUMNS, BIB_ROWS, CAGE, layout};
use crate::device_gorget_projection::PROJECTION;

// Collar and bib share a seam; bounded alternating support fits settle it.
const SUPPORT_PASSES: usize = 4;
const SUPPORT_TOLERANCE_METRES: f32 = 0.00001;

impl DeviceWearer<'_> {
    /// Seat the shared seam, then measure the collar on its final seating rays.
    pub(crate) fn record_gorget_support(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        samples: &Buffer,
        bib_padding: f32,
        collar_padding: f32,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<()> {
        for _ in 0..SUPPORT_PASSES {
            self.record_collar_fit(batch, fit, samples, collar_padding)?;
            self.record_bib_fit(batch, fit, samples, bib_padding, layers)?;
        }
        self.record_collar_fit(batch, fit, samples, collar_padding)?;
        self.record_bib_measure(batch, fit, samples, bib_padding, layers)?;
        dispatch(
            self,
            batch,
            &format!("{}{VALIDATE_BIB}", layout()),
            &[write("fit", fit)],
            &[Word::F("support_tolerance", SUPPORT_TOLERANCE_METRES)],
            Grid::Singles([1, 1, 1].into()),
        )
    }

    pub(crate) fn record_collar_fit(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        samples: &Buffer,
        padding: f32,
    ) -> Result<()> {
        let cage = format!("fn cage_word(i: u32) -> f32 {{ return fit[i]; }}\n{CAGE}");
        dispatch(
            self,
            batch,
            &format!("{}{cage}{PROJECTION}{MEASURE}", layout()),
            &[
                read_u32("faces", &self.body.faces),
                read("points", samples),
                write("fit", fit),
            ],
            &[
                Word::U("faces_count", self.body.face_count),
                Word::F("padding", padding),
            ],
            Grid::Items((BIB_ROWS * BIB_COLUMNS).into()),
        )?;
        let envelope = ENVELOPE
            .replace("BIB_RAW", "COLLAR_RAW")
            .replace("fit[BIB +", "fit[COLLAR +");
        dispatch(
            self,
            batch,
            &format!("{}{envelope}", layout()),
            &[write("fit", fit)],
            &[],
            Grid::Items((BIB_ROWS * BIB_COLUMNS).into()),
        )
    }
}

const VALIDATE_BIB: &str = r#"
@compute @workgroup_size(1)
fn main() {
    for (var i = 0u; i < BIB_ROWS * BIB_COLUMNS; i += 1u) {
        if (fit[BIB_RAW + i] > params.support_tolerance) {
            fit[FIT_FAILED] = 1.0;
        }
    }
}
"#;

const MEASURE: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= BIB_ROWS * BIB_COLUMNS) { return; }
    let t = f32(index / BIB_COLUMNS) / f32(BIB_ROWS - 1u);
    let angle = TAU * f32(index % BIB_COLUMNS) / f32(BIB_COLUMNS - 1u);
    let p = collar_point(t, control_angle(angle));
    let direction = vec3<f32>(sin(angle), 0.0, cos(angle));
    let tangent = vec3<f32>(cos(angle), 0.0, -sin(angle));
    let query = vec2<f32>(dot(p, tangent), p.y);
    var deepest = -INFINITY;
    for (var f = 0u; f < params.faces_count; f += 1u) {
        var corners: array<vec3<f32>, 3>;
        for (var c = 0u; c < 3u; c += 1u) {
            let point = points_at(faces[f * 3u + c]);
            corners[c] = vec3<f32>(dot(point, tangent), point.y, dot(point, direction));
        }
        let hit = ray_depth(query, corners[0], corners[1], corners[2]);
        if (hit.y != 0.0) { deepest = max(deepest, hit.x); }
    }
    // A later support pass may follow bib seating. Keep the existing outward
    // fit and add only the new clearance deficit measured at the shifted ray.
    fit[COLLAR_RAW + index] = max(deepest + params.padding - dot(p, direction), 0.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armor_frames::Wearer;
    use crate::device_gorget_cage::*;
    use fabelgeist_armor::ArmorGpu;

    #[test]
    fn intermediate_neck_bulge_and_shared_bib_seam_clear_the_measured_body() {
        let gpu = ArmorGpu::open().unwrap();
        // A local neck bulge missed by sections at y=0 and y=0.1.
        let positions = [
            [-0.02, 0.035, 0.07],
            [0.02, 0.035, 0.07],
            [-0.02, 0.065, 0.07],
            [0.02, 0.065, 0.07],
        ];
        let normals = [[0.0, 0.0, 1.0]; 4];
        let faces = [[0, 1, 2], [1, 3, 2]];
        let indices = [[0; 8]; 4];
        let weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 4];
        let names = vec!["c_neck".into()];
        let states = [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]];
        let host = Wearer {
            positions: &positions,
            normals: &normals,
            faces: &faces,
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &names,
            joints: &states,
        };
        let body = host.upload(&gpu).unwrap();
        let wearer = DeviceWearer {
            gpu: &gpu,
            body: &body,
            host: &host,
        };
        let mut words = vec![0.0_f32; GORGET_FIT_WORDS as usize];
        words[3] = 1.0;
        words[7] = 1.0;
        words[11] = 1.0;
        words[COLLAR_RADIUS as usize] = 0.04;
        words[COLLAR_RADIUS as usize + 1] = 0.04;
        words[BASE_RADIUS as usize] = 0.04;
        words[FRONT as usize] = 0.04;
        words[BACK as usize] = 0.04;
        words[PLANES as usize] = 0.1;
        words[OUTER_WIDTH as usize] = 0.1;
        words[OUTER_WIDTH as usize + 1] = 0.1;
        let fit = gpu.upload(BufferUpload::from_elements(&words)).unwrap();
        let samples = gpu.upload(BufferUpload::from_elements(&positions)).unwrap();
        let result = gpu.scratch(6 * 12, "collar regression samples").unwrap();
        let mut batch = gpu.batch("collar bulge regression");
        wearer
            .record_collar_fit(&mut batch, &fit, &samples, 0.002)
            .unwrap();
        let cage = format!("fn cage_word(i: u32) -> f32 {{ return fit[i]; }}\n{CAGE}");
        dispatch(
            &wearer,
            &mut batch,
            &format!("{}{cage}{EVALUATE}", layout()),
            &[read("fit", &fit), write("points", &result)],
            &[],
            Grid::Singles([1, 1, 1].into()),
        )
        .unwrap();
        batch.submit();
        let support = gpu.read::<f32>(&fit).unwrap();
        assert_eq!(support[15], 0.0, "final sampled support failed readiness");
        let points = gpu.read::<[f32; 3]>(&result).unwrap();
        for point in &points[..3] {
            assert!(point[2] >= 0.072 - 1e-6, "{point:?}");
        }
        let distance = points[3]
            .iter()
            .zip(points[4])
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt();
        assert!(distance <= 1e-6, "collar/bib seam separates by {distance}m");
    }

    #[test]
    fn bib_seating_rechecks_the_shifted_collar_against_a_side_neck_bulge() {
        let gpu = ArmorGpu::open().unwrap();
        let positions = [
            [0.04, -0.02, -0.02],
            [0.04, 0.03, -0.02],
            [0.04, -0.02, 0.02],
            [0.04, 0.03, 0.02],
            [0.09, 0.012, -0.01],
            [0.09, 0.018, -0.01],
            [0.09, 0.012, 0.01],
            [0.09, 0.018, 0.01],
            [0.09, 0.006, -0.01],
            [0.09, 0.0075, -0.01],
            [0.09, 0.006, 0.01],
            [0.09, 0.0075, 0.01],
        ];
        let normals = [[1.0, 0.0, 0.0]; 12];
        let faces = [
            [0, 1, 2],
            [1, 3, 2],
            [4, 5, 6],
            [5, 7, 6],
            [8, 9, 10],
            [9, 11, 10],
        ];
        let indices = [[0; 8]; 12];
        let weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 12];
        let names = vec!["c_neck".into()];
        let states = [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]];
        let host = Wearer {
            positions: &positions,
            normals: &normals,
            faces: &faces,
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &names,
            joints: &states,
        };
        let body = host.upload(&gpu).unwrap();
        let wearer = DeviceWearer {
            gpu: &gpu,
            body: &body,
            host: &host,
        };
        let mut words = vec![0.0_f32; GORGET_FIT_WORDS as usize];
        words[3] = 1.0;
        words[7] = 1.0;
        words[11] = 1.0;
        words[COLLAR_RADIUS as usize] = 0.04;
        words[COLLAR_RADIUS as usize + 1] = 0.04;
        words[BASE_RADIUS as usize] = 0.04;
        words[FRONT as usize] = 0.04;
        words[BACK as usize] = 0.04;
        words[PLANES as usize] = 0.02;
        words[HEM as usize + 1] = -0.1;
        words[OUTER_WIDTH as usize] = 0.1;
        words[OUTER_WIDTH as usize + 1] = 0.1;
        let fit = gpu.upload(BufferUpload::from_elements(&words)).unwrap();
        let samples = gpu.upload(BufferUpload::from_elements(&positions)).unwrap();
        let result = gpu.scratch(6 * 12, "collar regression samples").unwrap();
        let mut batch = gpu.batch("collar bulge regression");
        let layer_positions = [
            [-0.3, -0.0017, -0.3],
            [0.3, -0.0017, -0.3],
            [-0.3, -0.0017, 0.3],
            [0.3, -0.0017, 0.3],
            [0.08, 0.010, -0.3],
            [0.14, 0.010, -0.3],
            [0.08, 0.010, 0.3],
            [0.14, 0.010, 0.3],
        ];
        let layer_faces = [[0, 1, 2], [1, 3, 2], [4, 5, 6], [5, 7, 6]];
        let layer = crate::armor_layer::ArmorLayerSurface {
            positions: &layer_positions,
            faces: &layer_faces,
            joint_indices: &indices[..8],
            joint_weights: &weights[..8],
        };
        wearer
            .record_gorget_support(&mut batch, &fit, &samples, 0.002, 0.002, &[layer])
            .unwrap();
        let cage = format!("fn cage_word(i: u32) -> f32 {{ return fit[i]; }}\n{CAGE}");
        dispatch(
            &wearer,
            &mut batch,
            &format!("{}{cage}{EVALUATE_SIDE}", layout()),
            &[read("fit", &fit), write("points", &result)],
            &[],
            Grid::Singles([1, 1, 1].into()),
        )
        .unwrap();
        batch.submit();
        let support = gpu.read::<f32>(&fit).unwrap();
        assert_eq!(support[15], 0.0, "final sampled support failed readiness");
        let points = gpu.read::<[f32; 3]>(&result).unwrap();
        let bib = points[5];
        assert!(
            bib[0] < 0.08 || bib[0] > 0.14 || bib[1] >= 0.012 - 1e-6,
            "final seam correction moved the bib into lower support: {bib:?}"
        );
        let point = points[1];
        assert!(
            point[0] >= 0.092 - 1e-6 || point[1] < 0.012 || point[1] > 0.018,
            "final collar enters the shifted body support: {point:?}"
        );
        let distance = points[3]
            .iter()
            .zip(points[4])
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt();
        assert!(distance <= 1e-6, "collar/bib seam separates by {distance}m");
        // A final measured deficit at the last sample must fail readiness too;
        // successful surface output cannot mask a nonconvergent support fit.
        let mut unresolved = support;
        unresolved[(BIB_RAW + BIB_ROWS * BIB_COLUMNS - 1) as usize] =
            SUPPORT_TOLERANCE_METRES * 2.0;
        let unresolved = gpu
            .upload(BufferUpload::from_elements(&unresolved))
            .unwrap();
        let mut failure = gpu.batch("unresolved final bib support");
        dispatch(
            &wearer,
            &mut failure,
            &format!("{}{VALIDATE_BIB}", layout()),
            &[write("fit", &unresolved)],
            &[Word::F("support_tolerance", SUPPORT_TOLERANCE_METRES)],
            Grid::Singles([1, 1, 1].into()),
        )
        .unwrap();
        failure.submit();
        assert_eq!(gpu.read::<f32>(&unresolved).unwrap()[15], 1.0);
    }

    const EVALUATE_SIDE: &str = r#"
@compute @workgroup_size(1)
fn main() {
    points_set(0u, gorget_collar(0.4, PI * 0.5));
    points_set(1u, gorget_collar(0.5, PI * 0.5));
    points_set(2u, gorget_collar(0.6, PI * 0.5));
    points_set(3u, gorget_collar(1.0, PI * 0.5));
    points_set(4u, gorget_bib(0.0, PI * 0.5));
    points_set(5u, gorget_bib(1.0 / 16.0, PI * 0.5));
}
"#;

    const EVALUATE: &str = r#"
@compute @workgroup_size(1)
fn main() {
    points_set(0u, gorget_collar(0.4, 0.0));
    points_set(1u, gorget_collar(0.5, 0.0));
    points_set(2u, gorget_collar(0.6, 0.0));
    points_set(3u, gorget_collar(1.0, 0.0));
    points_set(4u, gorget_bib(0.0, 0.0));
}
"#;
}
