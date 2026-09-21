//! Garments that start as a surface fitted around the wearer rather than as
//! flat panels to sew. Sewn from flat panels, a hood slides off the head: its
//! zero-length seams close by the shortest way round, under the chin.
use super::*;
use crate::device_coif::{CoifCarrier, fitted_coif_carrier};
use adventuresim_armor_model::{CoifDesign, PartFrame};
use fabelgeist_cloth::{GarmentMesh, topology};
use fabelgeist_math::Vec2;
use std::collections::{HashMap, HashSet};

/// Head-frame lateral coordinates this close to zero lie on the centre line.
const CENTRE_TOLERANCE_M: f32 = 1e-5;
/// Conjugate gradient steps for the conformal starting map.
const CONFORMAL_ITERATIONS: usize = 4000;
/// Local/global rounds that pull the conformal map towards true size.
const RIGID_ROUNDS: usize = 24;
/// Conjugate gradient steps per round, warm-started from the last one.
const RIGID_ITERATIONS: usize = 400;
const SOLVER_TOLERANCE: f64 = 1e-14;

/// The coif's cloth mesh, fitted around the wearer described by `input`.
pub(super) fn coif(
    input: &DrapeInput,
    design: &CoifDesign,
    fabric: &Fabric,
) -> Result<GarmentMesh> {
    let normals = normals(&input.positions, &input.faces);
    let wearer = crate::armor_frames::Wearer {
        faces: &input.faces,
        positions: &input.positions,
        normals: &normals,
        joint_indices: &input.indices,
        joint_weights: &input.weights,
        joint_names: &input.names,
        joints: &input.joints,
    };
    let (frame, carrier) = fitted_coif_carrier(crate::armor_gpu()?, &wearer, design)?;
    surface_mesh(&frame, &carrier, fabric.density)
}

/// A cloth mesh whose rest shape is the fitted surface itself.
///
/// The surface is a ring around the face, so its mail texture cannot lie flat
/// in one piece. It is cut once up the front -- down the forehead to the face
/// opening, and from under the chin down the front flap -- and the copies
/// along the cut are sewn back together.
pub(super) fn surface_mesh(
    frame: &PartFrame,
    carrier: &CoifCarrier,
    density: f32,
) -> Result<GarmentMesh> {
    let count = carrier.positions.len();
    anyhow::ensure!(
        count > 0
            && carrier.indices.len().is_multiple_of(3)
            && carrier.indices.iter().all(|&i| (i as usize) < count),
        "invalid fitted surface"
    );
    let whole: Vec<[u32; 3]> = carrier.indices.as_chunks::<3>().0.to_vec();
    let cut = FrontCut::new(&carrier.positions, &whole);
    anyhow::ensure!(
        !cut.seams.is_empty(),
        "the fitted surface has no front centre line"
    );
    let local: Vec<[f32; 3]> = cut
        .origins
        .iter()
        .map(|&origin| carrier.positions[origin as usize])
        .collect();
    let positions: Vec<Vec3> = local.iter().map(|&p| vector(frame.point(p))).collect();
    let back_hem = local
        .iter()
        .enumerate()
        .filter(|(_, p)| p[0].abs() <= CENTRE_TOLERANCE_M && p[2] < 0.0)
        .min_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map(|(index, _)| index as u32)
        .context("the fitted surface has no back centre line")?;

    let mut mesh = GarmentMesh {
        material: unwrap(&positions, &cut.faces, [0, back_hem])?,
        positions,
        triangles: cut.faces,
        seams: cut.seams,
        panel_offsets: vec![0, local.len() as u32],
        panel_names: vec!["coif".into()],
        ..Default::default()
    };
    let stretch = topology::build(&mesh.triangles);
    mesh.rest_lengths = stretch
        .edges
        .iter()
        .map(|&[a, b]| (mesh.positions[a as usize] - mesh.positions[b as usize]).length())
        .collect();
    mesh.edges = stretch.edges;
    // Hinges come from the uncut surface, so the cut bends like the cloth
    // around it instead of creasing. Originals keep their indices.
    for bend in topology::build(&whole).bends {
        let points = bend.particles().map(|i| mesh.positions[i as usize]);
        let Some(weights) = topology::bending_weights(points) else {
            continue;
        };
        let rest = points
            .iter()
            .zip(&weights)
            .fold(Vec3::default(), |sum, (p, &k)| sum + *p * k)
            .length();
        mesh.bends.push(bend);
        mesh.bend_weights.push([
            weights[0], weights[1], weights[2], weights[3], rest, 0.0, 0.0, 0.0,
        ]);
    }
    mesh.masses = topology::vertex_masses(&mesh.positions, &mesh.triangles, density);
    Ok(mesh)
}

