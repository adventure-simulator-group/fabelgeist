//! Boundary-first breastplate generation.
//!
//! The main plate owns one canonical constrained triangulation. Every fixed
//! canonical vertex is evaluated directly on one smooth, body-supported curve
//! network; morphs never lift a boundary cage harmonically in three dimensions.
//! The torso crown stays low-frequency and defensive; wearer samples constrain
//! clearance and semantic anchors rather than imprinting anatomy into the form.

use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    ArmorMorph, BreastplateDesign, GenerateError, GeneratedArmor, Millimeters, Permille,
    TorsoShoulderSample, TorsoSurface, TorsoUpperRigAnchors, breastplate_design_hash,
    breastplate_topology::{BreastplateBoundaryEdge, CanonicalBreastplateTopology},
    validate_breastplate,
};

// Three physical rows match the target-measured ruled skirt aspect ratio; more
// rows make the wide wearer's lateral cells unnecessarily slender.
const SKIRT_ROWS: usize = 3;
const ANTERIOR_CLEARANCE_Q: f32 = 0.58;

fn topology_acceptance_designs(requested: &BreastplateDesign) -> Vec<BreastplateDesign> {
    let mut designs = vec![requested.clone()];
    for extreme in [
        BreastplateDesign {
            neck_width: Permille(200),
            neck_depth: Permille(0),
            arm_opening_depth: Permille(100),
            waist_width: Permille(550),
            stomach_height: Permille(0),
            rigidity: Permille(0),
            wrap: Permille(0),
            crown: Millimeters(0),
            skirt_length: Permille(40),
            skirt_flare: Millimeters(0),
            ..BreastplateDesign::default()
        },
        BreastplateDesign {
            neck_width: Permille(700),
            neck_depth: Permille(500),
            arm_opening_depth: Permille(600),
            waist_width: Permille(1_000),
            stomach_height: Permille(500),
            rigidity: Permille(1_000),
            wrap: Permille(1_000),
            crown: Millimeters(80),
            skirt_length: Permille(250),
            skirt_flare: Millimeters(120),
            ..BreastplateDesign::default()
        },
    ] {
        if !designs.contains(&extreme) {
            designs.push(extreme);
        }
    }
    designs
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], factor: f32) -> [f32; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
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

fn length(vector: [f32; 3]) -> f32 {
    dot(vector, vector).sqrt()
}

fn normalized(vector: [f32; 3]) -> Result<[f32; 3], GenerateError> {
    let magnitude = length(vector);
    (magnitude > 1e-8)
        .then(|| scale(vector, magnitude.recip()))
        .ok_or(GenerateError::Degenerate)
}

#[derive(Clone, Copy)]
struct Frame {
    lateral: [f32; 3],
    vertical: [f32; 3],
    front: [f32; 3],
}

fn frame(
    vertices: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    front_hint: [f32; 3],
) -> Result<Frame, GenerateError> {
    let front_hint = normalized(front_hint)?;
    let count = vertices.len() as f32;
    let mean_level = semantic_coordinates
        .iter()
        .map(|coordinate| coordinate[1])
        .sum::<f32>()
        / count;
    let mean = vertices.iter().fold([0.0; 3], |sum, point| {
        add(sum, scale(*point, count.recip()))
    });
    let variance = semantic_coordinates
        .iter()
        .map(|coordinate| (coordinate[1] - mean_level).powi(2))
        .sum::<f32>();
    let regression =
        semantic_coordinates
            .iter()
            .zip(vertices)
            .fold([0.0; 3], |sum, (coordinate, position)| {
                add(
                    sum,
                    scale(
                        sub(*position, mean),
                        (coordinate[1] - mean_level) / variance.max(1e-8),
                    ),
                )
            });
    let vertical = normalized(sub(
        regression,
        scale(front_hint, dot(regression, front_hint)),
    ))?;
    // Preserve the authored anterior direction. Rig anchor arrays are named
    // left/right, but their model-space ordering is not an axis convention;
    // using that ordering to flip lateral also flipped the entire plate onto
    // the character's back for rigs whose left side is +X.
    let lateral = normalized(cross(vertical, front_hint))?;
    let front = front_hint;
    Ok(Frame {
        lateral,
        vertical,
        front,
    })
}

fn coordinate(point: [f32; 3], frame: Frame) -> [f32; 3] {
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

fn world_with_upright_upper(point: [f32; 3], frame: Frame, upper_weight: f32) -> [f32; 3] {
    let original = world(point, frame);
    // The torso regression frame is intentionally wearer-specific, but its
    // small lateral pitch used to turn equal left/right yoke coordinates into
    // visibly different exported heights and depths.  Above the yoke seam,
    // remove only that lateral contribution while retaining the fitted
    // longitudinal (vertical/front) plane.  The smooth blend locks the
    // accepted lower torso and makes symmetric authored inputs genuinely
    // symmetric in the exported character frame.
    let center = world([0.0, point[1], point[2]], frame);
    let upright = [original[0], original[1], center[2]];
    [
        original[0] + (upright[0] - original[0]) * upper_weight,
        original[1] + (upright[1] - original[1]) * upper_weight,
        original[2] + (upright[2] - original[2]) * upper_weight,
    ]
}

fn regression_height(
    level: f32,
    vertices: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    vertical: [f32; 3],
) -> f32 {
    let count = vertices.len() as f32;
    let mean_level = semantic_coordinates
        .iter()
        .map(|coordinate| coordinate[1])
        .sum::<f32>()
        / count;
    let mean_height = vertices
        .iter()
        .map(|point| dot(*point, vertical))
        .sum::<f32>()
        / count;
    let variance = semantic_coordinates
        .iter()
        .map(|coordinate| (coordinate[1] - mean_level).powi(2))
        .sum::<f32>();
    let slope = semantic_coordinates
        .iter()
        .zip(vertices)
        .map(|(coordinate, point)| {
            (coordinate[1] - mean_level) * (dot(*point, vertical) - mean_height)
        })
        .sum::<f32>()
        / variance.max(1e-8);
    mean_height + slope * (level - mean_level)
}

fn local_support(
    target: [f32; 2],
    clearance_vertices: &[TorsoShoulderSample],
    frame: Frame,
) -> f32 {
    let mut samples = clearance_vertices
        .iter()
        .map(|sample| {
            let local = coordinate(sample.position, frame);
            let distance = (local[0] - target[0]).powi(2) + (local[1] - target[1]).powi(2);
            (distance, local[2])
        })
        .collect::<Vec<_>>();
    samples.sort_by(|left, right| left.0.total_cmp(&right.0));
    let nearest = samples.iter().take(6).collect::<Vec<_>>();
    let radius = nearest
        .last()
        .map_or(0.0, |sample| sample.0.sqrt())
        .max(0.010);
    nearest
        .into_iter()
        .map(|(distance, depth)| depth + (radius - distance.sqrt()).max(0.0) * 0.08)
        .fold(f32::NEG_INFINITY, f32::max)
}

fn mesh_front_support(
    target: [f32; 2],
    local_vertices: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> Option<f32> {
    let mut frontmost = None::<f32>;
    for face in faces {
        let [a, b, c] = face.map(|index| local_vertices[index as usize]);
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
            frontmost = Some(frontmost.map_or(depth, |existing| existing.max(depth)));
        }
    }
    frontmost
}

fn coronal_depth(
    canonical_level: f32,
    levels: &[f32],
    depths: &[f32],
) -> Result<f32, GenerateError> {
    if levels.len() != depths.len() || levels.is_empty() {
        eprintln!(
            "breastplate invalid coronal levels {} depths {}",
            levels.len(),
            depths.len()
        );
        return Err(GenerateError::InvalidSurface);
    }
    // Body section midpoints locate the wearer's coronal axis but are not
    // authored garment stations. Interpolating every sample with a natural
    // cubic produced an 11 mm trough over 9 mm of rail height on Body01=-1.
    // Fit the single low-frequency axis that minimizes squared sample error;
    // exact radial clearance remains the separate closed-body contract.
    let count = levels.len() as f32;
    let mean_level = levels.iter().sum::<f32>() / count;
    let mean_depth = depths.iter().sum::<f32>() / count;
    let covariance = levels
        .iter()
        .zip(depths)
        .map(|(level, depth)| (level - mean_level) * (depth - mean_depth))
        .sum::<f32>();
    let variance = levels
        .iter()
        .map(|level| (level - mean_level).powi(2))
        .sum::<f32>()
        .max(1e-8);
    Ok(mean_depth + covariance / variance * (canonical_level - mean_level))
}

fn canonical_coronal_level(v: f32) -> f32 {
    0.20 + 0.72 * v.clamp(0.0, 1.0).powf(0.65)
}

fn cubic(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2], t: f32) -> [f32; 2] {
    let one = 1.0 - t;
    [
        one.powi(3) * a[0]
            + 3.0 * one * one * t * b[0]
            + 3.0 * one * t * t * c[0]
            + t.powi(3) * d[0],
        one.powi(3) * a[1]
            + 3.0 * one * one * t * b[1]
            + 3.0 * one * t * t * c[1]
            + t.powi(3) * d[1],
    ]
}

/// Authored rounded arm opening between the outer shoulder and the true
/// mid-axillary endpoint.  Its endpoint tangents deliberately match the
/// shoulder bridge and short side transition: body anchors locate the curve,
/// but no intermediate anatomy sample is interpolated into its shape.
fn armhole_curve(shoulder: [f32; 2], mid_axillary: [f32; 2], t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    // A quarter ellipse distributes the complete turn continuously over the
    // physical-arc samples. The former quartic hook concentrated more than 25
    // degrees at one ordinary armscye station and rendered as a tooth.
    let angle = std::f32::consts::FRAC_PI_2 * t;
    let lateral_station = angle.sin();
    let vertical_station = 1.0 - angle.cos();
    [
        shoulder[0] + (mid_axillary[0] - shoulder[0]) * lateral_station,
        shoulder[1] + (mid_axillary[1] - shoulder[1]) * vertical_station,
    ]
}

fn normalized2(vector: [f32; 2]) -> [f32; 2] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1])
        .sqrt()
        .max(1e-8);
    [vector[0] / length, vector[1] / length]
}

/// Target-fitted, globally smooth crown lobe. A two-parameter beta profile was
/// fitted to the wearer-matched approved reference's normalized side
/// silhouette. A=2, b=1 places the unique maximum at v=.667 while retaining
/// the same single restrained lobe. Body samples remain inequality constraints
/// and never become interpolated anatomical stations.
fn longitudinal_crown_shape(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    const A: f32 = 2.0;
    const PEAK_V: f32 = A / (A + 1.0);
    let peak = PEAK_V.powf(A) * (1.0 - PEAK_V);
    v.powf(A) * (1.0 - v) / peak
}

fn clamped_cubic_bspline_basis(count: usize, t: f32) -> Vec<f32> {
    debug_assert!(count >= 4);
    let t = t.clamp(0.0, 1.0);
    let degree = 3;
    let mut knots = vec![0.0_f32; count + degree + 1];
    let interior = count - degree - 1;
    for (index, knot) in knots.iter_mut().enumerate() {
        *knot = if index <= degree {
            0.0
        } else if index >= count {
            1.0
        } else {
            (index - degree) as f32 / (interior + 1) as f32
        };
    }
    let mut basis = vec![0.0_f32; count + degree];
    if t >= 1.0 {
        basis[count - 1] = 1.0;
    } else {
        for index in 0..count {
            basis[index] = (knots[index] <= t && t < knots[index + 1]) as u8 as f32;
        }
    }
    for order in 1..=degree {
        let previous = basis.clone();
        for index in 0..count {
            let left_denominator = knots[index + order] - knots[index];
            let right_denominator = knots[index + order + 1] - knots[index + 1];
            let left = if left_denominator > 1e-8 {
                (t - knots[index]) / left_denominator * previous[index]
            } else {
                0.0
            };
            let right = if right_denominator > 1e-8 {
                (knots[index + order + 1] - t) / right_denominator * previous[index + 1]
            } else {
                0.0
            };
            basis[index] = left + right;
        }
    }
    basis.truncate(count);
    basis
}

#[derive(Clone, Copy)]
struct SemanticMeasurements {
    waist: [f32; 2],
    mid_axillary: [f32; 2],
    shoulder: [f32; 2],
    neck: [f32; 2],
    neck_center_height: f32,
    torso_to_shoulder: f32,
}

fn section_half_width(measurements: SemanticMeasurements, v: f32) -> f32 {
    let axillary_v = ((measurements.mid_axillary[1] - measurements.waist[1])
        / (measurements.neck[1] - measurements.waist[1]).max(1e-6))
    .clamp(0.05, 0.95);
    let (a, b, t) = if v <= axillary_v {
        (
            measurements.waist[0],
            measurements.mid_axillary[0],
            v / axillary_v,
        )
    } else {
        (
            measurements.mid_axillary[0],
            measurements.shoulder[0],
            (v - axillary_v) / (1.0 - axillary_v),
        )
    };
    let t = t * t * (3.0 - 2.0 * t);
    a + (b - a) * t
}

fn side_wrap_weight(v: f32) -> f32 {
    // The complete lower side follows the fitted coronal axis. Only the
    // armscye turns forward; its broad endpoint-flat exit avoids the bilateral
    // lobe caused by completing that turn over the old .18 span. An earlier
    // lower enter blend moved the rail 15 mm over one 9 mm station and was the
    // geometric source of the horizontal band.
    let leave = ((v - 0.50) / 0.42).clamp(0.0, 1.0);
    let leave = leave * leave * leave * (leave * (leave * 6.0 - 15.0) + 10.0);
    1.0 - leave
}

fn wrapped_lateral_x(parameter_x: f32, half_width: f32, v: f32) -> f32 {
    let sign = parameter_x.signum();
    let q = (parameter_x.abs() / half_width.max(1e-6)).clamp(0.0, 1.0);
    let outer = ((q - 0.75) / 0.25).clamp(0.0, 1.0);
    let outer = outer * outer * (3.0 - 2.0 * outer);
    let wrap = side_wrap_weight(v);
    let mapped = if parameter_x.abs() < half_width {
        parameter_x.abs() + 0.80 * wrap * outer * (half_width - parameter_x.abs())
    } else {
        parameter_x.abs()
    };
    sign * (mapped + 0.030 * wrap * outer)
}

