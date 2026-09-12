use std::f32::consts::{FRAC_PI_2, PI};

use super::PauldronDesign;
use crate::{GenerateError, PartFrame, PartMesh, plate_patch::fluted_patch};

const MAIN_ROWS: usize = 24;
const LAME_ROWS: usize = 8;
const DELTOID_JOIN_M: f32 = 0.040;
const CROWN_BOW_M: f32 = 0.012;
const MAIN_RIM_RESERVE_M: f32 = 0.006;
const LAME_WIDTH_TAPER_M: f32 = 0.014;
const LAME_HEIGHT_TAPER_M: f32 = 0.010;
const LAME_OVERLAP_M: f32 = 0.004;
const NECK_LAME_OVERLAP: f32 = 0.02;
// Reserve lap clearance when the fitted carrier tilts away from the formed
// separation direction during identity morph interpolation.
const NECK_LAME_SPACING_GAUGES: f32 = 2.0;
const ARM_CROWN_SCALE: f32 = 1.1;
const ARM_LAME_SCALE: f32 = 1.04;
const WING_RETURN_TAPER: f32 = 0.08;
const MEDIAL_WIDTH_SCALE: f32 = 1.1;
const LAP_GAUGE_RESERVE: f32 = 1.25;
const MAIN_JOIN_GAUGE_RESERVE: f32 = 2.0;
const MINIMUM_ARM_CLEARANCE_M: f32 = 0.002;

pub(super) struct Saddle<'a> {
    design: &'a PauldronDesign,
    frame: &'a PartFrame,
    medial: [f32; 3],
    up: [f32; 3],
    front_sign: f32,
}

impl<'a> Saddle<'a> {
    pub(super) fn new(
        design: &'a PauldronDesign,
        frame: &'a PartFrame,
    ) -> Result<Self, GenerateError> {
        let axis = frame.axes[1];
        let horizontal_length = axis[0].hypot(axis[2]);
        if horizontal_length < 0.01 {
            return Err(GenerateError::InvalidSurface);
        }
        let world_medial = [
            axis[0] / horizontal_length,
            0.0,
            axis[2] / horizontal_length,
        ];
        Ok(Self {
            design,
            frame,
            medial: frame.axes.map(|a| dot(a, world_medial)),
            up: frame.axes.map(|a| a[1]),
            front_sign: frame.axes[0][2].signum(),
        })
    }

    fn width(&self) -> f32 {
        let wall = self.design.gauge.thickness.metres();
        let requested = self.frame.half_extents[0]
            + self.design.gauge.clearance.metres()
            + wall
            + self.design.arm_allowance.metres();
        requested
            .max(self.frame.half_extents[0] + wall + LAME_WIDTH_TAPER_M + MINIMUM_ARM_CLEARANCE_M)
    }

    fn height(&self) -> f32 {
        let envelope = self.frame.half_extents[2]
            + self.design.gauge.clearance.metres()
            + self.design.gauge.thickness.metres()
            + LAME_HEIGHT_TAPER_M;
        (self.frame.half_extents[2] * self.design.crown_height.unit()).max(envelope)
    }

    fn lap_step(&self, taper: f32) -> f32 {
        let d = self.design;
        let relief = d
            .fluting
            .as_ref()
            .map_or(0.0, |pattern| pattern.depth.metres());
        (taper / f32::from(d.lower_lames))
            .max(d.gauge.thickness.metres() * LAP_GAUGE_RESERVE + relief)
    }

    fn lap_reserve(&self, taper: f32) -> f32 {
        (self.lap_step(taper) - taper / f32::from(self.design.lower_lames))
            * f32::from(self.design.lower_lames - 1)
    }

    fn theta(&self, u: f32, v: f32) -> f32 {
        let d = self.design;
        let rear_return = d.outline.rear_return.radians();
        let front_return = d.outline.front_return.radians();
        let (rear, front) = if self.front_sign > 0.0 {
            (rear_return, front_return)
        } else {
            (front_return, rear_return)
        };
        let main_end = 1.0 - d.outline.upper_span.unit() + NECK_LAME_OVERLAP;
        let corner_span = main_end * d.outline.corner_rounding.unit();
        let corner = rounded_end(v, corner_span);
        let arm_wrap = d.outline.arm_wrap.radians();
        let rear = arm_wrap + (rear - arm_wrap) * corner;
        let front = arm_wrap + (front - arm_wrap) * corner;
        -rear + (rear + front) * u
    }

