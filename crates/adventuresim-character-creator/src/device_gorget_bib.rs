//! The gorget bib's directional seating cage on the device: in each direction
//! around the neck, the bib moves along a line that lifts over the trapezius
//! at the sides and turns into depth at the front and back, until it clears
//! the highest body or completed lower-equipment surface under it.

use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_gpu::prelude::BufferUpload;

use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_gorget_cage::{BIB_COLUMNS, BIB_ROWS, CAGE, layout};
use crate::device_gorget_projection::PROJECTION;

impl DeviceWearer<'_> {
    /// Seat the bib on the union of the body and completed lower equipment.
    /// Lower surfaces are in the same canonical unposed frame as the body;
    /// neck/collar measurements continue to use body anatomy alone.
    pub(crate) fn record_bib_fit(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        samples: &Buffer,
        padding: f32,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<()> {
        self.record_bib_measure(batch, fit, samples, padding, layers)?;
        dispatch(
            self,
            batch,
            &format!("{}{ENVELOPE}", layout()),
            &[write("fit", fit)],
            &[],
            Grid::Items(BIB_ROWS * BIB_COLUMNS),
        )
    }

    /// Measure the final bib rays without changing their fitted geometry.
    pub(crate) fn record_bib_measure(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        samples: &Buffer,
        padding: f32,
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<()> {
        let triangles = layers
            .iter()
            .flat_map(|layer| {
                layer
                    .faces
                    .iter()
                    .map(|face| face.map(|i| layer.positions[i as usize]))
            })
            .collect::<Vec<_>>();
        let layer_faces = triangles.len() as u32;
        let triangles = self.gpu.upload(BufferUpload::from_elements(&triangles))?;
        let cage = format!("fn cage_word(i: u32) -> f32 {{\n    return fit[i];\n}}\n{CAGE}");
        dispatch(
            self,
            batch,
            &format!("{}{cage}{PROJECTION}{MEASURE}", layout()),
            &[
                read_u32("faces", &self.body.faces),
                read("points", samples),
                read("extra", &triangles),
                write("fit", fit),
            ],
            &[
                Word::U("faces_count", self.body.face_count),
                Word::U("layer_faces", layer_faces),
                Word::F("padding", padding),
            ],
            Grid::Items(BIB_ROWS * BIB_COLUMNS),
        )
    }
}

/// One invocation per measured sample, including the collar seam: the deepest
/// support crossing of the sample's ray, and the offset that clears it.
pub(crate) const MEASURE: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= BIB_ROWS * BIB_COLUMNS) {
        return;
    }
    let row = index / BIB_COLUMNS;
    let column = index % BIB_COLUMNS;
    let t = f32(row) / f32(BIB_ROWS - 1u);
    let angle = TAU * f32(column) / f32(BIB_COLUMNS - 1u);
    let control = control_angle(angle);
    let p = bib_point(t, control);
    let direction = bib_direction(angle);
    let query = vec2<f32>(p.x, p.y * direction.z - p.z * direction.y);
    let upper_height = bib_point(0.0, control).y;
    let origin = p.y * direction.y + p.z * direction.z;
    var deepest = -INFINITY;
    var hit = false;
    for (var f = 0u; f < params.faces_count + params.layer_faces; f = f + 1u) {
        var corners: array<vec3<f32>, 3>;
        for (var c = 0u; c < 3u; c += 1u) {
            if (f < params.faces_count) {
                corners[c] = points_at(faces[f * 3u + c]);
            } else {
                corners[c] = host_local(frame_at(0u), extra_at((f - params.faces_count) * 3u + c));
            }
        }
        let crossing = ray_depth(
            query,
            rotated(corners[0], direction),
            rotated(corners[1], direction),
            rotated(corners[2], direction),
        );
        if (crossing.y == 0.0) {
            continue;
        }
        // The head above the collar is not shoulder.
        let hit_height = query.y * direction.z + crossing.x * direction.y;
        if (f >= params.faces_count || hit_height <= upper_height + SECTION_HEIGHT_TOLERANCE_M) {
            deepest = max(deepest, crossing.x);
            hit = true;
        }
    }
    // Corrections accumulate outward only. Subsequent collar seating can
    // move this ray, but must not discard an already measured enclosure.
    let offset = select(0.0, max(deepest + params.padding - origin, 0.0), hit);
    fit[BIB_RAW + index] = offset;
}
"#;