fn inverse_wrapped_lateral_x(desired_x: f32, half_width: f32, v: f32) -> f32 {
    let sign = desired_x.signum();
    let desired = desired_x.abs();
    let mut low = 0.0_f32;
    let mut high = desired + 0.040;
    for _ in 0..32 {
        let middle = (low + high) * 0.5;
        if wrapped_lateral_x(middle, half_width, v).abs() < desired {
            low = middle;
        } else {
            high = middle;
        }
    }
    sign * (low + high) * 0.5
}

fn lateral_x_to_section_arc(x: f32, half_width: f32) -> f32 {
    let _ = half_width;
    x
}

fn section_arc_to_lateral_x(arc: f32, half_width: f32) -> f32 {
    let _ = half_width;
    arc
}

fn semantic_parameter_boundary(
    boundary: &[[f32; 3]],
    measurements: SemanticMeasurements,
    frame: Frame,
) -> Vec<[f32; 2]> {
    let waist_y = measurements.waist[1];
    let height = (measurements.neck[1] - waist_y).max(1e-6);
    boundary
        .iter()
        .map(|position| {
            let local = coordinate(*position, frame);
            let v = ((local[1] - waist_y) / height).clamp(0.0, 1.0);
            let half_width = section_half_width(measurements, v);
            [inverse_wrapped_lateral_x(local[0], half_width, v), local[1]]
        })
        .collect()
}

fn semantic_measurements(
    design: &BreastplateDesign,
    positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    anchors: TorsoUpperRigAnchors,
    frame: Frame,
) -> SemanticMeasurements {
    let neck = coordinate(anchors.neck_base, frame);
    let shoulders = anchors.shoulders.map(|point| coordinate(point, frame));
    let clavicles = anchors.clavicles.map(|point| coordinate(point, frame));
    let shoulder_half = (shoulders[1][0] - shoulders[0][0]).abs() * 0.5;
    let clavicle_half = (clavicles[1][0] - clavicles[0][0]).abs() * 0.5;
    let fallback_body_half = positions
        .iter()
        .map(|point| (dot(*point, frame.lateral) - neck[0]).abs())
        .fold(0.0_f32, f32::max);
    let mut physical_half_samples = positions
        .iter()
        .zip(semantic_coordinates)
        .filter(|(_, coordinate)| {
            (0.30..=1.02).contains(&coordinate[0].abs()) && (0.08..=0.62).contains(&coordinate[1])
        })
        .map(|(point, coordinate)| {
            (dot(*point, frame.lateral) - neck[0]).abs() / coordinate[0].abs()
        })
        .filter(|sample| sample.is_finite())
        .collect::<Vec<_>>();
    physical_half_samples.sort_by(f32::total_cmp);
    let body_half = physical_half_samples
        .get(physical_half_samples.len() / 2)
        .copied()
        .unwrap_or(fallback_body_half);
    let waist_height = regression_height(
        0.02 + design.stomach_height.unit() * 0.50,
        positions,
        semantic_coordinates,
        frame.vertical,
    );
    let lateral_envelope_at = |height: f32, half_window: f32| {
        let mut samples = positions
            .iter()
            .zip(semantic_coordinates)
            .filter(|(_, semantic)| semantic[0].abs() <= 1.05)
            .map(|(point, _)| coordinate(*point, frame))
            .filter(|point| (point[1] - height).abs() <= half_window)
            .map(|point| point[0].abs())
            .filter(|sample| sample.is_finite())
            .collect::<Vec<_>>();
        samples.sort_by(f32::total_cmp);
        samples
            .get(samples.len().saturating_mul(9) / 10)
            .copied()
            .unwrap_or(body_half)
    };
    let shoulder_height = shoulders.iter().map(|point| point[1]).sum::<f32>() * 0.5 + 0.043;
    // The neck-base anchor already lies on the wearer.  Raising the design
    // another 32 mm made the short bridge climb almost vertically before
    // dropping to the shoulder, which rendered as a floating triangular fin.
    // A small authored allowance produces a shallow U while leaving the thin
    // bridge aligned with the actual shoulder slope.
    let top_height = neck[1] - 0.015;
    let height = top_height - waist_height;
    let axillary_height =
        shoulder_height - height * (0.337 + design.arm_opening_depth.unit() * 0.100);
    // A front breastplate reaches the lateral centerline even when tapered;
    // width parameters change that side reach modestly rather than collapsing
    // the waist endpoint back onto the front abdominal projection.
    // Keep the lower plate closer to the target's modest taper. This remains
    // body-relative, so a parameter or morph cannot author an absolute-size
    // waist that fits only the neutral wearer.
    // Preserve the authored narrow front-apron proportion on shoulder-dominant
    // wearers, but transition toward the actual coronal torso reach when the
    // lower torso is wider than the shoulder anchors.  This is a wearer-shape
    // adaptation, not a target-pixel width: Body01=-1 remains at the authored
    // .782 scale while broad supported morphs cannot leave the body envelope
    // protruding through the armhole-to-waist side panel.
    let torso_to_shoulder = body_half / shoulder_half.max(1e-6);
    let broad_torso = ((torso_to_shoulder - 1.18) / 0.22).clamp(0.0, 1.0);
    let broad_torso = broad_torso * broad_torso * (3.0 - 2.0 * broad_torso);
    let authored_waist = height * (0.310 + design.waist_width.unit() * 0.055);
    let waist_body_half = lateral_envelope_at(waist_height, 0.022);
    let waist_half = (authored_waist + (body_half - authored_waist).max(0.0) * broad_torso).max(
        waist_body_half + design.clearance.metres() + design.wall_thickness.metres() * 0.5 + 0.006,
    );
    let shoulder_dominant = ((0.98 - torso_to_shoulder) / 0.08).clamp(0.0, 1.0);
    let shoulder_dominant = shoulder_dominant * shoulder_dominant * (3.0 - 2.0 * shoulder_dominant);
    let lateral_clearance = design.clearance.metres()
        + design.wall_thickness.metres() * 0.5
        + 0.008
        + 0.006 * shoulder_dominant;
    let authored_side = height * (0.286 + design.wrap.unit() * 0.024);
    let axillary_body_half = lateral_envelope_at(axillary_height, 0.022);
    let side_half = (authored_side + (body_half - authored_side).max(0.0) * broad_torso)
        .max(shoulder_half * 0.64)
        .max(axillary_body_half)
        + lateral_clearance;
    // Put the terminal on the front of the acromion/body envelope. The old
    // multiplier extended it beyond the body; the earlier over-correction put
    // it too far inward to support a genuine C-shaped armscye and rebound.
    let shoulder_terminal = (height * 0.341)
        .min(shoulder_half * 0.998)
        .max(clavicle_half * 0.86);
    let neck_half = (height * (0.241 + design.neck_width.unit() * 0.058))
        .max(clavicle_half * 0.54)
        .min(shoulder_terminal * 0.80);
    let neck_depth = height * (0.072 + design.neck_depth.unit() * 0.027);
    SemanticMeasurements {
        waist: [waist_half, waist_height],
        mid_axillary: [side_half, axillary_height],
        shoulder: [shoulder_terminal, shoulder_height],
        neck: [neck_half, top_height],
        neck_center_height: top_height - neck_depth,
        torso_to_shoulder,
    }
}

fn semantic_boundary(
    semantic_ranges: &[crate::breastplate_topology::SemanticBoundaryRange],
    design: &BreastplateDesign,
    positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    anchors: TorsoUpperRigAnchors,
    clearance_vertices: &[TorsoShoulderSample],
    frame: Frame,
) -> Vec<[f32; 3]> {
    let measurements =
        semantic_measurements(design, positions, semantic_coordinates, anchors, frame);
    let upper_height = measurements.neck[1] - measurements.waist[1];
    let armhole_terminal = [
        measurements.mid_axillary[0],
        measurements.mid_axillary[1] - upper_height * 0.020,
    ];
    let mut boundary_local =
        vec![[0.0; 3]; semantic_ranges.iter().map(|range| range.segments).sum()];
    // The body verifier uses exact nearest projected vertices whereas the
    // generator uses a six-sample support envelope. Keep a small modelling
    // tolerance between those two conservative approximations.
    let clearance = design.clearance.metres() + design.wall_thickness.metres() * 0.5 + 0.0085;
    for range in semantic_ranges {
        let curve = |t: f32| match range.edge {
            BreastplateBoundaryEdge::Neck => {
                let centered = 2.0 * t - 1.0;
                // Authored U half-profile with a shallow center and a smooth
                // vertical rise into the bridge.
                let half = cubic(
                    [0.0, 0.0],
                    [0.92, 0.0],
                    [1.0, 0.0],
                    [1.0, 1.0],
                    centered.abs(),
                );
                let x = centered.signum() * measurements.neck[0] * half[0];
                let bowl = half[1];
                let y = measurements.neck_center_height
                    + (measurements.neck[1] - measurements.neck_center_height) * bowl;
                [x, y]
            }
            BreastplateBoundaryEdge::RightShoulder => {
                let arm_tangent = normalized2([
                    armhole_terminal[0] - measurements.shoulder[0],
                    (armhole_terminal[1] - measurements.shoulder[1]) * 0.50,
                ]);
                let mut point = cubic(
                    measurements.neck,
                    [measurements.neck[0], measurements.neck[1] + 0.0062],
                    [
                        measurements.shoulder[0] - arm_tangent[0] * 0.006,
                        measurements.shoulder[1] - arm_tangent[1] * 0.006,
                    ],
                    measurements.shoulder,
                    t,
                );
                let shape = (16.0 * t * t * (1.0 - t) * (1.0 - t)).powi(2);
                point[0] += 0.020 * shape * (2.0 * t - 1.0);
                point[1] += 0.003 * shape;
                point
            }
            BreastplateBoundaryEdge::RightArmhole => {
                armhole_curve(measurements.shoulder, armhole_terminal, t)
            }
            BreastplateBoundaryEdge::RightSide => cubic(
                armhole_terminal,
                [armhole_terminal[0] + 0.008, armhole_terminal[1] - 0.0044],
                [
                    measurements.waist[0] + 0.004,
                    measurements.waist[1] + (armhole_terminal[1] - measurements.waist[1]) * 0.30,
                ],
                measurements.waist,
                t,
            ),
            BreastplateBoundaryEdge::Waist => {
                let x = measurements.waist[0] * (1.0 - 2.0 * t);
                [
                    x,
                    measurements.waist[1] - 0.004 * (std::f32::consts::PI * t).sin(),
                ]
            }
            BreastplateBoundaryEdge::LeftSide => {
                let p = cubic(
                    measurements.waist,
                    [
                        measurements.waist[0] + 0.004,
                        measurements.waist[1]
                            + (armhole_terminal[1] - measurements.waist[1]) * 0.30,
                    ],
                    [armhole_terminal[0] + 0.008, armhole_terminal[1] - 0.0044],
                    armhole_terminal,
                    t,
                );
                [-p[0], p[1]]
            }
            BreastplateBoundaryEdge::LeftArmhole => {
                let p = armhole_curve(measurements.shoulder, armhole_terminal, 1.0 - t);
                [-p[0], p[1]]
            }
            BreastplateBoundaryEdge::LeftShoulder => {
                let arm_tangent = normalized2([
                    armhole_terminal[0] - measurements.shoulder[0],
                    (armhole_terminal[1] - measurements.shoulder[1]) * 0.50,
                ]);
                let u = 1.0 - t;
                let mut p = cubic(
                    measurements.shoulder,
                    [
                        measurements.shoulder[0] - arm_tangent[0] * 0.006,
                        measurements.shoulder[1] - arm_tangent[1] * 0.006,
                    ],
                    [measurements.neck[0], measurements.neck[1] + 0.0062],
                    measurements.neck,
                    t,
                );
                let shape = (16.0 * u * u * (1.0 - u) * (1.0 - u)).powi(2);
                p[0] += 0.020 * shape * (2.0 * u - 1.0);
                p[1] += 0.003 * shape;
                [-p[0], p[1]]
            }
        };
        let dense = (0..=256)
            .map(|step| curve(step as f32 / 256.0))
            .collect::<Vec<_>>();
        let mut lengths = vec![0.0_f32];
        for pair in dense.windows(2) {
            lengths.push(
                lengths.last().copied().unwrap()
                    + ((pair[1][0] - pair[0][0]).powi(2) + (pair[1][1] - pair[0][1]).powi(2))
                        .sqrt(),
            );
        }
        let total = *lengths.last().unwrap();
        for offset in 0..range.segments {
            let fraction = offset as f32 / range.segments as f32;
            let fraction = if matches!(
                range.edge,
                BreastplateBoundaryEdge::RightShoulder | BreastplateBoundaryEdge::LeftShoulder
            ) {
                let exponent = (0.95
                    + ((design.neck_width.unit() - 0.26) / 0.44).clamp(0.0, 1.0) * 0.05)
                    .clamp(0.95, 1.0);
                fraction.powf(exponent)
            } else {
                fraction
            };
            let target = total * fraction;
            let upper = lengths.partition_point(|length| *length < target).min(256);
            let lower = upper.saturating_sub(1);
            let fraction = (target - lengths[lower]) / (lengths[upper] - lengths[lower]).max(1e-8);
            let x = dense[lower][0] + (dense[upper][0] - dense[lower][0]) * fraction;
            let y = dense[lower][1] + (dense[upper][1] - dense[lower][1]) * fraction;
            let depth = local_support([x, y], clearance_vertices, frame) + clearance;
            boundary_local[range.start + offset] = [x, y, depth];
        }
    }
    // Nearest-body support is deliberately only a clearance constraint, not
    // the authored depth curve. Independent nearest samples contain small
    // Voronoi-cell steps that become a visible ripple when interpolated over
    // the plate. Low-pass the closed perimeter, then translate the entire
    // fitted curve forward just enough to remain above every raw constraint.
    let required_depths = boundary_local
        .iter()
        .map(|point| point[2])
        .collect::<Vec<_>>();
    let semantic_range = |edge: BreastplateBoundaryEdge| {
        semantic_ranges
            .iter()
            .find(|range| range.edge == edge)
            .expect("complete semantic boundary")
    };
    let neck = semantic_range(BreastplateBoundaryEdge::Neck);
    let right_shoulder = semantic_range(BreastplateBoundaryEdge::RightShoulder);
    let right_armhole = semantic_range(BreastplateBoundaryEdge::RightArmhole);
    let right_side = semantic_range(BreastplateBoundaryEdge::RightSide);
    let left_armhole = semantic_range(BreastplateBoundaryEdge::LeftArmhole);
    let left_shoulder = semantic_range(BreastplateBoundaryEdge::LeftShoulder);
    let upper = |index: usize| {
        (neck.start..neck.start + neck.segments).contains(&index)
            || (right_shoulder.start..right_shoulder.start + right_shoulder.segments)
                .contains(&index)
            || (right_armhole.start..right_armhole.start + right_armhole.segments).contains(&index)
            || (left_armhole.start..left_armhole.start + left_armhole.segments).contains(&index)
            || (left_shoulder.start..left_shoulder.start + left_shoulder.segments).contains(&index)
    };
    // Body points locate the openings, but never become yoke surface samples.
    // Fit one authored transverse crown between a center-front neck support
    // and the torso-side seam, then translate the whole yoke forward by the
    // single worst clearance deficit. This retains a non-anatomical defensive
    // form instead of reproducing bilateral body volumes.
    let side_depth = required_depths[right_side.start];
    let neck_center = neck.start + neck.segments / 2;
    let center_depth = required_depths[neck_center].max(side_depth + 0.035);
    for (index, point) in boundary_local.iter_mut().enumerate() {
        if upper(index) {
            let q = (point[0].abs() / measurements.mid_axillary[0].max(1e-6)).clamp(0.0, 1.0);
            let crown = (std::f32::consts::FRAC_PI_2 * q).cos();
            point[2] = side_depth + (center_depth - side_depth) * crown;
        }
    }
    let authored_yoke_lift = boundary_local
        .iter()
        .zip(&required_depths)
        .enumerate()
        .filter(|(index, _)| upper(*index))
        .map(|(_, (authored, required))| required - authored[2])
        .fold(0.0_f32, f32::max);
    for (index, point) in boundary_local.iter_mut().enumerate() {
        if upper(index) {
            point[2] += authored_yoke_lift;
        }
    }
    let raw_depths = boundary_local
        .iter()
        .map(|point| point[2])
        .collect::<Vec<_>>();
    let mut fitted_depths = raw_depths.clone();
    for _ in 0..96 {
        let previous = fitted_depths.clone();
        let count = previous.len();
        for index in 0..count {
            fitted_depths[index] = previous[index] * 0.5
                + (previous[(index + count - 1) % count] + previous[(index + 1) % count]) * 0.25;
        }
    }
    let conservative_lift = required_depths
        .iter()
        .zip(&fitted_depths)
        .map(|(required, fitted)| required - fitted)
        .fold(0.0_f32, f32::max);
    boundary_local
        .into_iter()
        .zip(fitted_depths)
        .map(|(mut point, fitted)| {
            point[2] = fitted + conservative_lift;
            world(point, frame)
        })
        .collect()
}