/// Faces left of the centre line take copies of the front centre vertices.
struct FrontCut {
    faces: Vec<[u32; 3]>,
    /// The source vertex of every vertex; originals keep their own index.
    origins: Vec<u32>,
    /// Each copied vertex and its copy.
    seams: Vec<[u32; 2]>,
}

impl FrontCut {
    fn new(positions: &[[f32; 3]], faces: &[[u32; 3]]) -> Self {
        let left =
            |face: &[u32; 3]| face.iter().map(|&i| positions[i as usize][0]).sum::<f32>() < 0.0;
        let front_centre = |i: u32| {
            let p = positions[i as usize];
            p[0].abs() <= CENTRE_TOLERANCE_M && p[2] > 0.0
        };
        // Only a vertex both sides use is on the cut; copying one that only
        // left faces use would strand the original outside every triangle.
        let right_used: HashSet<u32> = faces
            .iter()
            .filter(|face| !left(face))
            .flatten()
            .copied()
            .collect();
        let mut origins: Vec<u32> = (0..positions.len() as u32).collect();
        let mut copies = HashMap::new();
        let mut seams = Vec::new();
        let mut cut_faces = Vec::with_capacity(faces.len());
        for face in faces {
            let mut face = *face;
            if left(&face) {
                for vertex in &mut face {
                    if front_centre(*vertex) && right_used.contains(vertex) {
                        *vertex = *copies.entry(*vertex).or_insert_with(|| {
                            let copy = origins.len() as u32;
                            origins.push(*vertex);
                            seams.push([*vertex, copy]);
                            copy
                        });
                    }
                }
            }
            cut_faces.push(face);
        }
        Self {
            faces: cut_faces,
            origins,
            seams,
        }
    }
}

/// One triangle's rest shape, for the unwrap.
struct Rest {
    face: [u32; 3],
    /// Gradient of each corner's hat function in the triangle's own plane.
    gradients: [[f64; 2]; 3],
    /// Square root of the area, so each row weighs by area.
    weight: f64,
}

/// A sparse least-squares row: coefficients on free unknowns, and the target.
type Row = (Vec<(usize, f64)>, f64);

/// Flatten a cut surface for its mail texture, in metres.
///
/// A conformal map (least-squares conformal maps) keeps ring shapes but lets
/// area drift towards the flaps and crown; a few as-rigid-as-possible rounds
/// then pull every triangle back towards its own size, so rings neither
/// shear nor swell. `pins` fix the orientation: the first maps to the origin
/// and the second straight below it.
fn unwrap(positions: &[Vec3], faces: &[[u32; 3]], pins: [u32; 2]) -> Result<Vec<Vec2>> {
    let rests = rest_triangles(positions, faces);
    anyhow::ensure!(!rests.is_empty(), "the fitted surface has no area");
    let mut flat = conformal(positions, &rests, pins)?;
    for _ in 0..RIGID_ROUNDS {
        flat = rigid_round(&rests, &flat, pins[0] as usize);
    }
    Ok(flat
        .into_iter()
        .map(|[u, v]| Vec2::new(u as f32, v as f32))
        .collect())
}

fn rest_triangles(positions: &[Vec3], faces: &[[u32; 3]]) -> Vec<Rest> {
    let point = |i: u32| {
        let p = positions[i as usize];
        [p.x as f64, p.y as f64, p.z as f64]
    };
    faces
        .iter()
        .filter_map(|&face| {
            let [a, b, c] = face.map(point);
            let e1 = sub(b, a);
            let e2 = sub(c, a);
            let normal = cross(e1, e2);
            let area2 = norm(normal);
            let length1 = norm(e1);
            if area2 < 1e-14 || length1 < 1e-12 {
                return None;
            }
            let x_axis = scale(e1, 1.0 / length1);
            let y_axis = cross(normal, x_axis);
            let y_axis = scale(y_axis, 1.0 / norm(y_axis));
            let q = [
                [0.0, 0.0],
                [length1, 0.0],
                [dot(e2, x_axis), dot(e2, y_axis)],
            ];
            // grad(hat_i) = rot90(q[i + 2] - q[i + 1]) / 2A, with rot90(x, y) = (-y, x).
            let gradients = std::array::from_fn(|corner| {
                let next = q[(corner + 1) % 3];
                let after = q[(corner + 2) % 3];
                [-(after[1] - next[1]) / area2, (after[0] - next[0]) / area2]
            });
            Some(Rest {
                face,
                gradients,
                weight: (area2 * 0.5).sqrt(),
            })
        })
        .collect()
}

