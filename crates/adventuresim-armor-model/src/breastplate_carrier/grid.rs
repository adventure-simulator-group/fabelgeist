//! Build connected carrier and skirt grids.

use super::*;

impl MidMesh {
    /// Mirror the right cut's diagonal at the left cut. A rising armscye can
    /// make the boundary quad concave in its facing projection; the opposite
    /// diagonal then leaves the cut and overlaps its wall during deformation.
    /// This material-column rule is identical for every body and morph.
    pub(super) fn triangulate_left_cut(&mut self) {
        let end = self.skirt_face_start.unwrap_or(self.faces.len());
        for pair in self.faces[..end].as_chunks_mut::<2>().0 {
            let [a, b, c] = pair[0];
            if (a as usize).is_multiple_of(self.main_columns) {
                let d = pair[1][2];
                pair[0] = [a, b, d];
                pair[1] = [b, c, d];
            }
        }
    }
}

pub(super) fn append_grid(mesh: &mut MidMesh, grid: &[Vec<[f32; 3]>], rear: bool) -> Vec<Vec<u32>> {
    let mut ids = Vec::with_capacity(grid.len());
    for row in grid {
        let mut id_row = Vec::with_capacity(row.len());
        for point in row {
            id_row.push(mesh.positions.len() as u32);
            mesh.positions.push(*point);
        }
        ids.push(id_row);
    }
    append_grid_faces(mesh, &ids, rear);
    ids
}

pub(super) fn append_grid_faces(mesh: &mut MidMesh, ids: &[Vec<u32>], rear: bool) {
    for rows in ids.windows(2) {
        for column in 0..rows[0].len() - 1 {
            let (a, b, d, c) = (
                rows[0][column],
                rows[0][column + 1],
                rows[1][column],
                rows[1][column + 1],
            );
            if rear {
                mesh.faces.extend([[a, c, b], [a, d, c]]);
            } else {
                mesh.faces.extend([[a, b, c], [a, c, d]]);
            }
        }
    }
}

