//! Boundary-aligned front and rear breastplate carriers.
//!
//! The plates are authored as smooth low-dimensional lofts. Neck, armscye,
//! shoulder-band and skirt boundaries are evaluated in each loft's material
//! chart, so changing an opening cannot fold an otherwise regular carrier.

use std::collections::BTreeMap;

use crate::{
    ArmorMorph, BreastplateDesign, GenerateError, GeneratedArmor, TorsoClearancePose,
    TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, breastplate_design_hash,
    validate_breastplate,
};

const U_SAMPLES: usize = 49;
const V_SAMPLES: usize = 33;
const BAND_SAMPLES: usize = 9;
const SKIRT_SAMPLES: usize = 9;
const REFERENCE_RIG_NECK_HEIGHT: f32 = 1.441_910_7;
const REFERENCE_CARRIER_TOP_HEIGHT: f32 = 1.480;
const REFERENCE_SEMANTIC_HEIGHT: f32 = 0.502_445;
const REFERENCE_SHOULDER_HALF_WIDTH: f32 = 0.175_861_03;
const REFERENCE_TORSO_HALF_WIDTH: f32 = 0.187_162_74;
const REFERENCE_SECTION_CENTER_DEPTH: f32 = 0.020_309_05;
const REFERENCE_SECTION_RADIUS: f32 = 0.121_758_32;
const FIT_SURFACE_MARGIN: f32 = 0.001_5;
const MAX_FIT_CORRECTION: f32 = 0.060;

const FRONT_HEIGHTS: [f32; 8] = [1.038, 1.105, 1.185, 1.285, 1.355, 1.400, 1.445, 1.480];
const FRONT_RADIUS_X: [f32; 8] = [0.178, 0.180, 0.181, 0.183, 0.185, 0.174, 0.142, 0.132];
const FRONT_RADIUS_Z: [f32; 8] = [0.143, 0.153, 0.156, 0.151, 0.136, 0.115, 0.080, 0.050];
const FRONT_TRIM_HEIGHTS: [f32; 10] = [
    1.038, 1.105, 1.185, 1.250, 1.300, 1.335, 1.370, 1.400, 1.445, 1.480,
];
const FRONT_LIMIT_DEGREES: [f32; 10] = [92.0, 94.0, 94.0, 92.0, 78.0, 68.0, 58.0, 60.0, 72.0, 96.0];
const FRONT_NECK_Y: [f32; 3] = [1.425, 1.433, 1.458];

const BACK_HEIGHTS: [f32; 8] = [1.040, 1.120, 1.200, 1.280, 1.340, 1.400, 1.445, 1.480];
const BACK_RADIUS_X: [f32; 8] = [0.168, 0.170, 0.180, 0.195, 0.210, 0.205, 0.170, 0.150];
const BACK_RADIUS_Z: [f32; 8] = [0.096, 0.072, 0.085, 0.106, 0.109, 0.105, 0.088, 0.078];
const BACK_TRIM_HEIGHTS: [f32; 9] = [
    1.040, 1.120, 1.200, 1.280, 1.340, 1.380, 1.420, 1.460, 1.480,
];
const BACK_LIMIT_DEGREES: [f32; 9] = [75.0, 76.0, 78.0, 68.0, 60.0, 55.0, 52.0, 65.0, 70.0];
const BACK_NECK_Y: [f32; 3] = [1.420, 1.430, 1.455];
const NECK_U: [f32; 3] = [0.0, 0.25, 0.50];

#[derive(Clone, Copy)]
struct Frame {
    lateral: [f32; 3],
    vertical: [f32; 3],
    front: [f32; 3],
}

#[derive(Clone, Copy)]
struct Wearer<'a> {
    frame: Frame,
    anchors: TorsoUpperRigAnchors,
    clearance: &'a TorsoClearancePose,
    source_faces: &'a [[u32; 3]],
    torso_faces: &'a [[u32; 3]],
    x_scale: f32,
    shoulder_x_scale: f32,
    y_scale: f32,
    z_scale: f32,
    lateral_origin: f32,
    coronal_origin: f32,
}

#[derive(Default)]
struct MidMesh {
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    skirt_face_start: Option<usize>,
}

struct SolidMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    source_mid_indices: Vec<usize>,
}

