use super::kernels::{Params, dispatch};
use super::shape_wgsl::design_words;
use crate::{BreastplateDesign, Permille, gpu::ArmorGpu};
use fabelgeist_gpu::prelude::BufferUpload;

#[test]
fn armscye_width_moves_its_boundary_without_moving_the_neck_or_waist() {
    let gpu = ArmorGpu::open().unwrap();
    for rear in [false, true] {
        let mut cases = Vec::new();
        for width in [100, 500, 750, 1000] {
            let design = BreastplateDesign {
                arm_opening_width: Permille(width),
                ..Default::default()
            };
            let plate = gpu
                .upload(BufferUpload::from_elements(&design_words(&design)))
                .unwrap();
            let points = gpu.scratch(5 * 12, "armscye boundary samples").unwrap();
            let mut batch = gpu.batch("independent armscye width");
            dispatch(
                &gpu,
                &mut batch,
                SAMPLE,
                &[("points", true)],
                Params {
                    rear,
                    ..Default::default()
                },
                &[("plate", &plate), ("points", &points)],
                1,
            )
            .unwrap();
            batch.submit();
            cases.push(gpu.read::<[f32; 3]>(&points).unwrap());
        }
        for narrowed in &cases[..3] {
            assert_eq!(narrowed[0], cases[3][0], "inner neckline moved");
            assert_eq!(narrowed[1], cases[3][1], "neck/armscye junction moved");
            assert_eq!(narrowed[3], cases[3][3], "waist side return moved");
        }
        assert!(cases[0][2][0] < cases[1][2][0]);
        assert!(cases[1][2][0] < cases[2][2][0]);
        assert!(cases[2][2][0] < cases[3][2][0]);
        assert!(cases[0][4][0] < cases[1][4][0]);
        assert!(cases[1][4][0] < cases[2][4][0]);
        assert!(cases[2][4][0] < cases[3][4][0]);
        assert_eq!(
            cases[0][2][1], cases[3][2][1],
            "opening depth was coupled to width"
        );
    }
}

#[test]
fn armscye_trim_preserves_the_retained_carrier_sections() {
    let gpu = ArmorGpu::open().unwrap();
    for rear in [false, true] {
        let mut cases = Vec::new();
        for width in [100, 500, 1000] {
            let design = BreastplateDesign {
                arm_opening_width: Permille(width),
                ..Default::default()
            };
            let plate = gpu
                .upload(BufferUpload::from_elements(&design_words(&design)))
                .unwrap();
            let points = gpu.scratch(3 * 12, "retained carrier sections").unwrap();
            let mut batch = gpu.batch("armscye only trims the carrier");
            dispatch(
                &gpu,
                &mut batch,
                RETAINED,
                &[("points", true)],
                Params {
                    rear,
                    ..Default::default()
                },
                &[("plate", &plate), ("points", &points)],
                1,
            )
            .unwrap();
            batch.submit();
            cases.push(gpu.read::<[f32; 3]>(&points).unwrap());
        }
        assert_eq!(cases[0], cases[2]);
        assert_eq!(cases[1], cases[2]);
    }
}

const RETAINED: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read_write> points: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;
@compute @workgroup_size(1)
fn main() {
    let rear = params.rear != 0u;
    let y = neckline_y(rear, 1.0);
    let columns = array<f32, 3>(0.25, 0.5, 0.55);
    for (var i = 0u; i < 3u; i += 1u) {
        let u = columns[i];
        points_set(i, vec3<f32>(chart_theta(rear, u, y), y, 0.0));
    }
}
"#;

#[test]
fn flank_return_preserves_the_neckline_at_coupled_neck_depths() {
    let gpu = ArmorGpu::open().unwrap();
    for rear in [false, true] {
        for neck_depth in [600, 1000, 1400] {
            let mut cases = Vec::new();
            for side_return in [500, 750, 1000] {
                let design = BreastplateDesign {
                    neck_depth: Permille(neck_depth),
                    side_return: Permille(side_return),
                    ..Default::default()
                };
                let plate = gpu
                    .upload(BufferUpload::from_elements(&design_words(&design)))
                    .unwrap();
                let points = gpu.scratch(5 * 12, "flank boundary samples").unwrap();
                let mut batch = gpu.batch("independent flank return");
                dispatch(
                    &gpu,
                    &mut batch,
                    SAMPLE,
                    &[("points", true)],
                    Params {
                        rear,
                        ..Default::default()
                    },
                    &[("plate", &plate), ("points", &points)],
                    1,
                )
                .unwrap();
                batch.submit();
                cases.push(gpu.read::<[f32; 3]>(&points).unwrap());
            }
            for narrowed in &cases[..2] {
                assert_eq!(narrowed[..2], cases[2][..2], "neckline moved");
            }
            assert!(cases[0][3][0] < cases[1][3][0]);
            assert!(cases[1][3][0] < cases[2][3][0]);
        }
    }
}