pub(super) fn main_grid(
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<Vec<Vec<[f32; 3]>>, GenerateError> {
    let bottom = if rear {
        BACK_HEIGHTS[0]
    } else {
        FRONT_HEIGHTS[0]
    };
    let columns = chart_columns(rear, design);
    let mut grid = Vec::with_capacity(V_SAMPLES);
    for row in 0..V_SAMPLES {
        let t = row as f32 / (V_SAMPLES - 1) as f32;
        let mut coarse = Vec::with_capacity(U_SAMPLES);
        for column in 0..U_SAMPLES {
            let u = -1.0 + 2.0 * column as f32 / (U_SAMPLES - 1) as f32;
            let top = if rear {
                back_top_y(u, design)
            } else {
                front_top_y(u, design)
            };
            let y = bottom + t * (top - bottom);
            let theta = if rear {
                back_theta(u, y, design)
            } else {
                front_theta(u, y, design)
            };
            coarse.push(carrier_point(rear, theta, y, wearer, design)?.0);
        }
        // Refine the regular carrier rather than re-evaluating its offset
        // near small-curvature neckline regions at arbitrarily dense spacing.
        grid.push(
            columns
                .iter()
                .map(|u| {
                    let u = fan_coordinate(*u, t, rear, design);
                    let sample = ((u + 1.0) * 0.5 * (U_SAMPLES - 1) as f32)
                        .clamp(0.0, (U_SAMPLES - 1) as f32);
                    let lower = (sample.floor() as usize).min(U_SAMPLES - 2);
                    let blend = sample - lower as f32;
                    add(
                        scale(coarse[lower], 1.0 - blend),
                        scale(coarse[lower + 1], blend),
                    )
                })
                .collect(),
        );
    }
    Ok(grid)
}

pub(super) fn skirt_grid(
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<Vec<Vec<[f32; 3]>>, GenerateError> {
    let bottom = if rear {
        BACK_HEIGHTS[0]
    } else {
        FRONT_HEIGHTS[0]
    };
    let drop = if rear { 0.056 } else { 0.058 };
    let radial_flare = design.skirt_flare.metres();
    let lateral_flare = radial_flare * if rear { 5.0 / 6.0 } else { 1.0 };
    let sagittal_flare = radial_flare * if rear { 1.0 } else { 0.75 };
    let flare_ratio = radial_flare / 0.030;
    let length_scale = design.skirt_length.unit();
    let columns = chart_columns(rear, design);
    let mut grid = Vec::with_capacity(SKIRT_SAMPLES);
    for row in 0..SKIRT_SAMPLES {
        let t = row as f32 / (SKIRT_SAMPLES - 1) as f32;
        let mut points = Vec::with_capacity(columns.len());
        for &material_u in &columns {
            let u = fan_coordinate(material_u, 0.0, rear, design);
            let seam_theta = if rear {
                back_theta(u, bottom, design)
            } else {
                front_theta(u, bottom, design)
            };
            if row == 0 {
                points.push(carrier_point(rear, seam_theta, bottom, wearer, design)?.0);
                continue;
            }
            let a0 = if rear {
                BACK_RADIUS_X[0]
            } else {
                FRONT_RADIUS_X[0]
            };
            let b0 = if rear {
                BACK_RADIUS_Z[0] * design.back_depth.unit()
            } else {
                FRONT_RADIUS_Z[0]
            };
            let edge_theta = if rear {
                back_theta(1.0, bottom, design)
            } else {
                front_theta(1.0, bottom, design)
            };
            let angular_inset = (2.0 * t * design.side_return.unit()).to_radians();
            let angle =
                seam_theta * (1.0 - angular_inset / edge_theta.abs().max(angular_inset + 1e-6));
            let x = (a0 + lateral_flare * t) * angle.sin();
            let point_drop = if rear {
                0.0
            } else {
                design.profile.waist_point.metres() * waist_point_weight(seam_theta, design)
            };
            let y = bottom - drop * length_scale * t - point_drop;
            let center_bias = if rear { -0.004 } else { 0.006 } * flare_ratio * t * (1.0 - u * u);
            let z = if rear {
                -(b0 + sagittal_flare * t) * angle.cos() + center_bias
            } else {
                let waist_projection =
                    front_raw(seam_theta, bottom, design)[2] - b0 * seam_theta.cos();
                (b0 + sagittal_flare * t) * angle.cos() + center_bias + waist_projection
            };
            let raw = mapped_point([x, y, z], rear, wearer, design);
            let (_, normal) = carrier_point(rear, seam_theta, bottom, wearer, design)?;
            let clearance = if rear {
                design.back_clearance.metres()
            } else {
                design.front_clearance.metres()
            };
            points.push(world(
                add(raw, scale(local(normal, wearer.frame), clearance)),
                wearer.frame,
            ));
        }
        grid.push(points);
    }
    Ok(grid)
}

pub(super) fn build_mid(
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<MidMesh, GenerateError> {
    let main = main_grid(rear, wearer, design)?;
    let skirt = skirt_grid(rear, wearer, design)?;
    let columns = chart_columns(rear, design);
    let mut mesh = MidMesh {
        main_columns: columns.len(),
        ..MidMesh::default()
    };
    let main_ids = append_grid(&mut mesh, &main, rear);
    let mut skirt_ids = vec![main_ids[0].clone()];
    for row in skirt.iter().skip(1) {
        let mut id_row = Vec::with_capacity(columns.len());
        for point in row {
            id_row.push(mesh.positions.len() as u32);
            mesh.positions.push(*point);
        }
        skirt_ids.push(id_row);
    }
    mesh.skirt_face_start = Some(mesh.faces.len());
    // The skirt rows run downward from the main grid's bottom row, so their
    // chart orientation is opposite the main grid's upward progression.
    append_grid_faces(&mut mesh, &skirt_ids, !rear);
    if !rear && design.profile.medial_ridge.0 > 0 {
        let mut coordinates = Vec::with_capacity(mesh.positions.len());
        for _ in 0..V_SAMPLES {
            coordinates.extend(columns.iter().copied());
        }
        for _ in 0..SKIRT_SAMPLES - 1 {
            coordinates.extend(columns.iter().copied());
        }
        mesh.medial_crease = coordinates.iter().map(|u| u.abs() < 1e-6).collect();
        mesh.crease_right = coordinates.iter().map(|u| *u > 1e-6).collect();
    }
    Ok(mesh)
}

#[cfg(test)]
mod cut_tests {
    use super::*;

    #[test]
    fn left_cut_diagonal_stays_inside_a_concave_armscye_projection() {
        let mut mesh = MidMesh {
            main_columns: 2,
            positions: vec![
                [-0.184_955, 1.381_396, 0.083_615],
                [-0.186_401, 1.390_387, 0.077_849],
                [-0.186_764, 1.389_292, 0.075_262],
                [-0.188_523, 1.396_664, 0.068_833],
            ],
            faces: vec![[0, 1, 3], [0, 3, 2]],
            ..MidMesh::default()
        };
        let carrier = mesh.positions.clone();
        assert!(face_normal(mesh.faces[1], &mesh.positions)[2] < 0.0);
        mesh.triangulate_left_cut();
        assert_eq!(mesh.positions, carrier);
        for face in &mesh.faces {
            assert!(face_normal(*face, &mesh.positions)[2] > 0.0);
        }
        solidify(mesh, 0.002).unwrap();
    }
}