fn main_mid_surface(
    topology: &CanonicalBreastplateTopology,
    design: &BreastplateDesign,
    positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    body_faces: &[[u32; 3]],
    anchors: TorsoUpperRigAnchors,
    clearance_vertices: &[TorsoShoulderSample],
    clearance_faces: &[[u32; 3]],
    coronal_levels: &[f32],
    coronal_depths: &[f32],
    frame: Frame,
) -> Result<(Vec<[f32; 3]>, Vec<[f32; 3]>), GenerateError> {
    let boundary = semantic_boundary(
        topology.semantic_ranges(),
        design,
        positions,
        semantic_coordinates,
        anchors,
        clearance_vertices,
        frame,
    );
    let measurements =
        semantic_measurements(design, positions, semantic_coordinates, anchors, frame);
    let waist_y = measurements.waist[1];
    let neck_y = measurements.neck[1];
    let boundary_domain = semantic_parameter_boundary(&boundary, measurements, frame);
    let section_half_width = |v: f32| section_half_width(measurements, v);
    let domain = topology.semantic_domain_from_boundary_mapped(&boundary_domain, |point| {
        let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
        [
            wrapped_lateral_x(point[0], section_half_width(v), v),
            point[1],
        ]
    });
    let required_depth = design.clearance.metres() + design.wall_thickness.metres() * 0.5 + 0.010;
    let local_clearance_vertices = clearance_vertices
        .iter()
        .map(|sample| coordinate(sample.position, frame))
        .collect::<Vec<_>>();
    let local_body_vertices = positions
        .iter()
        .map(|position| coordinate(*position, frame))
        .collect::<Vec<_>>();
    // Silhouette calibration against the matched wearer showed the previous
    // crown amplitude at 1.37x the approved target. Preserve the same global
    // analytic locus and slope distribution while scaling only its amplitude.
    let authored_crown = design.crown.metres() * 1.20;
    let exact_support_at = |sample_x: f32, y: f32| {
        let torso = mesh_front_support([sample_x, y], &local_body_vertices, body_faces);
        let clearance =
            mesh_front_support([sample_x, y], &local_clearance_vertices, clearance_faces);
        torso
            .into_iter()
            .chain(clearance)
            .reduce(f32::max)
            .unwrap_or_else(|| local_support([sample_x, y], clearance_vertices, frame))
    };
    const TRANSVERSE_KNOTS: [f32; 4] = [0.0, 0.42, 0.72, 1.0];
    let desired_depth = |v: f32, q: f32| -> Result<f32, GenerateError> {
        let y = waist_y + (neck_y - waist_y) * v;
        let half_width = section_half_width(v);
        let coronal = coronal_depth(canonical_coronal_level(v), coronal_levels, coronal_depths)?;
        let support = |sample_q: f32| {
            let x = half_width * sample_q;
            0.5 * (exact_support_at(-x, y) + exact_support_at(x, y)) + required_depth
        };
        let top_q = ((v - 0.65) / 0.35).clamp(0.0, 1.0);
        let top_blend = top_q * top_q * (3.0 - 2.0 * top_q);
        // At torso stations the mid-axillary rail turns around the body near
        // the coronal section rather than sitting on its anterior projection.
        // The offset fades into the supported upper yoke.
        // Below the supported upper yoke the semantic side rail lies on the
        // body's true coronal midplane.  It must not be pulled behind that
        // plane merely to make a frontal reach metric look large: this rail is
        // what makes the plate visibly occupy the lateral half-section.
        let side = coronal * (1.0 - top_blend) + support(0.96) * top_blend;
        let side = side + 0.008 * (1.0 - v).powi(4);
        let shape = longitudinal_crown_shape(v);
        let mut center = support(0.0) + authored_crown * shape;
        center = center.max(side + 0.040);
        let rounded = side
            + (center - side)
                * (std::f32::consts::FRAC_PI_2 * q.clamp(0.0, 1.0))
                    .cos()
                    .max(0.0)
                    .powf(1.15);
        // An anterior ray is useful over the front torso but ceases to describe
        // clearance as the shell turns toward the lateral plane. Fade that
        // inequality smoothly there; closed-body parity checks the side.
        let lateral = ((q - ANTERIOR_CLEARANCE_Q) / (1.0 - ANTERIOR_CLEARANCE_Q)).clamp(0.0, 1.0);
        let front_weight = 1.0 - lateral * lateral * (3.0 - 2.0 * lateral);
        Ok(rounded + (support(q) - rounded).max(0.0) * front_weight)
    };
    // Each transverse landmark follows one global quadratic envelope from
    // waist to yoke. Dense body samples only choose the minimum bow needed to
    // remain outside; no anatomical interior station is interpolated.
    let mut profiles = [[0.0_f32; 3]; 4];
    for (profile, q) in profiles.iter_mut().zip(TRANSVERSE_KNOTS) {
        let start = desired_depth(0.0, q)?;
        let end = desired_depth(1.0, q)?;
        let mut bow = if q == 0.0 { authored_crown } else { 0.0 };
        // Only genuinely anterior rails use front-ray body samples as
        // longitudinal inequalities. Lateral rails turn around the body and
        // are governed by the coronal section plus closed-body parity.
        let mut inequality_bow = 0.0_f32;
        for sample in 1..64 {
            let v = sample as f32 / 64.0;
            let shape = longitudinal_crown_shape(v);
            let linear = start + (end - start) * v;
            inequality_bow = inequality_bow.max((desired_depth(v, q)? - linear) / shape.max(1e-6));
        }
        // A partial low-frequency lift is the lateral analogue of the radial
        // clearance constraint: enough to keep the fair rail outside without
        // promoting one anterior shoulder sample into a frontal apron.
        let inequality_weight = if q <= 0.42 {
            1.0
        } else {
            0.85 - 0.35 * ((q - 0.72) / 0.28).clamp(0.0, 1.0)
        };
        bow = bow.max(inequality_bow * inequality_weight);
        // Endpoint-adjacent support samples must not turn into a global
        // barrel.  This limit applies to every rail: lateral clearance is
        // radial/closed-body, so a near-waist anterior sample must never be
        // amplified by division through the crown profile's double root.
        // The residual obstacle correction below remains exact and local.
        // The target-fitted authored lobe may be deliberately restrained.
        // Clearance must therefore select its own smooth low-frequency bow,
        // rather than falling through to pointwise anatomical corrections.
        // Clearance samples must not triple the authored crown.  That made a
        // compact wearer's upper front support become a 16 cm beta-profile
        // bow whose tip sat at the neckline.  The control-lattice inequality
        // solve below supplies any remaining smooth clearance; this profile
        // remains the designed low-frequency crown.
        bow = bow.min(authored_crown * 1.25);
        if q >= 0.72 {
            // Longitudinal front-ray clearance is not a valid reason to pull
            // the lateral rail forward. Fade the global bow out across the
            // lateral quadrant so q=1 remains the true coronal endpoint; the
            // closed-body test, rather than an anterior projection, governs
            // clearance there.
            let lateral = ((q - 0.72) / 0.28).clamp(0.0, 1.0);
            let lateral = lateral * lateral * (3.0 - 2.0 * lateral);
            bow *= 1.0 - 0.5 * lateral;
        }
        *profile = [start, end, bow];
    }
    let torso_depth = |v: f32, physical_q: f32| -> Result<f32, GenerateError> {
        let transverse = profiles.map(|profile| {
            profile[0] + (profile[1] - profile[0]) * v + profile[2] * longitudinal_crown_shape(v)
        });
        let center = transverse[0].max(transverse[3] + 0.004);
        let side = transverse[3];
        // One analytic convex transverse crown is the authored surface. The
        // intermediate support rails select clearance but are deliberately
        // not interpolated: doing so reproduced anatomical lobes and exposed
        // their knot curvature as vertical ribs in grazing light.
        let mut depth = side
            + (center - side)
                * (std::f32::consts::FRAC_PI_2 * physical_q.clamp(0.0, 1.0))
                    .cos()
                    .max(0.0)
                    .powf(1.15);
        // Restrict coronal support to the lateral quadrant. Extending it into
        // the anterior q=.5--.65 samples created ten-centimetre clearance
        // deficits which the low-order lattice necessarily propagated to the
        // otherwise safe side endpoint.
        let depth_blend_q = ((physical_q - 0.68) / 0.30).clamp(0.0, 1.0);
        let depth_blend_q = depth_blend_q * depth_blend_q * (3.0 - 2.0 * depth_blend_q);
        let coronal =
            coronal_depth(canonical_coronal_level(v), coronal_levels, coronal_depths)? + 0.0018;
        let blend = side_wrap_weight(v) * depth_blend_q;
        depth += (coronal - depth) * blend;
        // The registered yoke comparison isolates a small forward ridge on
        // the q=.4 rail. Fair it into its q=.2/.6 neighbours only above the
        // torso/yoke seam; the accepted lower crown and side rail are locked.
        let upper_yoke = ((v - 0.62) / 0.28).clamp(0.0, 1.0);
        let upper_yoke = upper_yoke * upper_yoke * (3.0 - 2.0 * upper_yoke);
        let local_q = ((physical_q - 0.20) / 0.40).clamp(0.0, 1.0);
        let local_q = (std::f32::consts::PI * local_q).sin().powi(2);
        depth -= 0.0068 * upper_yoke * local_q;
        let side_contact = ((physical_q - 0.92) / 0.08).clamp(0.0, 1.0);
        let side_contact = side_contact * side_contact * (3.0 - 2.0 * side_contact);
        depth += 0.0020 * side_wrap_weight(v) * side_contact;
        // The measured coronal rail had two isolated anterior outliers at the
        // waist corner and lower-chest station. Extend their correction over a
        // narrow C1 lateral band rather than branching on the boundary vertex
        // identity; otherwise the first interior ring acquires a depth crease.
        let rail = ((physical_q - 0.94) / 0.06).clamp(0.0, 1.0);
        let rail = rail * rail * (3.0 - 2.0 * rail);
        let waist_t = (v / 0.16).clamp(0.0, 1.0);
        let waist_t = waist_t * waist_t * (3.0 - 2.0 * waist_t);
        let chest_t = ((v - 0.25) / 0.30).clamp(0.0, 1.0);
        let chest_bump = (std::f32::consts::PI * chest_t).sin().powi(2);
        depth -= rail * (0.010 * (1.0 - waist_t) + 0.009 * chest_bump);
        Ok(depth)
    };
    let samples = domain
        .iter()
        .enumerate()
        .map(|(index, point)| -> Result<_, GenerateError> {
            let domain_x = point[0];
            let y = point[1];
            let v = ((y - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            let half_width = section_half_width(v);
            let is_boundary = index < boundary_domain.len();
            // Apply one monotone section-coordinate map to the entire surface,
            // including its semantic perimeter. Excluding the perimeter made
            // the shifted interior overtake the opening rail and fold; the
            // common map keeps the conic's parameter order while turning the
            // complete band toward the side rail.
            let x = wrapped_lateral_x(domain_x, half_width, v);
            let physical_q = (x.abs() / half_width).clamp(0.0, 1.0);
            if is_boundary {
                let expected = coordinate(boundary[index], frame)[0];
                if (x - expected).abs() > 2e-5 {
                    eprintln!("breastplate boundary map mismatch index {index}: {x} vs {expected}");
                    return Err(GenerateError::InvalidSurface);
                }
            }
            let depth = torso_depth(v, physical_q)?;
            // The low-frequency lattice and radial semantic rail jointly own
            // arm-root clearance. A former 10 mm post-hoc underarm bump ended
            // inside the armscye, creating a bilateral 26-degree tooth.
            // The loft controls own the shape. This symmetric residual guard
            // only corrects vertical interpolation/extrapolation where it
            // would cross the torso; it fades out before the true side plane,
            // whose clearance direction is radial rather than anterior.
            let required = 0.5 * (exact_support_at(-x.abs(), y) + exact_support_at(x.abs(), y))
                + required_depth;
            let lateral = ((physical_q - ANTERIOR_CLEARANCE_Q) / (1.0 - ANTERIOR_CLEARANCE_Q))
                .clamp(0.0, 1.0);
            let lateral = lateral * lateral * (3.0 - 2.0 * lateral);
            // The completed lateral-envelope fit owns arm-root clearance.
            // Re-enabling an anterior ray at q=1 above the underarm turns a
            // harmless arm projection into a broad B-spline lift and renders
            // the entire yoke as a detached forward horn.  Keep anterior
            // support on the actual front quadrant at every height; exact
            // closed-body parity governs the radial armscye/side quadrant.
            let front_constraint_weight = 1.0 - lateral;
            let constraint = (required - depth).max(0.0) * front_constraint_weight;
            Ok(([x, y, depth], constraint, true))
        })
        .collect::<Result<Vec<_>, _>>()?;
    const CONTROL_Q: usize = 7;
    const CONTROL_V: usize = 9;
    let constraint_rows = samples
        .iter()
        .map(|(point, deficit, enabled)| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            let q = (point[0].abs() / section_half_width(v).max(1e-6)).clamp(0.0, 1.0);
            let bq = clamped_cubic_bspline_basis(CONTROL_Q, q);
            let bv = clamped_cubic_bspline_basis(CONTROL_V, v);
            let mut weights = vec![0.0_f32; CONTROL_Q * CONTROL_V];
            for row in 0..CONTROL_V {
                for column in 0..CONTROL_Q {
                    weights[row * CONTROL_Q + column] = bv[row] * bq[column];
                }
            }
            (weights, if *enabled { *deficit } else { 0.0 })
        })
        .collect::<Vec<_>>();
    let mut controls = vec![0.0_f32; CONTROL_Q * CONTROL_V];
    const SCREEN: f32 = 0.08;
    const BENDING: f32 = 4.0;
    const STEP: f32 = 0.004;
    for _ in 0..420 {
        let mut gradient = controls
            .iter()
            .map(|value| SCREEN * value)
            .collect::<Vec<_>>();
        for row in 0..CONTROL_V {
            for column in 0..CONTROL_Q - 2 {
                let indices = [
                    row * CONTROL_Q + column,
                    row * CONTROL_Q + column + 1,
                    row * CONTROL_Q + column + 2,
                ];
                let difference =
                    controls[indices[0]] - 2.0 * controls[indices[1]] + controls[indices[2]];
                gradient[indices[0]] += BENDING * 2.0 * difference;
                gradient[indices[1]] -= BENDING * 4.0 * difference;
                gradient[indices[2]] += BENDING * 2.0 * difference;
            }
        }
        for column in 0..CONTROL_Q {
            for row in 0..CONTROL_V - 2 {
                let indices = [
                    row * CONTROL_Q + column,
                    (row + 1) * CONTROL_Q + column,
                    (row + 2) * CONTROL_Q + column,
                ];
                let difference =
                    controls[indices[0]] - 2.0 * controls[indices[1]] + controls[indices[2]];
                gradient[indices[0]] += BENDING * 2.0 * difference;
                gradient[indices[1]] -= BENDING * 4.0 * difference;
                gradient[indices[2]] += BENDING * 2.0 * difference;
            }
        }
        for (control, gradient) in controls.iter_mut().zip(gradient) {
            *control = (*control - STEP * gradient).max(0.0);
        }
        let mut maximum_violation = 0.0_f32;
        for _ in 0..3 {
            for (weights, required) in &constraint_rows {
                let value = weights
                    .iter()
                    .zip(&controls)
                    .map(|(weight, control)| weight * control)
                    .sum::<f32>();
                let violation = (*required - value).max(0.0);
                maximum_violation = maximum_violation.max(violation);
                if violation > 0.0 {
                    let norm = weights
                        .iter()
                        .map(|weight| weight * weight)
                        .sum::<f32>()
                        .max(1e-8);
                    for (control, weight) in controls.iter_mut().zip(weights) {
                        *control += violation * weight / norm;
                    }
                }
            }
        }
        if maximum_violation <= 2.5e-5 {
            break;
        }
    }
    let maximum_displacement = constraint_rows
        .iter()
        .map(|(weights, _)| {
            weights
                .iter()
                .zip(&controls)
                .map(|(weight, control)| weight * control)
                .sum::<f32>()
        })
        .fold(0.0_f32, f32::max);
    if maximum_displacement > 0.080 {
        let (worst, _) = constraint_rows
            .iter()
            .enumerate()
            .max_by(|(_, (left, _)), (_, (right, _))| {
                let value = |weights: &[f32]| {
                    weights
                        .iter()
                        .zip(&controls)
                        .map(|(weight, control)| weight * control)
                        .sum::<f32>()
                };
                value(left).total_cmp(&value(right))
            })
            .expect("surface has samples");
        let point = samples[worst].0;
        let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
        let q = (point[0].abs() / section_half_width(v).max(1e-6)).clamp(0.0, 1.0);
        eprintln!(
            "breastplate displacement cap {maximum_displacement} at q {q} v {v}: local=({:.6},{:.6},{:.6}) raw_deficit={:.6}",
            point[0], point[1], point[2], samples[worst].1
        );
        return Err(GenerateError::InvalidSurface);
    }
    let result = samples
        .into_iter()
        .zip(constraint_rows)
        .map(
            |((mut point, _, _), (weights, _))| -> Result<[f32; 3], GenerateError> {
                point[2] += weights
                    .iter()
                    .zip(&controls)
                    .map(|(weight, control)| weight * control)
                    .sum::<f32>();
                let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
                let q = (point[0].abs() / section_half_width(v).max(1e-6)).clamp(0.0, 1.0);

                // The lattice is a conservative global clearance solve.  On the
                // open upper-front quadrant it previously preserved a torso-scale
                // lift selected by a few shoulder samples, leaving the q=.2/.4/.6
                // rails 4--5 cm ahead of the wearer.  Fit that quadrant back to a
                // smooth, symmetric wearer-relative clearance after the solve.
                // This is a broad C1 yoke field (not a vertex push), and its 2.5 mm
                // verifier allowance is in addition to the authored 10 mm padding
                // gap and half the wall thickness.
                let upper = ((v - 0.57) / 0.05).clamp(0.0, 1.0);
                let upper = upper * upper * (3.0 - 2.0 * upper);
                // Keep one coherent fit across the neckline and its first inner
                // ring.  Any center-to-q=.2 ramp has to cross 3--4 cm in less than
                // one ring spacing and fails the physical angle gate.  The upper
                // yoke is therefore fitted as one fair band; the accepted lower
                // crown remains untouched below the smooth v=.57--.62 blend.
                // The fit is one field over the complete yoke. A former
                // q=.72--.85 cutoff crossed the armscye and made adjacent
                // physical-arc stations jump by four millimetres, producing
                // both the visible tooth and a 30-degree tangent reversal.
                let fit_weight = upper;
                let upright = ((v - 0.57) / 0.05).clamp(0.0, 1.0);
                let upright = upright * upright * (3.0 - 2.0 * upright);
                let mut output = world_with_upright_upper(point, frame, upright);

                // Fit the yoke back to the same low-frequency authored field used
                // by the global solve.  A former pointwise body-ray minimum here
                // reintroduced breast/axilla samples after fairing, creating a
                // 5--6 cm alternating depth jump and the visible horizontal band.
                // Body samples remain inequalities in the control solve above;
                // they never become final surface values.
                let mut fitted_point = point;
                fitted_point[2] = torso_depth(v, q)?;
                let fitted = world_with_upright_upper(fitted_point, frame, upright)[2];
                output[2] += (output[2].min(fitted) - output[2]) * fit_weight;
                Ok(output)
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    let result = fair_depth_lattice(result, &domain, waist_y, neck_y, section_half_width, frame);
    Ok((result, boundary))
}

fn fair_depth_lattice(
    mut positions: Vec<[f32; 3]>,
    domain: &[[f32; 2]],
    waist_y: f32,
    neck_y: f32,
    section_half_width: impl Fn(f32) -> f32,
    frame: Frame,
) -> Vec<[f32; 3]> {
    const Q: usize = 7;
    const V: usize = 9;
    const N: usize = Q * V;
    let targets = positions
        .iter()
        .map(|point| dot(*point, frame.front))
        .collect::<Vec<_>>();
    let weights = domain
        .iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            let q = (wrapped_lateral_x(point[0], section_half_width(v), v).abs()
                / section_half_width(v).max(1e-6))
            .clamp(0.0, 1.0);
            let bq = clamped_cubic_bspline_basis(Q, q);
            let bv = clamped_cubic_bspline_basis(V, v);
            let mut row = vec![0.0_f32; N];
            for y in 0..V {
                for x in 0..Q {
                    row[y * Q + x] = bv[y] * bq[x];
                }
            }
            row
        })
        .collect::<Vec<_>>();
    let mut matrix = vec![vec![0.0_f32; N]; N];
    let mut rhs = vec![0.0_f32; N];
    for (row, target) in weights.iter().zip(&targets) {
        for left in 0..N {
            rhs[left] += row[left] * target;
            for right in 0..N {
                matrix[left][right] += row[left] * row[right];
            }
        }
    }
    const BENDING: f32 = 0.12;
    let mut add_second_difference = |indices: [usize; 3]| {
        let coefficients = [1.0_f32, -2.0, 1.0];
        for left in 0..3 {
            for right in 0..3 {
                matrix[indices[left]][indices[right]] +=
                    BENDING * coefficients[left] * coefficients[right];
            }
        }
    };
    for y in 0..V {
        for x in 0..Q - 2 {
            add_second_difference([y * Q + x, y * Q + x + 1, y * Q + x + 2]);
        }
    }
    for x in 0..Q {
        for y in 0..V - 2 {
            add_second_difference([y * Q + x, (y + 1) * Q + x, (y + 2) * Q + x]);
        }
    }
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] += 1e-5;
    }
    let controls = solve_dense_system(matrix, rhs);
    let fitted = weights
        .iter()
        .map(|row| row.iter().zip(&controls).map(|(a, b)| a * b).sum::<f32>())
        .collect::<Vec<_>>();
    let lift = targets
        .iter()
        .zip(&fitted)
        .map(|(target, value)| target - value)
        .fold(0.0_f32, f32::max);
    for ((position, target), value) in positions.iter_mut().zip(targets).zip(fitted) {
        *position = add(*position, scale(frame.front, value + lift - target));
    }
    positions
}

