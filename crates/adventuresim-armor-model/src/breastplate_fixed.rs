//! Boundary-first breastplate generation.
//!
//! The main plate owns one canonical constrained triangulation. Every fixed
//! canonical vertex is evaluated directly on one smooth, body-supported curve
//! network; morphs never lift a boundary cage harmonically in three dimensions.
//! The torso crown stays low-frequency and defensive; wearer samples constrain
//! clearance and semantic anchors rather than imprinting anatomy into the form.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use crate::{
    ArmorMorph, BreastplateDesign, GenerateError, GeneratedArmor, TorsoShoulderSample,
    TorsoSurface, TorsoUpperRigAnchors,
    breastplate_crest_policy::CrestQueryPolicy,
    breastplate_design_hash,
    breastplate_family_quality::{improving_family, worst_first},
    breastplate_global_upper::{GlobalUpperLoft, UpperGuide},
    breastplate_joint_boundary::{JointReturn, derived_return_stations},
    breastplate_joint_profiles::JointProfiles,
    breastplate_loft_profiles::{
        LoftFitOptions, LoftProfiles, TorsoSlice, fit_loft_profiles, plate_fairing_multiplier,
    },
    breastplate_neckline::NecklineTreatment,
    breastplate_qp::{LinearConstraint, QpOptions, solve_dense_qp},
    breastplate_seated_band::{ShoulderSeat, fit_seated_band},
    breastplate_shoulder_band::{BandError, BandFraction, CrestSample, ShoulderBand},
    breastplate_shoulder_seat_query::{SeatedCrestQueries, hash_enclosure_geometry},
    breastplate_simple_rim::{DescendingArmOpening, RimPolicy, armhole_curve},
    breastplate_topology::{BreastplateBoundaryEdge, CanonicalBreastplateTopology},
    breastplate_whole_front::{WholeFrontPolicy, install_guide},
    validate_breastplate,
};

// Four physical rows keep both the default short skirt and the maximum design
// length inside the shared ten-degree triangle gate after curvature-sized
// waist sampling.  This changes only tessellation along the same ruled bell;
// seam, hem, flare, and morph connectivity remain exact.
const SKIRT_ROWS: usize = 4;
const ANTERIOR_CLEARANCE_Q: f32 = 0.58;

fn shoulder_band_error(error: BandError) -> GenerateError {
    eprintln!("shoulder band: {error}");
    GenerateError::InvalidSurface
}

fn whole_front_error(error: String) -> GenerateError {
    eprintln!("whole front: {error}");
    GenerateError::InvalidSurface
}

