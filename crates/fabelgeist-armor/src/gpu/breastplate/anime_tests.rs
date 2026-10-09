//! Exercise the actual deep-lap kernels, including clipped hem correspondence.
use super::super::anime_sampling::design_words;
use super::super::{
    shape_wgsl,
    topology::{SKIRT_SAMPLES, V_SAMPLES},
};
use super::*;
use crate::{BreastplateDesign, Millimeters, Permille};
use fabelgeist_gpu::prelude::BufferUpload;

#[test]
fn deep_laps_keep_noncollapsed_rows_and_distinct_metal_surfaces_at_the_hem() {
    let gpu = ArmorGpu::open().unwrap();
    let rows = V_SAMPLES + SKIRT_SAMPLES - 1;
    let width = 3;
    let mut plate_words = shape_wgsl::design_words(&BreastplateDesign::default());
    plate_words[32] = 1.0;
    plate_words[37] = 1.0;
    let plate = gpu
        .upload(BufferUpload::from_elements(&plate_words))
        .unwrap();
    for (lift, valid, half_width) in [(12, true, 0.06), (8, false, 0.06), (12, true, 0.15)] {
        let mut original = Vec::new();
        for side in 0..2 {
            for row in 0..rows {
                let y = if row < V_SAMPLES {
                    1.06 + 0.17 * row as f32 / (V_SAMPLES - 1) as f32
                } else {
                    1.06 - 0.04 * (row - V_SAMPLES + 1) as f32 / (SKIRT_SAMPLES - 1) as f32
                };
                for column in 0..width {
                    let x = (column as f32 - 1.0) * half_width;
                    let z = if side == 0 { 0.12 } else { -0.12 };
                    original.extend([
                        [x, y, z],
                        [x, y, z + if side == 0 { 0.004 } else { -0.004 }],
                    ]);
                }
            }
        }
        let references = (0..original.len() as u32 / 2)
            .map(|i| [i * 2, i * 2 + 1])
            .collect::<Vec<_>>();
        let original = gpu.upload(BufferUpload::from_elements(&original)).unwrap();
        let references = gpu
            .upload(BufferUpload::from_elements(&references))
            .unwrap();
        let columns = gpu
            .upload(BufferUpload::from_elements(&[
                -1.0_f32, 0.0, 1.0, -1.0, 0.0, 1.0,
            ]))
            .unwrap();

        let design = AnimeDesign {
            overlap: Millimeters(40),
            lap_lift: Millimeters(lift),
            chevron_slope: Permille(0),
            rear_chevron_slope: Permille(0),
            ..Default::default()
        };
        let design = gpu
            .upload(BufferUpload::from_elements(&design_words(&design)))
            .unwrap();
        let status = gpu.upload(BufferUpload::from_elements(&[0_u32])).unwrap();
        let bounds = gpu.scratch(32, "test course bounds").unwrap();
        let coordinates = (0..=6)
            .flat_map(|course| {
                let low = (1.02_f32 + 0.028 * course as f32 - if course == 0 { 0.0 } else { 0.04 })
                    .max(1.02);
                let high = if course == 6 {
                    1.23
                } else {
                    1.02 + 0.028 * (course + 1) as f32
                };
                (0..3).map(move |row| {
                    let q = low + (high - low) * row as f32 * 0.5;
                    [
                        ((rows - 1) * width + 1) as u32,
                        ((V_SAMPLES - 1) * width + 1) as u32,
                        ((q - 1.02) / 0.21).to_bits(),
                        course,
                    ]
                })
            })
            .collect::<Vec<_>>();
        let count = coordinates.len() as u32;
        let positions = gpu
            .scratch(count as u64 * 12, "test deep lap points")
            .unwrap();
        let links = gpu
            .scratch(count as u64 * 12, "test deep lap correspondence")
            .unwrap();
        let params = Params {
            count,
            width: width as u32,
            extra: width as u32,
            front_count: (rows * width) as u32,
            ..Default::default()
        };
        let mut batch = gpu.batch("deep lap geometry");
        dispatch(
            &gpu,
            &mut batch,
            include_str!("anime_bounds.wgsl"),
            &[("original", false)],
            params,
            &[
                ("plate", &plate),
                ("original", &original),
                ("references", &references),
                ("design", &design),
                ("bounds", &bounds),
                ("status", &status),
                ("columns", &columns),
            ],
            2,
        )
        .unwrap();
        if valid {
            dispatch(
                &gpu,
                &mut batch,
                include_str!("anime_resample.wgsl"),
                &[("original", false), ("positions", true)],
                params,
                &[
                    ("plate", &plate),
                    ("original", &original),
                    ("references", &references),
                    ("design", &design),
                    ("bounds", &bounds),
                    (
                        "coordinates",
                        &gpu.upload(BufferUpload::from_elements(&coordinates))
                            .unwrap(),
                    ),
                    ("positions", &positions),
                    ("links", &links),
                ],
                count,
            )
            .unwrap();
        }
        batch.submit();
        assert_eq!(
            gpu.read::<u32>(&status).unwrap()[0] == 0,
            valid,
            "metal separation guard did not match the generated courses"
        );
        if valid {
            let points = gpu.read::<[f32; 3]>(&positions).unwrap();
            assert!(points.iter().flatten().all(|v| v.is_finite()));
            for course in points.as_chunks::<3>().0 {
                assert!(course[0][1] >= 1.02 - 1e-6);
                assert!(course[0][1] < course[1][1] && course[1][1] < course[2][1]);
            }
            // Course one extends beyond the first exposed pitch. Its trimmed
            // lower edge has its original ramp, not the same lift as course two.
            assert!((points[3][1] - 1.02).abs() < 1e-6);
            assert!(points[3][2] > points[0][2] + 0.004);
            let correspondence = gpu.read::<u32>(&links).unwrap();
            assert!(correspondence.as_chunks::<3>().0.iter().all(|link| {
                link[..2]
                    .iter()
                    .all(|&i| i < rows as u32 * width as u32 * 4)
            }));
        }
    }
}
