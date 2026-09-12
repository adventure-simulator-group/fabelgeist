//! Long lamed thigh defenses suspended at the waist, with an open rear.
use crate::{
    DesignError, GarmentArmorDesign, GarmentPlateShape, GenerateError, Millimeters, PartMesh,
    Permille,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WrappedTassetDesign {
    /// Inner and outer returns, each as a fraction of a half circumference.
    pub inner_wrap: Permille,
    pub outer_wrap: Permille,
    /// Minimum distance of either inner edge from the body's medial plane.
    pub inner_gap: Millimeters,
    /// Rise of the upper suspension edge per metre from the medial plane.
    pub upper_edge_slope: Permille,
    /// Fraction of hip-to-knee distance reached by the lower hem.
    pub knee_reach: Permille,
    pub inner_cutaway: Millimeters,
    pub hem_rounding: Millimeters,
    /// Course at which a removable lower section begins; zero is unsplit.
    pub section_break: u8,
    pub section_gap: Millimeters,
}

impl Default for WrappedTassetDesign {
    fn default() -> Self {
        Self {
            inner_wrap: Permille(280),
            outer_wrap: Permille(620),
            inner_gap: Millimeters(10),
            upper_edge_slope: Permille(0),
            knee_reach: Permille(880),
            inner_cutaway: Millimeters(18),
            hem_rounding: Millimeters(18),
            section_break: 6,
            section_gap: Millimeters(3),
        }
    }
}

impl WrappedTassetDesign {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(150..=450).contains(&self.inner_wrap.0)
            || !(450..=750).contains(&self.outer_wrap.0)
            || !(650..=1100).contains(&self.knee_reach.0)
            || self.upper_edge_slope.0 > 500
            || self.inner_gap.0 > 80
            || self.inner_cutaway.0 > 50
            || self.hem_rounding.0 > 35
            || self.section_gap.0 > 8
            || self.section_break > 11
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TassetSide {
    Left,
    Right,
}

/// Physical reference heights before the plate's shaped boundary is applied.
#[derive(Clone, Copy, Debug)]
pub struct TassetSpan {
    hem: f32,
    suspension: f32,
}

impl TassetSpan {
    pub fn new(hem: f32, suspension: f32) -> Result<Self, GenerateError> {
        if !hem.is_finite() || !suspension.is_finite() || suspension <= hem {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(Self { hem, suspension })
    }

    pub fn axial(self, height: f32) -> f32 {
        (height - self.hem) / (self.suspension - self.hem)
    }

    fn height(self, axial: f32) -> f32 {
        self.hem + (self.suspension - self.hem) * axial
    }
}

/// Build one side on an anatomical carrier. The callback takes circumferential
/// angle and final physical height in metres, and returns the X/Z section point.
/// It must cover the shaped boundary, which can extend past the nominal span.
pub fn generate_wrapped_tasset(
    design: &GarmentArmorDesign,
    side: TassetSide,
    span: TassetSpan,
    point: impl Fn(f32, f32) -> [f32; 2],
) -> Result<PartMesh, GenerateError> {
    design.validate()?;
    let GarmentPlateShape::WrappedTassets(shape) = design.plate_shape else {
        return Err(GenerateError::InvalidSurface);
    };
    let count = usize::from(design.lame_count);
    let gauge = design.wall_thickness.metres();
    let sign = match side {
        TassetSide::Left => 1.0,
        TassetSide::Right => -1.0,
    };
    let outer = shape.outer_wrap.unit() * std::f32::consts::PI;
    let mut mesh = PartMesh::new();
    const OVERLAP_FRACTION: f32 = 0.14;
    const COURSE_ROWS: usize = 8;
    const LAP_LIFT_GAUGES: f32 = 2.5;
    for course in 0..count {
        let top = 1.0 - course as f32 / count as f32;
        let bottom = (1.0 - (course as f32 + 1.0 + OVERLAP_FRACTION) / count as f32).max(0.0);
        let invalid_carrier = std::cell::Cell::new(false);
        let patch = crate::plate_patch::fluted_patch(
            COURSE_ROWS,
            false,
            gauge,
            design.fluting.as_ref(),
            [bottom, top],
            |_, axial| gauge * LAP_LIFT_GAUGES * (top - axial) / (top - bottom),
            |u, v| {
                let axial = bottom + (top - bottom) * v;
                let across = match side {
                    TassetSide::Left => u,
                    TassetSide::Right => 1.0 - u,
                };
                let shaped = |angle, across| {
                    shaped_point(&shape, span, course, axial, across, angle, &point)
                };
                let inner =
                    inner_angle(&shape, sign, &|angle| shaped(angle, 0.0)).unwrap_or_else(|| {
                        invalid_carrier.set(true);
                        0.0
                    });
                let angle = sign * (inner + (outer - inner) * across);
                shaped(angle, across).unwrap_or_else(|| {
                    invalid_carrier.set(true);
                    [0.0; 3]
                })
            },
        );
        if invalid_carrier.get() {
            return Err(GenerateError::InvalidSurface);
        }
        mesh.append(patch?);
    }
    Ok(mesh)
}

/// Trim the front domain independently of the carrier's bulk shape. The front
/// half of the anatomical carrier must increase laterally with this angle.
fn inner_angle(
    shape: &WrappedTassetDesign,
    sign: f32,
    point: &impl Fn(f32) -> Option<[f32; 3]>,
) -> Option<f32> {
    const ANGLE_SOLVER_STEPS: usize = 24;
    let mut bounds = [
        -shape.inner_wrap.unit() * std::f32::consts::PI,
        (shape.outer_wrap.unit() * std::f32::consts::PI).min(std::f32::consts::FRAC_PI_2),
    ];
    let gap = shape.inner_gap.metres();
    let lateral_limit = sign * point(sign * bounds[1])?[0];
    if !lateral_limit.is_finite() || lateral_limit <= gap {
        return None;
    }
    if sign * point(sign * bounds[0])?[0] >= gap {
        return Some(bounds[0]);
    }
    for _ in 0..ANGLE_SOLVER_STEPS {
        let angle = (bounds[0] + bounds[1]) * 0.5;
        if sign * point(sign * angle)?[0] < gap {
            bounds[0] = angle;
        } else {
            bounds[1] = angle;
        }
    }
    Some(bounds[1])
}

/// Solve the sloped suspension against the section at its resulting height.
/// Rounding and cutaways select height before any anatomical radius is queried.
fn shaped_point(
    shape: &WrappedTassetDesign,
    span: TassetSpan,
    course: usize,
    axial: f32,
    across: f32,
    angle: f32,
    point: &impl Fn(f32, f32) -> [f32; 2],
) -> Option<[f32; 3]> {
    let mut base = span.height(axial)
        - (1.0 - across).powi(10) * shape.inner_cutaway.metres() * axial.powi(3)
        + shape.hem_rounding.metres() * (2.0 * across - 1.0).powi(8) * (1.0 - axial).powi(8);
    if shape.section_break > 0 && course >= usize::from(shape.section_break) {
        base -= shape.section_gap.metres();
    }
    let slope = shape.upper_edge_slope.unit() * axial.powi(4);
    let residual = |height| height - base - slope * point(angle, height)[0].abs();
    let mut bounds = [base, base];
    if slope > 0.0 {
        const BRACKET_EXPANSIONS: usize = 4;
        const HEIGHT_SOLVER_STEPS: usize = 24;
        let mut extent = span.suspension - span.hem;
        for _ in 0..BRACKET_EXPANSIONS {
            bounds[1] = base + extent;
            if residual(bounds[1]) >= 0.0 {
                break;
            }
            extent *= 2.0;
        }
        if !residual(bounds[1]).is_finite() || residual(bounds[1]) < 0.0 {
            return None;
        }
        for _ in 0..HEIGHT_SOLVER_STEPS {
            let height = (bounds[0] + bounds[1]) * 0.5;
            let error = residual(height);
            if !error.is_finite() {
                return None;
            }
            if error < 0.0 {
                bounds[0] = height;
            } else {
                bounds[1] = height;
            }
        }
    }
    let height = bounds[1];
    let [x, z] = point(angle, height);
    [x, height, z]
        .iter()
        .all(|v| v.is_finite())
        .then_some([x, height, z])
}