fn topology_acceptance_designs(requested: &BreastplateDesign) -> Vec<BreastplateDesign> {
    // Connectivity is frozen across BODY morphs of this requested garment.
    // A different armor recipe owns its own physical chart and topology;
    // independent parameter-extreme integration tests validate those meshes.
    // Requiring an unrelated zero-crown/tiny-neck recipe here coupled the
    // requested export to an artifact the caller never requested.
    vec![requested.clone()]
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

fn transverse_crown_shape(q: f32) -> f32 {
    let q = q.clamp(0.0, 1.0);
    (std::f32::consts::FRAC_PI_2 * q).cos().max(0.0).powf(1.58)
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

// Exact polynomial derivatives in f64 avoid subtracting nearly equal f32
// basis values over sub-millimetre finite-difference stencils.
fn cubic_basis_derivatives(count: usize, t: f64) -> [Vec<f64>; 3] {
    let knots = (0..count + 4)
        .map(|i| {
            if i <= 3 {
                0.0
            } else if i >= count {
                1.0
            } else {
                (i - 3) as f64 / (count - 3) as f64
            }
        })
        .collect::<Vec<_>>();
    let mut level = vec![0.0; count + 3];
    if t == 1.0 {
        level[count - 1] = 1.0;
    } else {
        for i in 0..count {
            level[i] = (knots[i] <= t && t < knots[i + 1]) as u8 as f64;
        }
    }
    let mut levels = vec![level.clone()];
    for degree in 1..=3 {
        let previous = level.clone();
        for i in 0..count {
            let a = knots[i + degree] - knots[i];
            let b = knots[i + degree + 1] - knots[i + 1];
            level[i] = if a > 0.0 {
                (t - knots[i]) * previous[i] / a
            } else {
                0.0
            } + if b > 0.0 {
                (knots[i + degree + 1] - t) * previous[i + 1] / b
            } else {
                0.0
            };
        }
        levels.push(level.clone());
    }
    let derivative = |degree: usize, lower: &[f64]| {
        let mut values = vec![0.0; count + 3];
        for i in 0..count {
            let a = knots[i + degree] - knots[i];
            let b = knots[i + degree + 1] - knots[i + 1];
            values[i] = if a > 0.0 {
                degree as f64 * lower[i] / a
            } else {
                0.0
            } - if b > 0.0 {
                degree as f64 * lower[i + 1] / b
            } else {
                0.0
            };
        }
        values
    };
    let first = derivative(3, &levels[2]);
    let second = derivative(3, &derivative(2, &levels[1]));
    [
        levels[3][..count].to_vec(),
        first[..count].to_vec(),
        second[..count].to_vec(),
    ]
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

impl SemanticMeasurements {
    fn armhole_terminal(self) -> [f32; 2] {
        [
            self.mid_axillary[0],
            self.mid_axillary[1] - (self.neck[1] - self.waist[1]) * 0.020,
        ]
    }
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
    semantic_boundary_from_measurements(
        semantic_ranges,
        design,
        measurements,
        clearance_vertices,
        frame,
        None,
    )
}

fn shoulder_bridge_curve(measurements: SemanticMeasurements, t: f32) -> [f32; 2] {
    // Match the actual U endpoint tangent and the quarter-ellipse armscye's
    // horizontal start tangent. The former vertical/diagonal handles plus a
    // 20mm polynomial hook belonged to incompatible historical outlines.
    let neck_tangent = normalized2([
        measurements.neck[0] * 0.12,
        (measurements.neck[1] - measurements.neck_center_height) * 0.58,
    ]);
    let chord = ((measurements.shoulder[0] - measurements.neck[0]).powi(2)
        + (measurements.shoulder[1] - measurements.neck[1]).powi(2))
    .sqrt();
    let handle = chord / 3.0;
    cubic(
        measurements.neck,
        [
            measurements.neck[0] + neck_tangent[0] * handle,
            measurements.neck[1] + neck_tangent[1] * handle,
        ],
        [measurements.shoulder[0] - handle, measurements.shoulder[1]],
        measurements.shoulder,
        t,
    )
}

fn semantic_boundary_from_measurements(
    semantic_ranges: &[crate::breastplate_topology::SemanticBoundaryRange],
    design: &BreastplateDesign,
    measurements: SemanticMeasurements,
    clearance_vertices: &[TorsoShoulderSample],
    frame: Frame,
    authored_arm: Option<&DescendingArmOpening>,
) -> Vec<[f32; 3]> {
    let armhole_terminal = measurements.armhole_terminal();
    let arm = |t| {
        authored_arm.map_or_else(
            || armhole_curve(measurements.shoulder, armhole_terminal, t),
            |curve| curve.at(t),
        )
    };
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
                    [0.50, 0.0],
                    [0.88, 0.42],
                    [1.0, 1.0],
                    centered.abs(),
                );
                let x = centered.signum() * measurements.neck[0] * half[0];
                let bowl = half[1];
                let y = measurements.neck_center_height
                    + (measurements.neck[1] - measurements.neck_center_height) * bowl;
                [x, y]
            }
            BreastplateBoundaryEdge::RightShoulder => shoulder_bridge_curve(measurements, t),
            BreastplateBoundaryEdge::RightArmhole => arm(t),
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
                let p = arm(1.0 - t);
                [-p[0], p[1]]
            }
            BreastplateBoundaryEdge::LeftShoulder => {
                let p = shoulder_bridge_curve(measurements, 1.0 - t);
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
            let crown = transverse_crown_shape(q);
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

#[derive(Clone)]
struct FittedDepthField {
    controls: Vec<f32>,
    bottom_y: f32,
    top_y: f32,
    half_extent: f32,
}

impl FittedDepthField {
    fn coordinates(&self, x: f32, y: f32) -> Result<[f32; 2], GenerateError> {
        let q = x.abs() / self.half_extent;
        let v = (y - self.bottom_y) / (self.top_y - self.bottom_y);
        if !q.is_finite() || !v.is_finite() || q > 1.0 || !(0.0..=1.0).contains(&v) {
            eprintln!("breastplate field query outside physical domain: x={x} y={y} q={q} v={v}");
            return Err(GenerateError::InvalidSurface);
        }
        Ok([q, v])
    }

    fn weights(&self, x: f32, y: f32) -> Result<Vec<f32>, GenerateError> {
        let [q, v] = self.coordinates(x, y)?;
        let bq = clamped_cubic_bspline_basis(7, q);
        let bv = clamped_cubic_bspline_basis(9, v);
        let mut weights = vec![0.0; 54];
        for (row, by) in bv.iter().enumerate() {
            for (column, bx) in bq.iter().enumerate() {
                let free = row * 6 + column.saturating_sub(1);
                weights[free] += by * bx;
            }
        }
        Ok(weights)
    }

    fn depth(&self, x: f32, y: f32) -> Result<f32, GenerateError> {
        Ok(self
            .weights(x, y)?
            .iter()
            .zip(&self.controls)
            .map(|(a, b)| a * b)
            .sum())
    }

    /// Derivatives with respect to outward |x|, shared by mirrored halves.
    fn transverse_derivatives(&self, x: f32, y: f32) -> Result<[Vec<f64>; 2], GenerateError> {
        let [dx, _, dxx, _, _] = self.physical_derivatives(x, y)?;
        Ok([dx, dxx])
    }

    /// Physical derivatives [d|x|, dy, d|x|², d|x|dy, dy²].
    fn physical_derivatives(&self, x: f32, y: f32) -> Result<[Vec<f64>; 5], GenerateError> {
        self.coordinates(x, y)?;
        let q = x.abs() as f64 / self.half_extent as f64;
        let v = (y as f64 - self.bottom_y as f64) / (self.top_y as f64 - self.bottom_y as f64);
        let [bq, dq, ddq] = cubic_basis_derivatives(7, q);
        let [bv, dv, ddv] = cubic_basis_derivatives(9, v);
        let width = self.half_extent as f64;
        let height = self.top_y as f64 - self.bottom_y as f64;
        let mut rows: [Vec<f64>; 5] = std::array::from_fn(|_| vec![0.0; 54]);
        for (row, by) in bv.iter().enumerate() {
            for column in 0_usize..7 {
                let free = row * 6 + column.saturating_sub(1);
                rows[0][free] += by * dq[column] / width;
                rows[1][free] += dv[row] * bq[column] / height;
                rows[2][free] += by * ddq[column] / width.powi(2);
                rows[3][free] += dv[row] * dq[column] / (width * height);
                rows[4][free] += ddv[row] * bq[column] / height.powi(2);
            }
        }
        Ok(rows)
    }
}

/// Compact torso/yoke prototype. The torso is parameterized by angle, never
/// by a depth graph at a vertical side tangent. The yoke is a single Hermite
/// return with position and longitudinal derivative inherited from the torso.
#[derive(Clone)]
struct AngularSurface {
    joint_profiles: Option<JointProfiles>,
    global_upper: Option<GlobalUpperLoft>,
    profiles: LoftProfiles,
    boundary_domain: Vec<[f32; 2]>,
    top_guides: Vec<[f32; 3]>,
    top_parameters: Vec<f32>,
    top_depth_controls: Vec<f32>,
    top_slope_controls: Vec<f32>,
    waist_y: f32,
    seam_y: f32,
    arc_scale: f32,
    side_trim: f32,
    trim_start_y: f32,
}

impl AngularSurface {
    fn coverage_angle(&self, y: f32) -> f32 {
        let t =
            ((y - self.trim_start_y) / (self.seam_y - self.trim_start_y).max(1e-6)).clamp(0.0, 1.0);
        std::f32::consts::FRAC_PI_2 - self.side_trim * t * t * (3.0 - 2.0 * t)
    }
    fn top(&self, u: f32) -> [f32; 3] {
        self.top_differential(u).0
    }

    fn top_differential(&self, u: f32) -> ([f32; 3], [f32; 3]) {
        let hi = self
            .top_parameters
            .partition_point(|v| *v < u)
            .clamp(1, self.top_parameters.len() - 1);
        let lo = hi - 1;
        let t = ((u - self.top_parameters[lo])
            / (self.top_parameters[hi] - self.top_parameters[lo]).max(1e-8))
        .clamp(0.0, 1.0);
        let a = self.top_guides[lo];
        let b = self.top_guides[hi];
        let basis = clamped_cubic_bspline_basis(self.top_depth_controls.len(), u);
        let tangent = |index: usize, axis: usize| {
            if index == 0 {
                if axis == 1 {
                    0.0
                } else {
                    (self.top_guides[1][axis] - self.top_guides[0][axis])
                        / (self.top_parameters[1] - self.top_parameters[0])
                }
            } else if index + 1 == self.top_guides.len() {
                let dy = (self.top_guides[index][1] - self.top_guides[index - 1][1])
                    / (self.top_parameters[index] - self.top_parameters[index - 1]);
                if axis == 0 {
                    let angle = self.coverage_angle(self.seam_y);
                    let a = self.profiles.eval(self.seam_y as f64)[0] as f32;
                    self.profiles.derivative(self.seam_y as f64)[0] as f32 * angle.sin() * dy
                        + a * angle.cos() * angle
                } else {
                    dy
                }
            } else {
                let before = (self.top_guides[index][axis] - self.top_guides[index - 1][axis])
                    / (self.top_parameters[index] - self.top_parameters[index - 1]);
                let after = (self.top_guides[index + 1][axis] - self.top_guides[index][axis])
                    / (self.top_parameters[index + 1] - self.top_parameters[index]);
                if before * after <= 0.0 {
                    0.0
                } else {
                    2.0 * before * after / (before + after)
                }
            }
        };
        let span = self.top_parameters[hi] - self.top_parameters[lo];
        let interpolate = |axis| {
            (2.0 * t.powi(3) - 3.0 * t * t + 1.0) * a[axis]
                + (t.powi(3) - 2.0 * t * t + t) * span * tangent(lo, axis)
                + (-2.0 * t.powi(3) + 3.0 * t * t) * b[axis]
                + (t.powi(3) - t * t) * span * tangent(hi, axis)
        };
        let position = [
            interpolate(0),
            interpolate(1),
            basis
                .iter()
                .zip(&self.top_depth_controls)
                .map(|(w, c)| w * c)
                .sum(),
        ];
        let derivative = |axis| {
            ((6.0 * t * t - 6.0 * t) * a[axis] + (-6.0 * t * t + 6.0 * t) * b[axis]) / span
                + (3.0 * t * t - 4.0 * t + 1.0) * tangent(lo, axis)
                + (3.0 * t * t - 2.0 * t) * tangent(hi, axis)
        };
        let dbasis = cubic_basis_derivatives(self.top_depth_controls.len(), u as f64)[1].clone();
        (
            position,
            [
                derivative(0),
                derivative(1),
                dbasis
                    .iter()
                    .zip(&self.top_depth_controls)
                    .map(|(a, b)| a * (*b as f64))
                    .sum::<f64>() as f32,
            ],
        )
    }

    fn local_normal(&self, point: [f32; 2]) -> Result<[f32; 3], GenerateError> {
        let u = (point[0] / self.arc_scale).abs().clamp(0.0, 1.0);
        let sign = if point[0] < 0.0 { -1.0 } else { 1.0 };
        let y = point[1];
        if let Some(joint) = &self.joint_profiles {
            let normal = joint.evaluate(u as f64, y as f64).1;
            return normalized([sign * normal[0] as f32, normal[1] as f32, normal[2] as f32]);
        }
        if let Some(global) = &self.global_upper
            && y > self.seam_y
        {
            let normal = global.evaluate(u as f64, y as f64).1;
            return normalized([sign * normal[0] as f32, normal[1] as f32, normal[2] as f32]);
        }
        let level = y.clamp(self.waist_y, self.seam_y) as f64;
        let [a, b, _] = self.profiles.eval(level);
        let [da, db, dc] = self.profiles.derivative(level);
        let angle = u as f64 * self.coverage_angle(y) as f64;
        let (top, top_u) = self.top_differential(u);
        let height = (top[1] - self.seam_y) as f64;
        if y <= self.seam_y + 1e-6 || height <= 1e-6 {
            // Ellipse-family normal is invariant to the angular trim's
            // coordinate redistribution. This is also the collapsed-yoke limit.
            return normalized([
                sign * (b * angle.sin()) as f32,
                (-(b * da * angle.sin().powi(2) + a * angle.cos() * (dc + db * angle.cos())))
                    as f32,
                (a * angle.cos()) as f32,
            ]);
        }
        let t = ((y - self.seam_y) as f64 / height).clamp(0.0, 1.0);
        let angle_scale = self.coverage_angle(self.seam_y) as f64;
        let [_, _, c] = self.profiles.eval(self.seam_y as f64);
        let x0 = a * angle.sin();
        let z0 = c + b * angle.cos();
        let xu0 = a * angle.cos() * angle_scale;
        let zu0 = -b * angle.sin() * angle_scale;
        let mx = da * angle.sin();
        let mz = dc + db * angle.cos();
        let mxu = da * angle.cos() * angle_scale;
        let mzu = -db * angle.sin() * angle_scale;
        let basis = cubic_basis_derivatives(self.top_slope_controls.len(), u as f64);
        let slope = basis[0]
            .iter()
            .zip(&self.top_slope_controls)
            .map(|(a, b)| a * (*b as f64))
            .sum::<f64>();
        let slope_u = basis[1]
            .iter()
            .zip(&self.top_slope_controls)
            .map(|(a, b)| a * (*b as f64))
            .sum::<f64>();
        let h00 = 2.0 * t.powi(3) - 3.0 * t * t + 1.0;
        let h10 = t.powi(3) - 2.0 * t * t + t;
        let h01 = -2.0 * t.powi(3) + 3.0 * t * t;
        let h11 = t.powi(3) - t * t;
        let d00 = 6.0 * t * t - 6.0 * t;
        let d10 = 3.0 * t * t - 4.0 * t + 1.0;
        let d01 = -d00;
        let d11 = 3.0 * t * t - 2.0 * t;
        let hu = top_u[1] as f64;
        let du = [
            h00 * xu0
                + h10 * (hu * mx + height * mxu)
                + h01 * top_u[0] as f64
                + h11 * (top_u[0] as f64 - xu0),
            t * hu,
            h00 * zu0
                + h10 * (hu * mz + height * mzu)
                + h01 * top_u[2] as f64
                + h11 * (hu * slope + height * slope_u),
        ];
        let dt = [
            d00 * x0 + d10 * height * mx + d01 * top[0] as f64 + d11 * (top[0] as f64 - x0),
            height,
            d00 * z0 + d10 * height * mz + d01 * top[2] as f64 + d11 * height * slope,
        ];
        let n = [
            du[1] * dt[2] - du[2] * dt[1],
            du[2] * dt[0] - du[0] * dt[2],
            du[0] * dt[1] - du[1] * dt[0],
        ];
        let magnitude = n.iter().map(|x| x * x).sum::<f64>().sqrt();
        if magnitude <= 1e-20 || !magnitude.is_finite() {
            return Err(GenerateError::Degenerate);
        }
        Ok([
            sign * (n[0] / magnitude) as f32,
            (n[1] / magnitude) as f32,
            (n[2] / magnitude) as f32,
        ])
    }

    fn local_position(&self, point: [f32; 2]) -> Result<[f32; 3], GenerateError> {
        let point = [point[0] / self.arc_scale, point[1]];
        if !point.iter().all(|value| value.is_finite())
            || point[0].abs() > 1.0001
            || point[1] < self.waist_y - 0.0001
        {
            eprintln!("angular surface query outside material domain: {point:?}");
            return Err(GenerateError::InvalidSurface);
        }
        let u = point[0].abs().clamp(0.0, 1.0);
        let sign = if point[0] < 0.0 { -1.0 } else { 1.0 };
        let y = point[1];
        if let Some(joint) = &self.joint_profiles {
            let position = joint.evaluate(u as f64, y as f64).0;
            return Ok([
                sign * position[0] as f32,
                position[1] as f32,
                position[2] as f32,
            ]);
        }
        if let Some(global) = &self.global_upper
            && y > self.seam_y
        {
            let position = global.evaluate(u as f64, y as f64).0;
            return Ok([
                sign * position[0] as f32,
                position[1] as f32,
                position[2] as f32,
            ]);
        }
        let angle = u * self.coverage_angle(y);
        let level = y.clamp(self.waist_y, self.seam_y) as f64;
        let [a, b, c] = self.profiles.eval(level);
        let lower = [a as f32 * angle.sin(), y, c as f32 + b as f32 * angle.cos()];
        if y <= self.seam_y + 1e-6 {
            return Ok([sign * lower[0], y, lower[2]]);
        }
        let top = self.top(u);
        let span = top[1] - self.seam_y;
        if span <= 1e-6 {
            return Ok([sign * lower[0], y, lower[2]]);
        }
        let t = ((y - self.seam_y) / span).clamp(0.0, 1.0);
        let [da, db, dc] = self.profiles.derivative(self.seam_y as f64);
        let start_x_slope = da as f32 * angle.sin();
        let start_z_slope = dc as f32 + db as f32 * angle.cos();
        let basis = clamped_cubic_bspline_basis(self.top_slope_controls.len(), u);
        let end_z_slope = basis
            .iter()
            .zip(&self.top_slope_controls)
            .map(|(w, c)| w * c)
            .sum::<f32>();
        let h00 = 2.0 * t.powi(3) - 3.0 * t * t + 1.0;
        let h10 = t.powi(3) - 2.0 * t * t + t;
        let h01 = -2.0 * t.powi(3) + 3.0 * t * t;
        let h11 = t.powi(3) - t * t;
        // The upper x tangent follows its whole meridian chord; z follows a
        // low-order shoulder-slope guide. No triangle-dependent correction.
        let end_x_slope = (top[0] - lower[0]) / span;
        Ok([
            sign * (h00 * lower[0]
                + h10 * span * start_x_slope
                + h01 * top[0]
                + h11 * span * end_x_slope),
            y,
            h00 * lower[2] + h10 * span * start_z_slope + h01 * top[2] + h11 * span * end_z_slope,
        ])
    }
}

fn angular_upper_measurements(
    mut measurements: SemanticMeasurements,
    design: &BreastplateDesign,
    anchors: TorsoUpperRigAnchors,
) -> SemanticMeasurements {
    let clavicle_half = anchors.clavicles.iter().map(|p| p[0].abs()).sum::<f32>() * 0.5;
    let shoulder_half = anchors.shoulders.iter().map(|p| p[0].abs()).sum::<f32>() * 0.5;
    let bridge_span = (shoulder_half - clavicle_half).max(0.001);
    // Openings scale with their anatomical span, not the stomach/waist height.
    // The U sits immediately below the neck base, and the bridge reaches the
    // shoulder instead of ending partway across the anterior clavicle.
    measurements.shoulder[0] = clavicle_half + bridge_span * 0.94;
    measurements.neck[0] = clavicle_half + bridge_span * (0.26 + 0.18 * design.neck_width.unit());
    measurements.neck_center_height =
        anchors.neck_base[1] - bridge_span * (0.045 + 0.10 * design.neck_depth.unit());
    measurements.neck[1] =
        measurements.neck_center_height + bridge_span * (0.11 + 0.12 * design.neck_depth.unit());
    measurements
}

fn outward_surface_anchor(
    origin: [f32; 3],
    direction: [f32; 3],
    padding: f32,
    vertices: &[TorsoShoulderSample],
    faces: &[[u32; 3]],
) -> Result<TorsoShoulderSample, GenerateError> {
    let direction = normalized(direction)?;
    let mut nearest = None::<(f32, [f32; 3], [f32; 3])>;
    for face in faces {
        let [a, b, c] = face.map(|i| &vertices[i as usize]);
        let e1 = sub(b.position, a.position);
        let e2 = sub(c.position, a.position);
        let p = cross(direction, e2);
        let det = dot(e1, p);
        if det.abs() < 1e-10 {
            continue;
        }
        let t = sub(origin, a.position);
        let u = dot(t, p) / det;
        let q = cross(t, e1);
        let v = dot(direction, q) / det;
        let distance = dot(e2, q) / det;
        if u < -1e-5 || v < -1e-5 || u + v > 1.00001 || distance <= 0.0 {
            continue;
        }
        let normal = normalized(add(
            add(scale(a.normal, 1.0 - u - v), scale(b.normal, u)),
            scale(c.normal, v),
        ))?;
        if dot(normal, direction) <= 0.0 {
            continue;
        }
        if nearest.as_ref().is_none_or(|hit| distance < hit.0) {
            nearest = Some((distance, add(origin, scale(direction, distance)), normal));
        }
    }
    nearest
        .map(|(_, point, normal)| TorsoShoulderSample {
            position: add(point, scale(normal, padding)),
            normal,
        })
        .ok_or(GenerateError::InvalidSurface)
}

fn closest_triangle_weights(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = sub(p, b);
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
    let cp = sub(p, c);
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
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let inv = 1.0 / (va + vb + vc);
    let v = vb * inv;
    let w = vc * inv;
    [1.0 - v - w, v, w]
}

fn nearest_surface_gap(p: [f32; 3], vertices: &[TorsoShoulderSample], faces: &[[u32; 3]]) -> f32 {
    let mut best = f32::INFINITY;
    let mut signed = f32::INFINITY;
    for face in faces {
        let samples = face.map(|i| &vertices[i as usize]);
        let w = closest_triangle_weights(
            p,
            samples[0].position,
            samples[1].position,
            samples[2].position,
        );
        let mut point = [0.0; 3];
        let mut normal = [0.0; 3];
        for i in 0..3 {
            point = add(point, scale(samples[i].position, w[i]));
            normal = add(normal, scale(samples[i].normal, w[i]));
        }
        let delta = sub(p, point);
        let distance = dot(delta, delta);
        if distance < best {
            best = distance;
            signed = distance.sqrt() * if dot(delta, normal) >= 0.0 { 1.0 } else { -1.0 };
        }
    }
    signed
}

fn arm_clearance_trim(
    profiles: &LoftProfiles,
    padding: f32,
    vertices: &[TorsoShoulderSample],
    faces: &[[u32; 3]],
    frame: Frame,
) -> Result<(f32, f32), GenerateError> {
    arm_clearance_trim_for_profile(
        profiles.height_range(),
        |y| profiles.eval(y),
        padding,
        vertices,
        faces,
        frame,
    )
}

fn arm_clearance_trim_for_profile(
    range: [f64; 2],
    evaluate: impl Fn(f64) -> [f64; 3],
    padding: f32,
    vertices: &[TorsoShoulderSample],
    faces: &[[u32; 3]],
    frame: Frame,
) -> Result<(f32, f32), GenerateError> {
    let [low, high] = range;
    let mut required = Vec::new();
    for row in 0..=32 {
        let y = low + (high - low) * row as f64 / 32.0;
        let [a, b, c] = evaluate(y);
        let mut safe = None;
        for step in 0..=35 {
            let trim = step as f32 * 0.01;
            let angle = std::f32::consts::FRAC_PI_2 - trim;
            let gap = [-1.0, 1.0]
                .into_iter()
                .map(|sign| {
                    let point = world(
                        [
                            sign * a as f32 * angle.sin(),
                            y as f32,
                            c as f32 + b as f32 * angle.cos(),
                        ],
                        frame,
                    );
                    nearest_surface_gap(point, vertices, faces)
                })
                .fold(f32::INFINITY, f32::min);
            if gap >= padding + 0.001 {
                safe = Some(trim);
                break;
            }
        }
        let trim = safe.ok_or_else(|| {
            eprintln!("angular side has no arm-safe trim within20deg at y={y}");
            GenerateError::InvalidSurface
        })?;
        required.push((y as f32, trim));
    }
    let Some(first) = required.iter().find(|(_, trim)| *trim > 0.0).map(|p| p.0) else {
        return Ok((0.0, low as f32));
    };
    let start = (first - 0.25 * (high - low) as f32).max(low as f32);
    let mut trim = 0.0_f32;
    for (y, need) in required {
        if need <= 0.0 {
            continue;
        }
        let t = ((y - start) / (high as f32 - start)).clamp(0.0, 1.0);
        let weight = t * t * (3.0 - 2.0 * t);
        if weight <= 0.0 {
            return Err(GenerateError::InvalidSurface);
        }
        trim = trim.max(need / weight);
    }
    if trim > 0.35 {
        eprintln!("angular side transition requires excessive trim={trim} start={start}");
        return Err(GenerateError::InvalidSurface);
    }
    eprintln!("angular arm-aware side trim={trim}rad, restoring coronal below y={start}");
    Ok((trim, start))
}

fn angular_affine_depth(surface: &AngularSurface, s: f64, y: f64) -> Option<Vec<f64>> {
    let u = s / surface.arc_scale as f64;
    if !(0.0..=1.0).contains(&u) || y < surface.seam_y as f64 {
        return None;
    }
    let top = surface.top(u as f32);
    let span = top[1] as f64 - surface.seam_y as f64;
    if span <= 1e-6 || y > top[1] as f64 {
        return None;
    }
    let t = (y - surface.seam_y as f64) / span;
    let h00 = 2.0 * t.powi(3) - 3.0 * t * t + 1.0;
    let h10 = t.powi(3) - 2.0 * t * t + t;
    let h01 = -2.0 * t.powi(3) + 3.0 * t * t;
    let h11 = t.powi(3) - t * t;
    let theta = u * surface.coverage_angle(surface.seam_y) as f64;
    let [_, b, c] = surface.profiles.eval(surface.seam_y as f64);
    let [_, db, dc] = surface.profiles.derivative(surface.seam_y as f64);
    let k = surface.top_depth_controls.len();
    let basis = cubic_basis_derivatives(k, u)[0].clone();
    let mut result = vec![h00 * (c + b * theta.cos()) + h10 * span * (dc + db * theta.cos())];
    result.extend(basis.iter().map(|w| h01 * w));
    result.extend(basis.iter().map(|w| h11 * span * w));
    Some(result)
}

fn interpolate_guide_gaps(
    parameters: &[f32],
    values: &[Option<f64>],
) -> Result<Vec<f64>, GenerateError> {
    if parameters.len() != values.len()
        || values.first().is_none_or(Option::is_none)
        || values.last().is_none_or(Option::is_none)
    {
        return Err(GenerateError::InvalidSurface);
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            if let Some(value) = value {
                return Ok(*value);
            }
            let left = (0..index)
                .rev()
                .find(|i| values[*i].is_some())
                .ok_or(GenerateError::InvalidSurface)?;
            let right = (index + 1..values.len())
                .find(|i| values[*i].is_some())
                .ok_or(GenerateError::InvalidSurface)?;
            let t = (parameters[index] - parameters[left]) as f64
                / (parameters[right] - parameters[left]) as f64;
            Ok(values[left].unwrap() * (1.0 - t) + values[right].unwrap() * t)
        })
        .collect()
}

fn add_angular_yoke_bending(
    surface: &AngularSurface,
    h: &mut [Vec<f64>],
    rhs: &mut [f64],
    length: f64,
    energy_multiplier: f64,
) -> Result<(), GenerateError> {
    // The variable part of full XYZ thin-plate energy is exactly the Z term:
    // X/Y are fixed by this compact embedding. Use physical angular arc and
    // height, never a singular Cartesian depth graph at the coronal rail.
    let mut samples = Vec::new();
    let step = 0.01 * (surface.arc_scale as f64).min(length / 0.20);
    let mut area = 0.0;
    for col in 0..32 {
        let u = (col as f64 + 0.5) / 32.0;
        let s = u * surface.arc_scale as f64;
        let span = surface.top(u as f32)[1] as f64 - surface.seam_y as f64;
        for row in 0..24 {
            let y = surface.seam_y as f64 + span * (row as f64 + 0.5) / 24.0;
            let mut stencil = Vec::new();
            for (ds, dy) in [
                (0.0, 0.0),
                (-step, 0.0),
                (step, 0.0),
                (0.0, -step),
                (0.0, step),
                (-step, -step),
                (-step, step),
                (step, -step),
                (step, step),
            ] {
                if let Some(v) = angular_affine_depth(surface, s + ds, y + dy) {
                    stencil.push(v);
                } else {
                    break;
                }
            }
            if stencil.len() != 9 {
                continue;
            }
            let w = surface.arc_scale as f64 / 32.0 * span / 24.0;
            let mut derivatives = vec![vec![0.0; 1 + rhs.len()]; 3];
            for i in 0..=rhs.len() {
                derivatives[0][i] =
                    (stencil[1][i] - 2.0 * stencil[0][i] + stencil[2][i]) / step.powi(2);
                derivatives[1][i] =
                    (stencil[3][i] - 2.0 * stencil[0][i] + stencil[4][i]) / step.powi(2);
                derivatives[2][i] = (stencil[8][i] - stencil[7][i] - stencil[6][i] + stencil[5][i])
                    / (4.0 * step.powi(2));
            }
            samples.push((w, derivatives));
            area += w;
        }
    }
    if area <= 0.0 {
        return Err(GenerateError::InvalidSurface);
    }
    for (w, derivatives) in samples {
        for (term, d) in derivatives.iter().enumerate() {
            let weight =
                length.powi(4) * energy_multiplier * w / area * if term == 2 { 2.0 } else { 1.0 };
            for i in 0..rhs.len() {
                rhs[i] -= weight * d[i + 1] * d[0];
                for j in 0..rhs.len() {
                    h[i][j] += weight * d[i + 1] * d[j + 1];
                }
            }
        }
    }
    eprintln!("angular yoke physical bending: length={length}m area={area}m2 step={step}m");
    Ok(())
}

fn fit_angular_yoke_clearance(
    surface: &mut AngularSurface,
    h: &[Vec<f64>],
    depth_rhs: &[f64],
    slope_rhs: &[f64],
    depth_exact: &[LinearConstraint],
    slope_exact: &[LinearConstraint],
    boundary_bounds: &[LinearConstraint],
    fairing_multiplier: f64,
    padding: f32,
    support: impl Fn(f32, f32) -> Option<f32>,
    frame: Frame,
) -> Result<(), GenerateError> {
    let k = depth_rhs.len();
    let span = surface
        .top_guides
        .iter()
        .map(|p| p[1] - surface.seam_y)
        .fold(0.0_f32, f32::max) as f64;
    let mut matrix = vec![vec![0.0; 2 * k]; 2 * k];
    let mut rhs = vec![0.0; 2 * k];
    for i in 0..k {
        let count = surface.top_guides.len() as f64;
        rhs[i] = depth_rhs[i] / count;
        rhs[k + i] = slope_rhs[i] * span * span / count;
        for j in 0..k {
            matrix[i][j] = h[i][j] / count;
            matrix[k + i][k + j] = h[i][j] * span * span / count;
        }
    }
    // Rigidity changes the energy weight, not the quadrature/stencil geometry.
    // Shrinking its numerical step together with smoothing length changes the
    // sampled objective too, so the slider no longer varies one fixed penalty.
    add_angular_yoke_bending(
        surface,
        &mut matrix,
        &mut rhs,
        0.20 * span,
        fairing_multiplier,
    )?;
    let lift_row = |row: &LinearConstraint, offset: usize| {
        let mut coefficients = vec![0.0; 2 * k];
        coefficients[offset..offset + k].copy_from_slice(&row.coefficients);
        LinearConstraint {
            coefficients,
            value: row.value,
        }
    };
    let exact = depth_exact
        .iter()
        .map(|r| lift_row(r, 0))
        .chain(slope_exact.iter().map(|r| lift_row(r, k)))
        .collect::<Vec<_>>();
    let mut bounds = boundary_bounds
        .iter()
        .map(|r| lift_row(r, 0))
        .collect::<Vec<_>>();
    let mut material_rows = Vec::new();
    for column in 0..=64 {
        let u = column as f32 / 64.0;
        let top = surface.top(u);
        let height = top[1] - surface.seam_y;
        if height <= 1e-6 {
            continue;
        }
        let basis = clamped_cubic_bspline_basis(k, u);
        for row in 1..=32 {
            let t = row as f32 / 32.0;
            let p = surface.local_position([u * surface.arc_scale, surface.seam_y + t * height])?;
            let Some(body) = support(p[0], p[1]) else {
                continue;
            };
            let h01 = -2.0 * t * t * t + 3.0 * t * t;
            let h11 = (t * t * t - t * t) * height;
            let mut coefficients = vec![0.0; 2 * k];
            let mut controlled = 0.0_f64;
            for i in 0..k {
                coefficients[i] = (h01 * basis[i]) as f64;
                coefficients[k + i] = (h11 * basis[i]) as f64;
                controlled += coefficients[i] * surface.top_depth_controls[i] as f64
                    + coefficients[k + i] * surface.top_slope_controls[i] as f64;
            }
            material_rows.push([
                bounds.len() as f64,
                u as f64,
                t as f64,
                p[0] as f64,
                p[1] as f64,
                p[2] as f64,
                (body + padding) as f64,
            ]);
            bounds.push(LinearConstraint {
                coefficients,
                value: (body + padding - p[2]) as f64 + controlled,
            });
        }
    }
    for i in 0..k {
        for sign in [-1.0, 1.0] {
            let mut coefficients = vec![0.0; 2 * k];
            coefficients[k + i] = sign;
            bounds.push(LinearConstraint {
                coefficients,
                value: -8.0,
            });
        }
    }
    if let Some(path) = std::env::var_os("BREASTPLATE_QP_DUMP") {
        let rows = |items: &[LinearConstraint]| {
            items
                .iter()
                .map(|r| {
                    format!(
                        "{{\"coefficients\":{:?},\"value\":{}}}",
                        r.coefficients, r.value
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        };
        let data = format!(
            "{{\"hessian\":{matrix:?},\"rhs\":{rhs:?},\"equalities\":[{}],\"lower_bounds\":[{}],\"material_bound_index_u_t_x_y_initialz_floor\":{material_rows:?},\"top_guides\":{:?},\"top_parameters\":{:?},\"seam_y\":{},\"frame_lateral_vertical_front\":{:?},\"controls\":\"depth[0..8]metres,slope[8..16]dimensionless\"}}",
            rows(&exact),
            rows(&bounds),
            surface.top_guides,
            surface.top_parameters,
            surface.seam_y,
            [frame.lateral, frame.vertical, frame.front]
        );
        std::fs::write(path, data).map_err(|_| GenerateError::InvalidSurface)?;
    }
    let solved =
        solve_dense_qp(&matrix, &rhs, &exact, &bounds, QpOptions::default()).map_err(|e| {
            eprintln!("angular coupled material yoke clearance: {e:?}");
            GenerateError::InvalidSurface
        })?;
    eprintln!(
        "angular coupled material yoke clearance: {} bounds, {:?}",
        bounds.len(),
        solved.diagnostics
    );
    surface.top_depth_controls = solved.coefficients[..k].iter().map(|v| *v as f32).collect();
    surface.top_slope_controls = solved.coefficients[k..].iter().map(|v| *v as f32).collect();
    Ok(())
}

fn support_path_slope(
    point: [f32; 2],
    lateral_slope: f32,
    support: impl Fn(f32, f32) -> Option<f32>,
) -> Option<f64> {
    if !lateral_slope.is_finite() {
        return None;
    }
    let step = 0.010;
    let a = support(point[0] - step * lateral_slope, point[1] - step)?;
    let b = support(point[0] + step * lateral_slope, point[1] + step)?;
    Some(((b - a) / (2.0 * step)) as f64)
}

fn fit_angular_surface(
    design: &BreastplateDesign,
    positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    body_faces: &[[u32; 3]],
    anchors: TorsoUpperRigAnchors,
    clearance_vertices: &[TorsoShoulderSample],
    clearance_faces: &[[u32; 3]],
    enclosure_vertices: &[TorsoShoulderSample],
    enclosure_faces: &[[u32; 3]],
    coronal_levels: &[f32],
    coronal_depths: &[f32],
    frame: Frame,
) -> Result<FittedBodySurface, GenerateError> {
    let layout = CanonicalBreastplateTopology::semantic_layout();
    let mut measurements =
        semantic_measurements(design, positions, semantic_coordinates, anchors, frame);
    // The torso fit domain is independent of the opening's wearer-relative
    // dimensions. Moving the neckline must not refit the accepted lower loft.
    let waist_y = measurements.waist[1];
    let seam_y = waist_y + 0.62 * (measurements.neck[1] - waist_y);
    let waist_radius_delta =
        0.055 * (measurements.neck[1] - waist_y) as f64 * (f64::from(design.waist_width.0) - 740.0)
            / 1_000.0;
    let fairing_multiplier = plate_fairing_multiplier(f64::from(design.rigidity.0) / 1_000.0);
    let padding = design.clearance.metres() + design.wall_thickness.metres() * 0.5 + 0.005;
    let band_policy = BandFraction::from_environment().map_err(shoulder_band_error)?;
    let crest_policy =
        CrestQueryPolicy::from_environment(band_policy).map_err(shoulder_band_error)?;
    let seat_policy =
        ShoulderSeat::from_environment(band_policy, crest_policy).map_err(shoulder_band_error)?;
    let rim_policy = RimPolicy::from_environment(seat_policy).map_err(shoulder_band_error)?;
    let whole_front_policy =
        WholeFrontPolicy::from_environment(rim_policy).map_err(whole_front_error)?;
    let neckline_policy =
        NecklineTreatment::from_environment(band_policy).map_err(shoulder_band_error)?;
    let crest_query_trace = std::cell::RefCell::new(Vec::new());
    let resolved_anchor =
        |anchor: [f32; 3], direction: [f32; 3]| -> Result<[f32; 3], GenerateError> {
            Ok(coordinate(
                outward_surface_anchor(
                    anchor,
                    direction,
                    padding,
                    clearance_vertices,
                    clearance_faces,
                )?
                .position,
                frame,
            ))
        };
    let clavicles = [
        resolved_anchor(anchors.clavicles[0], frame.front)?,
        resolved_anchor(anchors.clavicles[1], frame.front)?,
    ];
    let resolved_crest = |seed: [f32; 3]| -> Result<([f32; 3], [f32; 3]), GenerateError> {
        // Search the superior/anterior exit branch, not the nearest posterior
        // shoulder patch. The authored curve interpolates these sparse anchors
        // continuously; ray choices do not become per-vertex surface samples.
        let mut attempts = Vec::new();
        for &ratio in crest_policy.ratios() {
            let direction = add(frame.vertical, scale(frame.front, ratio));
            if let Ok(hit) = outward_surface_anchor(
                seed,
                direction,
                padding,
                clearance_vertices,
                clearance_faces,
            ) {
                attempts.push(serde_json::json!({ "ratio": ratio, "position": coordinate(hit.position, frame), "normal": coordinate(hit.normal, frame) }));
                if dot(hit.normal, frame.front) >= 0.15 && dot(hit.normal, frame.vertical) >= 0.45 {
                    if band_policy.is_some() {
                        crest_query_trace.borrow_mut().push(serde_json::json!({ "seed_world": seed, "seed_local": coordinate(seed, frame), "accepted_ratio": ratio, "attempts": attempts }));
                    }
                    return Ok((
                        coordinate(hit.position, frame),
                        coordinate(hit.normal, frame),
                    ));
                }
            } else {
                attempts.push(serde_json::json!({ "ratio": ratio, "hit": null }));
            }
        }
        if band_policy.is_some() {
            ShoulderBand::write_dump("shoulder-band-query-failure", &serde_json::json!({ "seed_world": seed, "seed_local": coordinate(seed, frame), "attempts": attempts })).map_err(shoulder_band_error)?;
        }
        Err(GenerateError::InvalidSurface)
    };
    let shoulders = [
        resolved_crest(anchors.shoulders[0])?.0,
        resolved_crest(anchors.shoulders[1])?.0,
    ];
    let local_anchors = TorsoUpperRigAnchors {
        neck_base: coordinate(anchors.neck_base, frame),
        clavicles: anchors.clavicles.map(|p| coordinate(p, frame)),
        shoulders: anchors.shoulders.map(|p| coordinate(p, frame)),
    };
    measurements = angular_upper_measurements(measurements, design, local_anchors);
    let crest_at = |x: f32| -> Result<([f32; 3], [f32; 3]), GenerateError> {
        let mut result = [0.0; 3];
        let mut normal = [0.0; 3];
        for side in 0..2 {
            let a = local_anchors.clavicles[side];
            let b = local_anchors.shoulders[side];
            let t = ((x - a[0].abs()) / (b[0].abs() - a[0].abs())).clamp(0.0, 1.0);
            let seed = add(
                scale(anchors.clavicles[side], 1.0 - t),
                scale(anchors.shoulders[side], t),
            );
            let (p, n) = canonicalize_lateral_anchor(resolved_crest(seed)?);
            result = add(result, scale(p, 0.5));
            normal = add(normal, scale(n, 0.5));
        }
        Ok((result, normalized(normal)?))
    };
    let (inner_crest, inner_normal) = crest_at(measurements.neck[0])?;
    let inner_fractions = std::array::from_fn(|side| {
        let a = local_anchors.clavicles[side][0].abs();
        let b = local_anchors.shoulders[side][0].abs();
        ((measurements.neck[0] - a) / (b - a)).clamp(0.0, 1.0)
    });
    let band = band_policy
        .map(|fraction| {
            ShoulderBand::from_queries(
                fraction,
                crest_policy,
                inner_fractions,
                CrestSample {
                    position: inner_crest.map(f64::from),
                    normal: inner_normal.map(f64::from),
                },
                |side, fraction| {
                    let seed = add(
                        scale(anchors.clavicles[side], 1.0 - fraction),
                        scale(anchors.shoulders[side], fraction),
                    );
                    let hit = resolved_crest(seed)
                        .map_err(|_| BandError::QueryFailed { side, fraction })?;
                    let (position, normal) = canonicalize_lateral_anchor(hit);
                    Ok(CrestSample {
                        position: position.map(f64::from),
                        normal: normal.map(f64::from),
                    })
                },
            )
        })
        .transpose();
    if band_policy.is_some() {
        ShoulderBand::write_dump(
            "crest-query-trace",
            &serde_json::json!({ "policy": crest_policy, "queries": *crest_query_trace.borrow() }),
        )
        .map_err(shoulder_band_error)?;
    }
    let mut band = band.map_err(shoulder_band_error)?;
    if let Some(seat) = seat_policy {
        let mut queries = SeatedCrestQueries::new(
            enclosure_vertices,
            enclosure_faces,
            frame.front,
            frame.vertical,
            seat,
            padding,
        );
        let seated = fit_seated_band(
            band.as_ref().ok_or(GenerateError::InvalidSurface)?,
            seat,
            inner_fractions,
            rim_policy,
            |side, fraction| {
                let seed = add(
                    scale(anchors.clavicles[side], 1.0 - fraction),
                    scale(anchors.shoulders[side], fraction),
                );
                let hit = queries.query(seed, side, fraction)?;
                let (position, normal) = canonicalize_lateral_anchor((
                    coordinate(hit.position.map(|v| v as f32), frame),
                    coordinate(hit.normal.map(|v| v as f32), frame),
                ));
                Ok(CrestSample {
                    position: position.map(f64::from),
                    normal: normal.map(f64::from),
                })
            },
        );
        queries.write_dump().map_err(shoulder_band_error)?;
        band = Some(seated.map_err(shoulder_band_error)?);
    }
    let (inner_crest, inner_normal) =
        if let Some(band) = band.as_ref().filter(|_| seat_policy.is_some()) {
            let inner = band.sample(0.0).map_err(shoulder_band_error)?;
            (
                inner.position.map(|v| v as f32),
                inner.normal.map(|v| v as f32),
            )
        } else {
            (inner_crest, inner_normal)
        };
    let (outer_crest, outer_normal) = if let Some(band) = &band {
        let outer = band.sample(1.0).map_err(shoulder_band_error)?;
        (
            outer.position.map(|v| v as f32),
            outer.normal.map(|v| v as f32),
        )
    } else {
        crest_at(measurements.shoulder[0])?
    };
    let height = outer_crest[1];
    eprintln!(
        "angular shoulder surface anchors clavicles={clavicles:?} shoulders={shoulders:?}; terminal y {} -> {height}",
        measurements.shoulder[1]
    );
    measurements.shoulder = [outer_crest[0], outer_crest[1]];
    measurements.neck = [inner_crest[0], inner_crest[1]];
    let authored_arm = rim_policy
        .arm_opening(band.as_ref(), measurements.armhole_terminal())
        .map_err(shoulder_band_error)?;
    let mut boundary = semantic_boundary_from_measurements(
        &layout,
        design,
        measurements,
        clearance_vertices,
        frame,
        authored_arm.as_ref(),
    );
    let band_stations = band
        .as_ref()
        .map(|b| b.stations(&layout))
        .transpose()
        .map_err(shoulder_band_error)?
        .unwrap_or_default();
    if let Some(band) = &band {
        let neckline = neckline_policy
            .curve(
                f64::from(measurements.neck_center_height),
                band.sample(0.0).map_err(shoulder_band_error)?,
            )
            .map_err(shoulder_band_error)?;
        let range = layout
            .iter()
            .find(|r| r.edge == BreastplateBoundaryEdge::Neck)
            .unwrap();
        let neckline_stations = neckline.stations(range);
        for (index, xy) in &neckline_stations {
            let depth = coordinate(boundary[*index], frame)[2];
            boundary[*index] = world([xy[0] as f32, xy[1] as f32, depth], frame);
        }
        ShoulderBand::write_dump("neckline", &serde_json::json!({
            "curve": neckline, "endpoint_tangent_xy": neckline.endpoint_tangent(),
            "role": { "edge": "Neck", "start": range.start, "segments": range.segments, "includes_shared_endpoint": true },
            "stations": neckline_stations,
            "policy": neckline_policy,
            "continuity": neckline_policy.continuity(),
            "band_inner": band.sample(0.0).map_err(shoulder_band_error)?,
            "target_corner_angles_rad": neckline.corner_angles(band.sample(0.0).map_err(shoulder_band_error)?),
        })).map_err(shoulder_band_error)?;
        for (index, sample) in &band_stations {
            boundary[*index] = world(sample.position.map(|v| v as f32), frame);
        }
        ShoulderBand::write_dump(
            "shoulder-band-context",
            &serde_json::json!({ "padding_m": padding, "band": band }),
        )
        .map_err(shoulder_band_error)?;
    }
    let local_body = positions
        .iter()
        .map(|p| coordinate(*p, frame))
        .collect::<Vec<_>>();
    let local_clearance = clearance_vertices
        .iter()
        .map(|p| coordinate(p.position, frame))
        .collect::<Vec<_>>();
    let mut slices = Vec::new();
    for row in 0..=64 {
        let height = waist_y + (seam_y - waist_y) * row as f32 / 64.0;
        let mut points = Vec::new();
        // Intersect the actual torso triangles, not a percentile of vertices.
        // Every segment endpoint is retained; ellipse convexity then covers
        // the complete straight section segment at this sampled height.
        for face in body_faces {
            let vertices = face.map(|i| local_body[i as usize]);
            for edge in 0..3 {
                let a = vertices[edge];
                let b = vertices[(edge + 1) % 3];
                if (a[1] <= height && b[1] >= height) || (b[1] <= height && a[1] >= height) {
                    let span = b[1] - a[1];
                    if span.abs() > 1e-8 {
                        let t = (height - a[1]) / span;
                        points.push([
                            (a[0] + t * (b[0] - a[0])) as f64,
                            (a[2] + t * (b[2] - a[2])) as f64,
                        ]);
                    }
                }
            }
        }
        if points.is_empty() {
            return Err(GenerateError::InvalidSurface);
        }
        slices.push(TorsoSlice {
            height: height as f64,
            points,
        });
    }
    let physical_coronal = coronal_levels
        .iter()
        .zip(coronal_depths)
        .map(|(level, depth)| {
            [
                regression_height(*level, positions, semantic_coordinates, frame.vertical) as f64,
                *depth as f64,
            ]
        })
        .collect::<Vec<_>>();
    let profiles = fit_loft_profiles(
        &slices,
        &physical_coronal,
        LoftFitOptions {
            clearance_m: padding as f64,
            crown_m: design.crown.metres() as f64,
            waist_radius_delta_m: waist_radius_delta,
            fairing_multiplier,
            ..LoftFitOptions::default()
        },
    )
    .map_err(|error| {
        eprintln!("angular torso profile: {error}");
        GenerateError::InvalidSurface
    })?;
    let (side_trim, trim_start_y) = arm_clearance_trim(
        &profiles,
        padding,
        clearance_vertices,
        clearance_faces,
        frame,
    )?;
    let seam_angle = (std::f32::consts::FRAC_PI_2 - side_trim) as f64;
    let mut local_boundary = boundary
        .iter()
        .map(|p| coordinate(*p, frame))
        .collect::<Vec<_>>();
    let waist = layout
        .iter()
        .find(|r| r.edge == BreastplateBoundaryEdge::Waist)
        .unwrap();
    for index in waist.start..=waist.start + waist.segments {
        local_boundary[index][1] = waist_y;
    }
    let neck = layout
        .iter()
        .find(|r| r.edge == BreastplateBoundaryEdge::Neck)
        .unwrap();
    let start = neck.start + neck.segments / 2;
    let mut top_guides = vec![local_boundary[start]];
    top_guides[0][0] = 0.0;
    for index in start + 1..local_boundary.len() {
        let p = local_boundary[index];
        if p[1] <= seam_y {
            let a = *top_guides.last().unwrap();
            let t = (seam_y - a[1]) / (p[1] - a[1]);
            top_guides.push([a[0] + t * (p[0] - a[0]), seam_y, 0.0]);
            break;
        }
        top_guides.push(p);
    }
    let [a_seam, b_seam, c_seam] = profiles.eval(seam_y as f64);
    let [da_seam, db_seam, dc_seam] = profiles.derivative(seam_y as f64);
    let side_x = a_seam * seam_angle.sin();
    let side_z = c_seam + b_seam * seam_angle.cos();
    let side_dx = da_seam * seam_angle.sin();
    let side_dz = dc_seam + db_seam * seam_angle.cos();
    // Route the lower armscye continuously into the ellipse, with its side
    // tangent at the join. Replacing only the final sample created a 13mm
    // lateral jump over 6mm height and a genuine tangent-plane discontinuity.
    let armhole = layout
        .iter()
        .find(|r| r.edge == BreastplateBoundaryEdge::RightArmhole)
        .unwrap();
    let return_index = (armhole.start + armhole.segments * 7 / 10)
        .saturating_sub(start)
        .clamp(1, top_guides.len() - 2);
    let return_start = top_guides[return_index];
    let upper_dxdy = (top_guides[return_index + 1][0] - top_guides[return_index - 1][0])
        / (top_guides[return_index + 1][1] - top_guides[return_index - 1][1]);
    let return_height = return_start[1] - seam_y;
    for p in &mut top_guides[return_index..] {
        let t = ((p[1] - seam_y) / return_height).clamp(0.0, 1.0);
        p[0] = (2.0 * t.powi(3) - 3.0 * t * t + 1.0) * side_x as f32
            + (t.powi(3) - 2.0 * t * t + t) * return_height * side_dx as f32
            + (-2.0 * t.powi(3) + 3.0 * t * t) * return_start[0]
            + (t.powi(3) - t * t) * return_height * upper_dxdy;
    }
    let mut top_parameters = vec![0.0];
    for pair in top_guides.windows(2) {
        let length = ((pair[1][0] - pair[0][0]).powi(2) + (pair[1][1] - pair[0][1]).powi(2)).sqrt();
        top_parameters.push(top_parameters.last().unwrap() + length);
    }
    let total = *top_parameters.last().unwrap();
    for u in &mut top_parameters {
        *u /= total;
    }
    let support = |x: f32, y: f32| {
        [-x.abs(), x.abs()]
            .into_iter()
            .filter_map(|x| {
                mesh_front_support([x, y], &local_body, body_faces)
                    .into_iter()
                    .chain(mesh_front_support(
                        [x, y],
                        &local_clearance,
                        clearance_faces,
                    ))
                    .reduce(f32::max)
            })
            .reduce(f32::max)
    };
    const K: usize = 8;
    let mut h = vec![vec![0.0; K]; K];
    let mut rhs = vec![0.0; K];
    let mut slope_rhs = vec![0.0; K];
    let mut lower_bounds = Vec::new();
    let mut guide_diagnostics = Vec::new();
    let bridge = layout
        .iter()
        .find(|r| r.edge == BreastplateBoundaryEdge::RightShoulder)
        .unwrap();
    let guide_floors = top_guides
        .iter()
        .map(|p| support(p[0], p[1]).map(|z| z as f64 + padding as f64))
        .collect::<Vec<_>>();
    let mut guide_targets = top_guides
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if (bridge.start..=bridge.start + bridge.segments).contains(&(start + i)) {
                if let Some(sample) = band_stations.get(&(start + i)) {
                    return Some(sample.position[2]);
                }
                let t =
                    ((p[0] - inner_crest[0]) / (outer_crest[0] - inner_crest[0])).clamp(0.0, 1.0);
                Some((inner_crest[2] + t * (outer_crest[2] - inner_crest[2])) as f64)
            } else {
                guide_floors[i].map(|z| z + 0.004)
            }
        })
        .collect::<Vec<_>>();
    *guide_targets.last_mut().unwrap() = Some(side_z);
    let guide_targets = interpolate_guide_gaps(&top_parameters, &guide_targets)?;
    for (i, p) in top_guides.iter_mut().enumerate() {
        let u = top_parameters[i];
        let row = clamped_cubic_bspline_basis(K, u)
            .into_iter()
            .map(f64::from)
            .collect::<Vec<_>>();
        let theta = u as f64 * seam_angle;
        let floor = guide_floors[i];
        let on_bridge = (bridge.start..=bridge.start + bridge.segments).contains(&(start + i));
        let bridge_t =
            ((p[0] - inner_crest[0]) / (outer_crest[0] - inner_crest[0])).clamp(0.0, 1.0);
        let target = guide_targets[i];
        let guide_normal = band_stations.get(&(start + i)).map_or_else(
            || {
                add(
                    scale(inner_normal, 1.0 - bridge_t),
                    scale(outer_normal, bridge_t),
                )
            },
            |sample| sample.normal.map(|v| v as f32),
        );
        let dxdy = if p[1] > seam_y + 1e-6 {
            (p[0] - a_seam as f32 * (u * seam_angle as f32).sin()) / (p[1] - seam_y)
        } else {
            (da_seam * theta.sin()) as f32
        };
        let body_slope = if on_bridge {
            (-(guide_normal[1] + guide_normal[0] * dxdy) / guide_normal[2]) as f64
        } else {
            // This Hermite tangent follows x(y), not a vertical line. Sampling
            // at fixed X omitted dz/dx * dx/dy at the armscye while the bridge
            // already included that chain-rule term through its body normal.
            support_path_slope([p[0], p[1]], dxdy, support)
                .unwrap_or(dc_seam + db_seam * theta.cos())
        };
        guide_diagnostics.push([
            i as f64,
            u as f64,
            p[0] as f64,
            p[1] as f64,
            target,
            floor.unwrap_or(0.0),
            if floor.is_some() { 1.0 } else { 0.0 },
            if on_bridge { 1.0 } else { 0.0 },
        ]);
        for j in 0..K {
            rhs[j] += row[j] * target;
            slope_rhs[j] += row[j] * body_slope;
            for k in 0..K {
                h[j][k] += row[j] * row[k];
            }
        }
        if u < 0.78 {
            // Shoulder/neck stand-off only. The lower armscye returns to the
            // coronal ellipse and is not an anterior-ray garment region.
            if let Some(floor) = floor {
                lower_bounds.push(LinearConstraint {
                    coefficients: row.clone(),
                    value: floor,
                });
            }
            // At a shoulder silhouette the +Z projection can graze a remote
            // body depth. Stand-off belongs to the actual3D crest guide, not
            // that unrelated depth interval. Keep the genuine body floor.
            // The 15mm cap is an anterior-depth modelling allowance beyond
            // an already normal-padded guide, NOT a 15mm normal-distance cap.
            if let Some(ceiling) = if on_bridge {
                Some(target + 0.015)
            } else {
                floor.map(|z| z + 0.015)
            } {
                lower_bounds.push(LinearConstraint {
                    coefficients: row.iter().map(|v| -v).collect(),
                    value: -ceiling,
                });
            }
        }
        p[2] = target as f32;
    }
    let fidelity_h = h.clone();
    // One smooth interpolating guide, not independent station depths.
    for i in 0..K {
        h[i][i] += 1e-8;
    }
    for i in 0..K - 2 {
        for (j, a) in [(i, 1.0), (i + 1, -2.0), (i + 2, 1.0)] {
            for (k, b) in [(i, 1.0), (i + 1, -2.0), (i + 2, 1.0)] {
                h[j][k] += 10.0 * a * b;
            }
        }
    }
    let mut endpoint = vec![0.0; K];
    endpoint[K - 1] = 1.0;
    let mut even_center = vec![0.0; K];
    even_center[0] = -1.0;
    even_center[1] = 1.0;
    let end_y_derivative = (top_guides.last().unwrap()[1] - top_guides[top_guides.len() - 2][1])
        / (top_parameters.last().unwrap() - top_parameters[top_parameters.len() - 2]);
    let mut end_derivative = vec![0.0; K];
    end_derivative[K - 2] = -3.0 * (K - 3) as f64;
    end_derivative[K - 1] = 3.0 * (K - 3) as f64;
    let exact = [
        LinearConstraint {
            coefficients: endpoint.clone(),
            value: side_z,
        },
        LinearConstraint {
            coefficients: even_center.clone(),
            value: 0.0,
        },
        LinearConstraint {
            coefficients: end_derivative.clone(),
            value: side_dz * end_y_derivative as f64 - b_seam * seam_angle.sin() * seam_angle,
        },
    ];
    let joint_requested = std::env::var_os("BREASTPLATE_DIAGNOSTIC_JOINT_PROFILES").is_some();
    let global_requested =
        std::env::var_os("BREASTPLATE_DIAGNOSTIC_GLOBAL_UPPER").is_some() || joint_requested;
    let depth = if global_requested {
        vec![0.0; K]
    } else {
        solve_dense_qp(&h, &rhs, &exact, &lower_bounds, QpOptions::default()).map_err(|e| {
            if let Some(path)=std::env::var_os("BREASTPLATE_QP_DUMP") {
                let rows=|items:&[LinearConstraint]|items.iter().map(|r|format!("{{\"coefficients\":{:?},\"value\":{}}}",r.coefficients,r.value)).collect::<Vec<_>>().join(",");
                let data=format!("{{\"hessian\":{h:?},\"rhs\":{rhs:?},\"equalities\":[{}],\"lower_bounds\":[{}],\"sample_index_u_x_y_target_floor_hit_bridge\":{guide_diagnostics:?}}}",rows(&exact),rows(&lower_bounds));
                let _=std::fs::write(path,data);
            }
            eprintln!("angular yoke guide: {e:?}");
            GenerateError::InvalidSurface
        })?.coefficients
    };
    let slope_exact = [
        LinearConstraint {
            coefficients: endpoint,
            value: side_dz,
        },
        LinearConstraint {
            coefficients: even_center,
            value: 0.0,
        },
        LinearConstraint {
            coefficients: end_derivative,
            value: -db_seam * seam_angle.sin() * seam_angle,
        },
    ];
    let slope = if global_requested {
        vec![0.0; K]
    } else {
        solve_dense_qp(&h, &slope_rhs, &slope_exact, &[], QpOptions::default())
            .map_err(|_| GenerateError::InvalidSurface)?
            .coefficients
    };
    // Guide/torso junction is exact in all coordinates.
    *top_guides.last_mut().unwrap() = [side_x as f32, seam_y, side_z as f32];
    let _ = da_seam;
    let mut angular = AngularSurface {
        joint_profiles: None,
        global_upper: None,
        profiles,
        boundary_domain: vec![],
        top_guides,
        top_parameters,
        top_depth_controls: depth.into_iter().map(|v| v as f32).collect(),
        top_slope_controls: slope.into_iter().map(|v| v as f32).collect(),
        waist_y,
        seam_y,
        arc_scale: (seam_angle * (a_seam + b_seam) * 0.5) as f32,
        side_trim,
        trim_start_y,
    };
    if !global_requested {
        fit_angular_yoke_clearance(
            &mut angular,
            &fidelity_h,
            &rhs,
            &slope_rhs,
            &exact,
            &slope_exact,
            &lower_bounds,
            fairing_multiplier,
            padding,
            support,
            frame,
        )?;
    }
    let upper_parameter = |p: [f32; 3]| {
        let mut best = (f32::INFINITY, 0.0);
        for (index, pair) in angular.top_guides.windows(2).enumerate() {
            let a = pair[0];
            let b = pair[1];
            let d = [b[0] - a[0], b[1] - a[1]];
            let t = (((p[0].abs() - a[0]) * d[0] + (p[1] - a[1]) * d[1])
                / (d[0] * d[0] + d[1] * d[1]).max(1e-12))
            .clamp(0.0, 1.0);
            let distance =
                (p[0].abs() - a[0] - t * d[0]).powi(2) + (p[1] - a[1] - t * d[1]).powi(2);
            if distance < best.0 {
                best = (
                    distance,
                    angular.top_parameters[index]
                        + t * (angular.top_parameters[index + 1] - angular.top_parameters[index]),
                );
            }
        }
        best.1 * p[0].signum()
    };
    let outline = local_boundary
        .iter()
        .map(|p| [p[0], p[1]])
        .collect::<Vec<_>>();
    let boundary_domain = local_boundary
        .iter()
        .map(|p| {
            if p[1] > seam_y {
                [upper_parameter(*p), p[1]]
            } else {
                let mut half = 0.0_f32;
                for (a, b) in outline
                    .iter()
                    .zip(outline.iter().cycle().skip(1))
                    .take(outline.len())
                {
                    if (a[1] <= p[1] && b[1] >= p[1]) || (b[1] <= p[1] && a[1] >= p[1]) {
                        let t = if (b[1] - a[1]).abs() > 1e-8 {
                            (p[1] - a[1]) / (b[1] - a[1])
                        } else {
                            0.0
                        };
                        half = half.max((a[0] + t * (b[0] - a[0])).abs());
                    }
                }
                [(p[0] / half.max(1e-6)).clamp(-1.0, 1.0), p[1]]
            }
        })
        .collect::<Vec<_>>();
    angular.boundary_domain = boundary_domain
        .into_iter()
        .map(|p| [p[0] * angular.arc_scale, p[1]])
        .collect();
    if global_requested {
        let outline_xy = angular
            .boundary_domain
            .iter()
            .map(|p| {
                angular
                    .local_position(*p)
                    .map(|p| [p[0] as f64, p[1] as f64])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let guides = angular
            .top_guides
            .iter()
            .enumerate()
            .map(|(i, p)| UpperGuide {
                xy: [p[0] as f64, p[1] as f64],
                target: p[2] as f64,
                ceiling: (start + i <= bridge.start + bridge.segments)
                    .then_some(p[2] as f64 + 0.015),
                normal: if (bridge.start..=bridge.start + bridge.segments).contains(&(start + i)) {
                    let t = ((p[0] - inner_crest[0]) / (outer_crest[0] - inner_crest[0]))
                        .clamp(0.0, 1.0);
                    Some(band_stations.get(&(start + i)).map_or_else(
                        || add(scale(inner_normal, 1.0 - t), scale(outer_normal, t)).map(f64::from),
                        |sample| sample.normal,
                    ))
                } else {
                    None
                },
            })
            .collect::<Vec<_>>();
        let high = guides.iter().map(|g| g.xy[1]).fold(seam_y as f64, f64::max);
        let global = GlobalUpperLoft {
            elliptical_depth: false,
            seam_y: seam_y as f64,
            span: high - seam_y as f64,
            angle: seam_angle,
            arc_scale: angular.arc_scale as f64,
            seam: [a_seam, b_seam, c_seam],
            seam_derivative: [da_seam, db_seam, dc_seam],
            radius_correction: [0.0; 2],
            prior_depth_correction: 0.0,
            depth_controls: vec![0.0; 32],
        }
        .fit(
            &guides,
            &outline_xy,
            padding as f64,
            fairing_multiplier,
            |x, y| support(x as f32, y as f32).map(f64::from),
        )
        .map_err(|e| {
            eprintln!("global upper loft failed: {e}");
            GenerateError::InvalidSurface
        })?;
        for (point, xy) in angular.boundary_domain.iter_mut().zip(&outline_xy) {
            if point[1] > seam_y {
                point[0] = xy[0].signum() as f32
                    * global
                        .parameter(*xy)
                        .map_err(|_| GenerateError::InvalidSurface)? as f32
                    * angular.arc_scale;
            }
        }
        if joint_requested {
            let old_domain = angular.boundary_domain.clone();
            let mut trim = side_trim;
            let mut trim_start = trim_start_y;
            let mut accepted = None;
            for iteration in 0..3 {
                let return_curve = JointReturn {
                    seam_y: seam_y as f64,
                    upper_xy: [return_start[0] as f64, return_start[1] as f64],
                    upper_dxdy: upper_dxdy as f64,
                    angle: (std::f32::consts::FRAC_PI_2 - trim) as f64,
                    guide_start: return_index,
                    boundary_stations: derived_return_stations(
                        &layout,
                        &outline_xy,
                        seam_y as f64,
                        return_start[1] as f64,
                    ),
                };
                let joint = JointProfiles::fit(
                    &angular.profiles,
                    &global,
                    &guides,
                    &outline_xy,
                    &old_domain,
                    return_curve,
                    padding as f64,
                    fairing_multiplier,
                    trim as f64,
                    trim_start as f64,
                    iteration,
                    |x, y| support(x as f32, y as f32).map(f64::from),
                )
                .map_err(|e| {
                    eprintln!("joint torso iteration {iteration} failed: {e}");
                    GenerateError::InvalidSurface
                })?;
                let (required, required_start) = arm_clearance_trim_for_profile(
                    angular.profiles.height_range(),
                    |y| joint.lower_eval(y),
                    padding,
                    clearance_vertices,
                    clearance_faces,
                    frame,
                )?;
                eprintln!(
                    "joint actual arm-trim check {iteration}: fitted {trim}@{trim_start}, required {required}@{required_start}"
                );
                // Retain the original semantic coverage unless the new shape
                // requires more trim. Never post-trim an already fitted field.
                if required <= trim + 1e-6
                    && (required == 0.0 || required_start >= trim_start - 1e-6)
                {
                    accepted = Some(joint);
                    break;
                }
                trim = trim.max(required);
                trim_start = trim_start.min(required_start);
            }
            let mut joint = accepted.ok_or_else(|| {
                eprintln!("joint torso arm-trim refit did not converge in three certified fits");
                GenerateError::InvalidSurface
            })?;
            if whole_front_policy == WholeFrontPolicy::WholeHeight {
                let band = band.as_ref().ok_or(GenerateError::InvalidSurface)?;
                let first_band_y = band.sample(0.0).map_err(shoulder_band_error)?.position[1]
                    .min(band.sample(1.0).map_err(shoulder_band_error)?.position[1]);
                let body = enclosure_vertices
                    .iter()
                    .map(|v| coordinate(v.position, frame).map(f64::from))
                    .collect::<Vec<_>>();
                install_guide(
                    &mut joint,
                    &body,
                    enclosure_faces,
                    f64::from(design.crown.metres()),
                    f64::from(padding),
                    first_band_y,
                )
                .map_err(whole_front_error)?;
            }
            angular.boundary_domain = joint.boundary_domain.clone();
            angular.side_trim = joint.side_trim as f32;
            angular.trim_start_y = joint.trim_start_y as f32;
            angular.global_upper = Some(joint.upper.clone());
            angular.joint_profiles = Some(joint);
        } else {
            angular.global_upper = Some(global);
        }
    }
    for (position, p) in boundary.iter_mut().zip(&angular.boundary_domain) {
        *position = world(angular.local_position(*p)?, frame);
    }
    if band.is_some() {
        let mut evidence = Vec::new();
        for (index, target) in &band_stations {
            let actual = angular.local_position(angular.boundary_domain[*index])?;
            let normal = angular.local_normal(angular.boundary_domain[*index])?;
            let normal_dot = normal
                .iter()
                .zip(target.normal)
                .map(|(a, b)| f64::from(*a) * b)
                .sum::<f64>();
            evidence.push(serde_json::json!({
                "station": index, "target": target, "actual_position": actual, "actual_normal": normal,
                "depth_error_m": f64::from(actual[2]) - target.position[2],
                "normal_error_degrees": normal_dot.clamp(-1.0, 1.0).acos().to_degrees(),
                "body_floor_m": support(actual[0], actual[1]).map(|z| z + padding),
            }));
        }
        ShoulderBand::write_dump("shoulder-band-fitted-stations", &evidence)
            .map_err(shoulder_band_error)?;
    }
    eprintln!(
        "angular loft fitted {} conservative slices; yoke {} guides; height {waist_y}..{seam_y}",
        slices.len(),
        angular.top_guides.len()
    );
    let report = &angular.profiles.report;
    eprintln!(
        "angular authored fit: waist radius delta={waist_radius_delta}m fairing={fairing_multiplier}; minimum requested/applied extra room={}/{}m saturated slices={}",
        report.minimum_requested_lateral_guard_m,
        report.minimum_applied_lateral_guard_m,
        report.lateral_guard_saturated_slice_count,
    );
    if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
        let report = &angular.profiles.report;
        let stations = (0..=32)
            .map(|i| {
                let y = waist_y as f64 + (seam_y as f64 - waist_y as f64) * i as f64 / 32.0;
                [
                    y,
                    angular.profiles.eval(y)[0],
                    angular.profiles.eval(y)[1],
                    angular.profiles.eval(y)[2],
                ]
            })
            .collect::<Vec<_>>();
        let mut data = format!(
            "{{\"height_range\":{:?},\"width_controls\":{:?},\"depth_controls\":{:?},\"stations_y_a_b_c\":{stations:?},\"anterior_samples\":{},\"inflated_ellipse_residual\":{},\"coronal_fit_error_m\":{},\"lateral_reserve_m\":{},\"top_guides\":{:?},\"top_parameters\":{:?},\"top_depth_controls\":{:?},\"top_slope_controls\":{:?},\"boundary_domain\":{:?},\"frame_lateral_vertical_front\":{:?},\"rig_neck_clavicles_shoulders\":{:?},\"resolved_clavicles_shoulders_local\":{:?}}}",
            angular.profiles.height_range(),
            angular.profiles.width_controls,
            angular.profiles.depth_controls,
            report.anterior_sample_count,
            report.max_inflated_ellipse_residual,
            report.max_coronal_fit_error_m,
            report.minimum_lateral_reserve_m,
            angular.top_guides,
            angular.top_parameters,
            angular.top_depth_controls,
            angular.top_slope_controls,
            angular.boundary_domain,
            [frame.lateral, frame.vertical, frame.front],
            [
                anchors.neck_base,
                anchors.clavicles[0],
                anchors.clavicles[1],
                anchors.shoulders[0],
                anchors.shoulders[1]
            ],
            [clavicles[0], clavicles[1], shoulders[0], shoulders[1]],
        );
        // The lower torso can restore full coronal coverage below a trimmed
        // upper side. These scalars are required to reconstruct non-neutral
        // wearers; a hardcoded right-angle replica is not the fitted field.
        data.pop();
        data.push_str(&format!(
            ",\"side_trim\":{},\"trim_start_y\":{},\"arc_scale_m\":{}}}",
            angular.side_trim, angular.trim_start_y, angular.arc_scale
        ));
        if let Some(global) = &angular.global_upper {
            data.pop();
            data.push_str(&format!(",\"global_upper\":{}}}", global.diagnostic_json()));
        }
        if let Some(joint) = &angular.joint_profiles {
            data.pop();
            data.push_str(&format!(
                ",\"joint_profiles\":{},\"lower_legacy_fields_are_baseline\":true}}",
                joint.diagnostic_json()
            ));
        }
        if let Some(band) = &band {
            data.pop();
            data.push_str(&format!(
                ",\"shoulder_band\":{}}}",
                serde_json::to_string(band).map_err(|_| GenerateError::InvalidSurface)?
            ));
        }
        std::fs::write(path, data).map_err(|_| GenerateError::InvalidSurface)?;
    }
    Ok(FittedBodySurface {
        field: FittedDepthField {
            controls: vec![],
            bottom_y: waist_y,
            top_y: seam_y,
            half_extent: 1.0,
        },
        angular: Some(angular),
        boundary,
        measurements,
        frame,
    })
}

struct FittedBodySurface {
    field: FittedDepthField,
    angular: Option<AngularSurface>,
    boundary: Vec<[f32; 3]>,
    measurements: SemanticMeasurements,
    frame: Frame,
}

fn signed_domain_area(domain: &[[f32; 2]], face: [u32; 3]) -> f64 {
    let [a, b, c] = face.map(|i| domain[i as usize].map(f64::from));
    ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) * 0.5
}

fn domain_orientation_report(
    domain: &[[f32; 2]],
    indices: &[u32],
    expected: f64,
) -> (f64, Vec<[u32; 3]>) {
    let mut min = f64::INFINITY;
    let mut faults = Vec::new();
    for face in indices.as_chunks::<3>().0 {
        let area = signed_domain_area(domain, *face) * expected;
        min = min.min(area);
        if !area.is_finite() || area <= 1e-12 {
            faults.push(*face);
        }
    }
    (min, faults)
}

#[derive(Clone, Copy, PartialEq)]
enum SurfaceEvaluation {
    Quality,
    Export,
}

fn topology_stage(stage: &str) -> Result<(), GenerateError> {
    eprintln!("breastplate topology stage: {stage}");
    if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
        let path = std::path::PathBuf::from(path).with_extension("stage.json");
        let data = format!("{{\"stage\":{stage:?}}}");
        std::fs::write(path, data).map_err(|_| GenerateError::InvalidSurface)?;
    }
    Ok(())
}

impl FittedBodySurface {
    fn evaluate(
        &self,
        topology: &CanonicalBreastplateTopology,
        purpose: SurfaceEvaluation,
    ) -> Result<(Vec<[f32; 3]>, Vec<[f32; 3]>), GenerateError> {
        if let Some(angular) = &self.angular {
            let domain = if topology.is_metric_panel() {
                topology
                    .metric_domain_from_boundary(&angular.boundary_domain, |point| {
                        angular
                            .local_position(point)
                            .map_err(|e| format!("Metric panel field: {e:?}"))
                    })
                    .map_err(|e| {
                        eprintln!("{e}");
                        GenerateError::InvalidSurface
                    })?
            } else {
                topology.semantic_domain_from_boundary_mapped(&angular.boundary_domain, |point| {
                    // Use intrinsic angular arc, not frontal x: dx/dtheta=0
                    // at the coronal endpoint is regular in 3D but collapses
                    // a front-projected boundary strip to zero chart width.
                    point
                })
            };
            let expected = topology
                .indices()
                .as_chunks::<3>()
                .0
                .iter()
                .map(|face| signed_domain_area(topology.canonical_positions(), *face))
                .sum::<f64>()
                .signum();
            let (minimum_area, faults) =
                domain_orientation_report(&domain, topology.indices(), expected);
            let positions = domain
                .iter()
                .map(|point| angular.local_position(*point).map(|p| world(p, self.frame)))
                .collect::<Result<Vec<_>, _>>()?;
            let normals = domain
                .iter()
                .map(|point| angular.local_normal(*point).map(|n| world(n, self.frame)))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP")
                .filter(|_| purpose == SurfaceEvaluation::Export || !faults.is_empty())
            {
                let path = std::path::PathBuf::from(path).with_extension("mesh.json");
                let data = format!(
                    "{{\"domain_arc_y\":{domain:?},\"indices\":{:?},\"mid_positions\":{positions:?},\"minimum_signed_area_m2\":{minimum_area},\"inverted_or_degenerate_faces\":{faults:?},\"arc_scale_m\":{},\"reference_domain\":{:?},\"boundary_count\":{},\"structured_samples\":{:?},\"metric_layout\":{}}}",
                    topology.indices(),
                    angular.arc_scale,
                    topology.canonical_positions(),
                    topology.boundary_vertices().len(),
                    (0..topology.canonical_positions().len())
                        .filter(|i| topology.is_structured_spoke_sample(*i))
                        .collect::<Vec<_>>(),
                    topology.metric_layout_json()
                );
                std::fs::write(path, data).map_err(|_| GenerateError::InvalidSurface)?;
            }
            if !faults.is_empty() {
                eprintln!(
                    "angular embedding has {} invalid triangles; minimumarea={minimum_area}; first={:?}",
                    faults.len(),
                    &faults[..faults.len().min(8)]
                );
                return Err(GenerateError::InvalidSurface);
            }
            return Ok((positions, normals));
        }
        let boundary = semantic_parameter_boundary(&self.boundary, self.measurements, self.frame);
        let waist = self.measurements.waist[1];
        let height = (self.measurements.neck[1] - waist).max(1e-6);
        let physical_x = |point: [f32; 2]| {
            let v = ((point[1] - waist) / height).clamp(0.0, 1.0);
            wrapped_lateral_x(point[0], section_half_width(self.measurements, v), v)
        };
        let positions = topology
            .semantic_domain_from_boundary_mapped(&boundary, |point| [physical_x(point), point[1]])
            .into_iter()
            .map(|point| {
                let x = physical_x(point);
                let v = ((point[1] - waist) / height).clamp(0.0, 1.0);
                let upper = ((v - 0.57) / 0.05).clamp(0.0, 1.0);
                let upper = upper * upper * (3.0 - 2.0 * upper);
                let base = world_with_upright_upper([x, point[1], 0.0], self.frame, upper);
                let depth = self.field.depth(x, point[1])?;
                Ok(add(
                    base,
                    scale(self.frame.front, depth - dot(base, self.frame.front)),
                ))
            })
            .collect::<Result<Vec<_>, GenerateError>>()?;
        let normals = main_radial_normals(topology, &positions, self.frame)?;
        Ok((positions, normals))
    }
}

// Fixed physical quadrature belongs to the semantic garment, not to a CDT.
// Moving/refining/reordering mesh vertices cannot reweight the surface fit.
fn material_contains(boundary: &[[f32; 2]], point: [f32; 2]) -> bool {
    let mut inside = false;
    for (a, b) in boundary
        .iter()
        .zip(boundary.iter().cycle().skip(1))
        .take(boundary.len())
    {
        let edge = [b[0] - a[0], b[1] - a[1]];
        let length2 = edge[0] * edge[0] + edge[1] * edge[1];
        let t = (((point[0] - a[0]) * edge[0] + (point[1] - a[1]) * edge[1]) / length2.max(1e-12))
            .clamp(0.0, 1.0);
        let distance2 =
            (point[0] - a[0] - t * edge[0]).powi(2) + (point[1] - a[1] - t * edge[1]).powi(2);
        if distance2 <= 1e-12 {
            return true;
        }
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

fn armhole_material_samples(boundary: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let layout = CanonicalBreastplateTopology::semantic_layout();
    let armhole = layout
        .iter()
        .find(|range| range.edge == BreastplateBoundaryEdge::RightArmhole)
        .expect("armscye semantics");
    let mut samples = Vec::new();
    // The middle armscye band only: preserve the shoulder return and the
    // underarm-to-coronal transition. Semantic progress, never artifact IDs.
    for sample in 0..=14 {
        let progress = 2.0 / 7.0 + sample as f32 / 28.0;
        let station = progress * armhole.segments as f32;
        let index = station.floor() as usize;
        let fraction = station - index as f32;
        let a = boundary[armhole.start + index];
        let b = boundary[armhole.start + index + 1];
        let edge = [
            a[0] + (b[0] - a[0]) * fraction,
            a[1] + (b[1] - a[1]) * fraction,
        ];
        for offset in [0.0, 0.0025, 0.005, 0.010, 0.015, 0.020, 0.025] {
            let point = [edge[0] - offset, edge[1]];
            if point[0] >= 0.0 && material_contains(boundary, point) {
                samples.push(point);
            }
        }
    }
    samples
}

fn fixed_surface_quadrature(boundary: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut result = boundary.to_vec();
    let half = boundary
        .iter()
        .map(|point| point[0].abs())
        .fold(0.0_f32, f32::max);
    let low = boundary
        .iter()
        .map(|point| point[1])
        .fold(f32::INFINITY, f32::min);
    let high = boundary
        .iter()
        .map(|point| point[1])
        .fold(f32::NEG_INFINITY, f32::max);
    for row in 0..64 {
        let y = low + (high - low) * (row as f32 + 0.5) / 64.0;
        for column in 0..49 {
            let x = half * (column as f32 / 24.0 - 1.0);
            if material_contains(boundary, [x, y]) {
                result.push([x, y]);
            }
        }
    }
    result
}

fn fit_main_surface(
    design: &BreastplateDesign,
    positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    body_faces: &[[u32; 3]],
    anchors: TorsoUpperRigAnchors,
    clearance_vertices: &[TorsoShoulderSample],
    clearance_faces: &[[u32; 3]],
    enclosure_vertices: &[TorsoShoulderSample],
    enclosure_faces: &[[u32; 3]],
    coronal_levels: &[f32],
    coronal_depths: &[f32],
    frame: Frame,
) -> Result<FittedBodySurface, GenerateError> {
    if std::env::var_os("BREASTPLATE_DIAGNOSTIC_ANGULAR_LOFT").is_some() {
        return fit_angular_surface(
            design,
            positions,
            semantic_coordinates,
            body_faces,
            anchors,
            clearance_vertices,
            clearance_faces,
            enclosure_vertices,
            enclosure_faces,
            coronal_levels,
            coronal_depths,
            frame,
        );
    }
    let semantic_layout = CanonicalBreastplateTopology::semantic_layout();
    let boundary = semantic_boundary(
        &semantic_layout,
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
    let physical_boundary = boundary_domain
        .iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            [
                wrapped_lateral_x(point[0], section_half_width(v), v),
                point[1],
            ]
        })
        .collect::<Vec<_>>();
    let domain = fixed_surface_quadrature(&physical_boundary)
        .into_iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            [
                inverse_wrapped_lateral_x(point[0], section_half_width(v), v),
                point[1],
            ]
        })
        .collect::<Vec<_>>();
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
    let exact_projection_at = |sample_x: f32, y: f32| {
        let torso = mesh_front_support([sample_x, y], &local_body_vertices, body_faces);
        let clearance =
            mesh_front_support([sample_x, y], &local_clearance_vertices, clearance_faces);
        torso.into_iter().chain(clearance).reduce(f32::max)
    };
    // Nearby support is only a soft authored-guide fallback. It must never
    // become an obstacle/contact constraint where no actual ray hit exists.
    let exact_support_at = |sample_x: f32, y: f32| {
        exact_projection_at(sample_x, y)
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
        let rounded = side + (center - side) * transverse_crown_shape(q);
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
        let transverse = transverse_crown_shape(physical_q);
        // The neckline occupies a narrow physical half-section, so the same
        // globally fair cosine crown can still turn 50+ degrees between its
        // first two arc-length stations. Flatten only the upper-center
        // parameterization with a C1 window; q=.18 and the complete lower
        // torso retain the accepted crown exactly.
        let upper_center = ((v - 0.80) / 0.16).clamp(0.0, 1.0);
        let upper_center = upper_center * upper_center * (3.0 - 2.0 * upper_center);
        let flattened_q = if physical_q < 0.18 {
            let t = physical_q / 0.18;
            let smoother = t.powi(3) * (t * (t * 6.0 - 15.0) + 10.0);
            physical_q * smoother
        } else {
            physical_q
        };
        let flattened = transverse_crown_shape(flattened_q);
        let transverse = transverse + (flattened - transverse) * upper_center;
        let mut depth = side + (center - side) * transverse;
        // Restrict coronal support to the lateral quadrant. Extending it into
        // the anterior q=.5--.65 samples created ten-centimetre clearance
        // deficits which the low-order lattice necessarily propagated to the
        // otherwise safe side endpoint.
        let old_blend = ((physical_q - 0.68) / 0.30).clamp(0.0, 1.0);
        let old_blend = old_blend * old_blend * (3.0 - 2.0 * old_blend);
        // The exact matched endpoint exposed a low-frequency 30--33 degree
        // turn one cell inside the upper side rail.  Additional physical rows
        // merely moved that hinge, proving that the coronal blend itself was
        // too concentrated.  Spread only the upper lateral transition over
        // q=.54--.98.  The lower shell, q<=.54 diagnostics, and q=1 coronal
        // endpoint remain bit-for-bit on their former branches.
        // Maximum authored crown increases the center-to-coronal depth span.
        // Give only that above-default design range more transverse distance
        // to make the same turn; otherwise its fair continuous field still
        // rotates 30--34 degrees per interior cell near the upper flank.  The
        // default 40 mm crown keeps the approved matched surface bit-for-bit.
        let crown_excess = ((design.crown.metres() - 0.040) / 0.040).clamp(0.0, 1.0);
        let wide_start = 0.54 - 0.34 * crown_excess;
        let wide_blend = ((physical_q - wide_start) / (0.98 - wide_start)).clamp(0.0, 1.0);
        let wide_blend = wide_blend * wide_blend * (3.0 - 2.0 * wide_blend);
        // Begin the maximum-crown transition below the first upper-lateral
        // cell so the same low-frequency turn is distributed vertically as
        // well as transversely. Since `wide_blend == old_blend` at the
        // approved default crown, this does not move its matched surface.
        let upper_lateral = ((v - 0.50) / 0.30).clamp(0.0, 1.0);
        let upper_lateral = upper_lateral * upper_lateral * (3.0 - 2.0 * upper_lateral);
        let depth_blend_q = old_blend + (wide_blend - old_blend) * upper_lateral;
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
    // The authored profile supplies only a soft target. The final physical
    // field owns ALL obstacle response; an earlier 420-step displacement
    // pre-solve both reintroduced semantic-coordinate aliasing and rejected
    // zero-crown designs at an unrelated 8cm intermediate cap.
    let result = domain
        .iter()
        .map(|point| -> Result<[f32; 3], GenerateError> {
            let y = point[1];
            let v = ((y - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            let half_width = section_half_width(v);
            let x = wrapped_lateral_x(point[0], half_width, v);
            let q = (x.abs() / half_width.max(1e-6)).clamp(0.0, 1.0);
            let upper = ((v - 0.57) / 0.05).clamp(0.0, 1.0);
            let upper = upper * upper * (3.0 - 2.0 * upper);
            Ok(world_with_upright_upper(
                [x, y, torso_depth(v, q)?],
                frame,
                upper,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut projection_missing_samples = Vec::new();
    let mut projection_radial_samples = Vec::new();
    let minimum_depths = domain
        .iter()
        .enumerate()
        .map(|(sample, point)| -> Result<Option<f32>, GenerateError> {
            let y = point[1];
            let v = ((y - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            let half_width = section_half_width(v);
            let x = wrapped_lateral_x(point[0], half_width, v);
            let q = (x.abs() / half_width.max(1e-6)).clamp(0.0, 1.0);
            // A mirrored field has to clear both sides independently. An
            // average can sit inside the farther-forward side of the wearer.
            let Some(required) = exact_projection_at(-x.abs(), y)
                .into_iter()
                .chain(exact_projection_at(x.abs(), y))
                .reduce(f32::max)
            else {
                projection_missing_samples.push(sample);
                return Ok(None);
            };
            let required = required + required_depth;
            // An obstacle inequality is the actual body support plus physical
            // clearance, never the authored surface. The upper yoke lies on
            // the anterior shoulder/arm-root envelope even at large q, so its
            // bridge and first armscye rows require these inequalities too.
            // Below the opening, the lateral surface turns onto the coronal
            // rail and closed-body/radial clearance owns that region instead.
            let underarm_v = ((measurements.mid_axillary[1] - waist_y)
                / (neck_y - waist_y).max(1e-6))
            .clamp(0.0, 1.0);
            if q > ANTERIOR_CLEARANCE_Q && v <= underarm_v + 0.05 {
                projection_radial_samples.push(sample);
                return Ok(None);
            }
            let upper = ((v - 0.57) / 0.05).clamp(0.0, 1.0);
            let upper = upper * upper * (3.0 - 2.0 * upper);
            Ok(Some(dot(
                world_with_upright_upper([x, y, required], frame, upper),
                frame.front,
            )))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(path) = std::env::var_os("BREASTPLATE_QP_DUMP") {
        let path = std::path::PathBuf::from(path).with_extension("projection.json");
        let valid = minimum_depths
            .iter()
            .filter(|value| value.is_some())
            .count();
        let dump = format!(
            "{{\"valid_anterior_constraint_count\":{valid},\"fallback_constraint_count\":0,\"missing_projection_samples\":{projection_missing_samples:?},\"radial_region_samples\":{projection_radial_samples:?}}}"
        );
        if let Err(error) = std::fs::write(path, dump) {
            eprintln!("breastplate body projection diagnostic dump failed: {error}");
        }
    }
    let resolved_boundary_depth = |sample_index: usize| -> Result<f32, GenerateError> {
        let shell = coordinate(boundary[sample_index], frame);
        exact_projection_at(-shell[0].abs(), shell[1])
            .into_iter()
            .chain(exact_projection_at(shell[0].abs(), shell[1]))
            .reduce(f32::max)
            .map(|depth| depth + required_depth)
            .ok_or_else(|| {
                eprintln!("breastplate upper anchor {sample_index} has no exact body projection at {shell:?}");
                GenerateError::InvalidSurface
            })
    };
    let semantic_range = |edge: BreastplateBoundaryEdge| {
        semantic_layout
            .iter()
            .find(|range| range.edge == edge)
            .expect("complete semantic boundary")
    };
    let neck = semantic_range(BreastplateBoundaryEdge::Neck);
    let right_shoulder = semantic_range(BreastplateBoundaryEdge::RightShoulder);
    let right_armhole = semantic_range(BreastplateBoundaryEdge::RightArmhole);
    let right_side = semantic_range(BreastplateBoundaryEdge::RightSide);
    let mut hard_depths = vec![
        (
            neck.start + neck.segments / 2,
            resolved_boundary_depth(neck.start + neck.segments / 2)?,
        ),
        (
            right_shoulder.start,
            resolved_boundary_depth(right_shoulder.start)?,
        ),
        (
            right_armhole.start,
            resolved_boundary_depth(right_armhole.start)?,
        ),
    ];
    // The lower side wrap is an invariant of the final field, not merely an
    // input target that a later global translation may erase. Constrain a
    // sparse set of q=1 samples to the analytic coronal rail; the intervening
    // values remain the same low-order longitudinal spline.
    for offset in (0..=right_side.segments).step_by(4) {
        let sample = right_side.start + offset;
        let point = domain[sample];
        let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
        let x = wrapped_lateral_x(point[0], section_half_width(v), v);
        let depth =
            coronal_depth(canonical_coronal_level(v), coronal_levels, coronal_depths)? + 0.0018;
        let upper = ((v - 0.57) / 0.05).clamp(0.0, 1.0);
        let upper = upper * upper * (3.0 - 2.0 * upper);
        hard_depths.push((
            sample,
            dot(
                world_with_upright_upper([x, point[1], depth], frame, upper),
                frame.front,
            ),
        ));
    }
    let field = fair_depth_lattice(
        boundary.len(),
        result,
        &domain,
        &minimum_depths,
        &hard_depths,
        waist_y,
        neck_y,
        section_half_width,
        frame,
    )?;
    Ok(FittedBodySurface {
        field,
        angular: None,
        boundary,
        measurements,
        frame,
    })
}

fn fair_depth_lattice(
    boundary_count: usize,
    positions: Vec<[f32; 3]>,
    domain: &[[f32; 2]],
    minimum_depths: &[Option<f32>],
    hard_depths: &[(usize, f32)],
    waist_y: f32,
    neck_y: f32,
    section_half_width: impl Fn(f32) -> f32,
    frame: Frame,
) -> Result<FittedDepthField, GenerateError> {
    const Q: usize = 7;
    // The surface is evaluated on |q| and mirrored.  Identifying the first
    // two clamped cubic controls makes d(depth)/dq exactly zero at q=0 for
    // every longitudinal row, rather than merely penalizing a crease in the
    // upper three rows after it appears.
    const FREE_Q: usize = Q - 1;
    const V: usize = 9;
    const N: usize = FREE_Q * V;
    // The neckline is not the top of the garment: upper bridges sit above it.
    // Preserve their physical height in the fitting chart rather than
    // collapsing distinct shoulder samples onto one clamped endpoint row.
    let fit_top_y = domain[..boundary_count]
        .iter()
        .map(|point| point[1])
        .fold(neck_y, f32::max)
        + 0.005;
    let fit_bottom_y = domain[..boundary_count]
        .iter()
        .map(|point| point[1])
        .fold(waist_y, f32::min);
    // Use an independent physical transverse chart as well. Semantic torso
    // widths may be narrower than an armscye/side curve at the same height;
    // dividing by them and clamping q collapsed distinct lateral points.
    // One constant physical extent makes the tensor chart injective and
    // avoids a hidden height-dependent derivative in the fitted field.
    let fit_half_extent = domain[..boundary_count]
        .iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            wrapped_lateral_x(point[0], section_half_width(v), v).abs()
        })
        .fold(0.0_f32, f32::max)
        + 0.005;
    let targets = positions
        .iter()
        .map(|point| dot(*point, frame.front))
        .collect::<Vec<_>>();
    // The upper yoke returns over clavicle/shoulder rather than continuing
    // the torso's defensive bow. Body samples are broad admissible intervals
    // and soft observations of this same low-order field, never interpolated
    // final vertices. A C1 window leaves the lower authored crown unchanged.
    let yoke_weights = domain
        .iter()
        .map(|point| {
            let v = (point[1] - waist_y) / (neck_y - waist_y).max(1e-6);
            let t = ((v - 0.70) / 0.16).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        })
        .collect::<Vec<_>>();
    let fit_targets = targets
        .iter()
        .zip(minimum_depths)
        .zip(&yoke_weights)
        .map(|((target, minimum), weight)| {
            minimum.map_or(*target, |minimum| {
                target + (minimum + 0.004 - target) * weight
            })
        })
        .collect::<Vec<_>>();
    let mut field = FittedDepthField {
        controls: vec![],
        bottom_y: fit_bottom_y,
        top_y: fit_top_y,
        half_extent: fit_half_extent,
    };
    let weights = domain
        .iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            field.weights(
                wrapped_lateral_x(point[0], section_half_width(v), v),
                point[1],
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let physical_samples = domain
        .iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            [
                wrapped_lateral_x(point[0], section_half_width(v), v),
                point[1],
            ]
        })
        .collect::<Vec<_>>();
    let grid_half_extent = physical_samples[..boundary_count]
        .iter()
        .map(|p| p[0].abs())
        .fold(0.0_f32, f32::max) as f64;
    let grid_low = physical_samples[..boundary_count]
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min) as f64;
    let grid_high = physical_samples[..boundary_count]
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max) as f64;
    let cell_area = (2.0 * grid_half_extent / 48.0) * ((grid_high - grid_low) / 64.0);
    // The fixed grid is midpoint-sampled vertically and trapezoidal in x.
    // Perimeter constraints remain active, but boundary points have no 2D
    // measure and are not extra area observations of the authored target.
    let quadrature_areas = physical_samples
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i < boundary_count {
                0.0
            } else {
                cell_area
                    * if ((p[0].abs() as f64) - grid_half_extent).abs() < 1e-6 {
                        0.5
                    } else {
                        1.0
                    }
            }
        })
        .collect::<Vec<_>>();
    let material_area = quadrature_areas.iter().sum::<f64>();
    if !material_area.is_finite() || material_area <= 0.0 {
        return Err(GenerateError::InvalidSurface);
    }
    let smoothing_fraction = if std::env::var_os("BREASTPLATE_DIAGNOSTIC_SURFACE_PREVIEW").is_some()
    {
        match std::env::var("BREASTPLATE_DIAGNOSTIC_SMOOTHING_FRACTION") {
            Ok(value) => {
                let fraction = value
                    .parse::<f64>()
                    .map_err(|_| GenerateError::InvalidSurface)?;
                if !fraction.is_finite() || !(0.005..=0.2).contains(&fraction) {
                    return Err(GenerateError::InvalidSurface);
                }
                fraction
            }
            Err(_) => 0.05,
        }
    } else {
        0.05
    };
    let smoothing_length = smoothing_fraction * (neck_y - waist_y) as f64;
    let bending_scale = smoothing_length.powi(4);
    let mut matrix = vec![vec![0.0_f64; N]; N];
    let mut rhs = vec![0.0_f64; N];
    let mut bending_rows = Vec::with_capacity(domain.len());
    for (sample, ((row, target), area)) in weights
        .iter()
        .zip(&fit_targets)
        .zip(&quadrature_areas)
        .enumerate()
    {
        let weight = area / material_area;
        if weight == 0.0 {
            bending_rows.push(None);
            continue;
        }
        let support = row
            .iter()
            .enumerate()
            .filter_map(|(i, a)| (a.abs() > 1e-12).then_some((i, *a as f64)))
            .collect::<Vec<_>>();
        for &(left, a) in &support {
            rhs[left] += weight * a * *target as f64;
            for &(right, b) in &support {
                matrix[left][right] += weight * a * b;
            }
        }
        let point = physical_samples[sample];
        let [_, _, dxx, dxy, dyy] = field.physical_derivatives(point[0], point[1])?;
        let derivatives = [dxx, dxy, dyy];
        for (derivative, multiplicity) in derivatives.iter().zip([1.0, 2.0, 1.0]) {
            let support = derivative
                .iter()
                .enumerate()
                .filter_map(|(i, a)| (a.abs() > 1e-12).then_some((i, *a)))
                .collect::<Vec<_>>();
            for &(left, a) in &support {
                for &(right, b) in &support {
                    matrix[left][right] += bending_scale * weight * multiplicity * a * b;
                }
            }
        }
        bending_rows.push(Some(derivatives));
    }
    // Data fixes the physical affine nullspace; this tiny numerical diagonal
    // only protects unused virtual-domain controls, not surface curvature.
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] += 1e-12;
    }
    // Anatomical proxy locations have physical fit tolerances; pinning eight
    // noisy coronal samples exactly forces the low-order field to overshoot
    // the adjacent armhole. Exact centre symmetry is already reduced into
    // the degrees of freedom. Fit intervals below retain body-relative wrap
    // without mistaking measured proxies for exact spline interpolation.
    let hard_rows = Vec::<LinearConstraint>::new();
    let mut inequality_rows = minimum_depths
        .iter()
        .enumerate()
        .filter_map(|(sample, required)| {
            required.map(|required| LinearConstraint {
                coefficients: weights[sample].iter().map(|value| *value as f64).collect(),
                value: required as f64,
            })
        })
        .collect::<Vec<_>>();
    let clearance_row_count = inequality_rows.len();
    for (sample, minimum) in minimum_depths.iter().enumerate() {
        if let Some(minimum) = minimum {
            let weight = yoke_weights[sample];
            if weight >= 0.5 {
                // Cap free inter-anchor inflation while allowing the fair
                // field 12--25mm above its body-clearance floor. This is an
                // inequality, not an anatomical surface interpolation.
                let maximum = minimum + 0.012 + 0.025 * (1.0 - weight);
                inequality_rows.push(LinearConstraint {
                    coefficients: weights[sample].iter().map(|value| -*value as f64).collect(),
                    value: -maximum as f64,
                });
            }
        }
    }
    let yoke_upper_row_count = inequality_rows.len() - clearance_row_count;
    // Order the lower five control rows of the defensive torso crown. The
    // upper four rows span the neck opening and separate shoulder bridges:
    // ordering that virtual, unmeshed half-section conflicts with the actual
    // shoulder return and axilla clearance. Their shape remains fair and
    // body-constrained, with the same exact even-C1 centre identification.
    const TORSO_MONOTONE_ROWS: usize = 5;
    for row in 0..TORSO_MONOTONE_ROWS {
        // The last control interval turns onto the radial/coronal side and is
        // not ordered by anterior depth. Keep the convex monotone contract on
        // the actual front quadrant; closed-body/coronal constraints own the
        // terminal lateral interval.
        for column in 0..FREE_Q - 2 {
            let mut weights = vec![0.0_f32; N];
            weights[row * FREE_Q + column] = 1.0;
            weights[row * FREE_Q + column + 1] = -1.0;
            inequality_rows.push(LinearConstraint {
                coefficients: weights.into_iter().map(f64::from).collect(),
                value: 0.0,
            });
        }
    }
    let monotonic_row_count = inequality_rows.len() - clearance_row_count - yoke_upper_row_count;
    let physical_boundary = domain[..boundary_count]
        .iter()
        .map(|point| {
            let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
            [
                wrapped_lateral_x(point[0], section_half_width(v), v),
                point[1],
            ]
        })
        .collect::<Vec<_>>();
    let material_fairness_samples = armhole_material_samples(&physical_boundary);
    for point in &material_fairness_samples {
        // Both conditions are local physical contracts: no outward return
        // (first derivative) and a defensive transverse section (second).
        // No rows are added across the neck hole or virtual shoulder domain.
        for coefficients in field.transverse_derivatives(point[0], point[1])? {
            inequality_rows.push(LinearConstraint {
                coefficients: coefficients.into_iter().map(|value| -value).collect(),
                value: 0.0,
            });
        }
    }
    for (anchor, (sample, depth)) in hard_depths.iter().enumerate() {
        let (minimum, maximum) = if anchor < 3 {
            // Neck/clavicle/shoulder use the identical evaluated obstacle
            // floor, avoiding separately transformed RHS rounding conflicts.
            let minimum = minimum_depths[*sample].ok_or(GenerateError::InvalidSurface)?;
            (minimum, minimum + 0.004)
        } else {
            // Coronal rail: at most 2mm anterior tolerance, up to10mm into
            // the half-wrap. Closed-body parity remains independent.
            (depth - 0.010, depth + 0.002)
        };
        let coefficients = weights[*sample]
            .iter()
            .map(|value| *value as f64)
            .collect::<Vec<_>>();
        inequality_rows.push(LinearConstraint {
            coefficients: coefficients.clone(),
            value: minimum as f64,
        });
        inequality_rows.push(LinearConstraint {
            coefficients: coefficients.iter().map(|value| -*value).collect(),
            value: -maximum as f64,
        });
    }
    let anchor_interval_row_count = hard_depths.len() * 2;
    let target_min = targets
        .iter()
        .copied()
        .chain(hard_depths.iter().map(|(_, depth)| *depth))
        .fold(f32::INFINITY, f32::min);
    let target_max = targets
        .iter()
        .copied()
        .chain(hard_depths.iter().map(|(_, depth)| *depth))
        .chain(minimum_depths.iter().filter_map(|depth| *depth))
        .fold(f32::NEG_INFINITY, f32::max);
    let control_min = target_min - 0.080;
    let control_max = target_max + 0.080;
    for control in 0..N {
        let mut lower = vec![0.0_f64; N];
        lower[control] = 1.0;
        inequality_rows.push(LinearConstraint {
            coefficients: lower.clone(),
            value: control_min as f64,
        });
        lower[control] = -1.0;
        inequality_rows.push(LinearConstraint {
            coefficients: lower,
            value: -control_max as f64,
        });
    }
    let hessian = matrix;
    if let Some(path) = std::env::var_os("BREASTPLATE_QP_DUMP") {
        let constraint_json = |rows: &[LinearConstraint]| {
            format!(
                "[{}]",
                rows.iter()
                    .map(|row| format!(
                        "{{\"coefficients\":{:?},\"value\":{}}}",
                        row.coefficients, row.value
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        let sample_rows = minimum_depths
            .iter()
            .enumerate()
            .filter_map(|(sample, value)| value.map(|_| sample))
            .collect::<Vec<_>>();
        let hard_json = hard_depths
            .iter()
            .map(|(sample, value)| format!("[{sample},{value}]"))
            .collect::<Vec<_>>()
            .join(",");
        let parameter_samples = domain
            .iter()
            .map(|point| {
                let v = ((point[1] - waist_y) / (neck_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
                let q =
                    wrapped_lateral_x(point[0], section_half_width(v), v).abs() / fit_half_extent;
                let fit_v =
                    ((point[1] - waist_y) / (fit_top_y - waist_y).max(1e-6)).clamp(0.0, 1.0);
                [q, fit_v]
            })
            .collect::<Vec<_>>();
        let mut dump = format!(
            "{{\"hessian\":{hessian:?},\"rhs\":{rhs:?},\"equalities\":{},\"lower_bounds\":{},\"hard_samples\":[{hard_json}],\"lower_sample_indices\":{sample_rows:?},\"sample_qv\":{parameter_samples:?},\"domain\":{domain:?},\"targets\":{targets:?},\"waist_y\":{waist_y},\"semantic_neck_y\":{neck_y},\"fit_top_y\":{fit_top_y},\"fit_half_extent\":{fit_half_extent},\"clearance_row_count\":{clearance_row_count},\"monotonic_row_count\":{monotonic_row_count},\"control_min\":{control_min},\"control_max\":{control_max}}}",
            constraint_json(&hard_rows),
            constraint_json(&inequality_rows),
        );
        dump.pop();
        dump.push_str(&format!(
            ",\"fit_targets\":{fit_targets:?},\"yoke_weights\":{yoke_weights:?},\"yoke_upper_row_count\":{yoke_upper_row_count},\"anchor_interval_row_count\":{anchor_interval_row_count},\"material_fairness_samples\":{material_fairness_samples:?},\"fit_bottom_y\":{fit_bottom_y},\"material_area_m2\":{material_area},\"smoothing_length_m\":{smoothing_length},\"quadrature_areas_m2\":{quadrature_areas:?}}}"
        ));
        if let Err(error) = std::fs::write(&path, dump) {
            eprintln!("breastplate QP diagnostic dump failed: {error}");
        }
    }
    let solution = solve_dense_qp(
        &hessian,
        &rhs,
        &hard_rows,
        &inequality_rows,
        QpOptions {
            max_sweeps: 50_000,
            primal_tolerance: 2.5e-5,
            equality_tolerance: 2.5e-6,
            // The area-normalized physical objective has units m². Certify
            // its dual gap well below the squared geometric tolerance.
            complementarity_tolerance: 1e-10,
            ..QpOptions::default()
        },
    )
    .map_err(|error| {
        eprintln!(
            "breastplate fair lattice QP {:?}: {} diagnostics={:?}",
            error.kind, error.message, error.diagnostics
        );
        GenerateError::InvalidSurface
    })?;
    let controls = solution
        .coefficients
        .iter()
        .map(|value| *value as f32)
        .collect::<Vec<_>>();
    let maximum_multiplier = solution
        .lower_bound_multipliers
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    if let Some(path) = std::env::var_os("BREASTPLATE_QP_DUMP") {
        let path = std::path::PathBuf::from(path).with_extension("solution.json");
        let diagnostics = &solution.diagnostics;
        let dump = format!(
            "{{\"coefficients\":{:?},\"sweeps\":{},\"max_equality_residual\":{},\"max_lower_bound_violation\":{},\"max_complementarity\":{},\"projected_stationarity_residual\":{},\"max_projected_update\":{},\"max_multiplier\":{maximum_multiplier}}}",
            solution.coefficients,
            diagnostics.sweeps,
            diagnostics.max_equality_residual,
            diagnostics.max_lower_bound_violation,
            diagnostics.max_complementarity,
            diagnostics.projected_stationarity_residual,
            diagnostics.max_projected_update,
        );
        if let Err(error) = std::fs::write(path, dump) {
            eprintln!("breastplate QP solution diagnostic dump failed: {error}");
        }
    }
    if std::env::var_os("BREASTPLATE_QP_DIAGNOSTICS").is_some() {
        eprintln!(
            "breastplate fair lattice QP diagnostics={:?} max_multiplier={maximum_multiplier}",
            solution.diagnostics
        );
    }
    field.controls = controls;
    if let Some(path) = std::env::var_os("BREASTPLATE_QP_DUMP") {
        let evaluate = |row: &[f64]| {
            row.iter()
                .zip(&field.controls)
                .map(|(a, b)| a * *b as f64)
                .sum::<f64>()
        };
        let mut data_energy = 0.0_f64;
        let mut bending_energy = [0.0_f64; 3];
        let mut region_energy = [0.0_f64; 4];
        for (sample, area) in quadrature_areas.iter().enumerate() {
            let Some(rows) = &bending_rows[sample] else {
                continue;
            };
            let weight = area / material_area;
            let point = physical_samples[sample];
            let difference = field.depth(point[0], point[1])? as f64 - fit_targets[sample] as f64;
            data_energy += 0.5 * weight * difference * difference;
            let region = usize::from(point[1] > waist_y + (neck_y - waist_y) * 0.6) * 2
                + usize::from(point[0].abs() as f64 > grid_half_extent * 0.5);
            for (axis, (row, multiplicity)) in rows.iter().zip([1.0, 2.0, 1.0]).enumerate() {
                let energy = 0.5 * bending_scale * weight * multiplicity * evaluate(row).powi(2);
                bending_energy[axis] += energy;
                region_energy[region] += energy;
            }
        }
        // A denser, staggered physical grid audits material between fit sites.
        let mut hessian_norms = Vec::<f64>::new();
        let mut positive_dx = 0;
        let mut positive_dxx = 0;
        let mut maximum = (0.0_f64, [0.0_f32; 2]);
        for row in 0..129 {
            let y = (grid_low + (grid_high - grid_low) * (row as f64 + 0.5) / 129.0) as f32;
            for column in 0..97 {
                let x = (grid_half_extent * (2.0 * (column as f64 + 0.5) / 97.0 - 1.0)) as f32;
                if !material_contains(&physical_boundary, [x, y]) {
                    continue;
                }
                let derivatives = field.physical_derivatives(x, y)?;
                let values = derivatives.map(|row| evaluate(&row));
                positive_dx += usize::from(values[0] > 1e-6);
                positive_dxx += usize::from(values[2] > 1e-6);
                let magnitude =
                    (values[2].powi(2) + 2.0 * values[3].powi(2) + values[4].powi(2)).sqrt();
                if magnitude > maximum.0 {
                    maximum = (magnitude, [x, y]);
                }
                hessian_norms.push(magnitude);
            }
        }
        hessian_norms.sort_by(f64::total_cmp);
        let count = hessian_norms.len();
        let percentile = |p: f64| hessian_norms[((count - 1) as f64 * p).round() as usize];
        let dump = format!(
            "{{\"area_m2\":{material_area},\"smoothing_length_m\":{smoothing_length},\"data_energy_m2\":{data_energy},\"bending_energy_xx_xy_yy_m2\":{bending_energy:?},\"bending_energy_lower_center_lower_side_upper_center_upper_side_m2\":{region_energy:?},\"heldout_samples\":{count},\"positive_outward_slope_samples\":{positive_dx},\"positive_transverse_second_derivative_samples\":{positive_dxx},\"graph_hessian_norm_per_m_p50_p95_p99_max\":[{},{},{},{}],\"maximum_hessian_point_xy\":{:?}}}",
            percentile(0.5),
            percentile(0.95),
            percentile(0.99),
            maximum.0,
            maximum.1,
        );
        let path = std::path::PathBuf::from(path).with_extension("fairness.json");
        if let Err(error) = std::fs::write(path, dump) {
            eprintln!("breastplate physical fairness diagnostic failed: {error}");
        }
    }
    Ok(field)
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
    // The waist boundary is sampled for the main-shell CDT, whose spacing is
    // intentionally coarser than the curvature-sized ruled skirt needs at
    // center front.  Re-sample every linear waist edge once for the separate,
    // duplicated skirt seam.  These midpoints lie exactly on the accepted
    // piecewise-linear seam, retain its sharp duplication, and halve the
    // adjacent-normal rotation without changing the main topology.
    let mut seam_points = Vec::with_capacity(seam_indices.len() * 2 - 1);
    for pair in seam_indices.windows(2) {
        let first = main[pair[0]];
        let second = main[pair[1]];
        seam_points.push(first);
        seam_points.push(scale(add(first, second), 0.5));
    }
    seam_points.push(main[*seam_indices.last().expect("waist seam")]);
    let columns = seam_points.len();
    // Default physical skirt length is 0.18 of the main semantic height.
    let length = torso_height * design.skirt_length.unit();
    let right = coordinate(seam_points[0], frame);
    let left = coordinate(*seam_points.last().expect("waist seam"), frame);
    let coronal_origin = (right[2] + left[2]) * 0.5;
    // Preserve row zero exactly, but let the bell leave that seam with a fair
    // transverse tangent.  The accepted waist polyline has a curvature-sized
    // center-front corner; extruding it verbatim repeats that corner down all
    // three skirt rows and produces a 38--42 degree vertical hinge.  Thirteen
    // open-curve Laplacian passes define a low-frequency ruled target while a
    // cubic ease-out keeps the seam positions exact and removes the repeated
    // kink by the first physical row.  Endpoints stay fixed, so the honest
    // coronal extent and terminal columns are unchanged.
    let mut fair_depths = seam_points
        .iter()
        .map(|point| coordinate(*point, frame)[2])
        .collect::<Vec<_>>();
    for _ in 0..13 {
        let previous = fair_depths.clone();
        for column in 1..columns - 1 {
            fair_depths[column] =
                previous[column] * 0.5 + (previous[column - 1] + previous[column + 1]) * 0.25;
        }
    }
    let mut skirt = Vec::with_capacity((SKIRT_ROWS + 1) * columns);
    for row in 0..=SKIRT_ROWS {
        let t = row as f32 / SKIRT_ROWS as f32;
        let fair_t = 1.0 - (1.0 - t).powi(4);
        for (column, &seam) in seam_points.iter().enumerate() {
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
            let raw_fair_delta = (fair_depths[column] - local[2]) * fair_t;
            // Fairing must not make a ruled column initially retreat toward
            // the wearer. Retain only the component aligned with this
            // column's horizontal radial generator; row zero remains exact.
            let fair_delta = if raw_fair_delta * radial[2] >= 0.0 {
                raw_fair_delta
            } else {
                0.0
            };
            skirt.push(add(
                seam,
                add(
                    scale(frame.vertical, -length * t),
                    add(
                        scale(frame.lateral, radial[0] / radial_length * lateral_flare),
                        scale(
                            frame.front,
                            radial[2] / radial_length * front_flare + fair_delta,
                        ),
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
) -> Result<(), GenerateError> {
    // Boundary traversal is a semantic ordering, not an orientation contract.
    // Each rim must oppose its incident outer face (and the reversed inner
    // face), regardless of which direction the caller lists the boundary.
    let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
    for &[a, b, c] in faces {
        if a == b || b == c || c == a {
            return Err(GenerateError::InvalidSurface);
        }
        for (a, b) in [(a, b), (b, c), (c, a)] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
    }
    if boundary.len() < 3
        || edges.values().any(|incidents| match incidents.as_slice() {
            [_] => false,
            [a, b] => *a != (b.1, b.0),
            _ => true,
        })
    {
        return Err(GenerateError::InvalidSurface);
    }
    let mut rim = Vec::with_capacity(boundary.len());
    let mut seen = BTreeSet::new();
    for index in 0..boundary.len() {
        let a = boundary[index];
        let b = boundary[(index + 1) % boundary.len()];
        let key = (a.min(b), a.max(b));
        let Some(incidents) = edges.get(&key) else {
            return Err(GenerateError::InvalidSurface);
        };
        if incidents.len() != 1 || !seen.insert(key) {
            return Err(GenerateError::InvalidSurface);
        }
        let reverse = incidents[0] == (a, b);
        rim.push((a + vertex_offset, b + vertex_offset, reverse));
    }
    if seen.len()
        != edges
            .values()
            .filter(|incidents| incidents.len() == 1)
            .count()
    {
        return Err(GenerateError::InvalidSurface);
    }
    for face in faces {
        let outer = face.map(|index| index + vertex_offset);
        indices.extend(outer);
        indices.extend([outer[0] + layer, outer[2] + layer, outer[1] + layer]);
    }
    for (a, b, reverse) in rim {
        // Retain the existing rim diagonal and geometric triangles; only
        // change their orientation when the sheet traversal requires it.
        if reverse {
            indices.extend([a, b + layer, b, a, a + layer, b + layer]);
        } else {
            indices.extend([a, b, b + layer, a, b + layer, a + layer]);
        }
    }
    Ok(())
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

/// Exactly both solidified main layers used by quality acceptance. This
/// deliberately has no render, skirt or body-binding attributes: generating
/// those for every topology candidate dominated the quality search cost.
struct PhysicalShape {
    positions: Vec<[f32; 3]>,
    inner_positions: Vec<[f32; 3]>,
}

type SurfaceFieldCache = BTreeMap<[u8; 32], FittedBodySurface>;

fn surface_field_key(
    design: &BreastplateDesign,
    surface: &TorsoSurface,
    positions: &[[f32; 3]],
    semantic: &[[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    clearance: &crate::TorsoClearancePose,
    coronal_depths: &[f32],
) -> Result<[u8; 32], GenerateError> {
    let mut hash = blake3::Hasher::new();
    hash.update(&breastplate_design_hash(design)?);
    let policy = crate::breastplate_joint_profiles::baseline_policy_values();
    hash_joint_construction_mode(
        &mut hash,
        std::env::var_os("BREASTPLATE_DIAGNOSTIC_JOINT_PROFILES").is_some(),
        [policy[0].as_str(), policy[1].as_str(), policy[2].as_str()],
    );
    let band_policy = BandFraction::from_environment().map_err(shoulder_band_error)?;
    let crest_policy =
        CrestQueryPolicy::from_environment(band_policy).map_err(shoulder_band_error)?;
    crest_policy.hash(&mut hash);
    let seat_policy =
        ShoulderSeat::from_environment(band_policy, crest_policy).map_err(shoulder_band_error)?;
    let rim_policy = RimPolicy::from_environment(seat_policy).map_err(shoulder_band_error)?;
    rim_policy.hash(&mut hash);
    WholeFrontPolicy::from_environment(rim_policy)
        .map_err(whole_front_error)?
        .hash(&mut hash);
    if let Some(seat) = seat_policy {
        seat.hash(&mut hash);
        hash_enclosure_geometry(
            &mut hash,
            &clearance.enclosure_vertices,
            &surface.clearance_mesh.enclosure_faces,
        );
    }
    NecklineTreatment::from_environment(band_policy)
        .map_err(shoulder_band_error)?
        .hash(&mut hash);
    if let Some(policy) = band_policy {
        policy.hash(&mut hash);
    }
    let mut floats = |values: Vec<f32>| {
        hash.update(&(values.len() as u64).to_le_bytes());
        for value in values {
            hash.update(&value.to_bits().to_le_bytes());
        }
    };
    floats(positions.iter().flatten().copied().collect());
    floats(semantic.iter().flatten().copied().collect());
    floats(front.to_vec());
    floats(
        anchors
            .neck_base
            .into_iter()
            .chain(anchors.clavicles.into_iter().flatten())
            .chain(anchors.shoulders.into_iter().flatten())
            .collect(),
    );
    floats(
        clearance
            .vertices
            .iter()
            .flat_map(|sample| sample.position.into_iter().chain(sample.normal))
            .collect(),
    );
    floats(coronal_depths.to_vec());
    floats(
        surface
            .coronal_anchors
            .iter()
            .map(|anchor| anchor.vertical)
            .collect(),
    );
    for faces in [&surface.faces, &surface.clearance_mesh.faces] {
        hash.update(&(faces.len() as u64).to_le_bytes());
        for index in faces.iter().flatten() {
            hash.update(&index.to_le_bytes());
        }
    }
    Ok(*hash.finalize().as_bytes())
}

fn hash_joint_construction_mode(hash: &mut blake3::Hasher, joint: bool, policy: [&str; 3]) {
    if joint {
        hash.update(b"joint-torso-elliptical-v1");
        for value in policy {
            hash.update(&(value.len() as u64).to_le_bytes());
            hash.update(value.as_bytes());
        }
    }
}

fn main_surface_with_cache(
    topology: &CanonicalBreastplateTopology,
    design: &BreastplateDesign,
    surface: &TorsoSurface,
    source_positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    clearance_pose: &crate::TorsoClearancePose,
    coronal_depths: &[f32],
    field_cache: &mut SurfaceFieldCache,
    purpose: SurfaceEvaluation,
) -> Result<(Vec<[f32; 3]>, Vec<[f32; 3]>, Frame), GenerateError> {
    let frame = frame(source_positions, semantic_coordinates, front)?;
    let coronal_levels = surface
        .coronal_anchors
        .iter()
        .map(|anchor| anchor.vertical)
        .collect::<Vec<_>>();
    let key = surface_field_key(
        design,
        surface,
        source_positions,
        semantic_coordinates,
        front,
        anchors,
        clearance_pose,
        coronal_depths,
    )?;
    // Lookup precedes body rays, authored profiles and QP assembly. Neither
    // cache identity nor fitting quadrature contains CDT vertices/indices.
    if let std::collections::btree_map::Entry::Vacant(entry) = field_cache.entry(key) {
        entry.insert(fit_main_surface(
            design,
            source_positions,
            semantic_coordinates,
            &surface.faces,
            anchors,
            &clearance_pose.vertices,
            &surface.clearance_mesh.faces,
            &clearance_pose.enclosure_vertices,
            &surface.clearance_mesh.enclosure_faces,
            &coronal_levels,
            coronal_depths,
            frame,
        )?);
    }
    let (positions, normals) = field_cache[&key].evaluate(topology, purpose)?;
    Ok((positions, normals, frame))
}

fn main_radial_normals(
    topology: &CanonicalBreastplateTopology,
    main: &[[f32; 3]],
    frame: Frame,
) -> Result<Vec<[f32; 3]>, GenerateError> {
    // This shared analytic thickness field must remain identical in the
    // quality-only and final export paths. Triangle/render normals are not a
    // substitute: they would make solidified positions topology-dependent.
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
        .map(|index| coordinate(main[index], frame)[2])
        .collect::<Vec<_>>();
    let coronal_origin = side_depths.iter().sum::<f32>() / side_depths.len().max(1) as f32;
    main.iter()
        .map(|position| {
            let local = coordinate(*position, frame);
            normalized(add(
                scale(frame.lateral, local[0]),
                scale(frame.front, local[2] - coronal_origin),
            ))
        })
        .collect()
}

fn physical_shape_with_cache(
    topology: &CanonicalBreastplateTopology,
    design: &BreastplateDesign,
    surface: &TorsoSurface,
    source_positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    clearance_pose: &crate::TorsoClearancePose,
    coronal_depths: &[f32],
    field_cache: &mut SurfaceFieldCache,
) -> Result<PhysicalShape, GenerateError> {
    let (main, normals, _frame) = main_surface_with_cache(
        topology,
        design,
        surface,
        source_positions,
        semantic_coordinates,
        front,
        anchors,
        clearance_pose,
        coronal_depths,
        field_cache,
        SurfaceEvaluation::Quality,
    )?;
    let half_wall = design.wall_thickness.metres() * 0.5;
    Ok(PhysicalShape {
        positions: offset_sheet(&main, &normals, half_wall),
        inner_positions: offset_sheet(&main, &normals, -half_wall),
    })
}

fn offset_sheet(mid: &[[f32; 3]], normals: &[[f32; 3]], offset: f32) -> Vec<[f32; 3]> {
    assert_eq!(mid.len(), normals.len());
    mid.iter()
        .zip(normals)
        .map(|(position, normal)| add(*position, scale(*normal, offset)))
        .collect()
}

fn generate_shape_with_cache(
    topology: &CanonicalBreastplateTopology,
    design: &BreastplateDesign,
    surface: &TorsoSurface,
    source_positions: &[[f32; 3]],
    semantic_coordinates: &[[f32; 2]],
    front: [f32; 3],
    anchors: TorsoUpperRigAnchors,
    clearance_pose: &crate::TorsoClearancePose,
    coronal_depths: &[f32],
    field_cache: &mut SurfaceFieldCache,
) -> Result<GeneratedShape, GenerateError> {
    let (main, mut mid_normals, frame) = main_surface_with_cache(
        topology,
        design,
        surface,
        source_positions,
        semantic_coordinates,
        front,
        anchors,
        clearance_pose,
        coronal_depths,
        field_cache,
        SurfaceEvaluation::Export,
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
    mid_normals.extend(open_normals(&mid[main_count..], &skirt_faces, frame.front)?);

    // Semantic boundary construction already places the mid-surface outside
    // the body by clearance plus half thickness. Solidification is therefore
    // symmetric about that authored sheet; applying clearance a second time
    // would shrink narrow openings and degrade their triangle quality.
    let outer_offset = design.wall_thickness.metres() * 0.5;
    let inner_offset = -design.wall_thickness.metres() * 0.5;
    let mut positions = offset_sheet(&mid, &mid_normals, outer_offset);
    positions.extend(offset_sheet(&mid, &mid_normals, inner_offset));
    // Thickness uses the analytic surface field above, but render normals must
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
    )?;
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
    )?;
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
        && !surface.clearance_mesh.base.vertices.is_empty()
        && surface
            .clearance_mesh
            .has_corresponding_domains(surface.morphs.len())
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
}

fn physical_refinement_candidates(
    topology: &CanonicalBreastplateTopology,
    shape: &PhysicalShape,
) -> (f32, f32, Vec<[f32; 2]>) {
    let mut minimum_angle = 180.0_f32;
    let mut maximum_aspect = 0.0_f32;
    let mut poor = Vec::<(f32, [f32; 2])>::new();
    for triangle in topology.indices().as_chunks::<3>().0 {
        let (angle, aspect) = [&shape.positions, &shape.inner_positions]
            .into_iter()
            .map(|layer| physical_triangle_quality(triangle.map(|index| layer[index as usize])))
            .fold((180.0_f32, 0.0_f32), |score, current| {
                (score.0.min(current.0), score.1.max(current.1))
            });
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
                if topology.is_metric_panel() {
                    let candidate = crate::breastplate_refinement::boundary_triangle_site([
                        first, second, interior,
                    ]);
                    poor.push((angle, candidate));
                    continue;
                }
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

fn physical_triangle_quality(points: [[f32; 3]; 3]) -> (f32, f32) {
    let p = points.map(|p| p.map(f64::from));
    let delta = |a: [f64; 3], b: [f64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let norm = |p: [f64; 3]| p.into_iter().map(|v| v * v).sum::<f64>().sqrt();
    let ab = delta(p[1], p[0]);
    let ac = delta(p[2], p[0]);
    let area = norm([
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]);
    let edges = [norm(delta(p[1], p[2])), norm(ac), norm(ab)];
    if !area.is_finite() || area <= 0.0 || edges.iter().any(|e| !e.is_finite() || *e <= 0.0) {
        return (0.0, f32::INFINITY);
    }
    let aspect = edges.into_iter().fold(0.0_f64, f64::max).powi(2) / area;
    let angle = (0..3)
        .map(|i| {
            let a = edges[(i + 1) % 3];
            let b = edges[(i + 2) % 3];
            ((a * a + b * b - edges[i] * edges[i]) / (2.0 * a * b))
                .clamp(-1.0, 1.0)
                .acos()
                .to_degrees()
        })
        .fold(180.0_f64, f64::min);
    (angle as f32, aspect as f32)
}

fn physical_vertex_quality(
    topology: &CanonicalBreastplateTopology,
    shape: &PhysicalShape,
    vertex: u32,
) -> (f32, f32) {
    let mut score = (180.0_f32, 0.0_f32);
    for face in topology
        .indices()
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|f| f.contains(&vertex))
    {
        for layer in [&shape.positions, &shape.inner_positions] {
            let (angle, aspect) = physical_triangle_quality(face.map(|i| layer[i as usize]));
            score.0 = score.0.min(angle);
            score.1 = score.1.max(aspect);
        }
    }
    score
}

fn physical_vertex_quality_family(
    topology: &CanonicalBreastplateTopology,
    base: &PhysicalShape,
    morphs: &[PhysicalShape],
    vertex: u32,
) -> (f32, f32) {
    let mut score = physical_vertex_quality(topology, base, vertex);
    for shape in morphs {
        let quality = physical_vertex_quality(topology, shape, vertex);
        score = (score.0.min(quality.0), score.1.max(quality.1));
    }
    score
}

fn physical_global_quality(
    topology: &CanonicalBreastplateTopology,
    surfaces: &[Vec<[f32; 3]>],
) -> (f32, f32, [u32; 3]) {
    physical_quality_summary(topology.indices().as_chunks::<3>().0, surfaces, false)
}

fn physical_search_quality(
    topology: &CanonicalBreastplateTopology,
    surfaces: &[Vec<[f32; 3]>],
) -> (f32, f32, [u32; 3]) {
    physical_quality_summary(
        topology.indices().as_chunks::<3>().0,
        surfaces,
        topology.is_metric_panel(),
    )
}

fn physical_quality_summary(
    faces: &[[u32; 3]],
    surfaces: &[Vec<[f32; 3]>],
    select_active_defect: bool,
) -> (f32, f32, [u32; 3]) {
    let mut score = (180.0_f32, 0.0_f32, [0_u32; 3]);
    let mut aspect_face = [0; 3];
    for face in faces {
        for layer in surfaces {
            let (angle, aspect) = physical_triangle_quality(face.map(|i| layer[i as usize]));
            if angle < score.0 {
                score.0 = angle;
                score.2 = *face;
            }
            if aspect > score.1 {
                aspect_face = *face;
            }
            score.1 = score.1.max(aspect);
        }
    }
    if select_active_defect && score.0 >= 10.0 && score.1 > 9.0 {
        score.2 = aspect_face;
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
        .filter(|f| f.contains(&vertex))
    {
        for layer in surfaces {
            let (angle, aspect) = physical_triangle_quality(face.map(|i| layer[i as usize]));
            score.0 = score.0.min(angle);
            score.1 = score.1.max(aspect);
        }
    }
    score
}

fn quality_shortfalls_do_not_worsen(before: (f32, f32), after: (f32, f32)) -> bool {
    [before.0, before.1, after.0, after.1]
        .iter()
        .all(|v| v.is_finite())
        && (10.0 - after.0).max(0.0) <= (10.0 - before.0).max(0.0) + 1e-5
        && (after.1 - 9.0).max(0.0) <= (before.1 - 9.0).max(0.0) + 1e-5
}

fn plateau_local_gain_improves(before: f32, after: f32, best_gain: f32) -> bool {
    after - before > best_gain.max(0.0) + 1e-5
}

fn quality_search_candidate_improves(
    aspect_focus: bool,
    best: [f32; 4],
    candidate: [f32; 4],
) -> bool {
    let [v, a, r, gain] = candidate;
    if v < best[0] - 1e-5 {
        return true;
    }
    if (v - best[0]).abs() > 1e-5 {
        return false;
    }
    if aspect_focus {
        // Once all angles pass, increasing a passing angle is not progress
        // against an unresolved aspect defect. A tied other aspect owner may
        // remain, so accept a strict baseline-relative local aspect reduction.
        r < best[2] - 1e-5
            || ((r - best[2]).abs() <= 1e-5 && plateau_local_gain_improves(0.0, gain, best[3]))
    } else {
        a > best[1] + 1e-5
            || ((a - best[1]).abs() <= 1e-5 && r < best[2] - 1e-5)
            || ((a - best[1]).abs() <= 1e-5
                && (r - best[2]).abs() <= 1e-5
                && plateau_local_gain_improves(0.0, gain, best[3]))
    }
}

fn metric_endpoint_guard(
    topology: &CanonicalBreastplateTopology,
    before: &[(f32, f32)],
    surfaces: &[Vec<[f32; 3]>],
) -> bool {
    before.len() == surfaces.len()
        && before.iter().zip(surfaces).all(|(old, surface)| {
            let (angle, aspect, _) =
                physical_global_quality(topology, std::slice::from_ref(surface));
            quality_shortfalls_do_not_worsen(*old, (angle, aspect))
        })
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
    let field_cache = std::cell::RefCell::new(SurfaceFieldCache::new());
    let generate_shape = |topology: &CanonicalBreastplateTopology,
                          design: &BreastplateDesign,
                          surface: &TorsoSurface,
                          positions: &[[f32; 3]],
                          semantic: &[[f32; 2]],
                          front: [f32; 3],
                          anchors: TorsoUpperRigAnchors,
                          clearance: &crate::TorsoClearancePose,
                          coronal: &[f32]| {
        physical_shape_with_cache(
            topology,
            design,
            surface,
            positions,
            semantic,
            front,
            anchors,
            clearance,
            coronal,
            &mut field_cache.borrow_mut(),
        )
    };
    // Explicitly diagnostic only: inspect a new fair field on the initial
    // canonical mesh before the expensive shared-quality search. No quality
    // gate is waived for production generation, which never sets this flag.
    let diagnostic_surface_preview =
        std::env::var_os("BREASTPLATE_DIAGNOSTIC_SURFACE_PREVIEW").is_some();
    if diagnostic_surface_preview {
        eprintln!(
            "breastplate DIAGNOSTIC surface preview: canonical topology quality is not certified"
        );
    }
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
            surface.clearance_mesh.morphs.len()
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
        &surface.clearance_mesh.base.vertices,
        base_frame,
    );
    let base_measurements = semantic_measurements(
        design,
        &base_positions,
        &base_semantic_coordinates,
        surface.upper_rig_anchors,
        base_frame,
    );
    let mut base_chart_boundary = base_boundary
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
    let mut base_domain =
        semantic_parameter_boundary(&base_boundary, base_measurements, base_frame);
    let mut extra_candidates = Vec::<[f32; 2]>::new();
    let base_coronal_depths = surface
        .coronal_anchors
        .iter()
        .map(|anchor| anchor.depth)
        .collect::<Vec<_>>();
    // Surface selection and quality bypass are independent. The explicit
    // angular flag can run the full production acceptance/optimization path;
    // only SURFACE_PREVIEW bypasses that path for diagnostic artifacts.
    let angular_chart = std::env::var_os("BREASTPLATE_DIAGNOSTIC_ANGULAR_LOFT").is_some();
    let metric_panel = std::env::var_os("BREASTPLATE_DIAGNOSTIC_METRIC_PANEL").is_some();
    if metric_panel && !angular_chart {
        eprintln!("Metric panel diagnostic requires angular field");
        return Err(GenerateError::InvalidSurface);
    }
    let mut metric_reference = None;
    if angular_chart {
        // Construct the design topology IN its actual intrinsic domain. A
        // legacy frontal chart's deep corner caps are not a valid embedding
        // of an angular waist merely because its outer strip can be mapped.
        let fitted = fit_angular_surface(
            design,
            &base_positions,
            &base_semantic_coordinates,
            &surface.faces,
            surface.upper_rig_anchors,
            &surface.clearance_mesh.base.vertices,
            &surface.clearance_mesh.faces,
            &surface.clearance_mesh.base.enclosure_vertices,
            &surface.clearance_mesh.enclosure_faces,
            &surface
                .coronal_anchors
                .iter()
                .map(|a| a.vertical)
                .collect::<Vec<_>>(),
            &base_coronal_depths,
            base_frame,
        )?;
        base_domain = fitted
            .angular
            .as_ref()
            .expect("angular fit")
            .boundary_domain
            .clone();
        base_chart_boundary = base_domain.clone();
        if metric_panel {
            metric_reference = fitted.angular.clone();
        }
        let key = surface_field_key(
            design,
            surface,
            &base_positions,
            &base_semantic_coordinates,
            surface.front,
            surface.upper_rig_anchors,
            &surface.clearance_mesh.base,
            &base_coronal_depths,
        )?;
        field_cache.borrow_mut().insert(key, fitted);
    }
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
    let mut best = None::<(f32, CanonicalBreastplateTopology, PhysicalShape)>;
    let mut refinement_iterations = 0_usize;
    let (mut topology, mut base) = loop {
        refinement_iterations += 1;
        topology_stage(&format!(
            "Steiner round {refinement_iterations}: construct with {} extra sites",
            extra_candidates.len()
        ))?;
        let topology = if let Some(angular) = &metric_reference {
            CanonicalBreastplateTopology::from_surface_metric_panel(
                &base_domain,
                &extra_candidates,
                |point| {
                    angular
                        .local_position(point)
                        .map_err(|e| format!("Metric panel field: {e:?}"))
                },
            )
            .map_err(|e| {
                eprintln!("{e}");
                GenerateError::InvalidSurface
            })?
        } else {
            CanonicalBreastplateTopology::from_metric_chart_with_candidates(
                &base_chart_boundary,
                &base_domain,
                &extra_candidates,
                if design.neck_width.unit() > 0.50 {
                    0.25
                } else {
                    0.50
                },
                !angular_chart,
                |chart| {
                    if angular_chart {
                        return chart;
                    }
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
            )
        };
        let base = generate_shape(
            &topology,
            design,
            surface,
            &base_positions,
            &base_semantic_coordinates,
            surface.front,
            surface.upper_rig_anchors,
            &surface.clearance_mesh.base,
            &base_coronal_depths,
        )?;
        if refinement_iterations == 1 {
            if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
                let path = std::path::PathBuf::from(path).with_extension("initial-topology.json");
                let data = format!(
                    "{{\"reference_domain\":{:?},\"indices\":{:?},\"boundary_count\":{},\"structured_samples\":{:?},\"metric_layout\":{}}}",
                    topology.canonical_positions(),
                    topology.indices(),
                    topology.boundary_vertices().len(),
                    (0..topology.canonical_positions().len())
                        .filter(|i| topology.is_structured_spoke_sample(*i))
                        .collect::<Vec<_>>(),
                    topology.metric_layout_json()
                );
                std::fs::write(path, data).map_err(|_| GenerateError::InvalidSurface)?;
            }
        }
        // Refine the design connectivity against every physical realization,
        // not only the base body. A coronal side rail can move materially on
        // a torso-wide morph while retaining the same semantic IDs; selecting
        // Steiner sites from the base alone left a boundary-adjacent triangle
        // acceptable on base but stretched on that supported morph.
        let mut minimum_angle = 180.0_f32;
        let mut maximum_aspect = 0.0_f32;
        let mut candidates = Vec::<[f32; 2]>::new();
        let mut assess = |shape: &PhysicalShape| {
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
            .zip(&surface.clearance_mesh.morphs)
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
        topology_stage(&format!(
            "Steiner round {refinement_iterations}: assessed angle {minimum_angle:.5}, aspect {maximum_aspect:.5}"
        ))?;
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
            || diagnostic_surface_preview
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

    if !diagnostic_surface_preview {
        if !topology.is_metric_panel() {
            topology_stage("shared rail search")?;
            // The neckline/shoulder corner is a reflex garment-pattern junction.  Its
            // outer curve samples are semantic landmarks and remain fixed, while the
            // middle sample of the derived inner cap is free to relax.  Optimize that
            // one non-landmark station against the actual curved base and envelope
            // morph surfaces; a chart-only optimum can still lose a degree after the
            // fair crown is evaluated in 3D.
            // A shallow ellipse can compress the physical side metric as strongly
            // as a curved neckline. Select stations by actual cross-body/layer
            // quality, not an inherited list of historical troublesome corners.
            // These positions belong to this exact current topology. Untouched
            // stations reuse them; an accepted trial transfers its complete family
            // alongside the topology, never a hash-based or cross-topology cache.
            let mut current_morph_shapes = Vec::with_capacity(surface.morphs.len());
            for index in 0..surface.morphs.len() {
                current_morph_shapes.push(generate_shape(
                    &topology,
                    design,
                    surface,
                    &surface.morphs[index].positions,
                    &surface.morph_semantic_coordinates[index],
                    surface.morph_fronts[index],
                    surface.morph_upper_rig_anchors[index],
                    &surface.clearance_mesh.morphs[index],
                    &surface.morph_coronal_depths[index],
                )?);
            }
            for station in 0..topology.boundary_vertices().len() {
                topology_stage(&format!("shared rail station {station}"))?;
                let vertex = topology.inner_rail_vertex(station).expect("semantic rail");
                let initial =
                    physical_vertex_quality_family(&topology, &base, &current_morph_shapes, vertex);
                if initial.0 >= 12.5 && initial.1 <= 7.5 {
                    continue;
                }
                let mut best_cap = (
                    initial.0,
                    initial.1,
                    topology.clone(),
                    base,
                    current_morph_shapes,
                );
                let mut endpoint_order = worst_first(
                    best_cap
                        .4
                        .iter()
                        .map(|shape| physical_vertex_quality(&topology, shape, vertex)),
                );
                let tangent_limit = 5;
                let normal_limit = 9;
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
                        let Ok(candidate_base) = generate_shape(
                            &candidate_topology,
                            design,
                            surface,
                            &base_positions,
                            &base_semantic_coordinates,
                            surface.front,
                            surface.upper_rig_anchors,
                            &surface.clearance_mesh.base,
                            &base_coronal_depths,
                        ) else {
                            continue;
                        };
                        let station_vertex = candidate_topology
                            .inner_rail_vertex(station)
                            .expect("inner semantic rail station");
                        let candidate_quality = physical_vertex_quality(
                            &candidate_topology,
                            &candidate_base,
                            station_vertex,
                        );
                        // Test the current limiting wearers first, but retain every
                        // endpoint in original morph order for any accepted trial.
                        // A partial min-angle/max-aspect bound only rejects trials
                        // that cannot beat the unchanged incumbent comparator.
                        if let Some(((minimum_angle, maximum_aspect), candidate_morph_shapes)) =
                            improving_family(
                                candidate_quality,
                                (best_cap.0, best_cap.1),
                                &endpoint_order,
                                |index| {
                                    let target = generate_shape(
                                        &candidate_topology,
                                        design,
                                        surface,
                                        &surface.morphs[index].positions,
                                        &surface.morph_semantic_coordinates[index],
                                        surface.morph_fronts[index],
                                        surface.morph_upper_rig_anchors[index],
                                        &surface.clearance_mesh.morphs[index],
                                        &surface.morph_coronal_depths[index],
                                    )
                                    .ok()?;
                                    let quality = physical_vertex_quality(
                                        &candidate_topology,
                                        &target,
                                        station_vertex,
                                    );
                                    Some((target, quality))
                                },
                            )
                        {
                            endpoint_order =
                                worst_first(candidate_morph_shapes.iter().map(|shape| {
                                    physical_vertex_quality(
                                        &candidate_topology,
                                        shape,
                                        station_vertex,
                                    )
                                }));
                            best_cap = (
                                minimum_angle,
                                maximum_aspect,
                                candidate_topology,
                                candidate_base,
                                candidate_morph_shapes,
                            );
                        }
                    }
                }
                topology = best_cap.2;
                base = best_cap.3;
                current_morph_shapes = best_cap.4;
            }
            // The right neckline/bridge reflex cap is a derived garment-pattern
            // vertex, not a landmark. Optimize its physical position across the same
            // wearer envelope after rail relaxation; widening the authored neckline
            // otherwise leaves its one fan triangle just over the aspect contract.
            topology_stage("shared cap search")?;
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
                    'quality_trial: for normal_step in -4_i32..=4 {
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
                        let Ok(candidate_base) = generate_shape(
                            &candidate_topology,
                            design,
                            surface,
                            &base_positions,
                            &base_semantic_coordinates,
                            surface.front,
                            surface.upper_rig_anchors,
                            &surface.clearance_mesh.base,
                            &base_coronal_depths,
                        ) else {
                            continue;
                        };
                        let cap_vertex = candidate_topology
                            .cap_vertex(cap_offset)
                            .expect("semantic cap vertex");
                        let (mut minimum_angle, mut maximum_aspect) = physical_vertex_quality(
                            &candidate_topology,
                            &candidate_base,
                            cap_vertex,
                        );
                        for (
                            morph_index,
                            (
                                (
                                    (((morph, anchors), semantic_coordinates), front),
                                    clearance_vertices,
                                ),
                                coronal_depths,
                            ),
                        ) in surface
                            .morphs
                            .iter()
                            .zip(&surface.morph_upper_rig_anchors)
                            .zip(&surface.morph_semantic_coordinates)
                            .zip(&surface.morph_fronts)
                            .zip(&surface.clearance_mesh.morphs)
                            .zip(&surface.morph_coronal_depths)
                            .enumerate()
                        {
                            if !representative_morphs.contains(&morph_index) {
                                continue;
                            }
                            let Ok(target) = generate_shape(
                                &candidate_topology,
                                design,
                                surface,
                                &morph.positions,
                                semantic_coordinates,
                                *front,
                                *anchors,
                                clearance_vertices,
                                coronal_depths,
                            ) else {
                                continue 'quality_trial;
                            };
                            let (angle, aspect) =
                                physical_vertex_quality(&candidate_topology, &target, cap_vertex);
                            minimum_angle = minimum_angle.min(angle);
                            maximum_aspect = maximum_aspect.max(aspect);
                        }
                        let violation = (10.0 - minimum_angle).max(0.0) * 100.0
                            + (maximum_aspect - 9.0).max(0.0) * 10.0;
                        let best_violation = (10.0 - best_cap.0).max(0.0) * 100.0
                            + (best_cap.1 - 9.0).max(0.0) * 10.0;
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
            topology_stage("fine rail search")?;
            for station in [0_usize, 1_usize, 37, 38, 179, 180] {
                let mut best_local = (
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    topology.clone(),
                    base,
                );
                for tangent_step in -8_i32..=8 {
                    'quality_trial: for normal_step in -8_i32..=8 {
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
                        let Ok(candidate_base) = generate_shape(
                            &candidate_topology,
                            design,
                            surface,
                            &base_positions,
                            &base_semantic_coordinates,
                            surface.front,
                            surface.upper_rig_anchors,
                            &surface.clearance_mesh.base,
                            &base_coronal_depths,
                        ) else {
                            continue;
                        };
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
                        let (mut minimum_angle, mut maximum_aspect) = physical_vertex_quality(
                            &candidate_topology,
                            &candidate_base,
                            rail_vertex,
                        );
                        let (angle, aspect) = physical_vertex_quality(
                            &candidate_topology,
                            &candidate_base,
                            cap_vertex,
                        );
                        minimum_angle = minimum_angle.min(angle);
                        maximum_aspect = maximum_aspect.max(aspect);
                        for (
                            morph_index,
                            (
                                (
                                    (((morph, anchors), semantic_coordinates), front),
                                    clearance_vertices,
                                ),
                                coronal_depths,
                            ),
                        ) in surface
                            .morphs
                            .iter()
                            .zip(&surface.morph_upper_rig_anchors)
                            .zip(&surface.morph_semantic_coordinates)
                            .zip(&surface.morph_fronts)
                            .zip(&surface.clearance_mesh.morphs)
                            .zip(&surface.morph_coronal_depths)
                            .enumerate()
                        {
                            if !representative_morphs.contains(&morph_index) {
                                continue;
                            }
                            let Ok(target) = generate_shape(
                                &candidate_topology,
                                design,
                                surface,
                                &morph.positions,
                                semantic_coordinates,
                                *front,
                                *anchors,
                                clearance_vertices,
                                coronal_depths,
                            ) else {
                                continue 'quality_trial;
                            };
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
                    'quality_trial: for y_step in 0_i32..=0 {
                        let mut candidate_topology = topology.clone();
                        if !candidate_topology.relax_free_vertex(
                            free_vertex,
                            x_step as f32 * 0.010,
                            y_step as f32 * 0.010,
                        ) {
                            continue;
                        }
                        let Ok(candidate_base) = generate_shape(
                            &candidate_topology,
                            design,
                            surface,
                            &base_positions,
                            &base_semantic_coordinates,
                            surface.front,
                            surface.upper_rig_anchors,
                            &surface.clearance_mesh.base,
                            &base_coronal_depths,
                        ) else {
                            continue;
                        };
                        let vertex = free_vertex as u32;
                        let (mut minimum_angle, mut maximum_aspect) =
                            physical_vertex_quality(&candidate_topology, &candidate_base, vertex);
                        for (
                            morph_index,
                            (
                                (
                                    (((morph, anchors), semantic_coordinates), front),
                                    clearance_vertices,
                                ),
                                coronal_depths,
                            ),
                        ) in surface
                            .morphs
                            .iter()
                            .zip(&surface.morph_upper_rig_anchors)
                            .zip(&surface.morph_semantic_coordinates)
                            .zip(&surface.morph_fronts)
                            .zip(&surface.clearance_mesh.morphs)
                            .zip(&surface.morph_coronal_depths)
                            .enumerate()
                        {
                            if !representative_morphs.contains(&morph_index) {
                                continue;
                            }
                            let Ok(target) = generate_shape(
                                &candidate_topology,
                                design,
                                surface,
                                &morph.positions,
                                semantic_coordinates,
                                *front,
                                *anchors,
                                clearance_vertices,
                                coronal_depths,
                            ) else {
                                continue 'quality_trial;
                            };
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
                    'quality_trial: for normal_step in 0_i32..=0 {
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
                        let Ok(candidate_base) = generate_shape(
                            &candidate_topology,
                            design,
                            surface,
                            &base_positions,
                            &base_semantic_coordinates,
                            surface.front,
                            surface.upper_rig_anchors,
                            &surface.clearance_mesh.base,
                            &base_coronal_depths,
                        ) else {
                            continue;
                        };
                        let cap_vertex = candidate_topology
                            .cap_vertex(cap_offset)
                            .expect("semantic cap vertex");
                        let (mut minimum_angle, mut maximum_aspect) = physical_vertex_quality(
                            &candidate_topology,
                            &candidate_base,
                            cap_vertex,
                        );
                        for (
                            morph_index,
                            (
                                (
                                    (((morph, anchors), semantic_coordinates), front),
                                    clearance_vertices,
                                ),
                                coronal_depths,
                            ),
                        ) in surface
                            .morphs
                            .iter()
                            .zip(&surface.morph_upper_rig_anchors)
                            .zip(&surface.morph_semantic_coordinates)
                            .zip(&surface.morph_fronts)
                            .zip(&surface.clearance_mesh.morphs)
                            .zip(&surface.morph_coronal_depths)
                            .enumerate()
                        {
                            if !representative_morphs.contains(&morph_index) {
                                continue;
                            }
                            let Ok(target) = generate_shape(
                                &candidate_topology,
                                design,
                                surface,
                                &morph.positions,
                                semantic_coordinates,
                                *front,
                                *anchors,
                                clearance_vertices,
                                coronal_depths,
                            ) else {
                                continue 'quality_trial;
                            };
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
                    &surface.clearance_mesh.base,
                    &base_coronal_depths,
                )?;
                surfaces.push(acceptance_base.positions[..main_vertex_count].to_vec());
                surfaces.push(acceptance_base.inner_positions[..main_vertex_count].to_vec());
                for (
                    ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                    coronal_depths,
                ) in surface
                    .morphs
                    .iter()
                    .zip(&surface.morph_upper_rig_anchors)
                    .zip(&surface.morph_semantic_coordinates)
                    .zip(&surface.morph_fronts)
                    .zip(&surface.clearance_mesh.morphs)
                    .zip(&surface.morph_coronal_depths)
                {
                    // Final topology acceptance includes every supplied body
                    // endpoint, not just the inexpensive seeding subset.
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
                    surfaces.push(target.inner_positions[..main_vertex_count].to_vec());
                }
            }
            Ok::<_, GenerateError>(surfaces)
        };
        topology_stage("full endpoint edge/vertex optimization")?;
        let shared_quality_surfaces = acceptance_surfaces(&topology)?;
        let flips = topology.optimize_physical_edges(&shared_quality_surfaces);
        topology_stage(&format!("full endpoint initial edge flips {flips}"))?;
        // A semantic-boundary relaxation changes the harmonic embedding of every
        // free CDT sample. On extreme parameter combinations an otherwise remote
        // interior one-ring can therefore become the last sub-10-degree region
        // even after all useful edge flips. Relax one vertex of that objectively
        // worst face in the physical chart, scoring this requested design on
        // the base and every supplied body endpoint. Other armor recipes own
        // independent physical charts and are tested in separate generations.
        for iteration in 0..128 {
            let quality_surfaces = acceptance_surfaces(&topology)?;
            // Relocating a derived sample changes which diagonal is best in
            // the physical inner/outer metric. Reusing only the initial CDT
            // flips can strand a valid vertex arrangement in a skinny fan.
            // Flips keep the semantic constraints and shared endpoint IDs.
            let flips = topology.optimize_physical_edges(&quality_surfaces);
            let (minimum_angle, maximum_aspect, worst_face) =
                physical_search_quality(&topology, &quality_surfaces);
            let aspect_focus =
                topology.is_metric_panel() && minimum_angle >= 10.0 && maximum_aspect > 9.0;
            topology_stage(&format!(
                "full endpoint iteration {iteration}: angle {minimum_angle:.6}, aspect {maximum_aspect:.6}, face {worst_face:?}, flips {flips}"
            ))?;
            if minimum_angle >= 10.0 && maximum_aspect <= 9.0 {
                topology_stage("full endpoint quality satisfied")?;
                break;
            }
            let endpoint_before = if topology.is_metric_panel() {
                quality_surfaces
                    .iter()
                    .map(|surface| {
                        let (a, b, _) =
                            physical_global_quality(&topology, std::slice::from_ref(surface));
                        (a, b)
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let search_started = std::time::Instant::now();
            let mut evaluated_candidates = 0;
            let violation = |angle: f32, aspect: f32| {
                (10.0 - angle).max(0.0) * 100.0 + (aspect - 9.0).max(0.0) * 10.0
            };
            let mut best = (
                violation(minimum_angle, maximum_aspect),
                minimum_angle,
                maximum_aspect,
                0.0,
                topology.clone(),
            );
            let boundary_count = topology.boundary_vertices().len();
            for &step_scale in topology.quality_search_scales() {
                let scale_started = std::time::Instant::now();
                for vertex in worst_face {
                    if topology.is_structured_spoke_sample(vertex as usize) {
                        continue;
                    }
                    let local_before_quality =
                        physical_vertex_quality_surfaces(&topology, &quality_surfaces, vertex);
                    let local_before = if aspect_focus {
                        -local_before_quality.1
                    } else {
                        local_before_quality.0
                    };
                    for [x_step, y_step] in topology.quality_move_stencil(vertex as usize) {
                        let [x_step, y_step] = [x_step * step_scale, y_step * step_scale];
                        let mut candidate_topology = topology.clone();
                        let vertex_index = vertex as usize;
                        let moved = if candidate_topology.is_metric_site(vertex_index) {
                            metric_reference.as_ref().is_some_and(|angular| {
                                candidate_topology.relax_metric_site_recipe(
                                    vertex_index,
                                    x_step,
                                    y_step,
                                    |point| {
                                        angular
                                            .local_position(point)
                                            .map_err(|e| format!("Metric recipe field: {e:?}"))
                                    },
                                )
                            })
                        } else if !candidate_topology.is_metric_panel()
                            && (boundary_count..boundary_count * 2).contains(&vertex_index)
                        {
                            candidate_topology.relax_inner_rail_station_preserving_embedding(
                                vertex_index - boundary_count,
                                x_step,
                                y_step,
                            )
                        } else {
                            candidate_topology.relax_free_vertex_preserving_embedding(
                                vertex_index,
                                x_step,
                                y_step,
                            )
                        };
                        if !moved {
                            continue;
                        }
                        let Ok(candidate_surfaces) = acceptance_surfaces(&candidate_topology)
                        else {
                            continue;
                        };
                        evaluated_candidates += 1;
                        if topology.is_metric_panel()
                            && !metric_endpoint_guard(
                                &candidate_topology,
                                &endpoint_before,
                                &candidate_surfaces,
                            )
                        {
                            continue;
                        }
                        let (angle, aspect, _) =
                            physical_global_quality(&candidate_topology, &candidate_surfaces);
                        let (local_angle, local_aspect) = physical_vertex_quality_surfaces(
                            &candidate_topology,
                            &candidate_surfaces,
                            vertex,
                        );
                        let local_after = if aspect_focus {
                            -local_aspect
                        } else {
                            local_angle
                        };
                        let candidate_violation = violation(angle, aspect);
                        if quality_search_candidate_improves(
                            aspect_focus,
                            [best.0, best.1, best.2, best.3],
                            [
                                candidate_violation,
                                angle,
                                aspect,
                                local_after - local_before,
                            ],
                        ) {
                            best = (
                                candidate_violation,
                                angle,
                                aspect,
                                local_after - local_before,
                                candidate_topology,
                            );
                        }
                    }
                }
                if topology.is_metric_panel() {
                    topology_stage(&format!(
                        "metric recipe search iteration {iteration} scale {step_scale}: {:.3}s, accepted={}",
                        scale_started.elapsed().as_secs_f64(),
                        best.4.reference_hash() != topology.reference_hash()
                    ))?;
                }
                // Preserve the complete coarse search and its selected result.
                // Refine the trust region only after every candidate at this scale
                // fails the same legal-move, all-endpoint and improvement checks.
                if best.4.reference_hash() != topology.reference_hash() {
                    break;
                }
            }
            if topology.is_metric_panel() {
                topology_stage(&format!(
                    "metric recipe search iteration {iteration}: {evaluated_candidates} evaluated candidates, {:.3}s",
                    search_started.elapsed().as_secs_f64()
                ))?;
            }
            if best.4.reference_hash() == topology.reference_hash()
                || best.0 > violation(minimum_angle, maximum_aspect) + 1e-5
            {
                topology_stage(&format!(
                    "full endpoint iteration {iteration}: no improving legal move"
                ))?;
                break;
            }
            topology_stage(&format!(
                "full endpoint iteration {iteration}: accepted angle {:.6}, aspect {:.6}, violation {:.6}",
                best.1, best.2, best.0
            ))?;
            topology = best.4;
        }
        // The aggregate optimizer can reach a fixed point whose tied worst score
        // is owned by a design extreme while the requested neutral surface still
        // has a different sub-10-degree free face. Give that requested surface a
        // bounded final coordinate relaxation, but accept moves only when the
        // complete acceptance set is no worse. This is still face-agnostic and
        // uses the same frozen semantic topology for every endpoint.
        for iteration in 0..64 {
            let current_base = generate_shape(
                &topology,
                design,
                surface,
                &base_positions,
                &base_semantic_coordinates,
                surface.front,
                surface.upper_rig_anchors,
                &surface.clearance_mesh.base,
                &base_coronal_depths,
            )?;
            let base_surfaces = [
                current_base.positions[..main_vertex_count].to_vec(),
                current_base.inner_positions[..main_vertex_count].to_vec(),
            ];
            let (base_angle, base_aspect, worst_face) =
                physical_search_quality(&topology, &base_surfaces);
            let aspect_focus =
                topology.is_metric_panel() && base_angle >= 10.0 && base_aspect > 9.0;
            topology_stage(&format!(
                "base both-layer polish {iteration}: angle {base_angle:.6}, aspect {base_aspect:.6}, face {worst_face:?}"
            ))?;
            if base_angle >= 10.0 && base_aspect <= 9.0 {
                topology_stage("base both-layer quality satisfied")?;
                break;
            }
            let current_acceptance = acceptance_surfaces(&topology)?;
            let endpoint_before = if topology.is_metric_panel() {
                current_acceptance
                    .iter()
                    .map(|surface| {
                        let (a, b, _) =
                            physical_global_quality(&topology, std::slice::from_ref(surface));
                        (a, b)
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
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
                0.0,
                topology.clone(),
            );
            let boundary_count = topology.boundary_vertices().len();
            for &step_scale in topology.quality_search_scales() {
                let scale_started = std::time::Instant::now();
                for vertex in worst_face {
                    if topology.is_structured_spoke_sample(vertex as usize) {
                        continue;
                    }
                    let local_before_quality =
                        physical_vertex_quality_surfaces(&topology, &base_surfaces, vertex);
                    let local_before = if aspect_focus {
                        -local_before_quality.1
                    } else {
                        local_before_quality.0
                    };
                    for [x_step, y_step] in topology.quality_move_stencil(vertex as usize) {
                        let [x_step, y_step] = [x_step * step_scale, y_step * step_scale];
                        let mut candidate_topology = topology.clone();
                        let vertex_index = vertex as usize;
                        let moved = if candidate_topology.is_metric_site(vertex_index) {
                            metric_reference.as_ref().is_some_and(|angular| {
                                candidate_topology.relax_metric_site_recipe(
                                    vertex_index,
                                    x_step,
                                    y_step,
                                    |point| {
                                        angular
                                            .local_position(point)
                                            .map_err(|e| format!("Metric recipe field: {e:?}"))
                                    },
                                )
                            })
                        } else if !candidate_topology.is_metric_panel()
                            && (boundary_count..boundary_count * 2).contains(&vertex_index)
                        {
                            candidate_topology.relax_inner_rail_station_preserving_embedding(
                                vertex_index - boundary_count,
                                x_step,
                                y_step,
                            )
                        } else {
                            candidate_topology.relax_free_vertex_preserving_embedding(
                                vertex_index,
                                x_step,
                                y_step,
                            )
                        };
                        if !moved {
                            continue;
                        }
                        let Ok(candidate_acceptance) = acceptance_surfaces(&candidate_topology)
                        else {
                            continue;
                        };
                        if topology.is_metric_panel()
                            && !metric_endpoint_guard(
                                &candidate_topology,
                                &endpoint_before,
                                &candidate_acceptance,
                            )
                        {
                            continue;
                        }
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
                            &surface.clearance_mesh.base,
                            &base_coronal_depths,
                        )?;
                        let candidate_base_surfaces = [
                            candidate_base.positions[..main_vertex_count].to_vec(),
                            candidate_base.inner_positions[..main_vertex_count].to_vec(),
                        ];
                        let (angle, aspect, _) =
                            physical_global_quality(&candidate_topology, &candidate_base_surfaces);
                        let candidate_violation = violation(angle, aspect);
                        let local_quality = physical_vertex_quality_surfaces(
                            &candidate_topology,
                            &candidate_base_surfaces,
                            vertex,
                        );
                        let local_after = if aspect_focus {
                            -local_quality.1
                        } else {
                            local_quality.0
                        };
                        if quality_search_candidate_improves(
                            aspect_focus,
                            [best.0, best.1, best.2, best.3],
                            [
                                candidate_violation,
                                angle,
                                aspect,
                                local_after - local_before,
                            ],
                        ) {
                            best = (
                                candidate_violation,
                                angle,
                                aspect,
                                local_after - local_before,
                                candidate_topology,
                            );
                        }
                    }
                }
                if topology.is_metric_panel() {
                    topology_stage(&format!(
                        "base metric recipe polish {iteration} scale {step_scale}: {:.3}s, accepted={}",
                        scale_started.elapsed().as_secs_f64(),
                        best.4.reference_hash() != topology.reference_hash()
                    ))?;
                }
                if best.4.reference_hash() != topology.reference_hash() {
                    break;
                }
            }
            if best.4.reference_hash() == topology.reference_hash()
                || best.0 > violation(base_angle, base_aspect) + 1e-5
            {
                topology_stage(&format!(
                    "base both-layer polish {iteration}: no improving legal move"
                ))?;
                break;
            }
            topology_stage(&format!(
                "base both-layer polish {iteration}: accepted angle {:.6}, aspect {:.6}, violation {:.6}",
                best.1, best.2, best.0
            ))?;
            topology = best.4;
        }
    }
    // Freeze topology first, then evaluate every surface on that exact final
    // domain. `base` may otherwise still describe the preceding best
    // iteration when the bounded relaxation exits without accepting its last
    // candidate, while morph endpoints are evaluated on the final topology.
    topology_stage("frozen export evaluation")?;
    let base = generate_shape_with_cache(
        &topology,
        design,
        surface,
        &base_positions,
        &base_semantic_coordinates,
        surface.front,
        surface.upper_rig_anchors,
        &surface.clearance_mesh.base,
        &base_coronal_depths,
        &mut field_cache.borrow_mut(),
    )?;
    let mid_count = base.mid_samples.len();
    if !diagnostic_surface_preview {
        validate_final_main_quality(&topology, &base.positions, mid_count, "base")?;
    }
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
        .zip(&surface.clearance_mesh.morphs)
        .zip(&surface.morph_coronal_depths)
        .map(
            |(
                ((((morph, anchors), semantic_coordinates), front), clearance_vertices),
                coronal_depths,
            )| {
                let target = generate_shape_with_cache(
                    &topology,
                    design,
                    surface,
                    &morph.positions,
                    semantic_coordinates,
                    *front,
                    *anchors,
                    clearance_vertices,
                    coronal_depths,
                    &mut field_cache.borrow_mut(),
                )?;
                if target.indices != base.indices || target.positions.len() != base.positions.len()
                {
                    eprintln!("breastplate morph topology mismatch {}", morph.name);
                    return Err(GenerateError::InvalidSurface);
                }
                if !diagnostic_surface_preview {
                    validate_final_main_quality(
                        &topology,
                        &target.positions,
                        mid_count,
                        &morph.name,
                    )?;
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

/// Rig anchor arrays can enumerate either side first. Reflect the surface
/// point and its normal together using the actual local lateral coordinate.
fn canonicalize_lateral_anchor(
    (mut position, mut normal): ([f32; 3], [f32; 3]),
) -> ([f32; 3], [f32; 3]) {
    let orientation = if position[0] < 0.0 { -1.0 } else { 1.0 };
    position[0] *= orientation;
    normal[0] *= orientation;
    (position, normal)
}

/// Optimization has a finite work budget, not permission to export an
/// uncertified result. Check both actual solidified main layers after the
/// shared topology is frozen, separately for every supplied body endpoint.
fn validate_final_main_quality(
    topology: &CanonicalBreastplateTopology,
    positions: &[[f32; 3]],
    mid_count: usize,
    endpoint: &str,
) -> Result<(), GenerateError> {
    let count = topology.canonical_positions().len();
    let surfaces = [
        positions[..count].to_vec(),
        positions[mid_count..mid_count + count].to_vec(),
    ];
    for layer in &surfaces {
        for face in topology.indices().as_chunks::<3>().0 {
            let [a, b, c] = face.map(|index| layer[index as usize]);
            let area = length(cross(sub(b, a), sub(c, a)));
            if !area.is_finite() || area <= 0.0 {
                eprintln!("breastplate final main degeneracy endpoint={endpoint} face={face:?}");
                return Err(GenerateError::InvalidSurface);
            }
        }
    }
    let (angle, aspect, face) = physical_global_quality(topology, &surfaces);
    if !angle.is_finite() || !aspect.is_finite() || angle < 10.0 || aspect > 9.0 {
        if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
            let path = std::path::PathBuf::from(path).with_extension("quality.json");
            let diagnostic = format!(
                "{{\"endpoint\":{endpoint:?},\"minimum_angle\":{angle},\"maximum_aspect\":{aspect},\"worst_face\":{face:?},\"main_count\":{count},\"mid_count\":{mid_count},\"positions\":{positions:?},\"indices\":{:?},\"canonical_domain\":{:?},\"metric_chart\":{:?}}}",
                topology.indices(),
                topology.canonical_positions(),
                topology.chart_positions(),
            );
            if let Err(error) = std::fs::write(path, diagnostic) {
                eprintln!("breastplate quality diagnostic write failed: {error}");
            }
        }
        eprintln!(
            "breastplate final main quality failed endpoint={endpoint} angle={angle} aspect={aspect} face={face:?}"
        );
        return Err(GenerateError::InvalidSurface);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_construction_mode_separates_field_cache_and_preserves_flag_off_key() {
        let mut original = blake3::Hasher::new();
        original.update(b"identical body design and anchors");
        let policy = ["grounded", "landmarks", "elliptical"];
        let mut legacy = original.clone();
        hash_joint_construction_mode(&mut legacy, false, policy);
        let mut joint = original.clone();
        hash_joint_construction_mode(&mut joint, true, policy);
        assert_eq!(original.finalize(), legacy.finalize());
        assert_ne!(original.finalize(), joint.finalize());
        for changed in [
            ["minimum-bending", "landmarks", "elliptical"],
            ["grounded", "dense", "elliptical"],
            ["grounded", "landmarks", "bernstein"],
        ] {
            let mut variant = original.clone();
            hash_joint_construction_mode(&mut variant, true, changed);
            assert_ne!(joint.finalize(), variant.finalize());
        }
    }

    fn assert_closed_oriented_edges(indices: &[u32]) {
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        for face in indices.chunks_exact(3) {
            for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
                edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
            }
        }
        for incidents in edges.values() {
            assert_eq!(incidents.len(), 2);
            assert_eq!(incidents[0], (incidents[1].1, incidents[1].0));
        }
    }

    #[test]
    fn solidification_orients_rims_from_faces_for_either_boundary_traversal() {
        let faces = [[0, 1, 2], [0, 2, 3]];
        for boundary in [[0, 1, 2, 3], [0, 3, 2, 1]] {
            let mut indices = Vec::new();
            append_closed_component(&mut indices, &faces, &boundary, 0, 4).unwrap();
            assert_closed_oriented_edges(&indices);
            assert_eq!(&indices[..12], &[0, 1, 2, 4, 6, 5, 0, 2, 3, 4, 7, 6]);
            let mut old_rims = Vec::new();
            for i in 0..boundary.len() {
                let a = boundary[i];
                let b = boundary[(i + 1) % boundary.len()];
                old_rims.extend([a, b, b + 4, a, b + 4, a + 4]);
            }
            for (actual, old) in indices[12..].chunks_exact(3).zip(old_rims.chunks_exact(3)) {
                let mut actual = actual.to_vec();
                let mut old = old.to_vec();
                actual.sort_unstable();
                old.sort_unstable();
                assert_eq!(actual, old, "rim diagonal must stay unchanged");
            }
        }
    }

    #[test]
    fn solidification_orients_both_separate_components_with_shared_layer_offset() {
        let mut indices = Vec::new();
        let main = [[0, 1, 2], [0, 2, 3]];
        let skirt = [[0, 2, 1], [0, 3, 2]];
        append_closed_component(&mut indices, &main, &[0, 3, 2, 1], 0, 8).unwrap();
        append_closed_component(&mut indices, &skirt, &[0, 3, 2, 1], 4, 8).unwrap();
        assert_closed_oriented_edges(&indices);
    }

    #[test]
    fn solidification_rejects_invalid_boundary_without_partially_appending() {
        let faces = [[0, 1, 2], [0, 2, 3]];
        for boundary in [vec![0, 1, 4, 3], vec![0, 1, 2], vec![0, 1, 2, 3, 0, 1]] {
            let mut indices = vec![99];
            assert!(append_closed_component(&mut indices, &faces, &boundary, 0, 4).is_err());
            assert_eq!(indices, [99]);
        }
        let mut indices = Vec::new();
        assert!(
            append_closed_component(&mut indices, &[[0, 1, 2], [0, 3, 2]], &[0, 1, 2, 3], 0, 4)
                .is_err()
        );
        assert!(indices.is_empty());
    }

    #[test]
    fn metric_recipe_endpoint_guard_does_not_hide_a_worse_body_behind_global_plateau() {
        assert!(quality_shortfalls_do_not_worsen((5.0, 15.0), (6.0, 14.0)));
        assert!(quality_shortfalls_do_not_worsen((12.0, 7.0), (11.0, 8.0)));
        assert!(!quality_shortfalls_do_not_worsen((12.0, 7.0), (9.9, 7.0)));
        assert!(!quality_shortfalls_do_not_worsen((6.0, 12.0), (5.9, 11.0)));
        assert!(!quality_shortfalls_do_not_worsen((6.0, 12.0), (7.0, 12.1)));
        assert!(!quality_shortfalls_do_not_worsen(
            (12.0, 7.0),
            (f32::NAN, 7.0)
        ));
        assert!(!quality_shortfalls_do_not_worsen(
            (12.0, 7.0),
            (12.0, f32::NAN)
        ));
        assert!(!quality_shortfalls_do_not_worsen(
            (12.0, 7.0),
            (f32::INFINITY, 7.0)
        ));
        // Endpoint A can hold the family's minimum at5 degrees; B still must
        // not regress from6 to5.5 simply because that global minimum is equal.
        let before = [(5.0, 15.0), (6.0, 12.0)];
        let after = [(5.0, 15.0), (5.5, 12.0)];
        assert!(
            !before
                .into_iter()
                .zip(after)
                .all(|(a, b)| quality_shortfalls_do_not_worsen(a, b))
        );
    }

    #[test]
    fn metric_recipe_plateau_requires_baseline_relative_improvement_not_absolute_angle() {
        assert!(!plateau_local_gain_improves(6.0, 6.0, 0.0));
        assert!(!plateau_local_gain_improves(6.0, 5.9, 0.0));
        assert!(plateau_local_gain_improves(6.0, 7.0, 0.0));
        // A different vertex starting at10 degrees must improve by more than
        // the current1-degree winner; its larger absolute angle is irrelevant.
        assert!(!plateau_local_gain_improves(10.0, 10.5, 1.0));
        assert!(plateau_local_gain_improves(5.0, 6.5, 1.0));
        assert!(!plateau_local_gain_improves(6.0, 5.9, -1.0));
    }

    #[test]
    fn metric_search_selects_aspect_owner_after_angles_pass_without_retargeting_legacy() {
        let faces = [[0, 1, 2], [3, 4, 5]];
        // Right triangle has the lower (passing)10.5-degree angle, while the
        // second, obtuse isosceles triangle has11-degree angles but aspect>9.
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 10.5f32.to_radians().tan(), 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 11.0f32.to_radians().tan(), 0.0],
        ];
        let legacy = physical_quality_summary(&faces, std::slice::from_ref(&points), false);
        let metric = physical_quality_summary(&faces, std::slice::from_ref(&points), true);
        assert!(metric.0 >= 10.0 && metric.1 > 9.0);
        assert_eq!(legacy.2, faces[0]);
        assert_eq!(metric.2, faces[1]);
        assert_eq!((legacy.0, legacy.1), (metric.0, metric.1));
        let mut angle_fails = points;
        angle_fails[2][1] = 9.5f32.to_radians().tan();
        assert_eq!(
            physical_quality_summary(&faces, &[angle_fails], true).2,
            faces[0]
        );
    }

    #[test]
    fn aspect_focused_search_rejects_passing_angle_only_plateaus() {
        let best = [2.3, 10.1, 9.23, 0.0];
        assert!(!quality_search_candidate_improves(
            true,
            best,
            [2.3, 10.5, 9.23, 0.0]
        ));
        assert!(!quality_search_candidate_improves(
            true,
            best,
            [2.3, 10.5, 9.23, -0.1]
        ));
        assert!(quality_search_candidate_improves(
            true,
            best,
            [2.3, 10.0, 9.23, 0.1]
        ));
        assert!(quality_search_candidate_improves(
            true,
            best,
            [1.0, 10.0, 9.1, 0.0]
        ));
        // With an actual angle failure, its original priority remains active.
        assert!(quality_search_candidate_improves(
            false,
            [100.0, 9.0, 8.0, 0.0],
            [90.0, 9.1, 8.0, 0.0]
        ));
    }

    #[test]
    fn armscye_support_tangent_includes_lateral_path_derivative() {
        let support = |x: f32, y: f32| Some(0.1 - 0.8 * x + 0.3 * y);
        for dxdy in [-0.4, 0.0, 0.4] {
            let slope = support_path_slope([0.2, 1.3], dxdy, support).unwrap();
            let expected = 0.3 - 0.8 * f64::from(dxdy);
            assert!((slope - expected).abs() < 3e-6);
            let body_normal = [0.8, -0.3, 1.0];
            assert!(dot(body_normal, [dxdy, 1.0, slope as f32]).abs() < 3e-6);
        }
        assert!(support_path_slope([0.2, 1.3], f32::NAN, support).is_none());
        assert!(support_path_slope([0.2, 1.3], 0.4, |_, _| None).is_none());
    }

    #[test]
    fn mirrored_crest_normals_follow_geometry_not_anchor_array_order() {
        let right = ([0.12, 0.56, 0.10], [0.3, 0.8, 0.5]);
        let left = ([-0.12, 0.56, 0.10], [-0.3, 0.8, 0.5]);
        assert_eq!(canonicalize_lateral_anchor(left), right);
        assert_eq!(canonicalize_lateral_anchor(right), right);
        for pair in [[left, right], [right, left]] {
            let anchors = pair.map(canonicalize_lateral_anchor);
            assert_eq!(anchors[0], anchors[1]);
        }
    }

    #[test]
    fn physical_triangle_quality_is_scale_invariant_and_rejects_degeneracy() {
        for scale in [1e-6_f32, 1.0, 1e6] {
            let triangle = [[0., 0., 0.], [1., 0., 0.], [0.5, 3.0_f32.sqrt() * 0.5, 0.]]
                .map(|point| point.map(|v| v * scale));
            let (angle, aspect) = physical_triangle_quality(triangle);
            assert!((angle - 60.0).abs() < 1e-4);
            assert!((aspect - 2.0 / 3.0_f32.sqrt()).abs() < 1e-5);
            let (angle, aspect) = physical_triangle_quality([[0.0; 3]; 3]);
            assert_eq!(angle, 0.0);
            assert!(aspect.is_infinite());
        }
    }

    #[test]
    fn final_quality_certificate_rejects_collapsed_exported_layers() {
        let topology = CanonicalBreastplateTopology::generate();
        let count = topology.canonical_positions().len();
        let collapsed = vec![[0.0; 3]; count * 2];
        assert!(validate_final_main_quality(&topology, &collapsed, count, "test").is_err());
    }

    #[test]
    fn missing_upper_support_interpolates_3d_anchors_without_depth_fallback() {
        let values = interpolate_guide_gaps(
            &[0.0, 0.2, 0.4, 0.7, 1.0],
            &[Some(-0.04), Some(-0.10), None, None, Some(-0.08)],
        )
        .unwrap();
        assert!((values[2] + 0.095).abs() < 1e-8);
        assert!((values[3] + 0.0875).abs() < 1e-8);
        assert!(values[1..].iter().all(|v| (-0.10..=-0.08).contains(v)));
        assert!(interpolate_guide_gaps(&[0.0, 1.0], &[None, Some(0.0)]).is_err());
    }

    #[test]
    fn surface_anchor_uses_first_outward_crossing_and_normal_padding() {
        let vertices = [
            TorsoShoulderSample {
                position: [-1.0, -1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            TorsoShoulderSample {
                position: [1.0, -1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            TorsoShoulderSample {
                position: [0.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            TorsoShoulderSample {
                position: [-1.0, -1.0, 2.0],
                normal: [0.0, 0.0, 1.0],
            },
            TorsoShoulderSample {
                position: [1.0, -1.0, 2.0],
                normal: [0.0, 0.0, 1.0],
            },
            TorsoShoulderSample {
                position: [0.0, 1.0, 2.0],
                normal: [0.0, 0.0, 1.0],
            },
        ];
        let hit = outward_surface_anchor(
            [0.0; 3],
            [0.0, 0.0, 1.0],
            0.015,
            &vertices,
            &[[3, 4, 5], [0, 1, 2]],
        )
        .unwrap();
        assert!((hit.position[2] - 1.015).abs() < 1e-6);
        assert_eq!(hit.normal, [0.0, 0.0, 1.0]);
        assert!(
            outward_surface_anchor([0.0; 3], [0.0, 0.0, -1.0], 0.015, &vertices, &[[0, 1, 2]])
                .is_err()
        );
    }

    #[test]
    fn angular_openings_follow_upper_rig_not_garment_height() {
        let original = SemanticMeasurements {
            waist: [0.2, 1.0],
            mid_axillary: [0.22, 1.3],
            shoulder: [0.14, 1.5],
            neck: [0.1, 1.49],
            neck_center_height: 1.47,
            torso_to_shoulder: 1.0,
        };
        let anchors = TorsoUpperRigAnchors {
            neck_base: [0.0, 1.5, 0.0],
            clavicles: [[0.03, 1.45, 0.0], [-0.03, 1.45, 0.0]],
            shoulders: [[0.18, 1.46, 0.0], [-0.18, 1.46, 0.0]],
        };
        let a = angular_upper_measurements(original, &BreastplateDesign::default(), anchors);
        let mut longer = original;
        longer.waist[1] -= 0.2;
        longer.neck = [0.16, 1.6];
        let b = angular_upper_measurements(longer, &BreastplateDesign::default(), anchors);
        assert_eq!(a.neck, b.neck);
        assert_eq!(a.neck_center_height, b.neck_center_height);
        assert_eq!(a.shoulder, b.shoulder);
        assert_eq!(a.waist, original.waist);
        assert_eq!(a.mid_axillary, original.mid_axillary);
        assert!(a.shoulder[0] > 0.16 && a.neck[0] < 0.09);
        assert!((anchors.neck_base[1] - a.neck_center_height) < 0.015);
    }

    #[test]
    fn simple_bridge_matches_neckline_and_armscye_tangents_without_lobes() {
        let m = SemanticMeasurements {
            waist: [0.2, 0.0],
            mid_axillary: [0.22, 0.3],
            shoulder: [0.14, 0.5],
            neck: [0.1, 0.49],
            neck_center_height: 0.47,
            torso_to_shoulder: 1.0,
        };
        let eps = 0.0001;
        let a = shoulder_bridge_curve(m, 0.0);
        let b = shoulder_bridge_curve(m, eps);
        let start = normalized2([b[0] - a[0], b[1] - a[1]]);
        let neck = normalized2([m.neck[0] * 0.12, (m.neck[1] - m.neck_center_height) * 0.58]);
        assert!(start[0] * neck[0] + start[1] * neck[1] > 0.9999);
        let a = shoulder_bridge_curve(m, 1.0 - eps);
        let b = shoulder_bridge_curve(m, 1.0);
        let end = normalized2([b[0] - a[0], b[1] - a[1]]);
        assert!(end[0] > 0.9999);
        let samples = (0..=100)
            .map(|i| shoulder_bridge_curve(m, i as f32 / 100.0))
            .collect::<Vec<_>>();
        assert!(samples.windows(2).all(|p| p[1][0] >= p[0][0]));
        assert!(
            samples
                .iter()
                .all(|p| p[0] >= m.neck[0] && p[0] <= m.shoulder[0])
        );
    }

    #[test]
    fn full_domain_orientation_check_catches_interior_fold_and_respects_metric_units() {
        let domain = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.5, 0.5]];
        let indices = [0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];
        let (area, faults) = domain_orientation_report(&domain, &indices, 1.0);
        assert!(faults.is_empty());
        let scaled = domain.map(|p| [p[0] * 0.25, p[1] * 0.25]);
        let (scaled_area, scaled_faults) = domain_orientation_report(&scaled, &indices, 1.0);
        assert!(scaled_faults.is_empty());
        assert!((scaled_area - area * 0.25_f64.powi(2)).abs() < 1e-12);
        let mut folded = domain;
        folded[4] = [1.1, 0.5];
        assert_eq!(
            domain_orientation_report(&folded, &indices, 1.0).1,
            vec![[1, 2, 4]]
        );
    }

    #[test]
    fn angular_torso_yoke_matches_position_and_longitudinal_derivative() {
        let waist_y = 1.0_f32;
        let seam_y = 1.3_f32;
        let profiles = fit_loft_profiles(
            &[waist_y, seam_y].map(|y| TorsoSlice {
                height: y as f64,
                points: vec![[-0.2, 0.0], [0.0, 0.12], [0.2, 0.0]],
            }),
            &[[waist_y as f64, 0.0], [seam_y as f64, 0.0]],
            LoftFitOptions::default(),
        )
        .unwrap();
        let mut surface = AngularSurface {
            joint_profiles: None,
            global_upper: None,
            profiles,
            boundary_domain: vec![],
            top_guides: vec![[0.0, 1.6, 0.0], [0.24, seam_y, 0.0]],
            top_parameters: vec![0.0, 1.0],
            top_depth_controls: vec![0.02; 8],
            top_slope_controls: vec![-0.4; 8],
            waist_y,
            seam_y,
            arc_scale: 0.25,
            side_trim: 0.0,
            trim_start_y: waist_y,
        };
        for u in [0.15, 0.35, 0.55, 0.85] {
            for t in [0.1_f32, 0.7, 1.0] {
                let point = |u: f32, t: f32| {
                    surface
                        .local_position([
                            u * surface.arc_scale,
                            seam_y + t * (surface.top(u)[1] - seam_y),
                        ])
                        .unwrap()
                };
                let e = 0.0005;
                let du = sub(point(u + e, t), point(u - e, t));
                let dt = sub(point(u, (t + e).min(1.0)), point(u, (t - e).max(0.0)));
                let geometric =
                    normalized(cross(normalized(du).unwrap(), normalized(dt).unwrap())).unwrap();
                let domain = [
                    u * surface.arc_scale,
                    seam_y + t * (surface.top(u)[1] - seam_y),
                ];
                let normal = surface.local_normal(domain).unwrap();
                assert!(
                    dot(normal, geometric) > 0.999,
                    "surface normal mismatch at u={u} t={t}: {:?} {:?}",
                    normal,
                    geometric
                );
                let p = point(u, t);
                let outer = add(p, scale(normal, 0.0015));
                let inner = add(p, scale(normal, -0.0015));
                assert!((dot(sub(outer, inner), geometric) - 0.003).abs() < 0.000004);
                let mirror = surface.local_normal([-domain[0], domain[1]]).unwrap();
                assert_eq!(normal[0], -mirror[0]);
                assert_eq!(normal[1], mirror[1]);
                assert_eq!(normal[2], mirror[2]);
            }
        }
        for u in [0.05, 0.25, 0.55, 0.8] {
            for t in [0.2, 0.5, 0.8] {
                let y = seam_y + t * (surface.top(u)[1] - seam_y);
                let s = u * surface.arc_scale;
                let row = angular_affine_depth(&surface, s as f64, y as f64).unwrap();
                let predicted = row[0]
                    + row[1..]
                        .iter()
                        .zip(
                            surface
                                .top_depth_controls
                                .iter()
                                .chain(&surface.top_slope_controls),
                        )
                        .map(|(a, b)| a * (*b as f64))
                        .sum::<f64>();
                let actual = surface.local_position([s, y]).unwrap()[2];
                assert!(
                    (predicted - actual as f64).abs() < 2e-7,
                    "linear yoke row differs at u={u},t={t}"
                );
            }
        }
        let mut bending = vec![vec![0.0; 16]; 16];
        let mut bending_rhs = vec![0.0; 16];
        add_angular_yoke_bending(&surface, &mut bending, &mut bending_rhs, 0.03, 1.0).unwrap();
        let mut weaker = vec![vec![0.0; 16]; 16];
        let mut weaker_rhs = vec![0.0; 16];
        add_angular_yoke_bending(&surface, &mut weaker, &mut weaker_rhs, 0.03, 0.01).unwrap();
        for (actual, reference) in weaker.iter().flatten().zip(bending.iter().flatten()) {
            assert!((actual - 0.01 * reference).abs() <= 1e-11 * reference.abs().max(1.0));
        }
        for (actual, reference) in weaker_rhs.iter().zip(&bending_rhs) {
            assert!((actual - 0.01 * reference).abs() <= 1e-11 * reference.abs().max(1.0));
        }
        for i in 0..16 {
            for j in 0..16 {
                assert!((bending[i][j] - bending[j][i]).abs() < 1e-10);
            }
        }
        for frequency in [0.3_f64, 1.1, 2.7] {
            let v = (0..16)
                .map(|i| (i as f64 * frequency).sin())
                .collect::<Vec<_>>();
            let energy = (0..16)
                .map(|i| (0..16).map(|j| v[i] * bending[i][j] * v[j]).sum::<f64>())
                .sum::<f64>();
            assert!(energy >= -1e-10);
        }
        for u in [0.0, 0.2, 0.4, 0.6] {
            let epsilon = 0.0001;
            let left = surface
                .local_position([u * surface.arc_scale, seam_y - epsilon])
                .unwrap();
            let center = surface
                .local_position([u * surface.arc_scale, seam_y])
                .unwrap();
            let right = surface
                .local_position([u * surface.arc_scale, seam_y + epsilon])
                .unwrap();
            for axis in [0, 2] {
                let dl = (center[axis] - left[axis]) / epsilon;
                let dr = (right[axis] - center[axis]) / epsilon;
                assert!(
                    (dl - dr).abs() < 0.012,
                    "C1 join mismatch u={u} axis={axis} left={dl} right={dr}"
                );
            }
            let mirror = surface
                .local_position([-u * surface.arc_scale, seam_y])
                .unwrap();
            assert_eq!(center[0], -mirror[0]);
            assert_eq!(center[2], mirror[2]);
        }
        assert!(surface.local_position([1.1, 1.1]).is_err());
        assert!(surface.local_position([0.3, 0.9]).is_err());
        for y in [1.32, 1.38, 1.48] {
            let epsilon = 0.0001;
            let center = surface.local_position([0.0, y]).unwrap();
            let left = surface.local_position([-epsilon, y]).unwrap();
            let right = surface.local_position([epsilon, y]).unwrap();
            assert!(((center[2] - left[2]) - (right[2] - center[2])).abs() / epsilon < 0.005);
        }
        assert!((surface.top(0.00001)[1] - surface.top(0.0)[1]).abs() < 1e-7);
        // The shrinking-span underarm corner additionally needs its boundary
        // tangent IN the torso tangent plane; fixed-u C1 alone is insufficient.
        let [a, b, c] = surface.profiles.eval(seam_y as f64);
        let [da, db, dc] = surface.profiles.derivative(seam_y as f64);
        let dy = (surface.top_guides[1][1] - surface.top_guides[0][1]) as f64;
        for trim in [0.0, 0.09] {
            surface.side_trim = trim;
            let theta = surface.coverage_angle(seam_y) as f64;
            surface.top_guides[1][0] = (a * theta.sin()) as f32;
            let side_z = c + b * theta.cos();
            let side_slope = dc + db * theta.cos();
            let dz = side_slope * dy - b * theta.sin() * theta;
            surface.top_depth_controls[7] = side_z as f32;
            surface.top_depth_controls[6] = (side_z - dz / 15.0) as f32;
            surface.top_slope_controls[7] = side_slope as f32;
            surface.top_slope_controls[6] = (side_slope + db * theta.sin() * theta / 15.0) as f32;
            let eps = 0.0001;
            let end = surface.top(1.0);
            let before = surface.top(1.0 - eps);
            let expected = [da * theta.sin() * dy + a * theta.cos() * theta, dy, dz];
            for axis in 0..3 {
                let actual = (end[axis] - before[axis]) / eps;
                assert!(
                    (actual as f64 - expected[axis]).abs() < 0.003,
                    "underarm tangent-plane mismatch axis={axis}, actual={actual}, expected={}",
                    expected[axis]
                );
            }
            for u in [0.0, 0.4, 0.9] {
                let p0 = surface
                    .local_position([u * surface.arc_scale, seam_y - eps])
                    .unwrap();
                let p1 = surface
                    .local_position([u * surface.arc_scale, seam_y])
                    .unwrap();
                let p2 = surface
                    .local_position([u * surface.arc_scale, seam_y + eps])
                    .unwrap();
                for axis in [0, 2] {
                    assert!(
                        ((p1[axis] - p0[axis]) - (p2[axis] - p1[axis])).abs() / eps < 0.02,
                        "trimmed C1 mismatch trim={trim} u={u}"
                    );
                }
            }
            let limit = surface.local_normal([surface.arc_scale, seam_y]).unwrap();
            let u = 0.999;
            let near = surface
                .local_normal([
                    u * surface.arc_scale,
                    seam_y + 0.1 * (surface.top(u)[1] - seam_y),
                ])
                .unwrap();
            assert!(
                dot(limit, near) > 0.99,
                "collapsed normal limit mismatch trim={trim}: {:?} {:?}",
                limit,
                near
            );
        }
    }

    #[test]
    fn quality_sheet_is_bit_exact_export_prefix_for_each_endpoint() {
        // The shared offset kernel operates independently of skirt suffix,
        // render normals and skin binding. Check multiple endpoint fields.
        for lateral_scale in [0.8, 1.0, 1.4] {
            let main = [
                [-0.2 * lateral_scale, 1.0, 0.05],
                [0.0, 1.4, 0.2],
                [0.2 * lateral_scale, 1.0, 0.05],
            ];
            let normals = main.map(|p| normalized([p[0], 0.0, p[2]]).unwrap());
            let quality = offset_sheet(&main, &normals, 0.002);
            let mut complete = main.to_vec();
            complete.push([0.0, 0.97, 0.23]);
            let mut complete_normals = normals.to_vec();
            complete_normals.push([0.0, 0.2, 0.98]);
            let exported = offset_sheet(&complete, &complete_normals, 0.002);
            assert_eq!(quality, exported[..main.len()]);
            for (index, p) in main.iter().enumerate() {
                assert_eq!(quality[index], add(*p, scale(normals[index], 0.002)));
            }
        }
    }

    #[test]
    fn retained_family_shapes_match_fresh_scores_before_and_after_rail_moves() {
        let canonical = CanonicalBreastplateTopology::generate();
        let boundary = &canonical.canonical_positions()[..canonical.boundary_vertices().len()];
        let mut topology = CanonicalBreastplateTopology::from_metric_chart_with_candidates(
            boundary,
            boundary,
            &[],
            0.5,
            false,
            |point| point,
        );
        let evaluate = |topology: &CanonicalBreastplateTopology, scale: f32| {
            let boundary = topology.canonical_positions()[..topology.boundary_vertices().len()]
                .iter()
                .map(|p| [scale * p[0], p[1]])
                .collect::<Vec<_>>();
            let domain = topology.semantic_domain_from_boundary(&boundary);
            let main = domain
                .iter()
                .map(|p| [p[0], p[1], 0.2 + 0.1 * p[0] * p[0] + 0.03 * p[1]])
                .collect::<Vec<_>>();
            let normals = domain
                .iter()
                .map(|p| normalized([-0.2 * p[0], -0.03, 1.0]).unwrap())
                .collect::<Vec<_>>();
            PhysicalShape {
                positions: offset_sheet(&main, &normals, 0.0015),
                inner_positions: offset_sheet(&main, &normals, -0.0015),
            }
        };
        let mut base = evaluate(&topology, 1.0);
        let mut retained = [0.98, 1.02]
            .map(|scale| evaluate(&topology, scale))
            .into_iter()
            .collect::<Vec<_>>();
        for moved in [false, true] {
            if moved {
                assert!((0..topology.boundary_vertices().len()).any(|station| {
                    topology.relax_inner_rail_station_preserving_embedding(station, 0.02, 0.0)
                }));
                // Accepted candidates transfer their own evaluated family;
                // no old-reference values may survive this transition.
                base = evaluate(&topology, 1.0);
                retained = [0.98, 1.02]
                    .map(|scale| evaluate(&topology, scale))
                    .into_iter()
                    .collect::<Vec<_>>();
            }
            for station in [0, 28, 96] {
                let vertex = topology.inner_rail_vertex(station).unwrap();
                let fresh_base = evaluate(&topology, 1.0);
                let fresh = [0.98, 1.02].map(|scale| evaluate(&topology, scale));
                let mut old_score = physical_vertex_quality(&topology, &fresh_base, vertex);
                for shape in &fresh {
                    let q = physical_vertex_quality(&topology, shape, vertex);
                    old_score = (old_score.0.min(q.0), old_score.1.max(q.1));
                }
                assert_eq!(
                    physical_vertex_quality_family(&topology, &base, &retained, vertex),
                    old_score
                );
                assert_eq!(base.positions, fresh_base.positions);
                assert_eq!(base.inner_positions, fresh_base.inner_positions);
                for (cached, recomputed) in retained.iter().zip(&fresh) {
                    assert_eq!(cached.positions, recomputed.positions);
                    assert_eq!(cached.inner_positions, recomputed.inner_positions);
                }
            }
        }
    }

    #[test]
    fn physical_hessian_has_affine_nullspace_and_correct_mixed_derivative() {
        let field = FittedDepthField {
            controls: vec![],
            bottom_y: 1.0,
            top_y: 1.5,
            half_extent: 0.3,
        };
        let knot = |count: usize, i: usize| {
            if i <= 3 {
                0.0
            } else if i >= count {
                1.0
            } else {
                (i - 3) as f64 / (count - 3) as f64
            }
        };
        let mut affine = vec![0.0; 54];
        let mut xxy = vec![0.0; 54];
        for row in 0..9 {
            let gy = (1..=3).map(|k| knot(9, row + k)).sum::<f64>() / 3.0;
            let y = field.bottom_y as f64 + gy * (field.top_y - field.bottom_y) as f64;
            for column in 0_usize..7 {
                let q = [
                    knot(7, column + 1),
                    knot(7, column + 2),
                    knot(7, column + 3),
                ];
                let x2 = (q[0] * q[1] + q[0] * q[2] + q[1] * q[2]) / 3.0
                    * (field.half_extent as f64).powi(2);
                let free = row * 6 + column.saturating_sub(1);
                affine[free] = 0.2 + 0.3 * y;
                xxy[free] = x2 * y;
            }
        }
        let x = 0.13_f32;
        let y = 1.31_f32;
        let derivatives = field.physical_derivatives(x, y).unwrap();
        let dot = |row: &[f64], controls: &[f64]| {
            row.iter().zip(controls).map(|(a, b)| a * b).sum::<f64>()
        };
        for row in &derivatives[2..] {
            assert!(dot(row, &affine).abs() < 1e-11);
        }
        assert!((dot(&derivatives[2], &xxy) - 2.0 * y as f64).abs() < 1e-11);
        assert!((dot(&derivatives[3], &xxy) - 2.0 * x as f64).abs() < 1e-11);
        assert!(dot(&derivatives[4], &xxy).abs() < 1e-11);
        let energy = |rows: &[Vec<f64>; 5], c: &[f64], length: f64| {
            length.powi(4)
                * (dot(&rows[2], c).powi(2)
                    + 2.0 * dot(&rows[3], c).powi(2)
                    + dot(&rows[4], c).powi(2))
        };
        assert!(energy(&derivatives, &xxy, 0.02) > 0.0);
        let scaled = FittedDepthField {
            controls: vec![],
            bottom_y: 2.0,
            top_y: 3.0,
            half_extent: 0.6,
        };
        let scaled_rows = scaled.physical_derivatives(2.0 * x, 2.0 * y).unwrap();
        let scaled_controls = xxy.iter().map(|c| 2.0 * c).collect::<Vec<_>>();
        assert!(
            (energy(&scaled_rows, &scaled_controls, 0.04) / energy(&derivatives, &xxy, 0.02) - 4.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn topology_acceptance_is_scoped_to_the_requested_garment_recipe() {
        let requested = BreastplateDesign::default();
        assert_eq!(
            topology_acceptance_designs(&requested),
            vec![requested.clone()]
        );
        let mut other_recipe = requested.clone();
        other_recipe.crown.0 = 80;
        other_recipe.neck_width.0 = 700;
        assert_eq!(
            topology_acceptance_designs(&other_recipe),
            vec![other_recipe.clone()]
        );
        assert_ne!(
            breastplate_design_hash(&requested).unwrap(),
            breastplate_design_hash(&other_recipe).unwrap()
        );
    }

    #[test]
    fn analytic_cubic_derivatives_match_known_polynomials() {
        for t in [0.0, 0.01, 0.25, 0.5, 0.9, 1.0] {
            let [basis, first, second] = cubic_basis_derivatives(4, t);
            assert!((basis.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(first.iter().sum::<f64>().abs() < 1e-12);
            assert!(second.iter().sum::<f64>().abs() < 1e-12);
            assert!((basis[3] - t.powi(3)).abs() < 1e-12);
            assert!((first[3] - 3.0 * t * t).abs() < 1e-12);
            assert!((second[3] - 6.0 * t).abs() < 1e-12);
            let linear = [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];
            assert!(
                (first.iter().zip(linear).map(|(a, b)| a * b).sum::<f64>() - 1.0).abs() < 1e-12
            );
            assert!(
                second
                    .iter()
                    .zip(linear)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
                    .abs()
                    < 1e-12
            );
        }
    }

    #[test]
    fn material_domain_excludes_neck_hole_but_keeps_bridge() {
        let boundary = [
            [-1.0, 0.0],
            [1.0, 0.0],
            [1.0, 2.0],
            [0.4, 2.0],
            [0.4, 1.5],
            [-0.4, 1.5],
            [-0.4, 2.0],
            [-1.0, 2.0],
        ];
        assert!(!material_contains(&boundary, [0.0, 1.7]));
        assert!(material_contains(&boundary, [0.0, 1.3]));
        assert!(material_contains(&boundary, [0.6, 1.7]));
        assert!(material_contains(&boundary, [0.4, 1.7]));
        assert!(!material_contains(&boundary, [1.1, 1.7]));
        let grid = fixed_surface_quadrature(&boundary);
        assert!(
            grid.iter()
                .all(|point| material_contains(&boundary, *point))
        );
    }

    #[test]
    fn fitted_field_queries_do_not_collapse_upper_or_lateral_coordinates() {
        let field = FittedDepthField {
            controls: vec![0.0; 54],
            bottom_y: 1.0,
            top_y: 1.5,
            half_extent: 0.3,
        };
        assert_ne!(
            field.coordinates(0.15, 1.42).unwrap(),
            field.coordinates(0.15, 1.46).unwrap()
        );
        assert_ne!(
            field.coordinates(0.22, 1.35).unwrap(),
            field.coordinates(0.26, 1.35).unwrap()
        );
        assert!(field.coordinates(0.301, 1.35).is_err());
        assert!(field.coordinates(0.15, 1.501).is_err());
        assert!(field.coordinates(0.15, 0.999).is_err());
    }

    #[test]
    fn fitted_field_is_unchanged_by_query_order_or_added_mesh_vertices() {
        let field = FittedDepthField {
            controls: (0..54).map(|i| (i as f32 * 0.1).sin() * 0.02).collect(),
            bottom_y: 1.0,
            top_y: 1.5,
            half_extent: 0.3,
        };
        let queries = [[0.0, 1.1], [0.12, 1.3], [0.25, 1.42]];
        let before = queries.map(|p| field.depth(p[0], p[1]).unwrap().to_bits());
        for p in [[0.17, 1.21], [0.07, 1.47]]
            .into_iter()
            .chain(queries.into_iter().rev())
        {
            assert!(field.depth(p[0], p[1]).unwrap().is_finite());
        }
        assert_eq!(
            before,
            queries.map(|p| field.depth(p[0], p[1]).unwrap().to_bits())
        );
        let boundary = [[-0.3, 1.0], [0.3, 1.0], [0.3, 1.5], [-0.3, 1.5]];
        let grid = fixed_surface_quadrature(&boundary);
        assert_eq!(grid, fixed_surface_quadrature(&boundary));
        assert!(
            grid[boundary.len()..]
                .iter()
                .all(|p| field.coordinates(p[0], p[1]).is_ok())
        );
    }

    #[test]
    fn fitted_field_cache_key_covers_design_body_morph_and_support_inputs() {
        let anchors = TorsoUpperRigAnchors {
            neck_base: [0.0, 1.4, 0.0],
            clavicles: [[0.0; 3]; 2],
            shoulders: [[0.0; 3]; 2],
        };
        let clearance = vec![TorsoShoulderSample {
            position: [0.1, 1.3, 0.0],
            normal: [0.0, 0.0, 1.0],
        }];
        let surface = TorsoSurface {
            domain: "cache-fixture".into(),
            front: [0.0, 0.0, 1.0],
            morph_fronts: vec![],
            upper_rig_anchors: anchors,
            morph_upper_rig_anchors: vec![],
            morph_semantic_coordinates: vec![],
            vertices: vec![],
            faces: vec![[0, 1, 2]],
            coronal_anchors: vec![crate::TorsoCoronalAnchor {
                vertical: 0.5,
                depth: 0.0,
            }],
            morph_coronal_depths: vec![],
            shoulder_envelope: vec![],
            morph_shoulder_envelopes: vec![],
            clearance_mesh: crate::TorsoClearanceMesh {
                base: crate::TorsoClearancePose {
                    vertices: clearance.clone(),
                    enclosure_vertices: vec![],
                },
                faces: vec![],
                enclosure_faces: vec![],
                morphs: vec![],
            },
            morphs: vec![],
        };
        let design = BreastplateDesign::default();
        let positions = [[0.0, 1.0, 0.0], [0.1, 1.0, 0.0], [0.0, 1.3, 0.0]];
        let semantic = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let key = |d: &BreastplateDesign,
                   p: &[[f32; 3]],
                   a,
                   c: &[TorsoShoulderSample],
                   depths: &[f32]| {
            let pose = crate::TorsoClearancePose {
                vertices: c.to_vec(),
                enclosure_vertices: vec![],
            };
            surface_field_key(d, &surface, p, &semantic, surface.front, a, &pose, depths).unwrap()
        };
        let original = key(&design, &positions, anchors, &clearance, &[0.0]);
        assert_eq!(
            original,
            key(&design, &positions, anchors, &clearance, &[0.0])
        );
        let mut changed_design = design.clone();
        changed_design.clearance.0 += 1;
        assert_ne!(
            original,
            key(&changed_design, &positions, anchors, &clearance, &[0.0])
        );
        changed_design = design.clone();
        changed_design.wall_thickness.0 += 1;
        assert_ne!(
            original,
            key(&changed_design, &positions, anchors, &clearance, &[0.0])
        );
        let mut morph = positions;
        morph[1][2] += 0.001;
        assert_ne!(original, key(&design, &morph, anchors, &clearance, &[0.0]));
        assert_ne!(
            original,
            key(&design, &positions, anchors, &clearance, &[0.001])
        );
        let mut changed_anchors = anchors;
        changed_anchors.shoulders[0][1] += 0.001;
        assert_ne!(
            original,
            key(&design, &positions, changed_anchors, &clearance, &[0.0])
        );
        let mut changed_clearance = clearance.clone();
        changed_clearance[0].position[2] += 0.001;
        assert_ne!(
            original,
            key(&design, &positions, anchors, &changed_clearance, &[0.0])
        );
    }

    #[test]
    fn tied_center_controls_make_every_mirrored_cubic_row_c1() {
        for row in 0..9 {
            let center = 0.12 + row as f32 * 0.009;
            let controls = [center, center, center - 0.02, 0.08, 0.06, 0.04, 0.02];
            let evaluate = |q: f32| {
                clamped_cubic_bspline_basis(7, q.abs())
                    .iter()
                    .zip(controls)
                    .map(|(weight, control)| weight * control)
                    .sum::<f32>()
            };
            let epsilon = 1e-4;
            let left_slope = (evaluate(0.0) - evaluate(-epsilon)) / epsilon;
            let right_slope = (evaluate(epsilon) - evaluate(0.0)) / epsilon;
            assert!(left_slope.abs() < 0.002);
            assert!(right_slope.abs() < 0.002);
            assert_eq!(evaluate(-0.15), evaluate(0.15));
        }
    }

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
