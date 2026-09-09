//! Build connected carrier, shoulder, and skirt grids.

use super::*;

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
    let mut grid = Vec::with_capacity(V_SAMPLES);
    for row in 0..V_SAMPLES {
        let t = row as f32 / (V_SAMPLES - 1) as f32;
        let mut points = Vec::with_capacity(U_SAMPLES);
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
            points.push(carrier_point(rear, theta, y, wearer, design)?.0);
        }
        grid.push(points);
    }
    Ok(grid)
}

pub(super) fn shoulder_grid(
    rear: bool,
    side: f32,
    main: &[Vec<[f32; 3]>],
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<Vec<Vec<[f32; 3]>>, GenerateError> {
    let start = if side > 0.0 { 36 } else { 0 };
    let width_angle = 25.0 * design.shoulder_band_width.metres() / 0.030;
    let mut grid = Vec::with_capacity(BAND_SAMPLES);
    for row in 0..BAND_SAMPLES {
        let t = row as f32 / (BAND_SAMPLES - 1) as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        let mut points = Vec::with_capacity(13);
        for k in 0..13 {
            let column = start + k;
            let r = k as f32 / 12.0;
            let outer = if side > 0.0 { r } else { 1.0 - r };
            let u = -1.0 + 2.0 * column as f32 / (U_SAMPLES - 1) as f32;
            let y0 = if rear {
                back_top_y(u, design)
            } else {
                front_top_y(u, design)
            };
            let theta0 = if rear {
                back_theta(u, y0, design)
            } else {
                front_theta(u, y0, design)
            };
            if row == 0 {
                points.push(main[V_SAMPLES - 1][column]);
                continue;
            }
            let (inner_angle, y_inner, y_outer) = if rear {
                (38.0, 1.470, 1.458)
            } else {
                (50.0, 1.475, 1.468)
            };
            let theta1 = (side * (inner_angle + width_angle * outer)).to_radians();
            let y1 = y_inner * (1.0 - outer) + y_outer * outer;
            let neck_y = if rear { BACK_NECK_Y } else { FRONT_NECK_Y };
            let neck_dy = cubic_eval(0.5, &NECK_U, &neck_y, true).1 * design.neck_depth.unit();
            let opening_scale = design.arm_opening_depth.unit();
            let trim_y0 = if rear {
                1.240 + (y0 - 1.240) * opening_scale
            } else {
                1.250 + (y0 - 1.250) * opening_scale
            };
            let (limit, dlimit) = if rear {
                cubic_eval(trim_y0, &BACK_TRIM_HEIGHTS, &BACK_LIMIT_DEGREES, false)
            } else {
                cubic_eval(trim_y0, &FRONT_TRIM_HEIGHTS, &FRONT_LIMIT_DEGREES, false)
            };
            let dlimit = dlimit * opening_scale;
            let inner_dtheta = side * (limit + 0.5 * dlimit * neck_dy).to_radians() * 0.18;
            let inner_dy = neck_dy * 0.18;
            let outer_dy = 0.080;
            let outer_dtheta = side * (dlimit * outer_dy).to_radians();
            let m0_theta = inner_dtheta * (1.0 - outer) + outer_dtheta * outer;
            let m0_y = inner_dy * (1.0 - outer) + outer_dy * outer;
            let m1_theta = 0.60 * (theta1 - theta0);
            let m1_y = 0.60 * (y1 - y0);
            let theta = h00 * theta0 + h10 * m0_theta + h01 * theta1 + h11 * m1_theta;
            let y = h00 * y0 + h10 * m0_y + h01 * y1 + h11 * m1_y;
            points.push(carrier_point(rear, theta, y, wearer, design)?.0);
        }
        grid.push(points);
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
    let mut grid = Vec::with_capacity(SKIRT_SAMPLES);
    for row in 0..SKIRT_SAMPLES {
        let t = row as f32 / (SKIRT_SAMPLES - 1) as f32;
        let mut points = Vec::with_capacity(U_SAMPLES);
        for column in 0..U_SAMPLES {
            let u = -1.0 + 2.0 * column as f32 / (U_SAMPLES - 1) as f32;
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
                BACK_RADIUS_Z[0]
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
            let y = bottom - drop * length_scale * t;
            let center_bias = if rear { -0.004 } else { 0.006 } * flare_ratio * t * (1.0 - u * u);
            let z = if rear {
                -(b0 + sagittal_flare * t) * angle.cos().powf(0.55) + center_bias
            } else {
                (b0 + sagittal_flare * t) * angle.cos() + center_bias
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
    let left = shoulder_grid(rear, -1.0, &main, wearer, design)?;
    let right = shoulder_grid(rear, 1.0, &main, wearer, design)?;
    let skirt = skirt_grid(rear, wearer, design)?;
    let mut mesh = MidMesh::default();
    let main_ids = append_grid(&mut mesh, &main, rear);
    for (band, start) in [(&left, 0usize), (&right, 36usize)] {
        let mut ids = vec![main_ids[V_SAMPLES - 1][start..start + 13].to_vec()];
        for row in band.iter().skip(1) {
            let mut id_row = Vec::with_capacity(13);
            for point in row {
                id_row.push(mesh.positions.len() as u32);
                mesh.positions.push(*point);
            }
            ids.push(id_row);
        }
        append_grid_faces(&mut mesh, &ids, rear);
    }
    let mut skirt_ids = vec![main_ids[0].clone()];
    for row in skirt.iter().skip(1) {
        let mut id_row = Vec::with_capacity(U_SAMPLES);
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
    Ok(mesh)
}