    fn wing_drop(&self, theta: f32, v: f32) -> f32 {
        let d = self.design;
        let anterior = theta * self.front_sign > 0.0;
        let wing = if anterior {
            d.outline.front_extension
        } else {
            d.outline.rear_extension
        };
        let edge_angle = if anterior {
            d.outline.front_return
        } else {
            d.outline.rear_return
        }
        .radians();
        let wing_start = d.outline.wing_start.radians();
        // Extension is the actual downward reach at the returned boundary.
        // Normalizing against PI made shorter, rounded returns suppress nearly
        // all of the requested wing depth.
        let wing_fraction = if edge_angle > wing_start {
            ((theta.abs() - wing_start) / (edge_angle - wing_start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let main_end = 1.0 - d.outline.upper_span.unit() + NECK_LAME_OVERLAP;
        let (position, rounding) = if anterior {
            (d.outline.front_wing_position, d.outline.front_wing_rounding)
        } else {
            (d.outline.rear_wing_position, d.outline.rear_wing_rounding)
        };
        wing.metres()
            * wing_fraction
            * wing_fraction
            * (3.0 - 2.0 * wing_fraction)
            * hanging_profile(v / main_end, position.unit(), rounding.unit())
    }

    pub(super) fn point(&self, u: f32, v: f32) -> [f32; 3] {
        let d = self.design;
        let theta = self.theta(u, v);
        let anterior = theta * self.front_sign > 0.0;
        let reach = if anterior {
            d.front_reach
        } else {
            d.rear_reach
        };
        let drop = if anterior { d.front_drop } else { d.rear_drop };
        let wing_drop = self.wing_drop(theta, v);
        let arm_wrap = d.outline.arm_wrap.radians();
        // Returned wings retain their medial reach. Turning sin² back toward
        // the arm folds the chart when chest clearance projects it in depth.
        let medial =
            d.neck_reach.metres() + reach.metres() * theta.abs().min(FRAC_PI_2).sin().powi(2);
        let join_gap = (d.gauge.thickness.metres() * MAIN_JOIN_GAUGE_RESERVE
            + d.fluting.as_ref().map_or(0.0, |f| f.depth.metres()))
        .max(MAIN_RIM_RESERVE_M);
        let width = self.width() + join_gap + self.lap_reserve(LAME_WIDTH_TAPER_M);
        let transverse = if theta.cos() >= 0.0 {
            theta.sin()
        } else {
            theta.sin().signum() * (1.0 - WING_RETURN_TAPER * (-theta.cos()))
        };
        let inner_x = width * transverse * MEDIAL_WIDTH_SCALE;
        let inner_z = self.height() * theta.cos() - drop.metres() * (-theta.cos()).max(0.0);
        let crown_height = (self.height() * ARM_CROWN_SCALE)
            .max(self.height() * ARM_LAME_SCALE + join_gap)
            + d.arm_allowance.metres()
            + self.lap_reserve(LAME_HEIGHT_TAPER_M);
        let mut outer = [
            width * theta.sin(),
            -DELTOID_JOIN_M,
            crown_height * theta.cos(),
        ];
        // Beyond the arm's open edge, continue the carrier downward without
        // curling its horizontal section back through the hanging wing.
        let return_shift = crown_height
            * (theta.abs().min(arm_wrap.min(FRAC_PI_2)).cos() - theta.cos())
            * self.medial[2];
        for (axis, point) in outer.iter_mut().enumerate() {
            *point += self.medial[axis] * return_shift;
        }
        std::array::from_fn(|i| {
            let inner =
                if i == 0 { inner_x } else { 0.0 } + self.medial[i] * medial + self.up[i] * inner_z;
            outer[i] * (1.0 - v)
                + inner * v
                + self.up[i] * (CROWN_BOW_M * (PI * v).sin() - wing_drop)
        })
    }
}

fn rounded_end(distance: f32, span: f32) -> f32 {
    let t = (distance / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hanging_profile(v: f32, position: f32, rounding: f32) -> f32 {
    let low_radius = position.min(1.0 - position) * (1.0 - rounding);
    let rising_span = position - low_radius;
    let falling_span = 1.0 - position - low_radius;
    rounded_end(v, rising_span) * rounded_end(1.0 - v, falling_span)
}

pub(crate) fn generate(d: &PauldronDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    super::PauldronCarrier::new(d, fit)?.mesh()
}

pub(super) fn plates(carrier: &super::PauldronCarrier) -> Result<PartMesh, GenerateError> {
    let d = &carrier.design;
    let saddle = Saddle::new(d, &carrier.frame)?;
    let gauge = d.gauge.thickness.metres();
    let neck_start = 1.0 - d.outline.upper_span.unit();
    let main_end = neck_start + NECK_LAME_OVERLAP;
    let mut result = fluted_patch(
        MAIN_ROWS,
        false,
        gauge,
        d.fluting.as_ref(),
        [0.0, main_end],
        |_, _| 0.0,
        |u, v| carrier.point(u, v * main_end, 0.0),
    )?;
    for index in 0..d.upper_lames {
        let step = d.outline.upper_span.unit() / f32::from(d.upper_lames);
        let start = neck_start + f32::from(index) * step;
        let end = (start + step + NECK_LAME_OVERLAP).min(1.0);
        result.append(fluted_patch(
            LAME_ROWS,
            false,
            gauge,
            d.fluting.as_ref(),
            [start, end],
            |_, _| 0.0,
            |u, v| {
                carrier.point(
                    u,
                    start + v * (end - start),
                    gauge * NECK_LAME_SPACING_GAUGES * f32::from(index + 1),
                )
            },
        )?);
    }
    for index in 0..d.lower_lames {
        let step = d.arm_length.metres() / f32::from(d.lower_lames);
        result.append(fluted_patch(
            LAME_ROWS,
            false,
            gauge,
            d.fluting.as_ref(),
            [
                1.0 - f32::from(index + 1) / f32::from(d.lower_lames),
                1.0 - f32::from(index) / f32::from(d.lower_lames)
                    + LAME_OVERLAP_M / d.arm_length.metres(),
            ],
            |_, _| 0.0,
            |u, v| {
                let theta = d.outline.arm_wrap.radians() * (2.0 * u - 1.0);
                let width = saddle.width() + saddle.lap_reserve(LAME_WIDTH_TAPER_M)
                    - saddle.lap_step(LAME_WIDTH_TAPER_M) * f32::from(index);
                let height = saddle.height() * ARM_LAME_SCALE
                    + d.arm_allowance.metres()
                    + saddle.lap_reserve(LAME_HEIGHT_TAPER_M)
                    - saddle.lap_step(LAME_HEIGHT_TAPER_M) * f32::from(index);
                [
                    theta.sin() * width,
                    -DELTOID_JOIN_M - (f32::from(index + 1) * step - v * (step + LAME_OVERLAP_M)),
                    theta.cos() * height,
                ]
            },
        )?);
    }
    Ok(result.transformed(&carrier.frame))
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