#[derive(Clone, Copy)]
struct SourceSample {
    face: usize,
    weights: [f32; 3],
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn normalized(a: [f32; 3]) -> Result<[f32; 3], GenerateError> {
    let magnitude = length(a);
    if magnitude > 1e-12 && magnitude.is_finite() {
        Ok(scale(a, magnitude.recip()))
    } else {
        eprintln!("breastplate cannot normalize {a:?}");
        Err(GenerateError::Degenerate)
    }
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn frame(
    _positions: &[[f32; 3]],
    _semantic: &[[f32; 2]],
    front_hint: [f32; 3],
) -> Result<Frame, GenerateError> {
    // MHR armor is authored in its stable Y-up model frame. The semantic
    // spine regression can tilt with posture and must not rotate an entire
    // rigid plate (a 4.5-degree tilt moved the rear plate by over 10 cm).
    let vertical = [0.0, 1.0, 0.0];
    let front = normalized([front_hint[0], 0.0, front_hint[2]])?;
    let lateral = normalized(cross(vertical, front))?;
    Ok(Frame {
        lateral,
        vertical,
        front,
    })
}

fn local(point: [f32; 3], frame: Frame) -> [f32; 3] {
    [
        dot(point, frame.lateral),
        dot(point, frame.vertical),
        dot(point, frame.front),
    ]
}

fn world(point: [f32; 3], frame: Frame) -> [f32; 3] {
    add(
        add(
            scale(frame.lateral, point[0]),
            scale(frame.vertical, point[1]),
        ),
        scale(frame.front, point[2]),
    )
}

fn cubic_eval(x: f32, xs: &[f32], ys: &[f32], zero_start: bool) -> (f32, f32) {
    debug_assert_eq!(xs.len(), ys.len());
    let mut slopes = Vec::with_capacity(xs.len());
    for index in 0..xs.len() {
        let slope = if index == 0 {
            if zero_start {
                0.0
            } else {
                (ys[1] - ys[0]) / (xs[1] - xs[0])
            }
        } else if index + 1 == xs.len() {
            (ys[index] - ys[index - 1]) / (xs[index] - xs[index - 1])
        } else {
            (ys[index + 1] - ys[index - 1]) / (xs[index + 1] - xs[index - 1])
        };
        slopes.push(slope);
    }
    if x <= xs[0] {
        return (ys[0], slopes[0]);
    }
    if x >= xs[xs.len() - 1] {
        return (ys[ys.len() - 1], slopes[slopes.len() - 1]);
    }
    let index = xs
        .windows(2)
        .position(|span| x <= span[1])
        .unwrap_or(xs.len() - 2);
    let h = xs[index + 1] - xs[index];
    let t = (x - xs[index]) / h;
    let t2 = t * t;
    let t3 = t2 * t;
    let value = (2.0 * t3 - 3.0 * t2 + 1.0) * ys[index]
        + (t3 - 2.0 * t2 + t) * h * slopes[index]
        + (-2.0 * t3 + 3.0 * t2) * ys[index + 1]
        + (t3 - t2) * h * slopes[index + 1];
    let derivative = (6.0 * t2 - 6.0 * t) * ys[index] / h
        + (3.0 * t2 - 4.0 * t + 1.0) * slopes[index]
        + (-6.0 * t2 + 6.0 * t) * ys[index + 1] / h
        + (3.0 * t2 - 2.0 * t) * slopes[index + 1];
    (value, derivative)
}

fn cubic(x: f32, xs: &[f32], ys: &[f32], zero_start: bool) -> f32 {
    cubic_eval(x, xs, ys, zero_start).0
}

fn valid_surface(surface: &TorsoSurface) -> bool {
    let count = surface.vertices.len();
    !surface.domain.is_empty()
        && count >= 3
        && !surface.faces.is_empty()
        && surface
            .faces
            .iter()
            .flatten()
            .all(|i| (*i as usize) < count)
        && surface.morphs.len() == surface.morph_fronts.len()
        && surface.morphs.len() == surface.morph_semantic_coordinates.len()
        && surface.morphs.len() == surface.morph_upper_rig_anchors.len()
        && surface.morphs.len() == surface.morph_coronal_depths.len()
        && surface
            .morphs
            .iter()
            .all(|m| m.positions.len() == count && m.normals.len() == count)
        && surface
            .morph_semantic_coordinates
            .iter()
            .all(|v| v.len() == count)
        && surface.coronal_anchors.len() >= 2
        && surface
            .morph_coronal_depths
            .iter()
            .all(|v| v.len() == surface.coronal_anchors.len())
        && surface
            .clearance_mesh
            .has_corresponding_domains(surface.morphs.len())
}

fn wearer<'a>(
    positions: &'a [[f32; 3]],
    semantic: &'a [[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    _coronal_depths: &'a [f32],
    clearance: &'a TorsoClearancePose,
    source_faces: &'a [[u32; 3]],
    torso_faces: &'a [[u32; 3]],
    shoulder_envelope: &[TorsoShoulderSample],
) -> Result<Wearer<'a>, GenerateError> {
    let frame = frame(positions, semantic, front)?;
    let neck = local(anchors.neck_base, frame);
    let shoulders = anchors.shoulders.map(|point| local(point, frame));
    let shoulder_half_width = shoulders
        .iter()
        .map(|point| (point[0] - neck[0]).abs())
        .sum::<f32>()
        * 0.5;
    let count = positions.len() as f32;
    let mean_level = semantic.iter().map(|p| p[1]).sum::<f32>() / count;
    let mean_height = positions.iter().map(|p| local(*p, frame)[1]).sum::<f32>() / count;
    let variance = semantic
        .iter()
        .map(|p| (p[1] - mean_level).powi(2))
        .sum::<f32>();
    let height_slope = semantic
        .iter()
        .zip(positions)
        .map(|(uv, p)| (uv[1] - mean_level) * (local(*p, frame)[1] - mean_height))
        .sum::<f32>()
        / variance.max(1e-8);
    let coronal_origin = _coronal_depths.iter().sum::<f32>() / _coronal_depths.len() as f32;
    let center_front = positions
        .iter()
        .zip(semantic)
        .filter_map(|(point, uv)| {
            (uv[0].abs() < 0.12 && (0.25..=0.75).contains(&uv[1])).then(|| local(*point, frame)[2])
        })
        .fold(f32::NEG_INFINITY, f32::max);
    if !center_front.is_finite() {
        return Err(GenerateError::InvalidSurface);
    }
    let top_height = shoulder_envelope
        .iter()
        .map(|sample| local(sample.position, frame)[1])
        .fold(neck[1], f32::max);
    let fit_bottom = neck[1] - 0.36 * (height_slope.abs() / REFERENCE_SEMANTIC_HEIGHT);
    let fit_top = neck[1] - 0.10 * (height_slope.abs() / REFERENCE_SEMANTIC_HEIGHT);
    let mut torso_min = [f32::INFINITY; 3];
    let mut torso_max = [f32::NEG_INFINITY; 3];
    for index in torso_faces.iter().flatten() {
        let point = local(
            clearance.enclosure_vertices[*index as usize].position,
            frame,
        );
        if (fit_bottom..=fit_top).contains(&point[1]) {
            for axis in 0..3 {
                torso_min[axis] = torso_min[axis].min(point[axis]);
                torso_max[axis] = torso_max[axis].max(point[axis]);
            }
        }
    }
    if std::env::var_os("BREASTPLATE_REPORT_FIT").is_some() {
        eprintln!(
            "breastplate wearer frame={:?} neck_y={} top_y={} shoulder_half={} semantic_height={} front_radius={} coronal={}",
            [frame.lateral, frame.vertical, frame.front],
            neck[1],
            top_height,
            shoulder_half_width,
            height_slope,
            center_front - coronal_origin,
            coronal_origin
        );
        eprintln!(
            "breastplate torso_fit_bounds y={fit_bottom}..{fit_top} min={torso_min:?} max={torso_max:?}"
        );
    }
    Ok(Wearer {
        frame,
        anchors,
        clearance,
        source_faces,
        torso_faces,
        x_scale: (torso_max[0].abs().max(torso_min[0].abs()) / REFERENCE_TORSO_HALF_WIDTH)
            .clamp(0.65, 1.55),
        shoulder_x_scale: (shoulder_half_width / REFERENCE_SHOULDER_HALF_WIDTH).clamp(0.65, 1.55),
        y_scale: (height_slope.abs() / REFERENCE_SEMANTIC_HEIGHT).clamp(0.70, 1.45),
        z_scale: (((torso_max[2] - torso_min[2]) * 0.5) / REFERENCE_SECTION_RADIUS)
            .clamp(0.65, 1.80),
        lateral_origin: neck[0],
        coronal_origin: (torso_min[2] + torso_max[2]) * 0.5,
    })
}