fn solve_dense_system(mut matrix: Vec<Vec<f32>>, mut rhs: Vec<f32>) -> Vec<f32> {
    for column in 0..rhs.len() {
        let pivot = (column..rhs.len())
            .max_by(|left, right| {
                matrix[*left][column]
                    .abs()
                    .total_cmp(&matrix[*right][column].abs())
            })
            .expect("dense system has a pivot");
        matrix.swap(column, pivot);
        rhs.swap(column, pivot);
        let diagonal = matrix[column][column]
            .abs()
            .max(1e-10)
            .copysign(matrix[column][column]);
        for row in column + 1..rhs.len() {
            let factor = matrix[row][column] / diagonal;
            for entry in column..rhs.len() {
                matrix[row][entry] -= factor * matrix[column][entry];
            }
            rhs[row] -= factor * rhs[column];
        }
    }
    let mut result = vec![0.0_f32; rhs.len()];
    for row in (0..rhs.len()).rev() {
        let remainder = (row + 1..rhs.len())
            .map(|column| matrix[row][column] * result[column])
            .sum::<f32>();
        result[row] =
            (rhs[row] - remainder) / matrix[row][row].abs().max(1e-10).copysign(matrix[row][row]);
    }
    result
}

fn skirt_surface(
    topology: &CanonicalBreastplateTopology,
    main: &[[f32; 3]],
    design: &BreastplateDesign,
    frame: Frame,
    torso_height: f32,
) -> (Vec<[f32; 3]>, usize) {
    let waist = topology
        .semantic_ranges()
        .iter()
        .find(|range| range.edge == BreastplateBoundaryEdge::Waist)
        .expect("canonical waist range");
    let seam_indices = (waist.start..=waist.start + waist.segments).collect::<Vec<_>>();
    let columns = seam_indices.len();
    // Default physical skirt length is 0.18 of the main semantic height.
    let length = torso_height * design.skirt_length.unit();
    let right = coordinate(main[seam_indices[0]], frame);
    let left = coordinate(main[*seam_indices.last().expect("waist seam")], frame);
    let coronal_origin = (right[2] + left[2]) * 0.5;
    let mut skirt = Vec::with_capacity((SKIRT_ROWS + 1) * columns);
    for row in 0..=SKIRT_ROWS {
        let t = row as f32 / SKIRT_ROWS as f32;
        for &boundary_index in &seam_indices {
            let seam = main[boundary_index];
            let local = coordinate(seam, frame);
            let radial = [local[0], 0.0, local[2] - coronal_origin];
            let radial_length = (radial[0] * radial[0] + radial[2] * radial[2])
                .sqrt()
                .max(1e-8);
            // A ruled skirt is linear from the exact waist seam to its hem.
            // Use one modest flare around the complete seam. The local radial
            // direction naturally rotates from anterior at center-front to
            // lateral at the coronal ends; circumferential attenuation made
            // the front a prow and the terminal columns rectangular tongues.
            let lateral_flare = design.skirt_flare.metres() * t * 0.204;
            let front_flare = design.skirt_flare.metres() * t * 0.17;
            skirt.push(add(
                seam,
                add(
                    scale(frame.vertical, -length * t),
                    add(
                        scale(frame.lateral, radial[0] / radial_length * lateral_flare),
                        scale(frame.front, radial[2] / radial_length * front_flare),
                    ),
                ),
            ));
        }
    }
    (skirt, columns)
}

