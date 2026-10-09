//! Fit a lateral flare from the actual retained facets in each lap.
//!
//! Plane support guides fitting; it is not a shell intersection certificate.
//! Final closed metal surfaces are checked independently at installation.
use super::cut_frame::CourseFrame;
use crate::{AnimeDesign, GenerateError};

/// Free space for seating adjacent metal courses beyond projected gauge.
const COURSE_SEATING_CLEARANCE_METERS: f64 = 0.0005;

pub(super) enum CourseSide {
    Front,
    Rear,
}

pub(super) struct CourseFlare(f32);

#[derive(Clone, Copy)]
struct Corner {
    point: [f64; 3],
    offset: [f64; 3],
    level: f64,
}

impl Corner {
    fn between(a: Self, b: Self, level: f64) -> Self {
        let blend = (level - a.level) / (b.level - a.level);
        Self {
            point: std::array::from_fn(|k| a.point[k] + blend * (b.point[k] - a.point[k])),
            offset: std::array::from_fn(|k| a.offset[k] + blend * (b.offset[k] - a.offset[k])),
            level,
        }
    }
}

impl CourseFlare {
    pub(super) fn fit(
        inner: &[[f32; 3]],
        outer: &[[f32; 3]],
        faces: &[[u32; 3]],
        frame: &CourseFrame,
        design: &AnimeDesign,
        side: CourseSide,
        bounds: [f32; 4],
    ) -> Result<Self, GenerateError> {
        let [floor, pitch, width, _] = bounds.map(f64::from);
        if !(width > 0.0 && width.is_finite() && pitch > 0.0) {
            return Err(GenerateError::InvalidSurface);
        }
        let overlap = f64::from(design.overlap.metres());
        let separation = f64::from(design.lap_lift.metres()) * pitch / (pitch + overlap);
        let (slope, depth_sign) = match side {
            CourseSide::Front => (design.chevron_slope, 1.0),
            CourseSide::Rear => (design.rear_chevron_slope, -1.0),
        };
        let slope = f64::from(slope.unit());
        let mut scale = 1.0_f64;
        for &face in faces {
            let corners = face.map(|i| {
                let point = frame.local_point(inner[i as usize]);
                let outside = frame.local_point(outer[i as usize]);
                Corner {
                    point,
                    offset: std::array::from_fn(|k| outside[k] - point[k]),
                    level: point[1] - slope * point[0].abs(),
                }
            });
            let normal = normal(corners.map(|c| c.point))?;
            for course in 1..=design.lame_count {
                let high = floor + pitch * f64::from(course);
                let retained = cut(
                    &cut(&corners, (high - overlap).max(floor), false),
                    high,
                    true,
                );
                for corner in retained {
                    let lateral = (normal[0] + slope * corner.point[0].signum() * normal[1])
                        * corner.point[0]
                        / width;
                    // A lateral correction cannot help a facet that faces
                    // the other way. Concave finite patches can be separated
                    // tangentially; do not reject them using this local proxy.
                    if lateral <= 0.0 {
                        continue;
                    }
                    let gauge = (0..3)
                        .map(|k| normal[k] * corner.offset[k])
                        .sum::<f64>()
                        .max(0.0);
                    let required = (gauge + COURSE_SEATING_CLEARANCE_METERS) / separation;
                    scale = scale.max((required - normal[2] * depth_sign) / lateral);
                }
            }
        }
        let scale = scale as f32;
        if !scale.is_finite() {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(Self(scale))
    }
    pub(super) fn scale(&self) -> f32 {
        self.0
    }
}

fn cut(polygon: &[Corner], boundary: f64, upper: bool) -> Vec<Corner> {
    let mut output = Vec::new();
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        let sign = if upper { 1.0 } else { -1.0 };
        let da = (a.level - boundary) * sign;
        let db = (b.level - boundary) * sign;
        if da <= 0.0 {
            output.push(a);
        }
        if da * db < 0.0 {
            output.push(Corner::between(a, b, boundary));
        }
    }
    output
}

fn normal(points: [[f64; 3]; 3]) -> Result<[f64; 3], GenerateError> {
    let a = std::array::from_fn::<_, 3, _>(|k| points[1][k] - points[0][k]);
    let b = std::array::from_fn::<_, 3, _>(|k| points[2][k] - points[0][k]);
    let n = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let length = n.iter().map(|v| v * v).sum::<f64>().sqrt();
    if !(length > 0.0 && length.is_finite()) {
        return Err(GenerateError::InvalidSurface);
    }
    Ok(n.map(|v| v / length))
}
