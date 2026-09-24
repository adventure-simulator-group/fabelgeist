//! Continuous bounds for disjoint groove lanes and local opposing walls.
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
        0.0,
        p.length,
        p.ricasso,
        p.point_start().unwrap_or(p.length),
    ];
    for g in &f.grooves {
        features.extend([
            g.start.get(),
            g.start.get() + g.entry_length.get(),
            g.end.get() - g.exit_length.get(),
            g.end.get(),
        ]);
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
/// A pointwise upper bound on a groove's trapezoidal material removal.
struct Cut {
    center: f64,
    mouth: f64,
    floor: f64,
    depth: f64,
}
impl Cut {
    fn at(&self, x: f64) -> f64 {
        let distance = (x - self.center).abs();
        if distance >= self.mouth {
            0.0
        } else if distance <= self.floor {
            self.depth
        } else {
            self.depth * (self.mouth - distance) / (self.mouth - self.floor)
        }
    }
    fn corners(&self) -> [f64; 4] {
        [
            self.center - self.mouth,
            self.center - self.floor,
            self.center + self.floor,
            self.center + self.mouth,
        ]
    }
}
impl Clearance<'_, '_> {
    fn inside(&self, envelopes: &[f64], width: f64, half_depth: f64) -> bool {
        if envelopes.iter().all(|q| *q == 0.0) {
            return true;
        }
        let flat = width * (1.0 - self.f.bevel_width_ratio.get());
        let slack = 64.0 * f64::EPSILON;
        let cuts: Vec<_> = self
            .f
            .grooves
            .iter()
            .zip(envelopes)
            .map(|(g, q)| Cut {
                center: g.lateral_position.get(),
                mouth: g.mouth_width.get() * q / (2.0 * flat),
                floor: g.mouth_width.get() * q * g.floor_width_ratio.get() / (2.0 * flat),
                depth: g.depth.get() * q * q,
            })
            .collect();
        for (i, (g, c)) in self.f.grooves.iter().zip(&cuts).enumerate() {
            if c.mouth * (1.0 + slack) >= (1.0 - c.center.abs()) * (1.0 - slack) {
                return false;
            }
            let faces = if g.faces == FullerFaces::Both {
                2.0
            } else {
                1.0
            };
            if faces * c.depth * (1.0 + slack) >= 2.0 * half_depth * (1.0 - slack) {
                return false;
            }
            for (h, d) in self.f.grooves.iter().zip(&cuts).skip(i + 1) {
                let same = [true, false]
                    .into_iter()
                    .any(|front| g.on_face(front) && h.on_face(front));
                if same
                    && c.mouth > 0.0
                    && d.mouth > 0.0
                    && (c.mouth + d.mouth) * (1.0 + slack)
                        >= (c.center - d.center).abs() * (1.0 - slack)
                {
                    return false;
                }
                let opposite = [true, false]
                    .into_iter()
                    .any(|front| g.on_face(front) && h.on_face(!front));
                if opposite
                    && c.corners().into_iter().chain(d.corners()).any(|x| {
                        (c.at(x) + d.at(x)) * (1.0 + slack) >= 2.0 * half_depth * (1.0 - slack)
                    })
                {
                    return false;
                }
            }
        }
        true
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
        if b != self.p.length || a < start {
            return false;
        }
        let delta = b - a;
        let mut envelopes = Vec::new();
        for g in &self.f.grooves {
            if g.end.get() <= a {
                envelopes.push(0.0);
            } else if g.end.get() == b && a >= b - g.exit_length.get() {
                // S(u)<=10u^3; division by the point's linear width bound
                // leaves positive powers of delta for both mouth and depth.
                envelopes.push(10.0 * (delta / g.exit_length.get()).powi(3));
            } else {
                return false;
            }
        }
        let width = point.minimum_width_slope() * delta;
        let depth = self.p.body(b)[1] * width / self.p.body(start)[0];
        self.inside(&envelopes, width, depth)
    }
    fn interval(&mut self, a: f64, b: f64, level: usize) -> Result<(), RecipeError> {
        if self.remaining == 0 {
            return Err(RecipeError::Budget);
        }
        self.remaining -= 1;
        if self.tip_interval(a, b) {
            return Ok(());
        }
        let envelopes: Vec<_> = self
            .f
            .grooves
            .iter()
            .map(|g| {
                let peak = ((g.start.get() + g.entry_length.get() + g.end.get()
                    - g.exit_length.get())
                    / 2.0)
                    .clamp(a, b);
                g.envelope(peak)
            })
            .collect();
        let [w, d] = self.lower_dimensions(a, b);
        if self.inside(&envelopes, w, d) {
            return Ok(());
        }
        let mid = (a + b) / 2.0;
        let envelopes: Vec<_> = self.f.grooves.iter().map(|g| g.envelope(mid)).collect();
        let [w, d] = self.p.dimensions(mid, self.point);
        if !self.inside(&envelopes, w, d) {
            return Err(RecipeError::Proportion);
        }
        if level >= MAX_CLEARANCE_DEPTH {
            return Err(RecipeError::Budget);
        }
        self.interval(a, mid, level + 1)?;
        self.interval(mid, b, level + 1)
    }
}