/// Expand support across the adjacent chart cells before interpolation. A
/// plate edge spanning a support boundary must clear both ends of that cell;
/// averaging its offsets can cut back through the lower plate's top edge.
/// Each pass spreads only new deficits, then adds them to its existing support;
/// previously fitted offsets must not diffuse again across neighbouring cells.
/// This sampled envelope is periodic around the neck, not a continuous proof.
pub(crate) const ENVELOPE: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= BIB_ROWS * BIB_COLUMNS) {
        return;
    }
    let row = i32(index / BIB_COLUMNS);
    let column = i32(index % BIB_COLUMNS);
    var value = -INFINITY;
    {
        let period = i32(BIB_COLUMNS) - 1;
        for (var dr = 0; dr < 3; dr = dr + 1) {
            let r = clamp(row + dr - 1, 0, i32(BIB_ROWS) - 1);
            for (var dc = 0; dc < 3; dc = dc + 1) {
                let c = ((column + dc - 1) % period + period) % period;
                value = max(value, fit[BIB_RAW + u32(r) * BIB_COLUMNS + u32(c)]);
            }
        }
    }
    fit[BIB + index] = fit[BIB + index] + value;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armor_frames::Wearer;
    use crate::device_gorget_cage::*;
    use fabelgeist_armor::ArmorGpu;
    use fabelgeist_compute::KernelBatchLabel;

    #[test]
    fn bib_seats_on_lower_equipment_in_the_canonical_wearer_frame() {
        let gpu = ArmorGpu::open().unwrap();
        for (translated, support_top) in [(false, 0.1), (true, 0.1), (false, 0.056), (true, 0.056)]
        {
            let world = |[x, y, z]: [f32; 3]| {
                if translated {
                    [z + 2.0, y + 3.0, -x - 4.0]
                } else {
                    [x, y, z]
                }
            };
            let body_points = [
                [-0.3, -0.15, 0.05],
                [0.3, -0.15, 0.05],
                [-0.3, 0.2, 0.05],
                [0.3, 0.2, 0.05],
            ];
            let positions = body_points.map(world);
            let faces = [[0, 1, 2], [1, 3, 2]];
            let normals = [[0.0, 0.0, 1.0]; 4];
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
            // Lower equipment reaches the collar seam but does not cover the
            // anatomical top opening. Its old pinned seam would intersect it.
            let lower = body_points.map(|[x, y, _]| world([x, y.min(support_top), 0.14]));
            let layer = ArmorLayerSurface {
                positions: &lower,
                faces: &faces,
                joint_indices: &indices,
                joint_weights: &weights,
            };
            let mut fitted = Vec::new();
            for layers in [&[][..], &[layer][..]] {
                let mut words = vec![0.0_f32; GORGET_FIT_WORDS as usize];
                words[3] = 1.0;
                words[7] = 1.0;
                words[11] = 1.0;
                if translated {
                    words[..12].copy_from_slice(&[
                        2.0, 3.0, -4.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0,
                    ]);
                }
                words[COLLAR_RADIUS as usize] = 0.03;
                words[COLLAR_RADIUS as usize + 1] = 0.03;
                words[BASE_RADIUS as usize] = 0.04;
                words[OUTER_WIDTH as usize..OUTER_WIDTH as usize + 2].fill(0.2);
                words[PLANES as usize] = 0.2;
                words[PLANES as usize + 2] = 0.1;
                words[HEM as usize..HEM as usize + 3].fill(-0.1);
                words[FRONT as usize..FRONT as usize + 3].fill(0.09);
                words[BACK as usize..BACK as usize + 3].fill(0.1);
                let fit = gpu.upload(BufferUpload::from_elements(&words)).unwrap();
                let samples = gpu
                    .upload(BufferUpload::from_elements(&body_points))
                    .unwrap();
                let output = gpu
                    .scratch(36 * 12, "seated collar and bib samples")
                    .unwrap();
                let mut batch = gpu.batch(KernelBatchLabel::from("gorget lower support test"));
                wearer
                    .record_bib_fit(&mut batch, &fit, &samples, 0.004, layers)
                    .unwrap();
                dispatch(
                    &wearer,
                    &mut batch,
                    &format!(
                        "{}fn cage_word(i: u32) -> f32 {{ return fit[i]; }}\n{CAGE}\n{EVALUATE}",
                        layout()
                    ),
                    &[read("fit", &fit), write("points", &output)],
                    &[],
                    Grid::Singles(1),
                )
                .unwrap();
                batch.submit();
                fitted.push(gpu.read::<[f32; 3]>(&output).unwrap());
            }
            assert!(
                fitted[1][..33]
                    .iter()
                    .all(|p| p[1] > support_top || p[2] >= 0.144 - 1e-5),
                "{fitted:?}"
            );
            // Test actual straight edges between the generated row samples at
            // the support's top boundary, not only points on the cubic cage.
            let mesh_rows = fitted[1][..33]
                .iter()
                .step_by(2)
                .copied()
                .collect::<Vec<_>>();
            for edge in mesh_rows.windows(2) {
                let [a, b] = [edge[0], edge[1]];
                if a[1] > support_top && b[1] <= support_top {
                    let t = (support_top - a[1]) / (b[1] - a[1]);
                    let depth = a[2] + t * (b[2] - a[2]);
                    assert!(depth >= 0.144 - 1e-5, "edge cuts the support: {a:?} {b:?}");
                }
            }
            assert!(
                fitted[0][..33].iter().all(|p| p[2] >= 0.09 - 1e-6),
                "support fitting contracted the authored carrier"
            );
            assert!(fitted[1][16][2] > fitted[0][16][2] + 0.05);
            assert_eq!(
                fitted[0][33], fitted[1][33],
                "lower equipment moved the top opening"
            );
            for (bib, collar) in fitted[1][0].iter().zip(&fitted[1][35]) {
                assert!(
                    (bib - collar).abs() < 1e-6,
                    "collar and bib separated at their shared seam"
                );
            }
            assert!(fitted.iter().flatten().flatten().all(|v| v.is_finite()));
        }
    }

    const EVALUATE: &str = r#"
@compute @workgroup_size(1)
fn main() {
    for (var i = 0u; i <= 32u; i += 1u) {
        points_set(i, gorget_bib(f32(i) / 32.0, 0.0));
    }
    points_set(33u, gorget_collar(0.0, 0.0));
    points_set(34u, gorget_collar(0.5, 0.0));
    points_set(35u, gorget_collar(1.0, 0.0));
}
"#;
}
