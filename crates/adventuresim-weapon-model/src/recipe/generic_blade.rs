//! Continuous outline and terminal section of an asymmetric forged blade.
use super::*;
use std::f64::consts::PI;

/// Full thickness at the cutting edge before an authored point begins.
pub(crate) const BLADE_EDGE_THICKNESS: f64 = 0.0006;
/// Fraction of root thickness removed by the body taper at its distal end.
const DISTAL_THICKNESS_REDUCTION: f64 = 0.65;

impl BladeParameters {
    pub(crate) fn heel_center(&self) -> [f64; 3] {
        [
            self.single_edge.map_or(0.0, Ratio::get) * self.width.get() / 2.0,
            0.0,
            0.0,
        ]
    }
    pub(crate) fn body_half_width(&self, y: f64) -> f64 {
        let t = (y / self.length.get()).clamp(0.0, 1.0);
        let tip = self.tip_width.map_or(0.025, Ratio::get);
        self.width.get() / 2.0
            * (tip + (1.0 - tip) * (1.0 - t).powf(self.taper.map_or(1.25, Ratio::get)))
            * (1.0 + self.belly.map_or(0.0, Ratio::get) * (PI * t).sin())
    }

    pub(crate) fn point_start(&self) -> Option<f64> {
        self.point
            .as_ref()
            .map(|p| p.start.get() * self.length.get())
    }

    pub(crate) fn point_curve(&self) -> Result<Option<PointCurve>, String> {
        let Some(point) = &self.point else {
            return Ok(None);
        };
        let y = self.point_start().unwrap();
        let t = point.start.get();
        let tip = self.tip_width.map_or(0.025, Ratio::get);
        let taper = self.taper.map_or(1.25, Ratio::get);
        let belly = self.belly.map_or(0.0, Ratio::get);
        let (sin, cos) = (PI * t).sin_cos();
        let slope = self.width.get() / (2.0 * self.length.get())
            * (-(1.0 - tip) * taper * (1.0 - t).powf(taper - 1.0) * (1.0 + belly * sin)
                + (tip + (1.0 - tip) * (1.0 - t).powf(taper)) * belly * PI * cos);
        PointCurve::new(
            y,
            self.length.get(),
            self.body_half_width(y),
            slope,
            point.roundness.get(),
        )
        .map(Some)
    }

    /// Half-width, full ridge thickness, and full edge thickness. The same
    /// envelope closes every transverse landmark at the exact authored tip.
    pub(crate) fn section_dimensions(&self, y: f64, point: Option<&PointCurve>) -> [f64; 3] {
        let t = (y / self.length.get()).clamp(0.0, 1.0);
        let mut width = self.body_half_width(y);
        let mut envelope = 1.0;
        if let (Some(start), Some(point)) = (self.point_start(), point)
            && y >= start
        {
            width = point.width_at(y);
            let q = width / self.body_half_width(start);
            envelope = q * (2.0 - q);
        }
        if self.section == Some(ForgedBladeSection::Diamond) {
            return [
                width,
                2.0 * width * self.thickness.get() / self.width.get(),
                0.0,
            ];
        }
        [
            width,
            self.thickness.get() * (1.0 - DISTAL_THICKNESS_REDUCTION * t) * envelope,
            BLADE_EDGE_THICKNESS * envelope,
        ]
    }

    pub(crate) fn validate_form(&self) -> Result<(), RecipeError> {
        let tip = self.tip_width.map_or(0.025, Ratio::get);
        let section_valid = if self.section == Some(ForgedBladeSection::Diamond) {
            self.single_edge.map_or(0.0, Ratio::get) == 0.0
        } else {
            self.thickness.get() * (1.0 - DISTAL_THICKNESS_REDUCTION) >= BLADE_EDGE_THICKNESS
        };
        let valid = self.taper.map_or(1.25, Ratio::get) > 0.0
            && section_valid
            && (-1.0..=1.0).contains(&self.single_edge.map_or(0.0, Ratio::get))
            && (0.0..=1.0).contains(&tip)
            && (tip > 0.0 || self.point.is_some())
            && self.belly.map_or(0.0, Ratio::get) > -1.0;
        if !valid
            || self.point.as_ref().is_some_and(|p| {
                !(p.start.get() > 0.0
                    && p.start.get() < 1.0
                    && (0.0..=1.0).contains(&p.roundness.get()))
            })
        {
            return Err(RecipeError::Proportion);
        }
        self.point_curve().map_err(|_| RecipeError::Proportion)?;
        Ok(())
    }
}
