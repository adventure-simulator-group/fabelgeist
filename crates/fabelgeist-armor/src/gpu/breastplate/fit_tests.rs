//! Actual fitting kernels preserve local support and the physical waist seam.
use super::{
    fit::RadialFit,
    fit_wgsl,
    kernels::{Params, dispatch},
    shape_wgsl,
    topology::{SKIRT_SAMPLES, V_SAMPLES},
};
use crate::{BreastplateDesign, Millimeters, gpu::ArmorGpu};
use fabelgeist_gpu::prelude::BufferUpload;

#[test]
fn body_support_uses_the_outer_envelope_and_preserves_the_chart_ray() {
    let gpu = ArmorGpu::open().unwrap();
    let width = 3;
    let count = width * (V_SAMPLES + SKIRT_SAMPLES - 1);
    let positions = gpu
        .upload(BufferUpload::from_elements(&vec![
            [0.02_f32, 1.0, 0.31];
            count
        ]))
        .unwrap();
    // Height-dependent section centres must not steer the chart's ray.
    let centers = gpu
        .upload(BufferUpload::from_elements(
            &(0..count)
                .map(|i| [0.05_f32, 0.08 + i as f32 * 0.001, 1.0, 2.0])
                .collect::<Vec<_>>(),
        ))
        .unwrap();
    let mut words = shape_wgsl::design_words(&BreastplateDesign {
        front_clearance: Millimeters(18),
        skirt_flare: Millimeters(0),
        ..Default::default()
    });
    words[32] = 1.0;
    words[37] = 1.0;
    words[44] = 1.0;
    words[45] = 0.02;
    words[46] = 0.05;
    let plate = gpu.upload(BufferUpload::from_elements(&words)).unwrap();
    // The chart origin is outside this slab. The first hit is an entry;
    // only the farther exit at z=.30 is its outer support.
    let body = gpu
        .upload(BufferUpload::from_elements(&[
            [-1.0_f32, 0.0, 0.25],
            [1.0, 0.0, 0.25],
            [0.0, 2.0, 0.25],
            [-1.0, 0.0, 0.30],
            [1.0, 0.0, 0.30],
            [0.0, 2.0, 0.30],
        ]))
        .unwrap();
    let faces = gpu
        .upload(BufferUpload::from_elements(&[[0_u32, 1, 2], [3, 4, 5]]))
        .unwrap();
    let support = gpu
        .scratch(count as u64 * 4, "outer envelope support")
        .unwrap();
    let envelope = gpu
        .scratch(count as u64 * 4, "outer envelope smooth")
        .unwrap();
    let status = gpu.upload(BufferUpload::from_elements(&[0_u32])).unwrap();
    let columns = gpu
        .upload(BufferUpload::from_elements(&[-1.0_f32, 0.0, 1.0]))
        .unwrap();
    let params = Params {
        count: count as u32,
        width: width as u32,
        torso_count: 2,
        ..Default::default()
    };
    let mut batch = gpu.batch("ray-preserving outer support");
    dispatch(
        &gpu,
        &mut batch,
        fit_wgsl::MEASURE,
        &[("positions", false), ("body_local", false)],
        params,
        &[
            ("plate", &plate),
            ("positions", &positions),
            ("centers", &centers),
            ("torso_faces", &faces),
            ("body_local", &body),
            ("support_radii", &support),
            ("columns", &columns),
            ("status", &status),
        ],
        count as u32,
    )
    .unwrap();
    dispatch(
        &gpu,
        &mut batch,
        fit_wgsl::ENVELOPE,
        &[],
        params,
        &[("support_radii", &support), ("envelope", &envelope)],
        count as u32,
    )
    .unwrap();
    dispatch(
        &gpu,
        &mut batch,
        fit_wgsl::APPLY,
        &[("positions", true)],
        params,
        &[
            ("plate", &plate),
            ("positions", &positions),
            ("centers", &centers),
            ("envelope", &envelope),
            ("support_radii", &support),
            ("status", &status),
        ],
        count as u32,
    )
    .unwrap();
    batch.submit();
    assert_eq!(gpu.read::<u32>(&status).unwrap()[0], 0);
    for point in gpu.read::<[f32; 3]>(&positions).unwrap() {
        assert!(
            (point[0] - 0.02).abs() < 1e-6,
            "fitting changed the chart angle"
        );
        assert!(
            (point[2] - (0.30 + Millimeters(18).metres())).abs() < 1e-6,
            "fitting selected the entry surface or changed clearance"
        );
    }
}

