//! Formed overlapping faceplates on a common removable buffe carrier.
use super::{CHART_HEIGHT_MM, Carrier, FACE_ROWS, pierced_patch};
use crate::{DesignError, GenerateError, Millimeters, PartMesh, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffeCourses {
    pub plate_count: u8,
    /// First seam's height, measured upward from chin to sight edge.
    pub lower_boundary: Permille,
    /// Second seam's height; used when there are three faceplates.
    pub upper_boundary: Permille,
    /// Physical vertical overlap at each descending edge.
    pub overlap: Millimeters,
    /// Additional air between courses, beyond twice the plate gauge.
    pub lap_clearance: Millimeters,
    /// Downward median point at the internal seams; outer edges stay fixed.
    pub boundary_drop: Millimeters,
}

impl Default for BuffeCourses {
    fn default() -> Self {
        Self {
            plate_count: 3,
            lower_boundary: Permille(350),
            upper_boundary: Permille(700),
            overlap: Millimeters(5),
            lap_clearance: Millimeters(1),
            boundary_drop: Millimeters(12),
        }
    }
}

impl BuffeCourses {
    pub fn validate(&self) -> Result<(), DesignError> {
        if !(2..=3).contains(&self.plate_count)
            || !(200..=700).contains(&self.lower_boundary.0)
            || !(450..=850).contains(&self.upper_boundary.0)
            || (self.plate_count == 3 && self.upper_boundary.0 < self.lower_boundary.0 + 180)
            || !(2..=12).contains(&self.overlap.0)
            || self.lap_clearance.0 > 4
            || self.boundary_drop.0 > 25
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }

    pub(super) fn generate(
        &self,
        carrier: &Carrier<'_>,
        gauge: f32,
    ) -> Result<PartMesh, GenerateError> {
        let boundaries = [
            0.0,
            self.lower_boundary.unit(),
            self.upper_boundary.unit(),
            1.0,
        ];
        let mut mesh = PartMesh::new();
        for index in 0..self.plate_count {
            let span = [
                boundaries[usize::from(index)],
                if index + 1 == self.plate_count {
                    1.0
                } else {
                    boundaries[usize::from(index + 1)]
                },
            ];
            let course = Course {
                design: self,
                carrier,
                span,
                index,
                gauge,
            };
            // Validate the physical trim before building closed surfaces. Very
            // short carriers cannot contain the requested lap and chevron.
            for column in 0..=80 {
                let [low, high] = course.extents(column as f32 / 80.0);
                if !low.is_finite()
                    || !high.is_finite()
                    || low < 0.0
                    || high > 1.0
                    || high - low <= gauge / carrier.half_height
                {
                    return Err(GenerateError::InvalidSurface);
                }
            }
            let upper = index + 1 == self.plate_count;
            let plate = if let Some(breaths) = carrier
                .design
                .breaths
                .filter(|b| upper && b.count_per_row > 0)
            {
                pierced_patch(
                    &breaths,
                    [
                        0.0,
                        CHART_HEIGHT_MM * f64::from(1.0 - span[0]) + f64::from(self.overlap.0),
                    ],
                    gauge,
                    |u, v| course.point(u, v),
                )?
            } else {
                crate::plate_patch::fluted_patch(
                    FACE_ROWS,
                    false,
                    gauge,
                    None,
                    span,
                    |_, _| 0.0,
                    |u, v| course.point(u, v),
                )?
            };
            mesh.append(plate);
        }
        Ok(mesh)
    }
}

struct Course<'a, 'b> {
    design: &'a BuffeCourses,
    carrier: &'a Carrier<'b>,
    span: [f32; 2],
    index: u8,
    gauge: f32,
}

impl Course<'_, '_> {
    fn extents(&self, u: f32) -> [f32; 2] {
        let angle = (2.0 * u - 1.0) * self.carrier.design.side_wrap.radians();
        let height = self.carrier.point(u, 1.0)[1] - self.carrier.point(u, 0.0)[1];
        let chevron = if angle.cos() > 0.0 {
            1.0 - angle.sin().abs()
        } else {
            0.0
        };
        let drop = self.design.boundary_drop.metres() * chevron / height;
        [
            self.span[0]
                - if self.index == 0 {
                    0.0
                } else {
                    drop + self.design.overlap.metres() / height
                },
            self.span[1] - if self.span[1] == 1.0 { 0.0 } else { drop },
        ]
    }

    fn point(&self, u: f32, v: f32) -> [f32; 3] {
        let [low, high] = self.extents(u);
        let mut point = self.carrier.point(u, low + (high - low) * v);
        let angle = (2.0 * u - 1.0) * self.carrier.design.side_wrap.radians();
        // Form the lower edge over its neighbor while retaining the fitted
        // sight and exposed top edges. Adding more courses must not keep
        // inflating the entire face defense away from the wearer.
        let lap = if self.index == 0 {
            0.0
        } else {
            (2.0 * self.gauge + self.design.lap_clearance.metres()) * (1.0 - v)
        };
        point[0] += lap * angle.sin();
        point[2] += lap * angle.cos();
        point
    }
}

/// A tangent V-section encloses the curved face rather than cutting into it.
/// Its apex uses the existing ridge height; sharpness blends toward formed flats.
pub(super) fn formed_ridge(angle: f32, depth: f32, height: f32, sharpness: f32) -> f32 {
    let front = angle.cos().max(0.0);
    let smooth = height * front.powi(4);
    let apex = depth + height;
    let angular = if height > 0.0 && front > depth / apex {
        let slope = ((apex / depth).powi(2) - 1.0).sqrt();
        (apex - depth * slope * angle.sin().abs() - depth * front).max(0.0)
    } else {
        0.0
    };
    smooth * (1.0 - sharpness) + angular * sharpness
}