fn mapped_height(reference_y: f32, wearer: Wearer<'_>, design: &BreastplateDesign) -> f32 {
    local(wearer.anchors.neck_base, wearer.frame)[1]
        + (reference_y - REFERENCE_RIG_NECK_HEIGHT) * wearer.y_scale * design.plate_length.unit()
}

fn lateral_scale(reference_y: f32, wearer: Wearer<'_>) -> f32 {
    let shoulder_blend = smoothstep((reference_y - 1.355) / 0.100);
    wearer.x_scale * (1.0 - shoulder_blend) + wearer.shoulder_x_scale * shoulder_blend
}

fn mapped_point(
    reference: [f32; 3],
    _rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> [f32; 3] {
    let vertical_t = ((reference[1] - FRONT_HEIGHTS[0])
        / (REFERENCE_RIG_NECK_HEIGHT - FRONT_HEIGHTS[0]))
        .clamp(0.0, 1.0);
    let waist_scale = 1.0 + (design.waist_width.unit() - 1.0) * (1.0 - smoothstep(vertical_t));
    [
        wearer.lateral_origin + reference[0] * lateral_scale(reference[1], wearer) * waist_scale,
        mapped_height(reference[1], wearer, design),
        wearer.coronal_origin + (reference[2] - REFERENCE_SECTION_CENTER_DEPTH) * wearer.z_scale,
    ]
}

fn front_top_y(u: f32, design: &BreastplateDesign) -> f32 {
    let depth = design.neck_depth.unit();
    let neck =
        FRONT_NECK_Y.map(|y| REFERENCE_RIG_NECK_HEIGHT - (REFERENCE_RIG_NECK_HEIGHT - y) * depth);
    let attach = REFERENCE_RIG_NECK_HEIGHT
        - (REFERENCE_RIG_NECK_HEIGHT - 1.390) * design.arm_opening_depth.unit();
    let a = u.abs();
    if a <= 0.5 {
        cubic(a, &NECK_U, &neck, true)
    } else {
        let s = smoothstep((a - 0.5) * 2.0);
        neck[2] * (1.0 - s) + attach * s
    }
}

fn back_top_y(u: f32, design: &BreastplateDesign) -> f32 {
    let depth = design.neck_depth.unit();
    let neck =
        BACK_NECK_Y.map(|y| REFERENCE_RIG_NECK_HEIGHT - (REFERENCE_RIG_NECK_HEIGHT - y) * depth);
    let attach = REFERENCE_RIG_NECK_HEIGHT
        - (REFERENCE_RIG_NECK_HEIGHT - 1.375) * design.arm_opening_depth.unit();
    let a = u.abs();
    if a <= 0.5 {
        cubic(a, &NECK_U, &neck, true)
    } else {
        let s = smoothstep((a - 0.5) * 2.0);
        neck[2] * (1.0 - s) + attach * s
    }
}

fn top_neck_scale(reference_y: f32, bottom: f32, top: f32, design: &BreastplateDesign) -> f32 {
    let blend = smoothstep((reference_y - bottom) / (top - bottom).max(1e-6));
    1.0 + (design.neck_width.unit() - 1.0) * blend
}

fn front_raw(theta: f32, y: f32, design: &BreastplateDesign) -> [f32; 3] {
    let a = cubic(y, &FRONT_HEIGHTS, &FRONT_RADIUS_X, false);
    let b = cubic(y, &FRONT_HEIGHTS, &FRONT_RADIUS_Z, false);
    let phase = ((y - 1.06) / 0.39).clamp(0.0, 1.0);
    let crown = design.front_crown.metres() * (std::f32::consts::PI * phase).sin().powi(2);
    let upper = smoothstep((y - 1.445) / 0.035);
    let recession = 0.050 * upper;
    let lift = 0.040 * upper;
    let sin = theta.sin();
    let cos = theta.cos();
    [
        a * sin,
        y,
        b * cos + crown * cos.powi(4) - recession + lift * sin.powi(2),
    ]
}

fn back_raw(theta: f32, y: f32) -> Result<[f32; 3], GenerateError> {
    let cos = theta.cos();
    if cos <= 0.0 {
        return Err(GenerateError::Degenerate);
    }
    let a = cubic(y, &BACK_HEIGHTS, &BACK_RADIUS_X, false);
    let b = cubic(y, &BACK_HEIGHTS, &BACK_RADIUS_Z, false);
    Ok([a * theta.sin(), y, -b * cos.powf(0.55)])
}

fn carrier_point(
    rear: bool,
    theta: f32,
    y: f32,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<([f32; 3], [f32; 3]), GenerateError> {
    let raw = if rear {
        back_raw(theta, y)?
    } else {
        front_raw(theta, y, design)
    };
    let du = 0.0005;
    let dy = 0.0002;
    let raw_theta = if rear {
        back_raw(theta + du, y)?
    } else {
        front_raw(theta + du, y, design)
    };
    let raw_y = if rear {
        back_raw(theta, y + dy)?
    } else {
        front_raw(theta, y + dy, design)
    };
    let center = mapped_point(raw, rear, wearer, design);
    let tangent_theta = sub(mapped_point(raw_theta, rear, wearer, design), center);
    let tangent_y = sub(mapped_point(raw_y, rear, wearer, design), center);
    let mut normal = normalized(cross(tangent_theta, tangent_y))?;
    if rear {
        normal = scale(normal, -1.0);
    }
    let clearance = if rear {
        design.back_clearance.metres()
    } else {
        design.front_clearance.metres()
    };
    Ok((
        world(add(center, scale(normal, clearance)), wearer.frame),
        world(normal, wearer.frame),
    ))
}

fn front_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let trim_y = 1.250 + (y - 1.250) * design.arm_opening_depth.unit();
    let limit =
        cubic(trim_y, &FRONT_TRIM_HEIGHTS, &FRONT_LIMIT_DEGREES, false) * design.side_return.unit();
    (limit * u * top_neck_scale(y, FRONT_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT, design))
        .to_radians()
}

fn back_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let trim_y = 1.240 + (y - 1.240) * design.arm_opening_depth.unit();
    let limit =
        cubic(trim_y, &BACK_TRIM_HEIGHTS, &BACK_LIMIT_DEGREES, false) * design.side_return.unit();
    (limit * u * top_neck_scale(y, BACK_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT, design))
        .to_radians()
}