#[test]
fn local_support_clears_its_body_ray_and_cannot_jump_from_skirt_to_neck() {
    let gpu = ArmorGpu::open().unwrap();
    let rows = V_SAMPLES + SKIRT_SAMPLES - 1;
    let width = 7;
    let count = rows * width;
    let spike = V_SAMPLES * width + width / 2;
    let mut input = vec![[0.0_f32, 1.0, 0.4]; count];
    input[spike] = [0.0, 1.1, 0.28];
    let positions = gpu.upload(BufferUpload::from_elements(&input)).unwrap();
    let centers = gpu
        .upload(BufferUpload::from_elements(&vec![
            [0.0_f32, 0.0, 1.0, 2.0];
            count
        ]))
        .unwrap();
    let mut words = shape_wgsl::design_words(&BreastplateDesign {
        skirt_flare: Millimeters(0),
        ..Default::default()
    });
    words[32] = 1.0;
    words[37] = 1.0;
    words[44] = 1.0;
    let plate = gpu.upload(BufferUpload::from_elements(&words)).unwrap();
    let body = gpu
        .upload(BufferUpload::from_elements(&[
            [-1.0_f32, 0.0, 0.25],
            [1.0, 0.0, 0.25],
            [0.0, 1.049, 0.25],
            [-1.0, 1.05, 0.3],
            [1.0, 1.05, 0.3],
            [0.0, 1.2, 0.3],
        ]))
        .unwrap();
    let faces = gpu
        .upload(BufferUpload::from_elements(&[[0_u32, 1, 2], [3, 4, 5]]))
        .unwrap();
    let support_radii = gpu
        .scratch(count as u64 * 4, "local support test support_radii")
        .unwrap();
    let envelope = gpu
        .scratch(count as u64 * 4, "local support test envelope")
        .unwrap();
    let status = gpu.upload(BufferUpload::from_elements(&[0_u32])).unwrap();
    let params = Params {
        count: count as u32,
        width: width as u32,
        torso_count: 2,
        ..Params::default()
    };
    let mut batch = gpu.batch("local support enclosure and seam");
    dispatch(
        &gpu,
        &mut batch,
        fit_wgsl::MEASURE,
        &[("positions", false), ("body_local", false)],
        params,
        &[
            ("plate", &plate),
            ("positions", &positions),
            ("centers", &centers),
            ("torso_faces", &faces),
            ("body_local", &body),
            ("support_radii", &support_radii),
            (
                "columns",
                &gpu.upload(BufferUpload::from_elements(
                    &(0..width)
                        .map(|c| -1.0_f32 + 2.0 * c as f32 / (width - 1) as f32)
                        .collect::<Vec<_>>(),
                ))
                .unwrap(),
            ),
            ("status", &status),
        ],
        count as u32,
    )
    .unwrap();
    dispatch(
        &gpu,
        &mut batch,
        fit_wgsl::ENVELOPE,
        &[],
        params,
        &[("support_radii", &support_radii), ("envelope", &envelope)],
        count as u32,
    )
    .unwrap();
    dispatch(
        &gpu,
        &mut batch,
        fit_wgsl::APPLY,
        &[("positions", true)],
        params,
        &[
            ("plate", &plate),
            ("positions", &positions),
            ("centers", &centers),
            ("envelope", &envelope),
            ("support_radii", &support_radii),
            (
                "columns",
                &gpu.upload(BufferUpload::from_elements(
                    &(0..width)
                        .map(|c| -1.0_f32 + 2.0 * c as f32 / (width - 1) as f32)
                        .collect::<Vec<_>>(),
                ))
                .unwrap(),
            ),
            ("status", &status),
        ],
        count as u32,
    )
    .unwrap();
    batch.submit();
    assert_eq!(gpu.read::<u32>(&status).unwrap()[0], 0);
    let required = gpu.read::<f32>(&support_radii).unwrap();
    let actual = gpu.read::<[f32; 3]>(&positions).unwrap();
    assert!(required[spike] > 0.3, "fixture did not need fitting");
    for (i, point) in actual.iter().enumerate() {
        assert!(
            point[2] >= required[i] - 1e-6,
            "local smoothing lost support at {i}"
        );
        assert_eq!(point[1], input[i][1], "fitting changed its measured height");
    }
    assert!(actual[spike][2] >= 0.31 - 1e-6, "body ray lacks clearance");
    assert!(
        actual[width / 2][2] > actual[V_SAMPLES * width][2],
        "waist seam is disconnected"
    );
    assert_eq!(
        actual[(V_SAMPLES - 1) * width + width / 2],
        actual[V_SAMPLES * width],
        "skirt support propagated to the distant neckline"
    );
    assert!(
        actual[V_SAMPLES * width][2] < input[V_SAMPLES * width][2],
        "an oversized carrier did not seat on its measured body"
    );
    assert!(
        actual.iter().all(|p| p[2] <= 0.311),
        "radius correction inflated already oversized neighbors"
    );
}