fn open_normals(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    front: [f32; 3],
) -> Result<Vec<[f32; 3]>, GenerateError> {
    let mut normals = vec![[0.0; 3]; positions.len()];
    for face in faces {
        let [a, b, c] = face.map(|index| positions[index as usize]);
        let normal = cross(sub(b, a), sub(c, a));
        for index in face {
            normals[*index as usize] = add(normals[*index as usize], normal);
        }
    }
    normals
        .into_iter()
        .map(|normal| normalized(if length(normal) > 1e-8 { normal } else { front }))
        .collect()
}

fn append_closed_component(
    indices: &mut Vec<u32>,
    faces: &[[u32; 3]],
    boundary: &[u32],
    vertex_offset: u32,
    layer: u32,
) {
    for face in faces {
        let outer = face.map(|index| index + vertex_offset);
        indices.extend(outer);
        indices.extend([outer[0] + layer, outer[2] + layer, outer[1] + layer]);
    }
    for index in 0..boundary.len() {
        let a = boundary[index] + vertex_offset;
        let b = boundary[(index + 1) % boundary.len()] + vertex_offset;
        indices.extend([a, b, b + layer, a, b + layer, a + layer]);
    }
}

fn nearest_sample(point: [f32; 3], surface: &TorsoSurface) -> Vec<(usize, f32)> {
    let mut candidates = surface
        .vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| (length(sub(vertex.position, point)), index))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut weights = candidates
        .into_iter()
        .take(4)
        .map(|(distance, index)| (index, 1.0 / distance.max(1e-4)))
        .collect::<Vec<_>>();
    let total = weights.iter().map(|(_, weight)| weight).sum::<f32>();
    for (_, weight) in &mut weights {
        *weight /= total;
    }
    weights
}

fn sampled_uv(sample: &[(usize, f32)], surface: &TorsoSurface) -> [f32; 2] {
    sample.iter().fold([0.0; 2], |mut result, (index, weight)| {
        for (axis, value) in result.iter_mut().enumerate() {
            *value += surface.vertices[*index].uv[axis] * weight;
        }
        result
    })
}

fn sampled_skin(sample: &[(usize, f32)], surface: &TorsoSurface) -> ([u32; 8], [f32; 8]) {
    let mut weights = BTreeMap::<u32, f32>::new();
    for (index, sample_weight) in sample {
        let vertex = surface.vertices[*index];
        for (joint, weight) in vertex.joint_indices.into_iter().zip(vertex.joint_weights) {
            *weights.entry(joint).or_default() += weight * sample_weight;
        }
    }
    let mut weights = weights.into_iter().collect::<Vec<_>>();
    weights.sort_by(|(joint_a, weight_a), (joint_b, weight_b)| {
        weight_b
            .partial_cmp(weight_a)
            .unwrap_or(Ordering::Equal)
            .then_with(|| joint_a.cmp(joint_b))
    });
    weights.truncate(8);
    let total = weights.iter().map(|(_, weight)| weight).sum::<f32>();
    let mut joints = [0; 8];
    let mut result = [0.0; 8];
    for (slot, (joint, weight)) in weights.into_iter().enumerate() {
        joints[slot] = joint;
        result[slot] = weight / total.max(1e-8);
    }
    (joints, result)
}

struct GeneratedShape {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    mid_samples: Vec<Vec<(usize, f32)>>,
}

