//! Stable authored meridians for an enclosing elliptical boot shaft.
use super::local;
use adventuresim_armor_model::PartFrame;
use bevy::math::{DMat4, DVec4};

pub(super) const RING_PLANE_TOLERANCE_M: f32 = 0.000001;

#[derive(Debug, PartialEq)]
pub(super) enum ShaftProfileError {
    IncompleteRings,
    InvalidEllipse {
        row: usize,
    },
    ReversedMeridian {
        row: usize,
        column: usize,
        height: f32,
        hem: f32,
        blend: f32,
    },
}

impl std::fmt::Display for ShaftProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompleteRings => write!(
                f,
                "boot shaft requires complete rings and a positive transition span"
            ),
            Self::InvalidEllipse { row } => {
                write!(f, "boot shaft row {row} has no enclosing ellipse")
            }
            Self::ReversedMeridian {
                row,
                column,
                height,
                hem,
                blend,
            } => write!(
                f,
                "boot shaft row {row} reverses angular order at column {column} (height {height}m, hem {hem}m, blend {blend})"
            ),
        }
    }
}

impl std::error::Error for ShaftProfileError {}

struct EllipseSection {
    center: [f32; 2],
    radii: [f32; 2],
}

impl EllipseSection {
    fn enclosing(points: &[[f32; 3]]) -> Option<Self> {
        let low = [0, 2].map(|axis| points.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min));
        let high = [0, 2].map(|axis| {
            points
                .iter()
                .map(|p| p[axis])
                .fold(f32::NEG_INFINITY, f32::max)
        });
        let scale: [f64; 2] = std::array::from_fn(|axis| f64::from(high[axis] - low[axis]));
        if scale.iter().any(|r| !r.is_finite() || *r <= 0.0) {
            return None;
        }
        let origin = [0, 2].map(|axis| {
            points.iter().map(|p| f64::from(p[axis])).sum::<f64>() / points.len() as f64
        });
        // Recover the carrier's ellipse even when projecting from a displaced
        // ankle concentrates most vertices on one side. A bounding-box center
        // would grow the opposite return because no vertex samples its tip.
        let mut normal = DMat4::ZERO;
        let mut rhs = DVec4::ZERO;
        for p in points {
            let x = (f64::from(p[0]) - origin[0]) / scale[0];
            let z = (f64::from(p[2]) - origin[1]) / scale[1];
            let row = DVec4::new(x * x, z * z, x, z);
            normal += DMat4::from_cols(row * row.x, row * row.y, row * row.z, row * row.w);
            rhs += row;
        }
        let determinant = normal.determinant();
        if !determinant.is_finite() || determinant <= 0.0 {
            return None;
        }
        let coefficients = normal.inverse() * rhs;
        if !coefficients.is_finite() || coefficients.x <= 0.0 || coefficients.y <= 0.0 {
            return None;
        }
        let quadratic = [coefficients.x, coefficients.y];
        let normalized_center = [
            -coefficients.z / (2.0 * quadratic[0]),
            -coefficients.w / (2.0 * quadratic[1]),
        ];
        let extent = 1.0
            + quadratic[0] * normalized_center[0].powi(2)
            + quadratic[1] * normalized_center[1].powi(2);
        let center = std::array::from_fn(|axis| {
            (origin[axis] + normalized_center[axis] * scale[axis]) as f32
        });
        let radii: [f32; 2] =
            std::array::from_fn(|axis| ((extent / quadratic[axis]).sqrt() * scale[axis]) as f32);
        let enclosing = points
            .iter()
            .map(|p| ((p[0] - center[0]) / radii[0]).hypot((p[2] - center[1]) / radii[1]))
            .fold(1.0_f32, f32::max);
        Some(Self {
            center,
            radii: radii.map(|radius| radius * enclosing),
        })
    }

    fn point(&self, angle: f32) -> [f32; 2] {
        [
            self.center[0] + self.radii[0] * angle.sin(),
            self.center[1] + self.radii[1] * angle.cos(),
        ]
    }
}