fn apply_support(
    gpu: &ArmorGpu,
    plate: &fabelgeist_gpu::prelude::Buffer,
    points: &[[f32; 3]],
    support: &[f32],
    envelope: &[f32],
    radial_fit: RadialFit,
    width: usize,
) -> (Vec<[f32; 3]>, u32) {
    let count = points.len();
    let positions = gpu.upload(BufferUpload::from_elements(points)).unwrap();
    let centers = gpu
        .upload(BufferUpload::from_elements(&vec![
            [0.0_f32, 0.0, 1.0, 2.0];
            count
        ]))
        .unwrap();
    let support = gpu.upload(BufferUpload::from_elements(support)).unwrap();
    let envelope = gpu.upload(BufferUpload::from_elements(envelope)).unwrap();
    let status = gpu.upload(BufferUpload::from_elements(&[0_u32])).unwrap();
    let mut batch = gpu.batch("final support application fixture");
    dispatch(
        gpu,
        &mut batch,
        fit_wgsl::APPLY,
        &[("positions", true)],
        Params {
            count: count as u32,
            width: width as u32,
            radial_fit,
            ..Default::default()
        },
        &[
            ("plate", plate),
            ("positions", &positions),
            ("centers", &centers),
            ("support_radii", &support),
            ("envelope", &envelope),
            ("status", &status),
        ],
        count as u32,
    )
    .unwrap();
    batch.submit();
    (
        gpu.read(&positions).unwrap(),
        gpu.read::<u32>(&status).unwrap()[0],
    )
}

fn fitted_frame(gpu: &ArmorGpu, design: &BreastplateDesign) -> fabelgeist_gpu::prelude::Buffer {
    let mut words = shape_wgsl::design_words(design);
    words[32] = 1.0;
    words[37] = 1.0;
    words[39] = 1.4419107;
    words[41] = 1.0;
    words[43] = 1.0;
    words[44] = 1.0;
    gpu.upload(BufferUpload::from_elements(&words)).unwrap()
}

#[test]
fn rear_enclosure_preserves_the_actual_preceding_lap() {
    let gpu = ArmorGpu::open().unwrap();
    let plate = fitted_frame(&gpu, &BreastplateDesign::default());
    let rows = V_SAMPLES + SKIRT_SAMPLES - 1;
    let width = 49;
    let front = (0..rows * width)
        .map(|i| [-0.15 + 0.3 * (i % width) as f32 / 48.0, 1.1, 0.12])
        .collect::<Vec<_>>();
    let back = (0..rows * width)
        .map(|i| {
            let x = -0.1 + 0.2 * (i % width) as f32 / 48.0;
            [x, 1.1, -(0.04 - x * x).sqrt()]
        })
        .collect::<Vec<_>>();
    let front = gpu.upload(BufferUpload::from_elements(&front)).unwrap();
    let back = gpu.upload(BufferUpload::from_elements(&back)).unwrap();
    let mut batch = gpu.batch("rear side lap followed by enclosure");
    for side in 0..2 {
        dispatch(
            &gpu,
            &mut batch,
            fit_wgsl::LAP,
            &[("positions", true)],
            Params {
                width: width as u32,
                front_count: width as u32,
                side,
                ..Default::default()
            },
            &[("plate", &plate), ("front", &front), ("positions", &back)],
            rows as u32,
        )
        .unwrap();
    }
    batch.submit();
    let lapped = gpu.read::<[f32; 3]>(&back).unwrap();
    assert!(lapped[width - 1][0] >= 0.157 - 1e-6);
    let support = vec![0.2; lapped.len()];
    let (actual, status) = apply_support(
        &gpu,
        &plate,
        &lapped,
        &support,
        &support,
        RadialFit::Enclose,
        width,
    );
    assert_eq!(status, 0);
    for (before, after) in lapped.iter().zip(&actual) {
        assert!(
            (before[0] - after[0]).abs() < 1e-6,
            "enclosure retracted a seated lap"
        );
    }
}