fn generate_shape(
    topology: &CanonicalBreastplateTopology,
    design: &BreastplateDesign,
    surface: &TorsoSurface,
    source_positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    clearance_vertices: &[TorsoShoulderSample],
    coronal_depths: &[f32],
) -> Result<GeneratedShape, GenerateError> {
    let frame = frame(source_positions, semantic_coordinates, front)?;
    let coronal_levels = surface
        .coronal_anchors
        .iter()
        .map(|anchor| anchor.vertical)
        .collect::<Vec<_>>();
    let (main, _) = main_mid_surface(
        topology,
        design,
        source_positions,
        semantic_coordinates,
        &surface.faces,
        anchors,
        clearance_vertices,
        &surface.clearance_mesh.faces,
        &coronal_levels,
        coronal_depths,
        frame,
    )?;
    let torso_height =
        regression_height(1.0, source_positions, semantic_coordinates, frame.vertical)
            - regression_height(0.0, source_positions, semantic_coordinates, frame.vertical);
    let (skirt, skirt_columns) = skirt_surface(topology, &main, design, frame, torso_height);
    let main_count = main.len();
    let mut mid = main;
    mid.extend(skirt);
    let mid_count = mid.len();

    // Canonical CDT winding is part of the fixed topology contract. Never
    // choose diagonals or flip faces independently for a body morph.
    let main_faces = topology.indices().as_chunks::<3>().0.to_vec();
    let skirt_faces_unoffset = (0..SKIRT_ROWS)
        .flat_map(|row| {
            (0..skirt_columns - 1).flat_map(move |column| {
                let a = (row * skirt_columns + column) as u32;
                let b = a + 1;
                let d = ((row + 1) * skirt_columns + column) as u32;
                let c = d + 1;
                [[a, b, c], [a, c, d]]
            })
        })
        .collect::<Vec<_>>();
    let skirt_faces = skirt_faces_unoffset;
    // Thickness follows the analytic convex crown, not triangle normals.  A
    // topology-smoothed normal field fed reflex cap connectivity back into the
    // solidified geometry, producing alternating teeth and quality oscillation
    // at otherwise valid semantic strips.  The front-to-coronal shell is
    // radial in each transverse section, so one topology-independent radial
    // field is the appropriate smooth normal (and is exactly symmetric).
    let side_depths = topology
        .semantic_ranges()
        .iter()
        .filter(|range| {
            matches!(
                range.edge,
                BreastplateBoundaryEdge::RightSide | BreastplateBoundaryEdge::LeftSide
            )
        })
        .flat_map(|range| range.start..=range.start + range.segments)
        .map(|index| coordinate(mid[index], frame)[2])
        .collect::<Vec<_>>();
    let coronal_origin = side_depths.iter().sum::<f32>() / side_depths.len().max(1) as f32;
    let mut mid_normals = mid[..main_count]
        .iter()
        .map(|position| {
            let local = coordinate(*position, frame);
            normalized(add(
                scale(frame.lateral, local[0]),
                scale(frame.front, local[2] - coronal_origin),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    mid_normals.extend(open_normals(&mid[main_count..], &skirt_faces, frame.front)?);

    // Semantic boundary construction already places the mid-surface outside
    // the body by clearance plus half thickness. Solidification is therefore
    // symmetric about that authored sheet; applying clearance a second time
    // would shrink narrow openings and degrade their triangle quality.
    let outer_offset = design.wall_thickness.metres() * 0.5;
    let inner_offset = -design.wall_thickness.metres() * 0.5;
    let positions = mid
        .iter()
        .zip(&mid_normals)
        .map(|(position, normal)| add(*position, scale(*normal, outer_offset)))
        .chain(
            mid.iter()
                .zip(&mid_normals)
                .map(|(position, normal)| add(*position, scale(*normal, inner_offset))),
        )
        .collect::<Vec<_>>();
    // Thickness uses the analytic radial field above, but render normals must
    // be derived from each finalized endpoint's actual oriented triangles.
    // Reusing the pre-solidification field (or the neutral endpoint's field)
    // leaves morph targets with normals that disagree with their geometry and
    // turns otherwise fair panel joins into dark bands and diagonal hollows.
    let outer_faces = main_faces
        .iter()
        .copied()
        .chain(
            skirt_faces
                .iter()
                .map(|face| face.map(|index| index + main_count as u32)),
        )
        .collect::<Vec<_>>();
    let outer_normals = open_normals(&positions[..mid_count], &outer_faces, frame.front)?;
    let inner_faces = outer_faces
        .iter()
        .map(|face| [face[0], face[2], face[1]])
        .collect::<Vec<_>>();
    let inner_normals = open_normals(
        &positions[mid_count..],
        &inner_faces,
        scale(frame.front, -1.0),
    )?;
    let normals = outer_normals.into_iter().chain(inner_normals).collect();

    let mut indices = Vec::new();
    append_closed_component(
        &mut indices,
        &main_faces,
        topology.boundary_vertices(),
        0,
        mid_count as u32,
    );
    let skirt_boundary = (0..skirt_columns as u32)
        .chain((1..=SKIRT_ROWS).map(|row| (row * skirt_columns + skirt_columns - 1) as u32))
        .chain(
            (0..skirt_columns - 1)
                .rev()
                .map(|column| (SKIRT_ROWS * skirt_columns + column) as u32),
        )
        .chain(
            (1..SKIRT_ROWS)
                .rev()
                .map(|row| (row * skirt_columns) as u32),
        )
        .collect::<Vec<_>>();
    append_closed_component(
        &mut indices,
        &skirt_faces,
        &skirt_boundary,
        main_count as u32,
        mid_count as u32,
    );
    let mid_samples = mid
        .iter()
        .map(|point| nearest_sample(*point, surface))
        .collect();
    Ok(GeneratedShape {
        positions,
        normals,
        indices,
        mid_samples,
    })
}

fn valid_surface(surface: &TorsoSurface) -> bool {
    !surface.domain.is_empty()
        && !surface.vertices.is_empty()
        && surface.vertices.len()
            == surface
                .morphs
                .first()
                .map_or(surface.vertices.len(), |morph| morph.positions.len())
        && !surface.clearance_mesh.vertices.is_empty()
        && surface.clearance_mesh.morph_vertices.len() == surface.morphs.len()
        && surface.morph_fronts.len() == surface.morphs.len()
        && surface.morph_semantic_coordinates.len() == surface.morphs.len()
        && surface.morph_upper_rig_anchors.len() == surface.morphs.len()
        && !surface.coronal_anchors.is_empty()
        && surface.morph_coronal_depths.len() == surface.morphs.len()
        && surface
            .morph_coronal_depths
            .iter()
            .all(|depths| depths.len() == surface.coronal_anchors.len())
        && surface
            .morphs
            .iter()
            .all(|morph| morph.positions.len() == surface.vertices.len())
        && surface
            .morph_semantic_coordinates
            .iter()
            .all(|coordinates| coordinates.len() == surface.vertices.len())
        && surface
            .clearance_mesh
            .morph_vertices
            .iter()
            .all(|vertices| vertices.len() == surface.clearance_mesh.vertices.len())
}

fn physical_refinement_candidates(
    topology: &CanonicalBreastplateTopology,
    shape: &GeneratedShape,
) -> (f32, f32, Vec<[f32; 2]>) {
    let mid = (0..topology.canonical_positions().len())
        .map(|index| shape.positions[index])
        .collect::<Vec<_>>();
    let mut minimum_angle = 180.0_f32;
    let mut maximum_aspect = 0.0_f32;
    let mut poor = Vec::<(f32, [f32; 2])>::new();
    for triangle in topology.indices().as_chunks::<3>().0 {
        let points = triangle.map(|index| mid[index as usize]);
        let edge = |a: [f32; 3], b: [f32; 3]| length(sub(b, a));
        let edges = [
            edge(points[1], points[2]),
            edge(points[2], points[0]),
            edge(points[0], points[1]),
        ];
        let twice_area = length(cross(sub(points[1], points[0]), sub(points[2], points[0])));
        let aspect = edges.iter().copied().fold(0.0, f32::max).powi(2) / twice_area.max(1e-10);
        let angle = (0..3)
            .map(|corner| {
                let a = edges[(corner + 1) % 3];
                let b = edges[(corner + 2) % 3];
                let opposite = edges[corner];
                ((a * a + b * b - opposite * opposite) / (2.0 * a * b).max(1e-10))
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees()
            })
            .fold(180.0_f32, f32::min);
        minimum_angle = minimum_angle.min(angle);
        maximum_aspect = maximum_aspect.max(aspect);
        if angle < 13.0 || aspect > 8.0 {
            let canonical = triangle.map(|index| topology.chart_positions()[index as usize]);
            let boundary_corners = triangle
                .iter()
                .enumerate()
                .filter(|(_, index)| **index < topology.boundary_vertices().len() as u32)
                .map(|(corner, _)| corner)
                .collect::<Vec<_>>();
            let candidate = if boundary_corners.len() == 2 {
                let first = canonical[boundary_corners[0]];
                let second = canonical[boundary_corners[1]];
                let interior = canonical[3 - boundary_corners[0] - boundary_corners[1]];
                let midpoint = [(first[0] + second[0]) * 0.5, (first[1] + second[1]) * 0.5];
                let toward = sub2d(interior, midpoint);
                let length = dot2d(toward, toward).sqrt().max(1e-8);
                let edge = dot2d(sub2d(second, first), sub2d(second, first)).sqrt();
                let inset = (edge * 3.0_f32.sqrt() * 0.5).min(0.0115);
                [
                    midpoint[0] + toward[0] * inset / length,
                    midpoint[1] + toward[1] * inset / length,
                ]
            } else {
                [
                    (canonical[0][0] + canonical[1][0] + canonical[2][0]) / 3.0,
                    (canonical[0][1] + canonical[1][1] + canonical[2][1]) / 3.0,
                ]
            };
            poor.push((angle, candidate));
        }
    }
    poor.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut candidates = Vec::<[f32; 2]>::new();
    for (_, candidate) in poor {
        let separated = topology
            .chart_positions()
            .iter()
            .chain(&candidates)
            .all(|point| {
                let delta = sub2d(candidate, *point);
                dot2d(delta, delta) >= 0.0012_f32.powi(2)
            });
        if separated {
            candidates.push(candidate);
            if candidates.len() == 8 {
                break;
            }
        }
    }
    (minimum_angle, maximum_aspect, candidates)
}

fn physical_vertex_quality(
    topology: &CanonicalBreastplateTopology,
    shape: &GeneratedShape,
    vertex: u32,
) -> (f32, f32) {
    topology
        .indices()
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|face| face.contains(&vertex))
        .fold((180.0_f32, 0.0_f32), |score, face| {
            let points = face.map(|index| shape.positions[index as usize]);
            let edges = [
                length(sub(points[1], points[2])),
                length(sub(points[2], points[0])),
                length(sub(points[0], points[1])),
            ];
            let twice_area = length(cross(sub(points[1], points[0]), sub(points[2], points[0])));
            let aspect =
                edges.iter().copied().fold(0.0_f32, f32::max).powi(2) / twice_area.max(1e-10);
            let angle = (0..3)
                .map(|corner| {
                    let a = edges[(corner + 1) % 3];
                    let b = edges[(corner + 2) % 3];
                    let opposite = edges[corner];
                    ((a * a + b * b - opposite * opposite) / (2.0 * a * b).max(1e-10))
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees()
                })
                .fold(180.0_f32, f32::min);
            (score.0.min(angle), score.1.max(aspect))
        })
}

fn physical_global_quality(
    topology: &CanonicalBreastplateTopology,
    surfaces: &[Vec<[f32; 3]>],
) -> (f32, f32, [u32; 3]) {
    let mut score = (180.0_f32, 0.0_f32, [0_u32; 3]);
    for face in topology.indices().as_chunks::<3>().0 {
        for positions in surfaces {
            let points = face.map(|index| positions[index as usize]);
            let edges = [
                length(sub(points[1], points[2])),
                length(sub(points[2], points[0])),
                length(sub(points[0], points[1])),
            ];
            let twice_area = length(cross(sub(points[1], points[0]), sub(points[2], points[0])));
            let aspect =
                edges.iter().copied().fold(0.0_f32, f32::max).powi(2) / twice_area.max(1e-10);
            let angle = (0..3)
                .map(|corner| {
                    let a = edges[(corner + 1) % 3];
                    let b = edges[(corner + 2) % 3];
                    let opposite = edges[corner];
                    ((a * a + b * b - opposite * opposite) / (2.0 * a * b).max(1e-10))
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees()
                })
                .fold(180.0_f32, f32::min);
            if angle < score.0 {
                score.0 = angle;
                score.2 = *face;
            }
            score.1 = score.1.max(aspect);
        }
    }
    score
}

fn physical_vertex_quality_surfaces(
    topology: &CanonicalBreastplateTopology,
    surfaces: &[Vec<[f32; 3]>],
    vertex: u32,
) -> (f32, f32) {
    let mut score = (180.0_f32, 0.0_f32);
    for face in topology
        .indices()
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|face| face.contains(&vertex))
    {
        for positions in surfaces {
            let points = face.map(|index| positions[index as usize]);
            let edges = [
                length(sub(points[1], points[2])),
                length(sub(points[2], points[0])),
                length(sub(points[0], points[1])),
            ];
            let twice_area = length(cross(sub(points[1], points[0]), sub(points[2], points[0])));
            let aspect =
                edges.iter().copied().fold(0.0_f32, f32::max).powi(2) / twice_area.max(1e-10);
            let angle = (0..3)
                .map(|corner| {
                    let a = edges[(corner + 1) % 3];
                    let b = edges[(corner + 2) % 3];
                    let opposite = edges[corner];
                    ((a * a + b * b - opposite * opposite) / (2.0 * a * b).max(1e-10))
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees()
                })
                .fold(180.0_f32, f32::min);
            score.0 = score.0.min(angle);
            score.1 = score.1.max(aspect);
        }
    }
    score
}

fn sub2d(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn dot2d(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

pub fn generate_breastplate(
    design: &BreastplateDesign,
    surface: &TorsoSurface,
) -> Result<GeneratedArmor, GenerateError> {
    validate_breastplate(design)?;
    if !valid_surface(surface) {
        eprintln!(
            "breastplate surface arrays invalid: vertices {} morphs {} fronts {} semantic {} anchors {} coronal {} clearance morphs {}",
            surface.vertices.len(),
            surface.morphs.len(),
            surface.morph_fronts.len(),
            surface.morph_semantic_coordinates.len(),
            surface.morph_upper_rig_anchors.len(),
            surface.morph_coronal_depths.len(),
            surface.clearance_mesh.morph_vertices.len()
        );
        return Err(GenerateError::InvalidSurface);
    }
    let base_positions = surface
        .vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<_>>();
    let base_semantic_coordinates = surface
        .vertices
        .iter()
        .map(|vertex| [vertex.lateral, vertex.vertical])
        .collect::<Vec<_>>();
    let base_frame = frame(&base_positions, &base_semantic_coordinates, surface.front)?;
    let semantic_layout = CanonicalBreastplateTopology::semantic_layout();
    let base_boundary = semantic_boundary(
        &semantic_layout,
        design,
        &base_positions,
        &base_semantic_coordinates,
        surface.upper_rig_anchors,
        &surface.clearance_mesh.vertices,
        base_frame,
    );
    let base_measurements = semantic_measurements(
        design,
        &base_positions,
        &base_semantic_coordinates,
        surface.upper_rig_anchors,
        base_frame,
    );
    let base_chart_boundary = base_boundary
        .iter()
        .map(|position| {
            let local = coordinate(*position, base_frame);
            let v = ((local[1] - base_measurements.waist[1])
                / (base_measurements.neck[1] - base_measurements.waist[1]).max(1e-6))
            .clamp(0.0, 1.0);
            let half_width = section_half_width(base_measurements, v);
            [lateral_x_to_section_arc(local[0], half_width), local[1]]
        })
        .collect::<Vec<_>>();
    let base_domain = semantic_parameter_boundary(&base_boundary, base_measurements, base_frame);
    let mut extra_candidates = Vec::<[f32; 2]>::new();
    let base_coronal_depths = surface
        .coronal_anchors
        .iter()
        .map(|anchor| anchor.depth)
        .collect::<Vec<_>>();
    // Iterative Steiner insertion only needs the physical envelope endpoints.
    // Evaluating all 90 identity directions at every insertion made asset
    // generation scale as morphs x iterations. Select the narrowest and
    // broadest torso/shoulder realizations for candidate placement; every
    // morph is still generated below and participates in the final shared
    // edge optimization and exported quality.
    let mut morph_ratios = Vec::<(usize, f32)>::new();
    for (index, (((morph, anchors), semantic_coordinates), front)) in surface
        .morphs
        .iter()
        .zip(&surface.morph_upper_rig_anchors)
        .zip(&surface.morph_semantic_coordinates)
        .zip(&surface.morph_fronts)
        .enumerate()
    {
        let morph_frame = frame(&morph.positions, semantic_coordinates, *front)?;
        let measurements = semantic_measurements(
            design,
            &morph.positions,
            semantic_coordinates,
            *anchors,
            morph_frame,
        );
        morph_ratios.push((index, measurements.torso_to_shoulder));
    }
    let mut representative_morphs = std::collections::BTreeSet::<usize>::new();
    if let Some((index, _)) = morph_ratios
        .iter()
        .min_by(|left, right| left.1.total_cmp(&right.1))
    {
        representative_morphs.insert(*index);
    }
    if let Some((index, _)) = morph_ratios
        .iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    {
        representative_morphs.insert(*index);
    }
    let mut best = None::<(f32, CanonicalBreastplateTopology, GeneratedShape)>;
    let mut refinement_iterations = 0_usize;
    let (mut topology, mut base) = loop {
        refinement_iterations += 1;
        let topology = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
            &base_chart_boundary,
            &base_domain,
            &extra_candidates,
            if design.neck_width.unit() > 0.50 {
                0.25
            } else {
                0.50
            },
            |chart| {
                let v = ((chart[1] - base_measurements.waist[1])
                    / (base_measurements.neck[1] - base_measurements.waist[1]).max(1e-6))
                .clamp(0.0, 1.0);
                let half_width = section_half_width(base_measurements, v);
                let physical_x = section_arc_to_lateral_x(chart[0], half_width);
                [
                    inverse_wrapped_lateral_x(physical_x, half_width, v),
                    chart[1],
                ]
            },
        );
        let base = generate_shape(
            &topology,
            design,
            surface,
            &base_positions,
            &base_semantic_coordinates,
            surface.front,
            surface.upper_rig_anchors,
            &surface.clearance_mesh.vertices,
            &base_coronal_depths,
        )?;
        // Refine the design connectivity against every physical realization,
        // not only the base body. A coronal side rail can move materially on
        // a torso-wide morph while retaining the same semantic IDs; selecting
        // Steiner sites from the base alone left a boundary-adjacent triangle
        // acceptable on base but stretched on that supported morph.
        let mut minimum_angle = 180.0_f32;
        let mut maximum_aspect = 0.0_f32;
        let mut candidates = Vec::<[f32; 2]>::new();
        let mut assess = |shape: &GeneratedShape| {
            let (angle, aspect, shape_candidates) =
                physical_refinement_candidates(&topology, shape);
            minimum_angle = minimum_angle.min(angle);
            maximum_aspect = maximum_aspect.max(aspect);
            candidates.extend(shape_candidates);
        };
        assess(&base);
        for (
            morph_index,
            (
                ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                coronal_depths,
            ),
        ) in surface
            .morphs
            .iter()
            .zip(&surface.morph_upper_rig_anchors)
            .zip(&surface.morph_semantic_coordinates)
            .zip(&surface.morph_fronts)
            .zip(&surface.clearance_mesh.morph_vertices)
            .zip(&surface.morph_coronal_depths)
            .enumerate()
        {
            if !representative_morphs.contains(&morph_index) {
                continue;
            }
            let target = generate_shape(
                &topology,
                design,
                surface,
                &morph.positions,
                semantic_coordinates,
                *front,
                *anchors,
                clearance_vertices,
                coronal_depths,
            )?;
            assess(&target);
        }
        // Keep a buffer for the shared morph surfaces: a base-only mesh that
        // stops exactly at the hard contract can lose quality when the same
        // semantic connectivity is evaluated on a different wearer.
        // First minimize hard-contract violations across every morph.  The
        // former additive buffer score preferred a 10.77-degree / 8.21-aspect
        // mesh over refinements that crossed the aspect gate but surrendered
        // a harmless fraction of the 12.5-degree reserve.
        let score = (10.0 - minimum_angle).max(0.0) * 100.0
            + (maximum_aspect - 8.0).max(0.0) * 10.0
            + (12.5 - minimum_angle).max(0.0) * 0.01
            + (maximum_aspect - 7.5).max(0.0) * 0.01;
        if best
            .as_ref()
            .is_none_or(|(best_score, _, _)| score < *best_score)
        {
            best = Some((score, topology, base));
        }
        if (minimum_angle >= 12.5 && maximum_aspect <= 7.5)
            || candidates.is_empty()
            || extra_candidates.len() >= 64
            || refinement_iterations >= 12
        {
            let (_, topology, base) = best.expect("at least one topology candidate");
            break (topology, base);
        }
        let previous_candidate_count = extra_candidates.len();
        for candidate in candidates {
            let separated = extra_candidates.iter().all(|point| {
                let delta = sub2d(candidate, *point);
                dot2d(delta, delta) >= 0.0012_f32.powi(2)
            });
            if separated {
                extra_candidates.push(candidate);
            }
        }
        if extra_candidates.len() == previous_candidate_count {
            let (_, topology, base) = best.expect("at least one topology candidate");
            break (topology, base);
        }
    };

    // The neckline/shoulder corner is a reflex garment-pattern junction.  Its
    // outer curve samples are semantic landmarks and remain fixed, while the
    // middle sample of the derived inner cap is free to relax.  Optimize that
    // one non-landmark station against the actual curved base and envelope
    // morph surfaces; a chart-only optimum can still lose a degree after the
    // fair crown is evaluated in 3D.
    for station in [
        0, 1, 2, 3, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 61, 62,
        159, 160, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191,
    ] {
        let mut best_cap = (f32::NEG_INFINITY, f32::INFINITY, topology.clone(), base);
        let tangent_limit = if matches!(station, 0 | 1 | 26 | 27) {
            5
        } else {
            3
        };
        let normal_limit = if matches!(station, 0 | 1 | 26 | 27) {
            9
        } else {
            6
        };
        for tangent_step in -tangent_limit..=tangent_limit {
            for normal_step in -normal_limit..=normal_limit {
                if tangent_step != 0 && normal_step != 0 {
                    continue;
                }
                let mut candidate_topology = topology.clone();
                if !candidate_topology.relax_inner_rail_station(
                    station,
                    tangent_step as f32 * 0.06,
                    normal_step as f32 * 0.06,
                ) {
                    continue;
                }
                let candidate_base = generate_shape(
                    &candidate_topology,
                    design,
                    surface,
                    &base_positions,
                    &base_semantic_coordinates,
                    surface.front,
                    surface.upper_rig_anchors,
                    &surface.clearance_mesh.vertices,
                    &base_coronal_depths,
                )?;
                let station_vertex = candidate_topology
                    .inner_rail_vertex(station)
                    .expect("inner semantic rail station");
                let (mut minimum_angle, mut maximum_aspect) =
                    physical_vertex_quality(&candidate_topology, &candidate_base, station_vertex);
                for (
                    morph_index,
                    (
                        ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                        coronal_depths,
                    ),
                ) in surface
                    .morphs
                    .iter()
                    .zip(&surface.morph_upper_rig_anchors)
                    .zip(&surface.morph_semantic_coordinates)
                    .zip(&surface.morph_fronts)
                    .zip(&surface.clearance_mesh.morph_vertices)
                    .zip(&surface.morph_coronal_depths)
                    .enumerate()
                {
                    if !representative_morphs.contains(&morph_index) {
                        continue;
                    }
                    let target = generate_shape(
                        &candidate_topology,
                        design,
                        surface,
                        &morph.positions,
                        semantic_coordinates,
                        *front,
                        *anchors,
                        clearance_vertices,
                        coronal_depths,
                    )?;
                    let (angle, aspect) =
                        physical_vertex_quality(&candidate_topology, &target, station_vertex);
                    minimum_angle = minimum_angle.min(angle);
                    maximum_aspect = maximum_aspect.max(aspect);
                }
                if minimum_angle > best_cap.0 + 1e-5
                    || ((minimum_angle - best_cap.0).abs() <= 1e-5
                        && maximum_aspect < best_cap.1 - 1e-5)
                {
                    best_cap = (
                        minimum_angle,
                        maximum_aspect,
                        candidate_topology,
                        candidate_base,
                    );
                }
            }
        }
        topology = best_cap.2;
        base = best_cap.3;
    }
    // The right neckline/bridge reflex cap is a derived garment-pattern
    // vertex, not a landmark. Optimize its physical position across the same
    // wearer envelope after rail relaxation; widening the authored neckline
    // otherwise leaves its one fan triangle just over the aspect contract.
    for (cap_offset, first_station, last_station) in [
        (0_usize, 0_usize, 1_usize),
        (2, 29, 29),
        (3, 39, 40),
        (4, 39, 40),
        (5, 180, 181),
        (6, 180, 181),
    ] {
        let mut best_cap = (f32::NEG_INFINITY, f32::INFINITY, topology.clone(), base);
        for tangent_step in -4_i32..=4 {
            for normal_step in -4_i32..=4 {
                if tangent_step != 0 && normal_step != 0 {
                    continue;
                }
                let mut candidate_topology = topology.clone();
                if !candidate_topology.relax_cap_vertex(
                    cap_offset,
                    first_station,
                    last_station,
                    tangent_step as f32 * 0.05,
                    normal_step as f32 * 0.05,
                ) {
                    continue;
                }
                let candidate_base = generate_shape(
                    &candidate_topology,
                    design,
                    surface,
                    &base_positions,
                    &base_semantic_coordinates,
                    surface.front,
                    surface.upper_rig_anchors,
                    &surface.clearance_mesh.vertices,
                    &base_coronal_depths,
                )?;
                let cap_vertex = candidate_topology
                    .cap_vertex(cap_offset)
                    .expect("semantic cap vertex");
                let (mut minimum_angle, mut maximum_aspect) =
                    physical_vertex_quality(&candidate_topology, &candidate_base, cap_vertex);
                for (
                    morph_index,
                    (
                        ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                        coronal_depths,
                    ),
                ) in surface
                    .morphs
                    .iter()
                    .zip(&surface.morph_upper_rig_anchors)
                    .zip(&surface.morph_semantic_coordinates)
                    .zip(&surface.morph_fronts)
                    .zip(&surface.clearance_mesh.morph_vertices)
                    .zip(&surface.morph_coronal_depths)
                    .enumerate()
                {
                    if !representative_morphs.contains(&morph_index) {
                        continue;
                    }
                    let target = generate_shape(
                        &candidate_topology,
                        design,
                        surface,
                        &morph.positions,
                        semantic_coordinates,
                        *front,
                        *anchors,
                        clearance_vertices,
                        coronal_depths,
                    )?;
                    let (angle, aspect) =
                        physical_vertex_quality(&candidate_topology, &target, cap_vertex);
                    minimum_angle = minimum_angle.min(angle);
                    maximum_aspect = maximum_aspect.max(aspect);
                }
                let violation = (10.0 - minimum_angle).max(0.0) * 100.0
                    + (maximum_aspect - 9.0).max(0.0) * 10.0;
                let best_violation =
                    (10.0 - best_cap.0).max(0.0) * 100.0 + (best_cap.1 - 9.0).max(0.0) * 10.0;
                if violation < best_violation - 1e-5
                    || ((violation - best_violation).abs() <= 1e-5
                        && (minimum_angle > best_cap.0 + 1e-5
                            || ((minimum_angle - best_cap.0).abs() <= 1e-5
                                && maximum_aspect < best_cap.1 - 1e-5)))
                {
                    best_cap = (
                        minimum_angle,
                        maximum_aspect,
                        candidate_topology,
                        candidate_base,
                    );
                }
            }
        }
        topology = best_cap.2;
        base = best_cap.3;
    }
    // Finish the wraparound neckline cap with a fine rail adjustment. The
    // coarse general rail search is intentionally broad, but this shared quad
    // can miss the morph aspect gate by a few hundredths. Score both the rail
    // station and its adjacent cap so the refinement cannot trade that tiny
    // aspect gain for a skinny cap fan.
    for station in [0_usize, 1_usize, 37, 38, 179, 180] {
        let mut best_local = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            topology.clone(),
            base,
        );
        for tangent_step in -8_i32..=8 {
            for normal_step in -8_i32..=8 {
                if tangent_step != 0 && normal_step != 0 {
                    continue;
                }
                let mut candidate_topology = topology.clone();
                if !candidate_topology.relax_inner_rail_station(
                    station,
                    tangent_step as f32 * 0.01,
                    normal_step as f32 * 0.01,
                ) {
                    continue;
                }
                let candidate_base = generate_shape(
                    &candidate_topology,
                    design,
                    surface,
                    &base_positions,
                    &base_semantic_coordinates,
                    surface.front,
                    surface.upper_rig_anchors,
                    &surface.clearance_mesh.vertices,
                    &base_coronal_depths,
                )?;
                let rail_vertex = candidate_topology
                    .inner_rail_vertex(station)
                    .expect("inner semantic rail station");
                let cap_vertex = candidate_topology
                    .cap_vertex(match station {
                        37 => 3,
                        38 => 4,
                        179 => 5,
                        180 => 6,
                        _ => 0,
                    })
                    .expect("semantic cap vertex");
                let (mut minimum_angle, mut maximum_aspect) =
                    physical_vertex_quality(&candidate_topology, &candidate_base, rail_vertex);
                let (angle, aspect) =
                    physical_vertex_quality(&candidate_topology, &candidate_base, cap_vertex);
                minimum_angle = minimum_angle.min(angle);
                maximum_aspect = maximum_aspect.max(aspect);
                for (
                    morph_index,
                    (
                        ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                        coronal_depths,
                    ),
                ) in surface
                    .morphs
                    .iter()
                    .zip(&surface.morph_upper_rig_anchors)
                    .zip(&surface.morph_semantic_coordinates)
                    .zip(&surface.morph_fronts)
                    .zip(&surface.clearance_mesh.morph_vertices)
                    .zip(&surface.morph_coronal_depths)
                    .enumerate()
                {
                    if !representative_morphs.contains(&morph_index) {
                        continue;
                    }
                    let target = generate_shape(
                        &candidate_topology,
                        design,
                        surface,
                        &morph.positions,
                        semantic_coordinates,
                        *front,
                        *anchors,
                        clearance_vertices,
                        coronal_depths,
                    )?;
                    for vertex in [rail_vertex, cap_vertex] {
                        let (angle, aspect) =
                            physical_vertex_quality(&candidate_topology, &target, vertex);
                        minimum_angle = minimum_angle.min(angle);
                        maximum_aspect = maximum_aspect.max(aspect);
                    }
                }
                let violation = (10.0 - minimum_angle).max(0.0) * 100.0
                    + (maximum_aspect - 9.0).max(0.0) * 10.0;
                if violation < best_local.0 - 1e-5
                    || ((violation - best_local.0).abs() <= 1e-5
                        && (minimum_angle > best_local.1 + 1e-5
                            || ((minimum_angle - best_local.1).abs() <= 1e-5
                                && maximum_aspect < best_local.2 - 1e-5)))
                {
                    best_local = (
                        violation,
                        minimum_angle,
                        maximum_aspect,
                        candidate_topology,
                        candidate_base,
                    );
                }
            }
        }
        topology = best_local.3;
        base = best_local.4;
    }
    // The final inserted Steiner site is the only free vertex that can remain
    // slightly anisotropic on a supported morph after shared insertion and
    // edge flips. Relax it in the physical chart rather than accepting a
    // morph-only sub-10-degree triangle or changing connectivity.
    {
        let free_vertex = topology.canonical_positions().len() - 1;
        let mut best_free = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            topology.clone(),
            base,
        );
        for x_step in 0_i32..=0 {
            for y_step in 0_i32..=0 {
                let mut candidate_topology = topology.clone();
                if !candidate_topology.relax_free_vertex(
                    free_vertex,
                    x_step as f32 * 0.010,
                    y_step as f32 * 0.010,
                ) {
                    continue;
                }
                let candidate_base = generate_shape(
                    &candidate_topology,
                    design,
                    surface,
                    &base_positions,
                    &base_semantic_coordinates,
                    surface.front,
                    surface.upper_rig_anchors,
                    &surface.clearance_mesh.vertices,
                    &base_coronal_depths,
                )?;
                let vertex = free_vertex as u32;
                let (mut minimum_angle, mut maximum_aspect) =
                    physical_vertex_quality(&candidate_topology, &candidate_base, vertex);
                for (
                    morph_index,
                    (
                        ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                        coronal_depths,
                    ),
                ) in surface
                    .morphs
                    .iter()
                    .zip(&surface.morph_upper_rig_anchors)
                    .zip(&surface.morph_semantic_coordinates)
                    .zip(&surface.morph_fronts)
                    .zip(&surface.clearance_mesh.morph_vertices)
                    .zip(&surface.morph_coronal_depths)
                    .enumerate()
                {
                    if !representative_morphs.contains(&morph_index) {
                        continue;
                    }
                    let target = generate_shape(
                        &candidate_topology,
                        design,
                        surface,
                        &morph.positions,
                        semantic_coordinates,
                        *front,
                        *anchors,
                        clearance_vertices,
                        coronal_depths,
                    )?;
                    let (angle, aspect) =
                        physical_vertex_quality(&candidate_topology, &target, vertex);
                    minimum_angle = minimum_angle.min(angle);
                    maximum_aspect = maximum_aspect.max(aspect);
                }
                let violation = (10.0 - minimum_angle).max(0.0) * 100.0
                    + (maximum_aspect - 9.0).max(0.0) * 10.0;
                if violation < best_free.0 - 1e-5
                    || ((violation - best_free.0).abs() <= 1e-5
                        && (minimum_angle > best_free.1 + 1e-5
                            || ((minimum_angle - best_free.1).abs() <= 1e-5
                                && maximum_aspect < best_free.2 - 1e-5)))
                {
                    best_free = (
                        violation,
                        minimum_angle,
                        maximum_aspect,
                        candidate_topology,
                        candidate_base,
                    );
                }
            }
        }
        topology = best_free.3;
        base = best_free.4;
    }
    // Rebalance the adjacent derived neck/bridge cap after moving the newest
    // Steiner site. The two variables serve different faces; optimizing the
    // cap second avoids forcing one free vertex to trade base quality against
    // the morph-only cap triangle.
    {
        let cap_offset = 2_usize;
        let mut best_cap = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            topology.clone(),
            base,
        );
        for tangent_step in 0_i32..=0 {
            for normal_step in 0_i32..=0 {
                let mut candidate_topology = topology.clone();
                if !candidate_topology.relax_cap_vertex(
                    cap_offset,
                    29,
                    29,
                    tangent_step as f32 * 0.020,
                    normal_step as f32 * 0.020,
                ) {
                    continue;
                }
                let candidate_base = generate_shape(
                    &candidate_topology,
                    design,
                    surface,
                    &base_positions,
                    &base_semantic_coordinates,
                    surface.front,
                    surface.upper_rig_anchors,
                    &surface.clearance_mesh.vertices,
                    &base_coronal_depths,
                )?;
                let cap_vertex = candidate_topology
                    .cap_vertex(cap_offset)
                    .expect("semantic cap vertex");
                let (mut minimum_angle, mut maximum_aspect) =
                    physical_vertex_quality(&candidate_topology, &candidate_base, cap_vertex);
                for (
                    morph_index,
                    (
                        ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                        coronal_depths,
                    ),
                ) in surface
                    .morphs
                    .iter()
                    .zip(&surface.morph_upper_rig_anchors)
                    .zip(&surface.morph_semantic_coordinates)
                    .zip(&surface.morph_fronts)
                    .zip(&surface.clearance_mesh.morph_vertices)
                    .zip(&surface.morph_coronal_depths)
                    .enumerate()
                {
                    if !representative_morphs.contains(&morph_index) {
                        continue;
                    }
                    let target = generate_shape(
                        &candidate_topology,
                        design,
                        surface,
                        &morph.positions,
                        semantic_coordinates,
                        *front,
                        *anchors,
                        clearance_vertices,
                        coronal_depths,
                    )?;
                    let (angle, aspect) =
                        physical_vertex_quality(&candidate_topology, &target, cap_vertex);
                    minimum_angle = minimum_angle.min(angle);
                    maximum_aspect = maximum_aspect.max(aspect);
                }
                let violation = (10.0 - minimum_angle).max(0.0) * 100.0
                    + (maximum_aspect - 9.0).max(0.0) * 10.0;
                if violation < best_cap.0 - 1e-5
                    || ((violation - best_cap.0).abs() <= 1e-5
                        && (minimum_angle > best_cap.1 + 1e-5
                            || ((minimum_angle - best_cap.1).abs() <= 1e-5
                                && maximum_aspect < best_cap.2 - 1e-5)))
                {
                    best_cap = (
                        violation,
                        minimum_angle,
                        maximum_aspect,
                        candidate_topology,
                        candidate_base,
                    );
                }
            }
        }
        topology = best_cap.3;
    }
    let main_vertex_count = topology.canonical_positions().len();
    let acceptance_designs = topology_acceptance_designs(design);
    let acceptance_surfaces = |candidate_topology: &CanonicalBreastplateTopology| {
        let mut surfaces = Vec::<Vec<[f32; 3]>>::new();
        for acceptance_design in &acceptance_designs {
            let acceptance_base = generate_shape(
                candidate_topology,
                acceptance_design,
                surface,
                &base_positions,
                &base_semantic_coordinates,
                surface.front,
                surface.upper_rig_anchors,
                &surface.clearance_mesh.vertices,
                &base_coronal_depths,
            )?;
            surfaces.push(acceptance_base.positions[..main_vertex_count].to_vec());
            if acceptance_design != design {
                continue;
            }
            for (
                morph_index,
                (
                    ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                    coronal_depths,
                ),
            ) in surface
                .morphs
                .iter()
                .zip(&surface.morph_upper_rig_anchors)
                .zip(&surface.morph_semantic_coordinates)
                .zip(&surface.morph_fronts)
                .zip(&surface.clearance_mesh.morph_vertices)
                .zip(&surface.morph_coronal_depths)
                .enumerate()
            {
                if !representative_morphs.contains(&morph_index) {
                    continue;
                }
                let target = generate_shape(
                    candidate_topology,
                    acceptance_design,
                    surface,
                    &morph.positions,
                    semantic_coordinates,
                    *front,
                    *anchors,
                    clearance_vertices,
                    coronal_depths,
                )?;
                surfaces.push(target.positions[..main_vertex_count].to_vec());
            }
        }
        Ok::<_, GenerateError>(surfaces)
    };
    let shared_quality_surfaces = acceptance_surfaces(&topology)?;
    topology.optimize_physical_edges(&shared_quality_surfaces);
    // A semantic-boundary relaxation changes the harmonic embedding of every
    // free CDT sample. On extreme parameter combinations an otherwise remote
    // interior one-ring can therefore become the last sub-10-degree region
    // even after all useful edge flips. Relax one vertex of that objectively
    // worst face in the physical chart, scoring the requested design and both
    // acceptance extremes on the base and representative wearers. This keeps
    // the canonical topology honest for the complete exported design space.
    for _ in 0..128 {
        let quality_surfaces = acceptance_surfaces(&topology)?;
        let (minimum_angle, maximum_aspect, worst_face) =
            physical_global_quality(&topology, &quality_surfaces);
        if minimum_angle >= 10.0 && maximum_aspect <= 9.0 {
            break;
        }
        let violation = |angle: f32, aspect: f32| {
            (10.0 - angle).max(0.0) * 100.0 + (aspect - 9.0).max(0.0) * 10.0
        };
        let mut best = (
            violation(minimum_angle, maximum_aspect),
            minimum_angle,
            maximum_aspect,
            f32::NEG_INFINITY,
            topology.clone(),
        );
        let boundary_count = topology.boundary_vertices().len();
        for vertex in worst_face {
            for (x_step, y_step) in [(-1, 0), (0, -1), (0, 1), (1, 0)] {
                let mut candidate_topology = topology.clone();
                let vertex_index = vertex as usize;
                let moved = if (boundary_count..boundary_count * 2).contains(&vertex_index) {
                    candidate_topology.relax_inner_rail_station_preserving_embedding(
                        vertex_index - boundary_count,
                        x_step as f32 * 0.06,
                        y_step as f32 * 0.06,
                    )
                } else {
                    candidate_topology.relax_free_vertex_preserving_embedding(
                        vertex_index,
                        x_step as f32 * 0.06,
                        y_step as f32 * 0.06,
                    )
                };
                if !moved {
                    continue;
                }
                let candidate_surfaces = acceptance_surfaces(&candidate_topology)?;
                let (angle, aspect, _) =
                    physical_global_quality(&candidate_topology, &candidate_surfaces);
                let (local_angle, _) = physical_vertex_quality_surfaces(
                    &candidate_topology,
                    &candidate_surfaces,
                    vertex,
                );
                let candidate_violation = violation(angle, aspect);
                if candidate_violation < best.0 - 1e-5
                    || ((candidate_violation - best.0).abs() <= 1e-5
                        && (angle > best.1 + 1e-5
                            || ((angle - best.1).abs() <= 1e-5 && aspect < best.2 - 1e-5)))
                    || ((candidate_violation - best.0).abs() <= 1e-5
                        && (angle - best.1).abs() <= 1e-5
                        && (aspect - best.2).abs() <= 1e-5
                        && local_angle > best.3 + 1e-5)
                {
                    best = (
                        candidate_violation,
                        angle,
                        aspect,
                        local_angle,
                        candidate_topology,
                    );
                }
            }
        }
        if best.4.reference_hash() == topology.reference_hash()
            || best.0 > violation(minimum_angle, maximum_aspect) + 1e-5
        {
            break;
        }
        topology = best.4;
    }
    // The aggregate optimizer can reach a fixed point whose tied worst score
    // is owned by a design extreme while the requested neutral surface still
    // has a different sub-10-degree free face. Give that requested surface a
    // bounded final coordinate relaxation, but accept moves only when the
    // complete acceptance set is no worse. This is still face-agnostic and
    // uses the same frozen semantic topology for every endpoint.
    for _ in 0..64 {
        let current_base = generate_shape(
            &topology,
            design,
            surface,
            &base_positions,
            &base_semantic_coordinates,
            surface.front,
            surface.upper_rig_anchors,
            &surface.clearance_mesh.vertices,
            &base_coronal_depths,
        )?;
        let base_surface = current_base.positions[..main_vertex_count].to_vec();
        let (base_angle, base_aspect, worst_face) =
            physical_global_quality(&topology, std::slice::from_ref(&base_surface));
        if base_angle >= 10.0 && base_aspect <= 9.0 {
            break;
        }
        let current_acceptance = acceptance_surfaces(&topology)?;
        let (acceptance_angle, acceptance_aspect, _) =
            physical_global_quality(&topology, &current_acceptance);
        let violation = |angle: f32, aspect: f32| {
            (10.0 - angle).max(0.0) * 100.0 + (aspect - 9.0).max(0.0) * 10.0
        };
        let current_acceptance_violation = violation(acceptance_angle, acceptance_aspect);
        let mut best = (
            violation(base_angle, base_aspect),
            base_angle,
            base_aspect,
            topology.clone(),
        );
        let boundary_count = topology.boundary_vertices().len();
        for vertex in worst_face {
            for (x_step, y_step) in [(-1, 0), (0, -1), (0, 1), (1, 0)] {
                let mut candidate_topology = topology.clone();
                let vertex_index = vertex as usize;
                let moved = if (boundary_count..boundary_count * 2).contains(&vertex_index) {
                    candidate_topology.relax_inner_rail_station_preserving_embedding(
                        vertex_index - boundary_count,
                        x_step as f32 * 0.06,
                        y_step as f32 * 0.06,
                    )
                } else {
                    candidate_topology.relax_free_vertex_preserving_embedding(
                        vertex_index,
                        x_step as f32 * 0.06,
                        y_step as f32 * 0.06,
                    )
                };
                if !moved {
                    continue;
                }
                let candidate_acceptance = acceptance_surfaces(&candidate_topology)?;
                let (candidate_acceptance_angle, candidate_acceptance_aspect, _) =
                    physical_global_quality(&candidate_topology, &candidate_acceptance);
                if violation(candidate_acceptance_angle, candidate_acceptance_aspect)
                    > current_acceptance_violation + 1e-5
                {
                    continue;
                }
                let candidate_base = generate_shape(
                    &candidate_topology,
                    design,
                    surface,
                    &base_positions,
                    &base_semantic_coordinates,
                    surface.front,
                    surface.upper_rig_anchors,
                    &surface.clearance_mesh.vertices,
                    &base_coronal_depths,
                )?;
                let candidate_base_surface = candidate_base.positions[..main_vertex_count].to_vec();
                let (angle, aspect, _) = physical_global_quality(
                    &candidate_topology,
                    std::slice::from_ref(&candidate_base_surface),
                );
                let candidate_violation = violation(angle, aspect);
                if candidate_violation < best.0 - 1e-5
                    || ((candidate_violation - best.0).abs() <= 1e-5
                        && (angle > best.1 + 1e-5
                            || ((angle - best.1).abs() <= 1e-5 && aspect < best.2 - 1e-5)))
                {
                    best = (candidate_violation, angle, aspect, candidate_topology);
                }
            }
        }
        if best.3.reference_hash() == topology.reference_hash()
            || best.0 >= violation(base_angle, base_aspect) - 1e-5
        {
            break;
        }
        topology = best.3;
    }
    // Freeze topology first, then evaluate every surface on that exact final
    // domain. `base` may otherwise still describe the preceding best
    // iteration when the bounded relaxation exits without accepting its last
    // candidate, while morph endpoints are evaluated on the final topology.
    base = generate_shape(
        &topology,
        design,
        surface,
        &base_positions,
        &base_semantic_coordinates,
        surface.front,
        surface.upper_rig_anchors,
        &surface.clearance_mesh.vertices,
        &base_coronal_depths,
    )?;
    let mid_count = base.mid_samples.len();
    let texcoords_mid = base
        .mid_samples
        .iter()
        .map(|sample| sampled_uv(sample, surface))
        .collect::<Vec<_>>();
    let skin_mid = base
        .mid_samples
        .iter()
        .map(|sample| sampled_skin(sample, surface))
        .collect::<Vec<_>>();
    let texcoords = texcoords_mid
        .iter()
        .copied()
        .chain(texcoords_mid.iter().copied())
        .collect();
    let joint_indices = skin_mid
        .iter()
        .map(|(joints, _)| *joints)
        .chain(skin_mid.iter().map(|(joints, _)| *joints))
        .collect();
    let joint_weights = skin_mid
        .iter()
        .map(|(_, weights)| *weights)
        .chain(skin_mid.iter().map(|(_, weights)| *weights))
        .collect();
    debug_assert_eq!(base.positions.len(), mid_count * 2);

    let morphs = surface
        .morphs
        .iter()
        .zip(&surface.morph_upper_rig_anchors)
        .zip(&surface.morph_semantic_coordinates)
        .zip(&surface.morph_fronts)
        .zip(&surface.clearance_mesh.morph_vertices)
        .zip(&surface.morph_coronal_depths)
        .map(
            |(
                ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                coronal_depths,
            )| {
                let target = generate_shape(
                    &topology,
                    design,
                    surface,
                    &morph.positions,
                    semantic_coordinates,
                    *front,
                    *anchors,
                    clearance_vertices,
                    coronal_depths,
                )?;
                if target.indices != base.indices || target.positions.len() != base.positions.len()
                {
                    eprintln!("breastplate morph topology mismatch {}", morph.name);
                    return Err(GenerateError::InvalidSurface);
                }
                Ok(ArmorMorph {
                    name: morph.name.clone(),
                    direct_positions: target.positions.clone(),
                    position_deltas: target
                        .positions
                        .iter()
                        .zip(&base.positions)
                        .map(|(target, base)| sub(*target, *base))
                        .collect(),
                    normal_deltas: target
                        .normals
                        .iter()
                        .zip(&base.normals)
                        .map(|(target, base)| sub(*target, *base))
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
        joint_indices,
        joint_weights,
        indices: base.indices,
        morphs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skirt_boundary_has_expected_cycle_length() {
        let columns = 9;
        let boundary = (0..columns)
            .chain((1..=SKIRT_ROWS).map(|row| row * columns + columns - 1))
            .chain(
                (0..columns - 1)
                    .rev()
                    .map(|column| SKIRT_ROWS * columns + column),
            )
            .chain((1..SKIRT_ROWS).rev().map(|row| row * columns))
            .collect::<Vec<_>>();
        assert_eq!(boundary.len(), 2 * columns + 2 * SKIRT_ROWS - 2);
    }

    #[test]
    fn wrapped_chart_round_trip_is_monotone_with_positive_jacobian() {
        let half_width = 0.21;
        for vertical_step in 0..=20 {
            let v = vertical_step as f32 / 20.0;
            let mut previous = wrapped_lateral_x(0.0, half_width, v);
            for lateral_step in 1..=120 {
                let parameter_x = 0.25 * lateral_step as f32 / 120.0;
                let physical_x = wrapped_lateral_x(parameter_x, half_width, v);
                assert!(physical_x > previous, "non-positive chart Jacobian at {v}");
                previous = physical_x;

                let inverse = inverse_wrapped_lateral_x(physical_x, half_width, v);
                assert!((inverse - parameter_x).abs() < 2e-5);
                let arc = lateral_x_to_section_arc(physical_x, half_width);
                let chart_round_trip = section_arc_to_lateral_x(arc, half_width);
                assert!((chart_round_trip - physical_x).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn authored_crown_has_one_peak_and_armhole_is_smoothly_monotone() {
        let samples = (0..=1_000)
            .map(|step| longitudinal_crown_shape(step as f32 / 1_000.0))
            .collect::<Vec<_>>();
        let peak_index = samples
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .unwrap()
            .0;
        let peak = peak_index as f32 / 1_000.0;
        assert!((peak - 2.0 / 3.0).abs() <= 0.002);
        assert!(
            samples[..=peak_index]
                .windows(2)
                .all(|pair| pair[1] >= pair[0])
        );
        assert!(
            samples[peak_index..]
                .windows(2)
                .all(|pair| pair[1] <= pair[0])
        );

        let shoulder = [0.12, 0.40];
        let underarm = [0.14, 0.22];
        let curve = (0..=200)
            .map(|step| armhole_curve(shoulder, underarm, step as f32 / 200.0))
            .collect::<Vec<_>>();
        assert_eq!(curve[0], shoulder);
        assert_eq!(*curve.last().unwrap(), underarm);
        assert!(curve.windows(2).all(|pair| pair[1][1] < pair[0][1]));
        assert!(curve.windows(2).all(|pair| pair[1][0] > pair[0][0]));
        let vertical_second_differences = curve
            .windows(3)
            .map(|points| points[2][1] - 2.0 * points[1][1] + points[0][1])
            .collect::<Vec<_>>();
        // The tailored armhole intentionally has a shallow S-curvature so it
        // can turn off the bridge and arrive tangent to the side rail. Float
        // sign-change counting is unstable at its exact zero-curvature
        // stations; bound the actual curvature magnitude instead, while the
        // strict x/y monotonicity above excludes scallops and double notches.
        assert!(
            vertical_second_differences
                .iter()
                .all(|difference| difference.abs() <= 3.1e-5)
        );
    }
}