/// The conformal starting map, oriented like the surface and scaled so its
/// typical triangle has its true size.
fn conformal(positions: &[Vec3], rests: &[Rest], pins: [u32; 2]) -> Result<Vec<[f64; 2]>> {
    let count = positions.len();
    let span = (positions[pins[0] as usize] - positions[pins[1] as usize]).length() as f64;
    anyhow::ensure!(span > 0.0, "texture unwrap pins coincide");
    // Variable 2i is u_i and 2i + 1 is v_i; pinned ones move to the targets.
    let pinned = |variable: usize| match variable {
        v if v == pins[0] as usize * 2 || v == pins[0] as usize * 2 + 1 => Some(0.0),
        v if v == pins[1] as usize * 2 => Some(0.0),
        v if v == pins[1] as usize * 2 + 1 => Some(-span),
        _ => None,
    };
    let mut free = vec![usize::MAX; count * 2];
    let mut unknowns = 0;
    for (variable, slot) in free.iter_mut().enumerate() {
        if pinned(variable).is_none() {
            *slot = unknowns;
            unknowns += 1;
        }
    }
    // Cauchy-Riemann per triangle: rot90(grad u) - grad v = 0.
    let mut rows: Vec<Row> = Vec::with_capacity(rests.len() * 2);
    for rest in rests {
        let mut row_x: Row = (Vec::new(), 0.0);
        let mut row_y: Row = (Vec::new(), 0.0);
        for (corner, g) in rest.gradients.iter().enumerate() {
            let vertex = rest.face[corner] as usize;
            let w = rest.weight;
            for (variable, x_coefficient, y_coefficient) in [
                (vertex * 2, -g[1] * w, g[0] * w),
                (vertex * 2 + 1, -g[0] * w, -g[1] * w),
            ] {
                match pinned(variable) {
                    Some(value) => {
                        row_x.1 -= x_coefficient * value;
                        row_y.1 -= y_coefficient * value;
                    }
                    None => {
                        row_x.0.push((free[variable], x_coefficient));
                        row_y.0.push((free[variable], y_coefficient));
                    }
                }
            }
        }
        rows.push(row_x);
        rows.push(row_y);
    }
    let x = least_squares(&rows, vec![0.0; unknowns], CONFORMAL_ITERATIONS);
    let value = |variable: usize| pinned(variable).unwrap_or_else(|| x[free[variable]]);
    let mut flat: Vec<[f64; 2]> = (0..count)
        .map(|i| [value(i * 2), value(i * 2 + 1)])
        .collect();

    // The rigid rounds fit rotations, so the start must not be mirrored.
    let signed: f64 = rests.iter().map(|rest| signed_area(&flat, rest.face)).sum();
    if signed < 0.0 {
        flat.iter_mut().for_each(|p| p[0] = -p[0]);
    }
    let mut scales: Vec<f64> = rests
        .iter()
        .filter_map(|rest| {
            let area = signed_area(&flat, rest.face).abs();
            (area > 0.0).then(|| rest.weight / area.sqrt())
        })
        .collect();
    anyhow::ensure!(!scales.is_empty(), "texture unwrap collapsed");
    scales.sort_by(f64::total_cmp);
    let to_metres = scales[scales.len() / 2];
    flat.iter_mut()
        .for_each(|p| *p = [p[0] * to_metres, p[1] * to_metres]);
    Ok(flat)
}