#[test]
fn final_smoothing_cannot_silently_exceed_the_outward_fit_bound() {
    let gpu = ArmorGpu::open().unwrap();
    let plate = fitted_frame(&gpu, &BreastplateDesign::default());
    let count = (V_SAMPLES + SKIRT_SAMPLES - 1) * 7;
    let points = vec![[0.0, 1.1, 0.1]; count];
    let support = vec![0.1; count];
    let envelope = vec![0.2; count];
    let (_, status) = apply_support(
        &gpu,
        &plate,
        &points,
        &support,
        &envelope,
        RadialFit::Seat,
        7,
    );
    assert_ne!(
        status, 0,
        "post-smoothing correction bypassed the 60 mm bound"
    );
}

#[test]
fn body_seating_retains_authored_skirt_flare_without_moving_the_waist() {
    let gpu = ArmorGpu::open().unwrap();
    let rows = V_SAMPLES + SKIRT_SAMPLES - 1;
    let width = 7;
    let count = rows * width;
    let points = gpu
        .upload(BufferUpload::from_elements(&vec![
            [0.0_f32, 1.1, -0.4];
            count
        ]))
        .unwrap();
    let centers = gpu
        .upload(BufferUpload::from_elements(&vec![
            [0.0_f32, 0.0, 1.0, 2.0];
            count
        ]))
        .unwrap();
    let body = gpu
        .upload(BufferUpload::from_elements(&[
            [-1.0_f32, 0.0, -0.25],
            [1.0, 0.0, -0.25],
            [0.0, 2.0, -0.25],
        ]))
        .unwrap();
    let faces = gpu
        .upload(BufferUpload::from_elements(&[[0_u32, 1, 2]]))
        .unwrap();
    let mut cases = Vec::new();
    for flare in [0, 20, 40] {
        let plate = fitted_frame(
            &gpu,
            &BreastplateDesign {
                skirt_flare: Millimeters(flare),
                ..Default::default()
            },
        );
        let support = gpu
            .scratch(count as u64 * 4, "skirt flare target radii")
            .unwrap();
        let status = gpu.upload(BufferUpload::from_elements(&[0_u32])).unwrap();
        let mut batch = gpu.batch("body-supported skirt flare");
        dispatch(
            &gpu,
            &mut batch,
            fit_wgsl::MEASURE,
            &[("positions", false), ("body_local", false)],
            Params {
                count: count as u32,
                width: width as u32,
                rear: true,
                torso_count: 1,
                ..Default::default()
            },
            &[
                ("plate", &plate),
                ("positions", &points),
                ("centers", &centers),
                ("body_local", &body),
                ("torso_faces", &faces),
                ("support_radii", &support),
                (
                    "columns",
                    &gpu.upload(BufferUpload::from_elements(
                        &(0..width)
                            .map(|c| -1.0_f32 + 2.0 * c as f32 / (width - 1) as f32)
                            .collect::<Vec<_>>(),
                    ))
                    .unwrap(),
                ),
                ("status", &status),
            ],
            count as u32,
        )
        .unwrap();
        batch.submit();
        assert_eq!(gpu.read::<u32>(&status).unwrap()[0], 0);
        let support = gpu.read::<f32>(&support).unwrap();
        let measured = gpu.upload(BufferUpload::from_elements(&support)).unwrap();
        let envelope = gpu
            .scratch(count as u64 * 4, "measured skirt envelope")
            .unwrap();
        let mut smoothing = gpu.batch("actual skirt support dilation");
        dispatch(
            &gpu,
            &mut smoothing,
            fit_wgsl::ENVELOPE,
            &[],
            Params {
                count: count as u32,
                width: width as u32,
                ..Default::default()
            },
            &[("support_radii", &measured), ("envelope", &envelope)],
            count as u32,
        )
        .unwrap();
        smoothing.submit();
        let envelope = gpu.read::<f32>(&envelope).unwrap();
        let input = vec![[0.0_f32, 1.1, -0.4]; count];
        let (actual, status) = apply_support(
            &gpu,
            &plate,
            &input,
            &support,
            &envelope,
            RadialFit::Seat,
            width,
        );
        assert_eq!(status, 0);
        assert!(actual.iter().zip(&support).all(|(p, s)| -p[2] >= *s - 1e-6));
        cases.push(actual.iter().map(|p| -p[2]).collect::<Vec<_>>());
    }
    assert!(
        cases[2][width / 2] - cases[0][width / 2] < 0.006,
        "local skirt smoothing excessively moved the waist"
    );
    let hem = count - width / 2 - 1;
    assert!(
        cases[0][hem] < cases[1][hem] && cases[1][hem] < cases[2][hem],
        "body seating erased skirt flare"
    );
    assert!(cases[2][hem] - cases[0][hem] > 0.035);
}
