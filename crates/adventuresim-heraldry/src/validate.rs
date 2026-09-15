//! Bounded documents and advisory heraldic checks.
use crate::{Error, document::*};
const MAX_COMPOSITION_DEPTH: usize = 5;
const MAX_CHARGES: usize = 128;
const MAX_ORDINARIES: usize = 64;
fn range(label: &str, value: f32, low: f32, high: f32) -> Result<(), Error> {
    if value.is_finite() && (low..=high).contains(&value) {
        Ok(())
    } else {
        Err(Error::Invalid(format!("{label} must be in {low}..={high}")))
    }
}
impl Document {
    pub fn validate(&self) -> Result<(), Error> {
        if self.name.len() > 160 {
            return Err(Error::Invalid("name exceeds 160 bytes".into()));
        }
        self.arms.check(0, &mut [0, 0])?;
        self.drawing.check()?;
        self.surface.check()?;
        range("view yaw", self.view.yaw.0, -180.0, 180.0)?;
        range("view pitch", self.view.pitch.0, -80.0, 80.0)?;
        range("light angle", self.view.light.0, -180.0, 180.0)?;
        range("exposure", self.view.exposure, -3.0, 3.0)?;
        range("zoom", self.view.zoom.0, 0.4, 3.0)
    }
    /// Advice never mutates or blocks a valid document.
    pub fn advice(&self) -> Vec<String> {
        let mut advice = Vec::new();
        self.arms.advise(&mut advice);
        advice
    }
}
impl ArmsDesign {
    fn check(&self, depth: usize, counts: &mut [usize; 2]) -> Result<(), Error> {
        if depth > MAX_COMPOSITION_DEPTH {
            return Err(Error::Invalid(
                "composition nesting exceeds five levels".into(),
            ));
        }
        counts[0] += self.charges.len();
        counts[1] += self.ordinaries.len();
        if counts[0] > MAX_CHARGES || counts[1] > MAX_ORDINARIES {
            return Err(Error::Invalid(
                "composition exceeds charge or ordinary limit".into(),
            ));
        }
        match &self.field {
            Field::Patterned { repeats, .. } if !(2..=16).contains(repeats) => {
                return Err(Error::Invalid("pattern repeats must be 2..=16".into()));
            }
            Field::Quarterly { quarters } => {
                for quarter in quarters.iter() {
                    quarter.check(depth + 1, counts)?;
                }
            }
            _ => (),
        }
        for ordinary in &self.ordinaries {
            range("ordinary width", ordinary.width.0, 0.03, 0.5)?;
        }
        for charge in &self.charges {
            for c in charge.center {
                range("charge center", c, 0.0, 1.0)?;
            }
            for s in charge.size {
                range("charge size", s.0, 0.03, 1.5)?;
            }
            range("charge rotation", charge.rotation.0, -180.0, 180.0)?;
            if let ChargeKind::Star { points } = charge.shape
                && !(3..=16).contains(&points)
            {
                return Err(Error::Invalid("star points must be 3..=16".into()));
            }
            if let Coloring::Counterchanged { tinctures } = charge.color
                && tinctures[0] == tinctures[1]
            {
                return Err(Error::Invalid(
                    "counterchanging needs distinct tinctures".into(),
                ));
            }
        }
        if let Some(inset) = &self.inescutcheon {
            inset.check(depth + 1, counts)?;
        }
        Ok(())
    }
    fn advise(&self, messages: &mut Vec<String>) {
        if let Field::Solid { tincture } = self.field {
            for c in &self.charges {
                if let Coloring::Solid { tincture: charge } = c.color
                    && charge.metal() == tincture.metal()
                {
                    messages.push("Charge and field belong to the same tincture class; check contrast and historical intent.".into());
                }
            }
        }
        for c in &self.charges {
            if (0..2).any(|axis| {
                c.center[axis] - c.size[axis].0 * 0.5 < 0.0
                    || c.center[axis] + c.size[axis].0 * 0.5 > 1.0
            }) {
                messages.push(
                    "A charge reaches beyond its field bounds; inspect the clipped contour.".into(),
                );
            }
        }
        if let Field::Quarterly { quarters } = &self.field {
            for q in quarters.iter() {
                q.advise(messages);
            }
        }
        if let Some(inset) = &self.inescutcheon {
            inset.advise(messages);
        }
    }
}
impl DrawingStyle {
    fn check(&self) -> Result<(), Error> {
        for (label, value) in [
            ("detail", self.detail),
            ("painted shadows", self.painted_modeling.shadows),
            ("painted highlights", self.painted_modeling.highlights),
            ("contour character", self.contour_character),
            ("asymmetry", self.asymmetry),
        ] {
            range(label, value.0, 0.0, 1.0)?;
        }
        range("stroke width", self.stroke_width.0, 0.001, 0.012)?;
        let e = &self.eagle;
        let l = &self.lion;
        for v in [
            e.body_width,
            e.wing_span,
            e.wing_lift,
            e.feather_length,
            e.neck_length,
            e.head_size,
            e.leg_spread,
            e.tail_spread,
            l.body_width,
            l.spine_arch,
            l.head_size,
            l.foreleg_reach,
            l.hindleg_spread,
            l.mane_fullness,
            l.tail_curl,
            l.paw_size,
        ] {
            range("animal proportion", v.0, 0.5, 1.5)?;
        }
        if !(6..=16).contains(&e.feather_count) {
            return Err(Error::Invalid("feather count must be 6..=16".into()));
        }
        Ok(())
    }
}
impl PaintedSurface {
    fn check(&self) -> Result<(), Error> {
        for v in [self.width, self.height] {
            range("surface dimension (mm)", v.0, 50.0, 2000.0)?;
        }
        range("support thickness (mm)", self.thickness.0, 3.0, 80.0)?;
        range("curvature (mm)", self.curvature.0, 0.0, self.width.0 * 0.25)?;
        range("shield shoulder", self.shoulder.0, 0.0, 0.3)?;
        range("shield point", self.point.0, 0.2, 0.8)?;
        for (name, v, max) in [
            ("ground", self.ground, 3.0),
            ("pigment", self.pigment, 0.3),
            ("substrate relief", self.substrate_relief, 2.0),
            ("brush relief", self.brush_relief, 0.2),
        ] {
            range(name, v.0, 0.0, max)?;
        }
        range("brush width", self.brush_width.0, 0.5, 40.0)?;
        range("brush angle", self.brush_angle.0, -180.0, 180.0)?;
        for paint in self.palette.0 {
            range("paint roughness", paint.appearance().roughness.0, 0.0, 1.0)?;
        }
        for v in [self.glaze, self.glaze_roughness] {
            range("surface response", v.0, 0.0, 1.0)?;
        }
        for finish in [self.gold, self.silver] {
            match finish {
                MetalFinish::WaterGilding { burnish } => range("burnishing", burnish.0, 0.0, 1.0)?,
                MetalFinish::MordantGilding { relief } => {
                    range("mordant relief (mm)", relief.0, 0.0, 0.1)?
                }
                MetalFinish::YellowGlazedSilver { depth } => {
                    range("yellow glaze depth", depth.0, 0.0, 1.0)?
                }
                MetalFinish::Pigment | MetalFinish::OilGilding => (),
            }
        }
        if matches!(self.silver, MetalFinish::YellowGlazedSilver { .. }) {
            return Err(Error::Invalid(
                "yellow-glazed silver represents Or, not Argent".into(),
            ));
        }
        Ok(())
    }
}