/// One local/global round: fit each triangle's nearest rotation to the
/// current map, then solve u and v for gradients that follow those rotations.
fn rigid_round(rests: &[Rest], flat: &[[f64; 2]], anchor: usize) -> Vec<[f64; 2]> {
    let count = flat.len();
    let free = |vertex: usize| if vertex < anchor { vertex } else { vertex - 1 };
    let mut rows: [Vec<Row>; 2] = [
        Vec::with_capacity(rests.len() * 2),
        Vec::with_capacity(rests.len() * 2),
    ];
    for rest in rests {
        let mut jacobian = [[0.0; 2]; 2];
        for (corner, g) in rest.gradients.iter().enumerate() {
            let p = flat[rest.face[corner] as usize];
            for axis in 0..2 {
                for k in 0..2 {
                    jacobian[axis][k] += p[axis] * g[k];
                }
            }
        }
        let angle = (jacobian[1][0] - jacobian[0][1]).atan2(jacobian[0][0] + jacobian[1][1]);
        let (sin, cos) = angle.sin_cos();
        let rotation = [[cos, -sin], [sin, cos]];
        for axis in 0..2 {
            for k in 0..2 {
                let mut row: Row = (Vec::new(), rotation[axis][k] * rest.weight);
                for (corner, g) in rest.gradients.iter().enumerate() {
                    let vertex = rest.face[corner] as usize;
                    let coefficient = g[k] * rest.weight;
                    if vertex == anchor {
                        row.1 -= coefficient * flat[anchor][axis];
                    } else {
                        row.0.push((free(vertex), coefficient));
                    }
                }
                rows[axis].push(row);
            }
        }
    }
    let mut next = flat.to_vec();
    for (axis, rows) in rows.iter().enumerate() {
        let start = (0..count)
            .filter(|&vertex| vertex != anchor)
            .map(|vertex| flat[vertex][axis])
            .collect();
        let solved = least_squares(rows, start, RIGID_ITERATIONS);
        for vertex in (0..count).filter(|&vertex| vertex != anchor) {
            next[vertex][axis] = solved[free(vertex)];
        }
    }
    next
}

/// Conjugate gradients on the normal equations, from `x`.
fn least_squares(rows: &[Row], mut x: Vec<f64>, iterations: usize) -> Vec<f64> {
    let apply = |x: &[f64]| -> Vec<f64> {
        rows.iter()
            .map(|(terms, _)| terms.iter().map(|&(j, k)| k * x[j]).sum())
            .collect()
    };
    let unknowns = x.len();
    let apply_transpose = |r: &[f64]| -> Vec<f64> {
        let mut out = vec![0.0; unknowns];
        for ((terms, _), &residual) in rows.iter().zip(r) {
            for &(j, k) in terms {
                out[j] += k * residual;
            }
        }
        out
    };
    let targets: Vec<f64> = rows.iter().map(|(_, rhs)| *rhs).collect();
    let mut residual: Vec<f64> = targets
        .iter()
        .zip(apply(&x))
        .map(|(target, value)| target - value)
        .collect();
    let mut gradient = apply_transpose(&residual);
    let mut direction = gradient.clone();
    let mut gamma: f64 = gradient.iter().map(|g| g * g).sum();
    let limit = apply_transpose(&targets)
        .iter()
        .map(|g| g * g)
        .sum::<f64>()
        .max(f64::MIN_POSITIVE)
        * SOLVER_TOLERANCE;
    for _ in 0..iterations {
        if gamma <= limit {
            break;
        }
        let q = apply(&direction);
        let curvature: f64 = q.iter().map(|v| v * v).sum();
        if curvature <= 0.0 {
            break;
        }
        let alpha = gamma / curvature;
        for (xi, di) in x.iter_mut().zip(&direction) {
            *xi += alpha * di;
        }
        for (ri, qi) in residual.iter_mut().zip(&q) {
            *ri -= alpha * qi;
        }
        gradient = apply_transpose(&residual);
        let next: f64 = gradient.iter().map(|g| g * g).sum();
        let beta = next / gamma;
        for (di, gi) in direction.iter_mut().zip(&gradient) {
            *di = gi + beta * *di;
        }
        gamma = next;
    }
    x
}

fn signed_area(flat: &[[f64; 2]], [a, b, c]: [u32; 3]) -> f64 {
    let [ta, tb, tc] = [a, b, c].map(|i| flat[i as usize]);
    ((tb[0] - ta[0]) * (tc[1] - ta[1]) - (tb[1] - ta[1]) * (tc[0] - ta[0])) * 0.5
}