const SAMPLE: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read_write> points: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;
@compute @workgroup_size(1)
fn main() {
    let rear = params.rear != 0u;
    let columns = array<f32, 3>(0.25, 0.5, 1.0);
    for (var i = 0u; i < 3u; i += 1u) {
        let y = neckline_y(rear, columns[i]);
        let u = select(columns[i], arm_opening_column(rear, y), i == 2u);
        points_set(i, vec3<f32>(chart_theta(rear, u, y), y, 0.0));
    }
    let y = bottom_height(rear);
    points_set(3u, vec3<f32>(chart_theta(rear, 1.0, y), y, 0.0));
    let mid_y = mix(y, neckline_y(rear, 1.0), 0.5);
    points_set(4u, vec3<f32>(chart_theta(rear, arm_opening_column(rear, mid_y), mid_y), mid_y, 0.0));
}
"#;

#[test]
fn coupled_flank_and_armscye_controls_cannot_reverse_chart_columns() {
    let gpu = ArmorGpu::open().unwrap();
    for (side_return, width, depth, neck, neck_width) in [
        (500, 500, 1300, 1000, 1000),
        (500, 750, 600, 1400, 1000),
        (500, 1000, 1400, 600, 1000),
        (750, 500, 1000, 600, 1000),
        (750, 750, 1300, 1400, 1000),
        (750, 1000, 600, 1000, 1000),
        (1080, 500, 1400, 1400, 1000),
        (1080, 750, 1000, 1000, 1000),
        (1080, 1000, 600, 600, 1000),
        (500, 500, 700, 1400, 1300),
        (1000, 100, 1100, 1000, 1000),
        (1080, 100, 1400, 1400, 1300),
    ] {
        let design = BreastplateDesign {
            side_return: Permille(side_return),
            arm_opening_width: Permille(width),
            arm_opening_depth: Permille(depth),
            neck_depth: Permille(neck),
            neck_width: Permille(neck_width),
            ..Default::default()
        };
        for rear in [false, true] {
            let plate = gpu
                .upload(BufferUpload::from_elements(&design_words(&design)))
                .unwrap();
            let points = gpu.scratch(49 * 33 * 12, "coupled chart ordering").unwrap();
            let mut batch = gpu.batch("chart columns retain their orientation");
            dispatch(
                &gpu,
                &mut batch,
                ORDERING,
                &[("points", true)],
                Params {
                    rear,
                    ..Default::default()
                },
                &[("plate", &plate), ("points", &points)],
                1,
            )
            .unwrap();
            batch.submit();
            let points = gpu.read::<[f32; 3]>(&points).unwrap();
            for (row, strip) in points.as_chunks::<49>().0.iter().enumerate() {
                for (column, pair) in strip.windows(2).enumerate() {
                    if pair[0][1] <= pair[0][2].min(pair[1][2]) {
                        assert!(
                            pair[1][0] > pair[0][0],
                            "reversed or collapsed chart side={side_return} width={width} depth={depth} neck={neck} rear={rear} row={row} column={column}: {pair:?}"
                        );
                    }
                }
            }
        }
    }
}

const ORDERING: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read_write> points: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;
@compute @workgroup_size(1)
fn main() {
    let rear = params.rear != 0u;
    for (var row = 0u; row < 33u; row += 1u) {
        let y = mix(bottom_height(rear), REFERENCE_CARRIER_TOP_HEIGHT, f32(row) / 32.0);
        for (var column = 0u; column < 49u; column += 1u) {
            let u = f32(column) / 48.0;
            points_set(row * 49u + column, vec3<f32>(chart_theta(rear, u, y), y, neckline_y(rear, u)));
        }
    }
}
"#;
