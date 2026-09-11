//! Align lateral lap width without warping the back shell in depth.
use super::*;

const LAP_CLEARANCE: f32 = 0.003;
const LAP_FULL_HEIGHT: f32 = 1.17;
const LAP_FADE_HEIGHT: f32 = 0.10;
const LAP_START_U: f32 = 0.50;

fn edge_at_height(points: &[[f32; 3]], height: f32) -> [f32; 3] {
    for pair in points.windows(2) {
        if pair[0][1] <= height && height <= pair[1][1] {
            let t = (height - pair[0][1]) / (pair[1][1] - pair[0][1]).max(1e-6);
            return add(scale(pair[0], 1.0 - t), scale(pair[1], t));
        }
    }
    if height < points[0][1] {
        points[0]
    } else {
        *points.last().unwrap()
    }
}

pub(super) fn align_back_lap_width(
    back: &mut MidMesh,
    front: &MidMesh,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) {
    let columns = back.main_columns;
    for (side, column) in [(-1.0, 0), (1.0, columns - 1)] {
        let mut front_edge: Vec<_> = front
            .positions
            .chunks_exact(columns)
            .map(|row| local(row[column], wearer.frame))
            .collect();
        front_edge.sort_by(|a, b| a[1].total_cmp(&b[1]));
        for row in back.positions.chunks_exact_mut(columns) {
            let original = row[column];
            let original_local = local(original, wearer.frame);
            let reference_y = reference_height(original, wearer, design);
            let height_blend = 1.0 - smoothstep((reference_y - LAP_FULL_HEIGHT) / LAP_FADE_HEIGHT);
            if height_blend == 0.0 {
                continue;
            }
            let front_point = edge_at_height(&front_edge, original_local[1]);
            // The back's sagittal contour already defines its arm cutaway.
            // Closing that opening by translating it to the front edge bends
            // a deep crease into the plate. Only align the fastening width.
            let target_x = front_point[0] + side * (design.wall_thickness.metres() + LAP_CLEARANCE);
            let offset = world([target_x - original_local[0], 0.0, 0.0], wearer.frame);
            for (index, point) in row.iter_mut().enumerate() {
                let u = -1.0 + 2.0 * index as f32 / (columns - 1) as f32;
                let lateral_blend = smoothstep((u * side - LAP_START_U) / (1.0 - LAP_START_U));
                *point = add(*point, scale(offset, height_blend * lateral_blend));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lap_alignment_preserves_depth_and_height_in_the_wearer_frame() {
        let frame = Frame {
            lateral: [0.0, 0.0, 1.0],
            vertical: [0.0, 1.0, 0.0],
            front: [-1.0, 0.0, 0.0],
        };
        let clearance = TorsoClearancePose {
            vertices: vec![],
            enclosure_vertices: vec![],
        };
        let wearer = Wearer {
            frame,
            anchors: TorsoUpperRigAnchors {
                neck_base: [0.0, REFERENCE_RIG_NECK_HEIGHT, 0.0],
                clavicles: [[0.0; 3]; 2],
                shoulders: [[0.0; 3]; 2],
            },
            clearance: &clearance,
            source_faces: &[],
            torso_faces: &[],
            x_scale: 1.0,
            shoulder_x_scale: 1.0,
            y_scale: 1.0,
            z_scale: 1.0,
            lateral_origin: 0.0,
            coronal_origin: 0.0,
        };
        let build = |radius: f32, depth: f32| MidMesh {
            main_columns: 5,
            positions: [1.10, 1.20, 1.30]
                .into_iter()
                .flat_map(|height| {
                    (-2..=2).map(move |column| {
                        world([column as f32 * radius / 2.0, height, depth], frame)
                    })
                })
                .collect(),
            ..MidMesh::default()
        };
        let front = build(0.20, 0.02);
        let mut back = build(0.22, -0.10);
        let original = back.positions.clone();
        align_back_lap_width(&mut back, &front, wearer, &BreastplateDesign::default());
        assert!((local(back.positions[0], frame)[0] - local(original[0], frame)[0]).abs() > 0.01);
        for (before, after) in original.iter().zip(&back.positions) {
            let before = local(*before, frame);
            let after = local(*after, frame);
            assert!((before[1] - after[1]).abs() < 1e-6);
            assert!(
                (before[2] - after[2]).abs() < 1e-6,
                "lap alignment warped the back in depth"
            );
        }
        assert_eq!(
            back.positions[10..],
            original[10..],
            "upper opening must remain unchanged"
        );
    }
}