/// Preserve angular correspondence when the calf and garment section centers
/// move relative to the ankle. Ray projection from each moving center otherwise
/// slides the authored vertices around the shaft between identity samples.
pub(super) fn regularize(
    carrier: &mut [[f32; 3]],
    frame: &PartFrame,
    hem: f32,
    transition_height: f32,
) -> Result<(), ShaftProfileError> {
    let points = carrier.iter().map(|p| local(frame, *p)).collect::<Vec<_>>();
    let Some(first) = points.first() else {
        return Err(ShaftProfileError::IncompleteRings);
    };
    let ring_size = points
        .iter()
        .take_while(|p| (p[1] - first[1]).abs() < RING_PLANE_TOLERANCE_M)
        .count();
    // The closed upper has complete rings followed by the sole's fan center.
    if ring_size < 3 || points.len() % ring_size != 1 || transition_height <= 0.0 {
        return Err(ShaftProfileError::IncompleteRings);
    }
    for (row, ring) in points.chunks_exact(ring_size).enumerate() {
        let t = ((ring[0][1] - hem) / transition_height).clamp(0.0, 1.0);
        if t == 0.0 {
            continue;
        }
        let Some(section) = EllipseSection::enclosing(ring) else {
            return Err(ShaftProfileError::InvalidEllipse { row });
        };
        let blend = t * t * (3.0 - 2.0 * t);
        if blend == 1.0 {
            // The fully regularized endpoint uses the authored chart alone.
            // Its incoming angular order does not contribute to this endpoint.
            for column in 0..ring_size {
                let angle = std::f32::consts::TAU * column as f32 / ring_size as f32;
                let [x, z] = section.point(angle);
                carrier[row * ring_size + column] = frame.point([x, ring[column][1], z]);
            }
            continue;
        }
        let mut first_angle = 0.0;
        let mut previous_angle = 0.0;
        for (column, before) in ring.iter().enumerate() {
            let normalized = [
                (before[0] - section.center[0]) / section.radii[0],
                (before[2] - section.center[1]) / section.radii[1],
            ];
            let mut angle = normalized[0].atan2(normalized[1]);
            if column == 0 {
                first_angle = angle;
            } else {
                while angle <= previous_angle {
                    angle += std::f32::consts::TAU;
                }
                if angle - first_angle >= std::f32::consts::TAU {
                    return Err(ShaftProfileError::ReversedMeridian {
                        row,
                        column,
                        height: before[1],
                        hem,
                        blend,
                    });
                }
            }
            previous_angle = angle;
            let target_angle = std::f32::consts::TAU * column as f32 / ring_size as f32;
            // A Cartesian chord cuts through an enclosing ellipse. Move in its
            // angle/radius chart; both angular sequences have the same winding.
            let target = section.point(angle + (target_angle - angle) * blend);
            let radius = normalized[0].hypot(normalized[1]);
            let radius = radius + (1.0 - radius) * blend;
            carrier[row * ring_size + column] = frame.point([
                section.center[0] + (target[0] - section.center[0]) * radius,
                before[1],
                section.center[1] + (target[1] - section.center[1]) * radius,
            ]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_transition_constructs_the_authored_chart_without_requiring_input_order() {
        let mut complete = carrier([0.0; 2], 1.0, 0.0);
        complete.swap(16 + 5, 16 + 6);
        let mut partial = complete.clone();
        for point in &mut partial[16..32] {
            point[1] = 0.5;
        }
        assert!(matches!(
            regularize(&mut partial, &frame(), 0.0, 1.0),
            Err(ShaftProfileError::ReversedMeridian { .. })
        ));
        regularize(&mut complete, &frame(), 0.0, 1.0).unwrap();
        for (column, point) in complete[16..32].iter().enumerate() {
            let angle = column as f32 * std::f32::consts::TAU / 16.0;
            assert!((point[0] - 0.1 * angle.sin()).abs() < 1e-6);
            assert!((point[2] - 0.08 * angle.cos()).abs() < 1e-6);
        }
    }

    fn frame() -> PartFrame {
        PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [1.0; 3],
        }
    }

    fn carrier(center: [f32; 2], scale: f32, phase: f32) -> Vec<[f32; 3]> {
        let mut points = Vec::new();
        for height in [0.0, 2.0] {
            for column in 0..16 {
                let angle = std::f32::consts::TAU * column as f32 / 16.0;
                let angle = angle + phase * (4.0 * angle).sin();
                points.push([
                    center[0] + scale * 0.1 * angle.sin(),
                    height,
                    center[1] + scale * 0.08 * angle.cos(),
                ]);
            }
        }
        points.push([center[0], 0.0, center[1]]);
        points
    }

    #[test]
    fn moving_sections_keep_meridians_without_moving_foot_or_sole_anchors() {
        let mut reference = carrier([0.0; 2], 1.0, 0.05);
        let original = reference.clone();
        let mut shifted = carrier([0.03, -0.04], 1.2, 0.15);
        assert!(regularize(&mut reference, &frame(), 0.0, 1.0).is_ok());
        assert!(regularize(&mut shifted, &frame(), 0.0, 1.0).is_ok());
        assert_eq!(&reference[..16], &original[..16]);
        assert_eq!(reference[32], original[32]);
        for (a, b) in reference[16..32].iter().zip(&shifted[16..32]) {
            assert!((b[0] - (a[0] * 1.2 + 0.03)).abs() < 1e-6);
            assert!((b[2] - (a[2] * 1.2 - 0.04)).abs() < 1e-6);
        }
    }

    #[test]
    fn collapsed_shaft_is_rejected() {
        let mut points = carrier([0.0; 2], 0.0, 0.0);
        assert!(regularize(&mut points, &frame(), 0.0, 1.0).is_err());
    }

    #[test]
    fn partial_chart_transitions_stay_on_shifted_uneven_ellipses() {
        for (center, radii) in [
            ([0.0, 0.0], [0.1, 0.08]),
            ([0.03, -0.04], [0.06, 0.11]),
            ([-0.05, 0.02], [0.15, 0.035]),
        ] {
            let mut points = Vec::new();
            for height in [0.0, 0.25, 0.5, 0.75, 1.0] {
                for column in 0..40 {
                    let u = column as f32 / 40.0 * std::f32::consts::TAU;
                    let angle = u + 0.8 * u.sin() + 0.3;
                    points.push([
                        center[0] + radii[0] * angle.sin(),
                        height,
                        center[1] + radii[1] * angle.cos(),
                    ]);
                }
            }
            points.push([center[0], 0.0, center[1]]);
            let original = points.clone();
            assert!(regularize(&mut points, &frame(), 0.0, 1.0).is_ok());
            assert_eq!(&points[..40], &original[..40]);
            assert_eq!(points[200], original[200]);
            for point in &points[..200] {
                let radius =
                    ((point[0] - center[0]) / radii[0]).hypot((point[2] - center[1]) / radii[1]);
                assert!(
                    (radius - 1.0).abs() < 1e-5,
                    "transition entered support: {radius}"
                );
            }
        }
    }

    #[test]
    fn uneven_samples_preserve_the_formed_ellipse_instead_of_inflating_its_return() {
        let points = (0..40)
            .map(|column| {
                let t = std::f32::consts::TAU * column as f32 / 40.0;
                let angle = t + 0.8 * t.sin() + 0.3;
                [0.03 + 0.1 * angle.sin(), 0.2, -0.04 + 0.08 * angle.cos()]
            })
            .collect::<Vec<_>>();
        let fitted = EllipseSection::enclosing(&points).unwrap();
        for (actual, expected) in fitted.center.into_iter().zip([0.03, -0.04]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        for (actual, expected) in fitted.radii.into_iter().zip([0.1, 0.08]) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
}