fn append_grid(mesh: &mut MidMesh, grid: &[Vec<[f32; 3]>], rear: bool) -> Vec<Vec<u32>> {
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

fn append_grid_faces(mesh: &mut MidMesh, ids: &[Vec<u32>], rear: bool) {
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

fn main_grid(
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

fn shoulder_grid(
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

fn skirt_grid(
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

fn build_mid(
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

fn face_normal(face: [u32; 3], positions: &[[f32; 3]]) -> [f32; 3] {
    let [a, b, c] = face.map(|index| positions[index as usize]);
    cross(sub(b, a), sub(c, a))
}

fn vertex_normals(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> Result<Vec<[f32; 3]>, GenerateError> {
    let mut normals = vec![[0.0; 3]; positions.len()];
    for (face_index, face) in faces.iter().enumerate() {
        let normal = face_normal(*face, positions);
        if dot(normal, normal) <= 1e-16 || normal.iter().any(|value| !value.is_finite()) {
            eprintln!("breastplate degenerate face {face_index} {face:?}");
            return Err(GenerateError::Degenerate);
        }
        for index in face {
            normals[*index as usize] = add(normals[*index as usize], normal);
        }
    }
    normals.into_iter().map(normalized).collect()
}

fn solidify(mid: MidMesh, thickness: f32) -> Result<SolidMesh, GenerateError> {
    let mid_normals = vertex_normals(&mid.positions, &mid.faces)?;
    let count = mid.positions.len() as u32;
    let mut positions = mid.positions.clone();
    positions.extend(
        mid.positions
            .iter()
            .zip(&mid_normals)
            .map(|(point, normal)| add(*point, scale(*normal, thickness))),
    );
    let mut source_mid_indices = (0..mid.positions.len())
        .chain(0..mid.positions.len())
        .collect::<Vec<_>>();
    let mut welded_indices = (0..count * 2).collect::<Vec<_>>();
    let mut skirt_inner = [0_u32; U_SAMPLES];
    let mut skirt_outer = [0_u32; U_SAMPLES];
    if mid.skirt_face_start.is_some() {
        for index in 0..U_SAMPLES {
            skirt_inner[index] = positions.len() as u32;
            positions.push(mid.positions[index]);
            source_mid_indices.push(index);
            welded_indices.push(index as u32);
            skirt_outer[index] = positions.len() as u32;
            positions.push(add(
                mid.positions[index],
                scale(mid_normals[index], thickness),
            ));
            source_mid_indices.push(index);
            welded_indices.push(index as u32 + count);
        }
    }
    let mut indices = Vec::with_capacity(mid.faces.len() * 6);
    for (face_index, [a, b, c]) in mid.faces.iter().enumerate() {
        let skirt = mid
            .skirt_face_start
            .is_some_and(|start| face_index >= start);
        let inner = |index: u32| {
            if skirt && (index as usize) < U_SAMPLES {
                skirt_inner[index as usize]
            } else {
                index
            }
        };
        let outer = |index: u32| {
            if skirt && (index as usize) < U_SAMPLES {
                skirt_outer[index as usize]
            } else {
                index + count
            }
        };
        indices.extend([outer(*a), outer(*b), outer(*c)]);
        indices.extend([inner(*c), inner(*b), inner(*a)]);
    }
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for [a, b, c] in &mid.faces {
        for (start, end) in [(*a, *b), (*b, *c), (*c, *a)] {
            let key = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            edges.entry(key).or_default().push((start, end));
        }
    }
    for uses in edges.values() {
        match uses.as_slice() {
            &[(a, b)] => indices.extend([a, b, b + count, a, b + count, a + count]),
            &[(first_start, first_end), (second_start, second_end)]
                if first_start == second_end && first_end == second_start => {}
            [_, _] => return Err(GenerateError::Degenerate),
            _ => return Err(GenerateError::Degenerate),
        }
    }
    let faces = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<_>>();
    let mut solid_edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for [a, b, c] in &faces {
        let [a, b, c] = [*a, *b, *c].map(|index| welded_indices[index as usize]);
        for (start, end) in [(a, b), (b, c), (c, a)] {
            let key = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            solid_edges.entry(key).or_default().push((start, end));
        }
    }
    if solid_edges
        .values()
        .any(|uses| !matches!(uses.as_slice(), &[(a, b), (c, d)] if a == d && b == c))
    {
        return Err(GenerateError::Degenerate);
    }
    let normals = vertex_normals(&positions, &faces)?;
    Ok(SolidMesh {
        positions,
        normals,
        indices,
        source_mid_indices,
    })
}

fn translate(mesh: &mut MidMesh, offset: [f32; 3]) {
    for position in &mut mesh.positions {
        *position = add(*position, offset);
    }
}

fn body_depth_range(point: [f32; 3], wearer: Wearer<'_>) -> Option<(f32, f32)> {
    let target = local(point, wearer.frame);
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    for face in wearer.torso_faces {
        let [a, b, c] = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        let denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        if denominator.abs() <= 1e-10 {
            continue;
        }
        let u =
            ((b[1] - c[1]) * (target[0] - c[0]) + (c[0] - b[0]) * (target[1] - c[1])) / denominator;
        let v =
            ((c[1] - a[1]) * (target[0] - c[0]) + (a[0] - c[0]) * (target[1] - c[1])) / denominator;
        let w = 1.0 - u - v;
        if u >= -1e-5 && v >= -1e-5 && w >= -1e-5 {
            let depth = u * a[2] + v * b[2] + w * c[2];
            minimum = minimum.min(depth);
            maximum = maximum.max(depth);
        }
    }
    minimum.is_finite().then_some((minimum, maximum))
}

fn body_lateral_range(point: [f32; 3], wearer: Wearer<'_>) -> Option<(f32, f32)> {
    let target = local(point, wearer.frame);
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    for face in wearer.torso_faces {
        let [a, b, c] = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        let denominator = (b[1] - c[1]) * (a[2] - c[2]) + (c[2] - b[2]) * (a[1] - c[1]);
        if denominator.abs() <= 1e-10 {
            continue;
        }
        let u =
            ((b[1] - c[1]) * (target[2] - c[2]) + (c[2] - b[2]) * (target[1] - c[1])) / denominator;
        let v =
            ((c[1] - a[1]) * (target[2] - c[2]) + (a[2] - c[2]) * (target[1] - c[1])) / denominator;
        let w = 1.0 - u - v;
        if u >= -1e-5 && v >= -1e-5 && w >= -1e-5 {
            let lateral = u * a[0] + v * b[0] + w * c[0];
            minimum = minimum.min(lateral);
            maximum = maximum.max(lateral);
        }
    }
    minimum.is_finite().then_some((minimum, maximum))
}

fn reference_height(point: [f32; 3], wearer: Wearer<'_>, design: &BreastplateDesign) -> f32 {
    let neck_y = local(wearer.anchors.neck_base, wearer.frame)[1];
    REFERENCE_RIG_NECK_HEIGHT
        + (local(point, wearer.frame)[1] - neck_y)
            / (wearer.y_scale * design.plate_length.unit()).max(1e-6)
}

#[derive(Clone, Copy, Debug, Default)]
struct FitProfile {
    center: [f32; 3],
    side: [f32; 3],
}

fn quadratic_weights(reference_y: f32, bottom: f32) -> [f32; 3] {
    let t = ((reference_y - bottom) / (REFERENCE_CARRIER_TOP_HEIGHT - bottom)).clamp(0.0, 1.0);
    [(1.0 - t).powi(2), 2.0 * t * (1.0 - t), t.powi(2)]
}

fn weighted_value(weights: [f32; 3], controls: [f32; 3]) -> f32 {
    weights
        .into_iter()
        .zip(controls)
        .map(|(weight, control)| weight * control)
        .sum()
}

fn torso_section_center(local_y: f32, wearer: Wearer<'_>) -> Option<[f32; 2]> {
    let mut minimum = [f32::INFINITY; 2];
    let mut maximum = [f32::NEG_INFINITY; 2];
    let mut crossings = 0usize;
    for face in wearer.torso_faces {
        let triangle = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            let da = a[1] - local_y;
            let db = b[1] - local_y;
            if (da < 0.0 && db < 0.0) || (da > 0.0 && db > 0.0) || (da - db).abs() <= 1e-9 {
                continue;
            }
            let t = da / (da - db);
            if !(-1e-5..=1.0 + 1e-5).contains(&t) {
                continue;
            }
            let crossing = [a[0] + (b[0] - a[0]) * t, a[2] + (b[2] - a[2]) * t];
            for axis in 0..2 {
                minimum[axis] = minimum[axis].min(crossing[axis]);
                maximum[axis] = maximum[axis].max(crossing[axis]);
            }
            crossings += 1;
        }
    }
    (crossings >= 4).then(|| {
        [
            (minimum[0] + maximum[0]) * 0.5,
            (minimum[1] + maximum[1]) * 0.5,
        ]
    })
}

fn radial_direction(point: [f32; 3], wearer: Wearer<'_>) -> Option<([f32; 3], f32, f32)> {
    let local_point = local(point, wearer.frame);
    let center = torso_section_center(local_point[1], wearer)?;
    let delta = [local_point[0] - center[0], 0.0, local_point[2] - center[1]];
    let radius = length(delta);
    (radius > 1e-6).then(|| {
        let direction = scale(delta, radius.recip());
        let side_blend = smoothstep((direction[0].abs() - 0.45) / 0.45);
        (direction, radius, side_blend)
    })
}

fn body_radial_extent(point: [f32; 3], direction: [f32; 3], wearer: Wearer<'_>) -> Option<f32> {
    let local_point = local(point, wearer.frame);
    let center = torso_section_center(local_point[1], wearer)?;
    let origin = [center[0], local_point[1], center[1]];
    let mut minimum = f32::INFINITY;
    for face in wearer.torso_faces {
        let [a, b, c] = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        let edge_ab = sub(b, a);
        let edge_ac = sub(c, a);
        let h = cross(direction, edge_ac);
        let determinant = dot(edge_ab, h);
        if determinant.abs() <= 1e-9 {
            continue;
        }
        let inverse = determinant.recip();
        let from_a = sub(origin, a);
        let u = inverse * dot(from_a, h);
        if !(-1e-5..=1.0 + 1e-5).contains(&u) {
            continue;
        }
        let q = cross(from_a, edge_ab);
        let v = inverse * dot(direction, q);
        if v < -1e-5 || u + v > 1.0 + 1e-5 {
            continue;
        }
        let distance = inverse * dot(edge_ac, q);
        if distance >= 1e-6 {
            minimum = minimum.min(distance);
        }
    }
    minimum.is_finite().then_some(minimum)
}

fn update_radial_profile(
    fit: &mut FitProfile,
    height_weights: [f32; 3],
    side_blend: f32,
    residual: f32,
) {
    let center_blend = 1.0 - side_blend;
    let denominator = height_weights
        .into_iter()
        .map(|weight| weight * weight * (center_blend * center_blend + side_blend * side_blend))
        .sum::<f32>();
    if residual > 0.0 && denominator > 1e-8 {
        for (index, weight) in height_weights.into_iter().enumerate() {
            fit.center[index] += residual * weight * center_blend / denominator;
            fit.side[index] += residual * weight * side_blend / denominator;
        }
    }
}

fn apply_fit(
    position: [f32; 3],
    original: [f32; 3],
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
    fit: FitProfile,
) -> [f32; 3] {
    let reference_y = reference_height(original, wearer, design);
    let bottom = if rear {
        BACK_HEIGHTS[0]
    } else {
        FRONT_HEIGHTS[0]
    };
    let weights = quadratic_weights(reference_y, bottom);
    let Some((direction, _, side_blend)) = radial_direction(original, wearer) else {
        return position;
    };
    let offset = weighted_value(weights, fit.center) * (1.0 - side_blend)
        + weighted_value(weights, fit.side) * side_blend;
    world(
        add(local(position, wearer.frame), scale(direction, offset)),
        wearer.frame,
    )
}

fn section_clearance_fit(
    mesh: &mut MidMesh,
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<FitProfile, GenerateError> {
    let original = mesh.positions.clone();
    let bottom = if rear {
        BACK_HEIGHTS[0]
    } else {
        FRONT_HEIGHTS[0]
    };
    let mut fit = FitProfile::default();
    for iteration in 0..=24 {
        for (position, original) in mesh.positions.iter_mut().zip(&original) {
            *position = apply_fit(*original, *original, rear, wearer, design, fit);
        }
        let mut constraint = None::<(f32, [f32; 3], f32, [f32; 3], f32)>;
        for (position, original) in mesh.positions.iter().zip(&original) {
            let reference_y = reference_height(*original, wearer, design);
            let weights = quadratic_weights(reference_y, bottom);
            let Some((direction, radius, side_blend)) = radial_direction(*position, wearer) else {
                continue;
            };
            if (rear && direction[2] >= 0.0) || (!rear && direction[2] <= 0.0) {
                continue;
            }
            let Some(body_radius) = body_radial_extent(*position, direction, wearer) else {
                continue;
            };
            let residual = body_radius + FIT_SURFACE_MARGIN - radius;
            if residual > 1e-5 {
                if constraint.is_none_or(|current| residual > current.0) {
                    constraint = Some((
                        residual,
                        weights,
                        side_blend,
                        local(*position, wearer.frame),
                        body_radius,
                    ));
                }
            }
        }
        let Some((residual, weights, side_blend, point, body_radius)) = constraint else {
            break;
        };
        if iteration == 24 {
            if std::env::var_os("BREASTPLATE_REPORT_FIT").is_some() {
                eprintln!(
                    "breastplate nonconverged section_fit rear={rear} residual={residual} witness={point:?} body_radius={body_radius}"
                );
            }
            return Err(GenerateError::InvalidSurface);
        }
        update_radial_profile(&mut fit, weights, side_blend, residual);
        if std::env::var_os("BREASTPLATE_REPORT_FIT").is_some() {
            eprintln!(
                "breastplate fit_iteration rear={rear} fit={fit:?} witness={point:?} body_radius={body_radius}"
            );
        }
    }
    if fit
        .center
        .into_iter()
        .chain(fit.side)
        .any(|value| !value.is_finite() || value > MAX_FIT_CORRECTION)
    {
        if std::env::var_os("BREASTPLATE_REPORT_FIT").is_some() {
            eprintln!("breastplate rejected section_fit rear={rear} fit={fit:?}");
        }
        return Err(GenerateError::InvalidSurface);
    }
    for (position, original) in mesh.positions.iter_mut().zip(&original) {
        *position = apply_fit(*original, *original, rear, wearer, design, fit);
    }
    Ok(fit)
}

fn build_pair(
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<(MidMesh, MidMesh), GenerateError> {
    if std::env::var_os("BREASTPLATE_REPORT_FIT").is_some() {
        for reference_y in [1.040_f32, 1.120, 1.200, 1.280, 1.340, 1.400] {
            let probe = world(
                [
                    wearer.lateral_origin,
                    mapped_height(reference_y, wearer, design),
                    wearer.coronal_origin,
                ],
                wearer.frame,
            );
            eprintln!(
                "breastplate lateral_station y={reference_y} range={:?}",
                body_lateral_range(probe, wearer)
            );
        }
    }
    let mut front = build_mid(false, wearer, design)?;
    let mut back = build_mid(true, wearer, design)?;
    let front_fit = section_clearance_fit(&mut front, false, wearer, design)?;
    let back_fit = section_clearance_fit(&mut back, true, wearer, design)?;
    if std::env::var_os("BREASTPLATE_REPORT_FIT").is_some() {
        eprintln!("breastplate section_fit front={front_fit:?} back={back_fit:?}");
    }
    let thickness = design.wall_thickness.metres();
    let front_normals = vertex_normals(&front.positions, &front.faces)?;
    let back_normals = vertex_normals(&back.positions, &back.faces)?;
    let front_min = front
        .positions
        .iter()
        .zip(front_normals)
        .flat_map(|(p, n)| {
            [
                dot(*p, wearer.frame.front),
                dot(add(*p, scale(n, thickness)), wearer.frame.front),
            ]
        })
        .fold(f32::INFINITY, f32::min);
    let back_max = back
        .positions
        .iter()
        .zip(back_normals)
        .flat_map(|(p, n)| {
            [
                dot(*p, wearer.frame.front),
                dot(add(*p, scale(n, thickness)), wearer.frame.front),
            ]
        })
        .fold(f32::NEG_INFINITY, f32::max);
    let required = design.plate_gap.metres();
    let correction = ((required - (front_min - back_max)) * 0.5).max(0.0);
    if correction > 0.0 {
        translate(&mut front, scale(wearer.frame.front, correction));
        translate(&mut back, scale(wearer.frame.front, -correction));
    }
    Ok((front, back))
}

fn combine(front: SolidMesh, back: SolidMesh) -> SolidMesh {
    let offset = front.positions.len() as u32;
    let front_mid_count = front
        .source_mid_indices
        .iter()
        .copied()
        .max()
        .map_or(0, |index| index + 1);
    let mut positions = front.positions;
    positions.extend(back.positions);
    let mut normals = front.normals;
    normals.extend(back.normals);
    let mut indices = front.indices;
    indices.extend(back.indices.into_iter().map(|index| index + offset));
    let mut source_mid_indices = front.source_mid_indices;
    source_mid_indices.extend(
        back.source_mid_indices
            .into_iter()
            .map(|index| index + front_mid_count),
    );
    SolidMesh {
        positions,
        normals,
        indices,
        source_mid_indices,
    }
}

fn source_sample(point: [f32; 3], wearer: Wearer<'_>, eligible_faces: &[usize]) -> SourceSample {
    let target = local(point, wearer.frame);
    let mut best = None::<(f32, SourceSample)>;
    for face_index in eligible_faces.iter().copied() {
        let face = wearer.source_faces[face_index];
        let indices = face.map(|index| index as usize);
        let triangle = indices.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index].position,
                wearer.frame,
            )
        });
        let weights = closest_triangle_weights(target, triangle);
        let closest = triangle
            .into_iter()
            .zip(weights)
            .fold([0.0; 3], |sum, (vertex, weight)| {
                add(sum, scale(vertex, weight))
            });
        let distance = dot(sub(closest, target), sub(closest, target));
        if best.is_none_or(|current| distance < current.0) {
            best = Some((
                distance,
                SourceSample {
                    face: face_index,
                    weights,
                },
            ));
        }
    }
    best.expect("validated torso has faces").1
}

fn closest_triangle_weights(point: [f32; 3], triangle: [[f32; 3]; 3]) -> [f32; 3] {
    let [a, b, c] = triangle;
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(point, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = sub(point, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = sub(point, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let denominator = (va + vb + vc).recip();
    let v = vb * denominator;
    let w = vc * denominator;
    [1.0 - v - w, v, w]
}

fn sampled_uv(sample: SourceSample, surface: &TorsoSurface) -> [f32; 2] {
    surface.clearance_mesh.enclosure_texcoord_faces[sample.face]
        .into_iter()
        .zip(sample.weights)
        .fold([0.0; 2], |sum, (index, weight)| {
            let uv = surface.clearance_mesh.enclosure_texcoords[index as usize];
            [sum[0] + uv[0] * weight, sum[1] + uv[1] * weight]
        })
}

fn sampled_skin(sample: SourceSample, surface: &TorsoSurface) -> ([u32; 8], [f32; 8]) {
    let mut merged = BTreeMap::<u32, f32>::new();
    for (vertex, source_weight) in surface.clearance_mesh.enclosure_faces[sample.face]
        .into_iter()
        .zip(sample.weights)
    {
        let vertex = vertex as usize;
        for (joint, weight) in surface.clearance_mesh.enclosure_joint_indices[vertex]
            .into_iter()
            .zip(surface.clearance_mesh.enclosure_joint_weights[vertex])
        {
            if weight > 1e-7 {
                *merged.entry(joint).or_default() += source_weight * weight;
            }
        }
    }
    let mut weights = merged.into_iter().collect::<Vec<_>>();
    weights.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    // Bevy currently consumes only JOINTS_0/WEIGHTS_0. Keep the strongest
    // four and renormalize them rather than exporting a silently lost tail.
    weights.truncate(4);
    let total = weights.iter().map(|v| v.1).sum::<f32>().max(1e-8);
    let mut joints = [0; 8];
    let mut result = [0.0; 8];
    for (slot, (joint, weight)) in weights.into_iter().enumerate() {
        joints[slot] = joint;
        result[slot] = weight / total;
    }
    (joints, result)
}

pub fn generate_breastplate(
    design: &BreastplateDesign,
    surface: &TorsoSurface,
) -> Result<GeneratedArmor, GenerateError> {
    validate_breastplate(design)?;
    if !valid_surface(surface) {
        return Err(GenerateError::InvalidSurface);
    }
    let base_positions = surface
        .vertices
        .iter()
        .map(|v| v.position)
        .collect::<Vec<_>>();
    let base_semantic = surface
        .vertices
        .iter()
        .map(|v| [v.lateral, v.vertical])
        .collect::<Vec<_>>();
    let base_coronal = surface
        .coronal_anchors
        .iter()
        .map(|v| v.depth)
        .collect::<Vec<_>>();
    let base_wearer = wearer(
        &base_positions,
        &base_semantic,
        surface.front,
        surface.upper_rig_anchors,
        &base_coronal,
        &surface.clearance_mesh.base,
        &surface.clearance_mesh.enclosure_faces,
        &surface.clearance_mesh.enclosure_torso_faces,
        &surface.shoulder_envelope,
    )?;
    let (front_mid, back_mid) = build_pair(base_wearer, design)?;
    let mid_positions = front_mid
        .positions
        .iter()
        .chain(&back_mid.positions)
        .copied()
        .collect::<Vec<_>>();
    let base = combine(
        solidify(front_mid, design.wall_thickness.metres())?,
        solidify(back_mid, design.wall_thickness.metres())?,
    );
    let source_face_indices = surface
        .clearance_mesh
        .enclosure_faces
        .iter()
        .copied()
        .enumerate()
        .map(|(index, face)| (face, index))
        .collect::<BTreeMap<_, _>>();
    let eligible_faces = surface
        .clearance_mesh
        .enclosure_torso_faces
        .iter()
        .map(|face| {
            source_face_indices
                .get(face)
                .copied()
                .ok_or(GenerateError::InvalidSurface)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let samples = mid_positions
        .iter()
        .map(|point| source_sample(*point, base_wearer, &eligible_faces))
        .collect::<Vec<_>>();
    let solid_samples = base
        .source_mid_indices
        .iter()
        .map(|index| samples[*index])
        .collect::<Vec<_>>();
    let texcoords = solid_samples
        .iter()
        .map(|sample| sampled_uv(*sample, surface))
        .collect::<Vec<_>>();
    let skin = solid_samples
        .iter()
        .map(|sample| sampled_skin(*sample, surface))
        .collect::<Vec<_>>();
    let morphs = surface
        .morphs
        .iter()
        .zip(&surface.morph_semantic_coordinates)
        .zip(&surface.morph_fronts)
        .zip(&surface.morph_upper_rig_anchors)
        .zip(&surface.morph_coronal_depths)
        .zip(&surface.clearance_mesh.morphs)
        .zip(&surface.morph_shoulder_envelopes)
        .map(
            |((((((morph, semantic), front), anchors), coronal), clearance), shoulders)| {
                let wearer = wearer(
                    &morph.positions,
                    semantic,
                    *front,
                    *anchors,
                    coronal,
                    clearance,
                    &surface.clearance_mesh.enclosure_faces,
                    &surface.clearance_mesh.enclosure_torso_faces,
                    shoulders,
                )?;
                let (front_mid, back_mid) = build_pair(wearer, design)?;
                let target = combine(
                    solidify(front_mid, design.wall_thickness.metres())?,
                    solidify(back_mid, design.wall_thickness.metres())?,
                );
                if target.indices != base.indices || target.positions.len() != base.positions.len()
                {
                    return Err(GenerateError::InvalidSurface);
                }
                Ok(ArmorMorph {
                    name: morph.name.clone(),
                    direct_positions: target.positions.clone(),
                    position_deltas: target
                        .positions
                        .iter()
                        .zip(&base.positions)
                        .map(|(a, b)| sub(*a, *b))
                        .collect(),
                    normal_deltas: target
                        .normals
                        .iter()
                        .zip(&base.normals)
                        .map(|(a, b)| sub(*a, *b))
                        .collect(),
                })
            },
        )
        .collect::<Result<Vec<_>, GenerateError>>()?;
    Ok(GeneratedArmor {
        design_hash: breastplate_design_hash(design)?,
        surface_domain: surface.domain.clone(),
        positions: base.positions,
        normals: base.normals,
        texcoords,
        joint_indices: skin.iter().map(|v| v.0).collect(),
        joint_weights: skin.iter().map(|v| v.1).collect(),
        indices: base.indices,
        morphs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_interpolates_control_values() {
        for (x, y) in FRONT_HEIGHTS.into_iter().zip(FRONT_RADIUS_X) {
            assert!((cubic(x, &FRONT_HEIGHTS, &FRONT_RADIUS_X, false) - y).abs() < 1e-6);
        }
    }

    #[test]
    fn neckline_has_bilateral_symmetry_and_a_round_center() {
        let design = BreastplateDesign::default();
        assert_eq!(front_top_y(-0.37, &design), front_top_y(0.37, &design));
        assert!((front_top_y(0.001, &design) - front_top_y(0.0, &design)).abs() < 1e-5);
        assert!(front_top_y(0.5, &design) > front_top_y(0.0, &design));
    }
}
