//! Continuous section bounds, including simultaneous groove/point closure.
use super::*;
use std::f64::consts::PI;
const MAX_CLEARANCE_INTERVALS: usize = 4096;
const MAX_CLEARANCE_DEPTH: usize = 18;

pub(super) fn check(
    p: &BladeProfile<'_>,
    f: &FullerParameters,
    point: Option<&PointCurve>,
) -> Result<(), RecipeError> {
    let mut clearance = Clearance {
        p,
        f,
        point,
        remaining: MAX_CLEARANCE_INTERVALS,
    };
    let mut features = vec![
        f.start.get(),
        f.start.get() + f.entry_length.get(),
        f.end.get() - f.exit_length.get(),
        f.end.get(),
    ];
    for station in [p.ricasso, p.point_start().unwrap_or(p.length)] {
        if station > f.start.get() && station < f.end.get() {
            features.push(station);
        }
    }
    features.sort_by(f64::total_cmp);
    features.dedup();
    for pair in features.windows(2) {
        clearance.interval(pair[0], pair[1], 0)?;
    }
    Ok(())
}
struct Clearance<'a, 'b> {
    p: &'a BladeProfile<'b>,
    f: &'a FullerParameters,
    point: Option<&'a PointCurve>,
    remaining: usize,
}
impl Clearance<'_, '_> {
    fn inside(&self, mouth: f64, depth: f64, width: f64, half_depth: f64) -> bool {
        let faces = if self.f.faces == FullerFaces::Both {
            2.0
        } else {
            1.0
        };
        // Reserve relative rounding slack for the finite products, powers and
        // trigonometric bounds; undecided near-contact intervals are rejected.
        let slack = 64.0 * f64::EPSILON;
        mouth * (1.0 + slack) < 2.0 * width * (1.0 - self.f.bevel_width_ratio.get()) * (1.0 - slack)
            && faces * depth * (1.0 + slack) < 2.0 * half_depth * (1.0 - slack)
    }
    fn lower_dimensions(&self, a: f64, b: f64) -> [f64; 2] {
        if self.p.point_start().is_some_and(|start| a >= start) {
            return self.p.dimensions(b, self.point);
        }
        let ta = ((a - self.p.ricasso) / (self.p.length - self.p.ricasso)).clamp(0.0, 1.0);
        let tb = ((b - self.p.ricasso) / (self.p.length - self.p.ricasso)).clamp(0.0, 1.0);
        let sa = (PI * ta).sin();
        let sb = (PI * tb).sin();
        let sine_min = sa.min(sb);
        let sine_max = if ta <= 0.5 && tb >= 0.5 {
            1.0
        } else {
            sa.max(sb)
        };
        let plan = match self.p.plan {
            BladePlan::Straight => 1.0,
            BladePlan::Leaf => 0.78 + 0.22 * sine_min,
            BladePlan::Cleaver => 0.9 + 0.24 * sine_min,
        };
        let belly = 1.0
            + self.p.belly
                * if self.p.belly >= 0.0 {
                    sine_min
                } else {
                    sine_max
                };
        [
            self.p.width / 2.0 * plan * (0.025 + 0.975 * (1.0 - tb).powf(self.p.taper)) * belly,
            self.p.body(b)[1],
        ]
    }
    fn tip_interval(&self, a: f64, b: f64) -> bool {
        let Some(point) = self.point else {
            return false;
        };
        let Some(start) = self.p.point_start() else {
            return false;
        };
        if b != self.p.length || self.f.end.get() != b || a < start {
            return false;
        }
        let delta = b - a;
        let exit = delta / self.f.exit_length.get();
        // S(u)<=10u^3. The point's Bernstein coefficients give a positive
        // linear width bound; e(q)=2q-q²>=q gives the same depth lower bound.
        let width = point.minimum_width_slope() * delta;
        let depth = self.p.body(b)[1] * width / self.p.body(start)[0];
        self.inside(
            self.f.mouth_width.get() * 10.0 * exit.powi(3),
            self.f.depth.get() * 100.0 * exit.powi(6),
            width,
            depth,
        )
    }
    fn interval(&mut self, a: f64, b: f64, level: usize) -> Result<(), RecipeError> {
        if self.remaining == 0 {
            return Err(RecipeError::Budget);
        }
        self.remaining -= 1;
        if let Some(start) = self.p.point_start()
            && a < start
            && start < b
        {
            self.interval(a, start, level + 1)?;
            return self.interval(start, b, level + 1);
        }
        if self.tip_interval(a, b) {
            return Ok(());
        }
        let peak = ((self.f.start.get() + self.f.entry_length.get() + self.f.end.get()
            - self.f.exit_length.get())
            / 2.0)
            .clamp(a, b);
        let q = self.f.envelope(peak);
        let [w, d] = self.lower_dimensions(a, b);
        if self.inside(
            self.f.mouth_width.get() * q,
            self.f.depth.get() * q * q,
            w,
            d,
        ) {
            return Ok(());
        }
        let mid = (a + b) / 2.0;
        let q = self.f.envelope(mid);
        let [w, d] = self.p.dimensions(mid, self.point);
        if !self.inside(
            self.f.mouth_width.get() * q,
            self.f.depth.get() * q * q,
            w,
            d,
        ) {
            return Err(RecipeError::Proportion);
        }
        if level >= MAX_CLEARANCE_DEPTH {
            return Err(RecipeError::Budget);
        }
        self.interval(a, mid, level + 1)?;
        self.interval(mid, b, level + 1)
    }
}
