//! Shared continuous blade plan and manufacturing clearance bounds.
use super::*;
use std::f64::consts::PI;

pub(crate) struct BladeProfile<'a> {
    pub(crate) length: f64,
    pub(crate) width: f64,
    pub(crate) thickness: f64,
    pub(crate) taper: f64,
    pub(crate) ricasso: f64,
    pub(crate) plan: BladePlan,
    pub(crate) belly: f64,
    pub(crate) curvature: f64,
    pub(crate) single_edge: f64,
    pub(crate) section: BladeCrossSection,
    pub(crate) fuller: Option<&'a FullerParameters>,
    pub(crate) point: Option<&'a BladePoint>,
}
impl<'a> From<&'a LoftedBladeParameters> for BladeProfile<'a> {
    fn from(p: &'a LoftedBladeParameters) -> Self {
        Self {
            length: p.length.get(),
            width: p.width.get(),
            thickness: p.thickness.get(),
            taper: p.taper.get(),
            ricasso: p.ricasso.get(),
            plan: p.plan,
            belly: p.belly.get(),
            curvature: p.curvature.get(),
            single_edge: p.single_edge.get(),
            section: p.section,
            fuller: p.fuller.as_ref(),
            point: p.point.as_ref(),
        }
    }
}
impl<'a> From<&'a SectionBladeParameters> for BladeProfile<'a> {
    fn from(p: &'a SectionBladeParameters) -> Self {
        Self {
            length: p.length.get(),
            width: p.width.get(),
            thickness: p.thickness.get(),
            taper: p.taper.map_or(0.8, Ratio::get),
            ricasso: 0.0,
            plan: BladePlan::Straight,
            belly: 0.0,
            curvature: 0.0,
            single_edge: 0.0,
            section: p.section.unwrap_or(BladeCrossSection::Diamond),
            fuller: p.fuller.as_ref(),
            point: p.point.as_ref(),
        }
    }
}
impl BladeProfile<'_> {
    fn progress(&self, y: f64) -> f64 {
        ((y - self.ricasso) / (self.length - self.ricasso)).clamp(0.0, 1.0)
    }
    pub(crate) fn body(&self, y: f64) -> [f64; 2] {
        let t = self.progress(y);
        let sine = (PI * t).sin();
        let plan = match self.plan {
            BladePlan::Straight => 1.0,
            BladePlan::Leaf => 0.78 + 0.22 * sine,
            BladePlan::Cleaver => 0.9 + 0.24 * sine,
        };
        let w = self.width / 2.0
            * plan
            * (0.025 + 0.975 * (1.0 - t).powf(self.taper))
            * (1.0 + self.belly * sine);
        [w, self.thickness / 2.0 * (1.0 - 0.72 * t)]
    }
    pub(crate) fn point_start(&self) -> Option<f64> {
        self.point
            .map(|p| self.ricasso + p.start.get() * (self.length - self.ricasso))
    }
    pub(crate) fn point_curve(&self) -> Result<Option<PointCurve>, String> {
        let Some(p) = self.point else { return Ok(None) };
        let y = self.point_start().unwrap();
        let t = self.progress(y);
        let (sine, cosine) = (PI * t).sin_cos();
        let (a, b) = match self.plan {
            BladePlan::Straight => (1.0, 0.0),
            BladePlan::Leaf => (0.78, 0.22),
            BladePlan::Cleaver => (0.9, 0.24),
        };
        let plan = a + b * sine;
        let taper = 0.025 + 0.975 * (1.0 - t).powf(self.taper);
        let belly = 1.0 + self.belly * sine;
        let derivative = self.width / 2.0
            * (b * PI * cosine * taper * belly
                - plan * 0.975 * self.taper * (1.0 - t).powf(self.taper - 1.0) * belly
                + plan * taper * self.belly * PI * cosine)
            / (self.length - self.ricasso);
        PointCurve::new(
            y,
            self.length,
            self.body(y)[0],
            derivative,
            p.roundness.get(),
        )
        .map(Some)
    }
    pub(crate) fn dimensions(&self, y: f64, point: Option<&PointCurve>) -> [f64; 2] {
        let [w, d] = self.body(y);
        if let (Some(start), Some(point)) = (self.point_start(), point)
            && y >= start
        {
            let width = point.width_at(y);
            let q = width / self.body(start)[0];
            return [width, d * (2.0 * q - q * q)];
        }
        [w, d]
    }
    pub(crate) fn center(&self, y: f64, width: f64) -> f64 {
        let t = self.progress(y);
        self.curvature * t * t + width * self.single_edge * 0.35
    }
    pub(crate) fn validate(&self) -> Result<(), RecipeError> {
        let require = |b| {
            if b {
                Ok(())
            } else {
                Err(RecipeError::Proportion)
            }
        };
        require((self.section == BladeCrossSection::Recessed) == self.fuller.is_some())?;
        if let Some(p) = self.point {
            require(
                p.start.get() > 0.0
                    && p.start.get() < 1.0
                    && (0.0..=1.0).contains(&p.roundness.get()),
            )?;
        }
        let point = self.point_curve().map_err(|_| RecipeError::Proportion)?;
        let Some(f) = self.fuller else { return Ok(()) };
        require(
            (0.0..1.0).contains(&f.bevel_width_ratio.get()) && f.bevel_width_ratio.get() > 0.0,
        )?;
        for groove in &f.grooves {
            require(
                groove.mouth_width.get() > 0.0
                    && groove.depth.get() > 0.0
                    && groove.floor_width_ratio.get() > 0.0
                    && groove.floor_width_ratio.get() < 1.0
                    && groove.lateral_position.get().abs() < 1.0
                    && groove.start.get() >= self.ricasso
                    && groove.end.get() <= self.length
                    && groove.start.get() < groove.end.get()
                    && groove.entry_length.get() > 0.0
                    && groove.exit_length.get() > 0.0
                    && groove.entry_length.get() + groove.exit_length.get()
                        <= groove.end.get() - groove.start.get(),
            )?;
        }
        super::blade_clearance::check(self, f, point.as_ref())
    }
}