type V = [f64; 3];
fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V, s: f64) -> V {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::gpu::{
        COIF_DRAPE_SECTIONS, FIT_PROFILE_WORD, frame_words, record_coif,
    };
    use std::f32::consts::TAU;

    fn head() -> PartFrame {
        PartFrame {
            origin: [0.0, 1.62, 0.02],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.075, 0.115, 0.1],
        }
    }

    /// The coif's own frame, then a head-proportioned drape: a neck boundary
    /// below the chin and flaps hanging straight down from it.
    fn fit_words(design: &CoifDesign, frame: &PartFrame) -> Vec<f32> {
        let own = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: frame.half_extents,
        };
        let mut words = frame_words(&own).to_vec();
        words.resize(FIT_PROFILE_WORD, 0.0);
        let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres();
        let [width, height, depth] = frame.half_extents.map(|e| e + gap);
        let front_height = -height * (1.0 + 0.65 * design.neck_coverage.unit());
        let side_height = front_height + height * 0.34;
        let back_height = front_height + height * 0.22;
        let (front_depth, back_depth) = (depth * 0.38, -depth * 1.32);
        words.extend([
            front_height,
            side_height,
            back_height,
            width * 1.5,
            -depth * 0.36,
            front_depth,
            back_depth,
        ]);
        // Each flap hangs from the neck columns a twelfth of a turn either
        // side of its centre line.
        let edge = (TAU / 12.0).cos().powi(2);
        for (end, depth, length, sign) in [
            (
                front_height,
                front_depth,
                design.front_flap_length.metres(),
                1.0,
            ),
            (
                back_height,
                back_depth,
                design.back_flap_length.metres(),
                -1.0,
            ),
        ] {
            let top = side_height + (end - side_height) * edge;
            for i in 0..COIF_DRAPE_SECTIONS {
                let t = i as f32 / (COIF_DRAPE_SECTIONS - 1) as f32;
                let z = depth + sign * length * (1.0 - t) * 0.25;
                words.extend([end - length + (top - end + length) * t, z, z]);
            }
        }
        words
    }

    fn coif_mesh() -> GarmentMesh {
        let gpu = crate::armor_gpu().unwrap();
        let design = CoifDesign::default();
        let frame = head();
        let fit = gpu.upload(&fit_words(&design, &frame)).unwrap();
        let placement = gpu.upload(&frame_words(&frame)).unwrap();
        let mut batch = gpu.batch("coif carrier test");
        let part = record_coif(gpu, &mut batch, &design, &fit, &placement).unwrap();
        batch.submit();
        let carrier = CoifCarrier::read(gpu, &part).unwrap();
        surface_mesh(&frame, &carrier, Fabric::CHAINMAIL.density).unwrap()
    }

    #[test]
    fn fitted_coif_is_one_cloth_surface_sewn_up_the_front() {
        let mesh = coif_mesh();
        assert!(!mesh.seams.is_empty());
        for &[a, b] in &mesh.seams {
            assert_eq!(mesh.positions[a as usize], mesh.positions[b as usize]);
            assert!(mesh.positions[a as usize].z > head().origin[2]);
        }
        assert!(
            mesh.masses.iter().all(|m| *m > 0.0),
            "a vertex is in no triangle"
        );
        assert!(mesh.rest_lengths.iter().all(|l| *l > 0.0));
        assert!(!mesh.bends.is_empty());
        let frame = head();
        let top = mesh.positions.iter().map(|p| p.y).fold(f32::MIN, f32::max);
        assert!(
            top > frame.origin[1] + frame.half_extents[1],
            "the crown is at {top}"
        );
    }

    #[test]
    fn mail_texture_lies_flat_without_folds_or_stretch() {
        let mesh = coif_mesh();
        let mut orientation = [0usize; 2];
        let mut ratios = Vec::new();
        for &[a, b, c] in &mesh.triangles {
            let [pa, pb, pc] = [a, b, c].map(|i| mesh.positions[i as usize]);
            let surface = (pb - pa).cross(pc - pa).length() * 0.5;
            if surface < 1e-9 {
                continue;
            }
            let [ta, tb, tc] = [a, b, c].map(|i| mesh.material[i as usize]);
            let flat = ((tb.x - ta.x) * (tc.y - ta.y) - (tb.y - ta.y) * (tc.x - ta.x)) * 0.5;
            orientation[usize::from(flat < 0.0)] += 1;
            ratios.push(flat.abs() / surface);
        }
        assert!(
            orientation[0] == 0 || orientation[1] == 0,
            "the texture folds over: {orientation:?} triangles each way"
        );
        ratios.sort_by(f32::total_cmp);
        let median = ratios[ratios.len() / 2];
        let low = ratios[ratios.len() / 20];
        let high = ratios[ratios.len() * 19 / 20];
        assert!((0.8..1.25).contains(&median), "median area ratio {median}");
        assert!(low > 0.5 && high < 2.0, "area ratios span {low}..{high}");
    }
}
