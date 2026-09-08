//! Experimental globally parameterized upper loft. Openings trim this field;
//! they do not set a different interpolation height for each meridian.

use crate::breastplate_qp::{LinearConstraint, QpOptions, solve_dense_qp};

const ANGULAR: usize = 8;
const HEIGHT: usize = 4;
const COUNT: usize = ANGULAR * HEIGHT;

#[path = "breastplate_elliptical_upper.rs"]
mod elliptical;

#[derive(Clone, Copy, Debug)]
struct FitPolicy {
    minimum_bending_radius: bool,
    dense_guides: bool,
    elliptical_depth: bool,
}

impl FitPolicy {
    fn diagnostic() -> Result<Self, String> {
        let option =
            |name: &str, default: &str, alternate: &str| match std::env::var(name).as_deref() {
                Err(std::env::VarError::NotPresent) => Ok(false),
                Ok(value) if value == default => Ok(false),
                Ok(value) if value == alternate => Ok(true),
                value => Err(format!(
                    "Invalid {name}: {value:?}; expected {default} or {alternate}"
                )),
            };
        Ok(Self {
            minimum_bending_radius: option(
                "BREASTPLATE_DIAGNOSTIC_UPPER_RADIUS_MODE",
                "grounded",
                "minimum-bending",
            )?,
            dense_guides: option(
                "BREASTPLATE_DIAGNOSTIC_UPPER_GUIDE_MODE",
                "landmarks",
                "dense",
            )?,
            elliptical_depth: option(
                "BREASTPLATE_DIAGNOSTIC_UPPER_DEPTH_MODE",
                "bernstein",
                "elliptical",
            )?,
        })
    }

    fn json(self) -> String {
        format!(
            "{{\"radius\":\"{}\",\"guides\":\"{}\",\"depth\":\"{}\"}}",
            if self.minimum_bending_radius {
                "minimum-bending"
            } else {
                "grounded"
            },
            if self.dense_guides {
                "dense"
            } else {
                "landmarks"
            },
            if self.elliptical_depth {
                "elliptical"
            } else {
                "bernstein"
            }
        )
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub(super) struct UpperGuide {
    pub xy: [f64; 2],
    pub target: f64,
    pub ceiling: Option<f64>,
    pub normal: Option<[f64; 3]>,
}

#[cfg(test)]
#[path = "breastplate_global_upper_tests.rs"]
mod tests;

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub(super) struct GlobalUpperLoft {
    #[cfg_attr(test, serde(default))]
    pub elliptical_depth: bool,
    pub seam_y: f64,
    pub span: f64,
    pub angle: f64,
    pub arc_scale: f64,
    pub seam: [f64; 3],
    pub seam_derivative: [f64; 3],
    pub radius_correction: [f64; 2],
    pub prior_depth_correction: f64,
    pub depth_controls: Vec<f64>,
}

fn bernstein(degree: usize, t: f64) -> Vec<f64> {
    (0..=degree)
        .map(|i| {
            let coefficient = (0..i).fold(1.0, |v, j| v * (degree - j) as f64 / (j + 1) as f64);
            coefficient * t.powi(i as i32) * (1.0 - t).powi((degree - i) as i32)
        })
        .collect()
}

fn basis(degree: usize, t: f64) -> [Vec<f64>; 3] {
    let b = bernstein(degree, t);
    let lower = bernstein(degree - 1, t);
    let second = bernstein(degree - 2, t);
    let at = |values: &[f64], index: isize| {
        usize::try_from(index)
            .ok()
            .and_then(|i| values.get(i))
            .copied()
            .unwrap_or(0.0)
    };
    let d = (0..=degree)
        .map(|i| degree as f64 * (at(&lower, i as isize - 1) - at(&lower, i as isize)))
        .collect();
    let dd = (0..=degree)
        .map(|i| {
            (degree * (degree - 1)) as f64
                * (at(&second, i as isize - 2) - 2.0 * at(&second, i as isize - 1)
                    + at(&second, i as isize))
        })
        .collect();
    [b, d, dd]
}

fn contains(outline: &[[f64; 2]], point: [f64; 2]) -> bool {
    let mut inside = false;
    for (a, b) in outline
        .iter()
        .zip(outline.iter().cycle().skip(1))
        .take(outline.len())
    {
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

impl GlobalUpperLoft {
    fn control_count(&self) -> usize {
        if self.elliptical_depth {
            elliptical::COUNT
        } else {
            COUNT
        }
    }
    pub fn radius(&self, y: f64) -> (f64, f64) {
        let v = (y - self.seam_y) / self.span;
        let [c2, c3] = self.radius_correction;
        (
            self.seam[0]
                + self.seam_derivative[0] * (y - self.seam_y)
                + c2 * v * v
                + c3 * v * v * v,
            self.seam_derivative[0] + (2.0 * c2 * v + 3.0 * c3 * v * v) / self.span,
        )
    }

    pub fn parameter(&self, xy: [f64; 2]) -> Result<f64, String> {
        let ratio = xy[0].abs() / self.radius(xy[1]).0;
        if ratio > self.angle.sin() + 1e-7 || !ratio.is_finite() {
            return Err(format!(
                "Upper trim exceeds fitted lateral radius: ratio={ratio} xy={xy:?} radius={} seam_y={} dy={} angle={} coefficients={:?}",
                self.radius(xy[1]).0,
                self.seam_y,
                xy[1] - self.seam_y,
                self.angle,
                self.radius_correction
            ));
        }
        // The existing lower surface stores its right angle in f32. Preserve
        // that exact endpoint rather than reflecting its f64 sine inverse to
        // the other side of pi/2 (the representations differ by ~4e-8).
        if ratio >= self.angle.sin() {
            return Ok(1.0);
        }
        Ok((ratio.asin() / self.angle).min(1.0))
    }

    fn rows(&self, u: f64, y: f64) -> [Vec<f64>; 6] {
        let v = (y - self.seam_y) / self.span;
        let height = basis(HEIGHT - 1, v);
        let theta = u * self.angle;
        let b = self.seam[1] + self.seam_derivative[1] * (y - self.seam_y);
        let mut rows = std::array::from_fn(|_| vec![0.0; self.control_count() + 1]);
        rows[0][0] = self.seam[2] + self.seam_derivative[2] * (y - self.seam_y) + b * theta.cos();
        rows[1][0] = -b * theta.sin() * self.angle;
        rows[2][0] = self.seam_derivative[2] + self.seam_derivative[1] * theta.cos();
        rows[3][0] = -b * theta.cos() * self.angle * self.angle;
        rows[5][0] = -self.seam_derivative[1] * theta.sin() * self.angle;
        if self.elliptical_depth {
            elliptical::fill_rows(&mut rows, &height, v, self.span, theta, self.angle);
            return rows;
        }
        let angular = basis(ANGULAR - 1, u);
        for i in 0..ANGULAR {
            for j in 0..HEIGHT {
                let h = v * v * height[0][j];
                let dh = (2.0 * v * height[0][j] + v * v * height[1][j]) / self.span;
                let ddh = (2.0 * height[0][j] + 4.0 * v * height[1][j] + v * v * height[2][j])
                    / self.span.powi(2);
                let k = 1 + i * HEIGHT + j;
                rows[0][k] = angular[0][i] * h;
                rows[1][k] = angular[1][i] * h;
                rows[2][k] = angular[0][i] * dh;
                rows[3][k] = angular[2][i] * h;
                rows[4][k] = angular[0][i] * ddh;
                rows[5][k] = angular[1][i] * dh;
            }
        }
        rows
    }

    pub fn evaluate(&self, u: f64, y: f64) -> ([f64; 3], [f64; 3]) {
        let rows = self.rows(u, y);
        let value = |row: &[f64]| {
            row[0]
                + row[1..]
                    .iter()
                    .zip(&self.depth_controls)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
        };
        let (a, da) = self.radius(y);
        let theta = u * self.angle;
        let xu = a * theta.cos() * self.angle;
        let xy = da * theta.sin();
        let zu = value(&rows[1]);
        let zy = value(&rows[2]);
        let normal = [-zu, zu * xy - xu * zy, xu];
        let norm = normal.iter().map(|n| n * n).sum::<f64>().sqrt();
        (
            [a * theta.sin(), y, value(&rows[0])],
            normal.map(|n| n / norm),
        )
    }

    fn legacy_radius_segment_bounds(&self, a: [f64; 2], b: [f64; 2]) -> Vec<LinearConstraint> {
        if a[1] <= self.seam_y + 1e-6 || b[1] <= self.seam_y + 1e-6 {
            return Vec::new();
        }
        let v0 = (a[1] - self.seam_y) / self.span;
        let v1 = (b[1] - self.seam_y) / self.span;
        let dv = v1 - v0;
        let dx = (b[0].abs() - a[0].abs()) / self.angle.sin();
        let endpoint = |v: f64, x: f64| {
            let row = [v * v, v * v * v];
            let derivative_row = [2.0 * v * dv, 3.0 * v * v * dv];
            let base = self.seam[0] + self.seam_derivative[0] * self.span * v
                - x.abs() / self.angle.sin()
                - 0.005 * v * v;
            let derivative_base = self.seam_derivative[0] * self.span * dv - dx - 0.010 * v * dv;
            (row, derivative_row, base, derivative_base)
        };
        let (r0, dr0, b0, db0) = endpoint(v0, a[0]);
        let (r1, dr1, b1, db1) = endpoint(v1, b[0]);
        [
            (r0, b0),
            ([r0[0] + dr0[0] / 3.0, r0[1] + dr0[1] / 3.0], b0 + db0 / 3.0),
            ([r1[0] - dr1[0] / 3.0, r1[1] - dr1[1] / 3.0], b1 - db1 / 3.0),
            (r1, b1),
        ]
        .into_iter()
        .map(|(row, base)| LinearConstraint {
            coefficients: row.to_vec(),
            value: -base,
        })
        .collect()
    }

    #[cfg(test)]
    fn fit_radius(&mut self, guides: &[UpperGuide], outline_xy: &[[f64; 2]]) -> Result<(), String> {
        self.fit_radius_policy(guides, outline_xy, false)
    }

    fn fit_radius_policy(
        &mut self,
        guides: &[UpperGuide],
        outline_xy: &[[f64; 2]],
        minimum_bending: bool,
    ) -> Result<(), String> {
        let mut bounds = Vec::new();
        let derivative = self.seam_derivative[0] * self.span;
        if self.seam[0] <= 0.0 || self.seam[0] + derivative / 3.0 <= 0.0 {
            return Err("Upper radius has nonpositive fixed seam controls".into());
        }
        bounds.push(LinearConstraint {
            coefficients: vec![1.0, 0.0],
            value: 3e-6 - 3.0 * self.seam[0] - 2.0 * derivative,
        });
        bounds.push(LinearConstraint {
            coefficients: vec![1.0, 1.0],
            value: 1e-6 - self.seam[0] - derivative,
        });
        for xy in guides
            .iter()
            .map(|g| g.xy)
            .chain(outline_xy.iter().copied())
        {
            let dy = xy[1] - self.seam_y;
            if dy <= 1e-6 {
                continue;
            }
            let v = dy / self.span;
            bounds.push(LinearConstraint {
                coefficients: vec![v * v, v * v * v],
                // A seam-vanishing guard keeps upper trims off a singular
                // frontal projection without moving their authored X/Y.
                value: xy[0].abs() / self.angle.sin() + 0.005 * v * v
                    - self.seam[0]
                    - self.seam_derivative[0] * dy,
            });
        }
        // Minimize departure from the lower radius/value-slope continuation,
        // not bending alone, which extrapolates a lower flare into a huge top.
        // Final edges follow the field in parameter space, not XY chords.
        let bending = 0.002;
        let matrix = if minimum_bending {
            // Full candidate20c policy, including its conservative upper-only
            // chord surrogate, is retained solely for controlled ablation.
            for (a, b) in outline_xy
                .iter()
                .zip(outline_xy.iter().cycle().skip(1))
                .take(outline_xy.len())
            {
                bounds.extend(self.legacy_radius_segment_bounds(*a, *b));
            }
            [vec![4.0, 6.0], vec![6.0, 12.0]]
        } else {
            [
                vec![1.0 / 5.0 + 4.0 * bending, 1.0 / 6.0 + 6.0 * bending],
                vec![1.0 / 6.0 + 6.0 * bending, 1.0 / 7.0 + 12.0 * bending],
            ]
        };
        let result = solve_dense_qp(&matrix, &[0.0, 0.0], &[], &bounds, QpOptions::default())
            .map_err(|e| format!("Global upper radius: {e:?}"))?;
        self.radius_correction.copy_from_slice(&result.coefficients);
        Ok(())
    }

    pub fn fit(
        self,
        guides: &[UpperGuide],
        outline_xy: &[[f64; 2]],
        padding: f64,
        fairing: f64,
        support: impl Fn(f64, f64) -> Option<f64>,
    ) -> Result<Self, String> {
        let policy = FitPolicy::diagnostic()?;
        self.fit_with_policy(guides, outline_xy, padding, fairing, support, policy)
    }

    fn fit_with_policy(
        mut self,
        guides: &[UpperGuide],
        outline_xy: &[[f64; 2]],
        padding: f64,
        fairing: f64,
        support: impl Fn(f64, f64) -> Option<f64>,
        policy: FitPolicy,
    ) -> Result<Self, String> {
        self.elliptical_depth = policy.elliptical_depth;
        self.depth_controls = vec![0.0; self.control_count()];
        if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
            let guides_json = guides
                .iter()
                .map(|g| {
                    format!(
                        "{{\"xy\":{:?},\"target\":{},\"ceiling\":{},\"normal\":{}}}",
                        g.xy,
                        g.target,
                        g.ceiling
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "null".into()),
                        g.normal
                            .map(|v| format!("{v:?}"))
                            .unwrap_or_else(|| "null".into())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let data = format!(
                "{{\"surface\":{},\"guides\":[{guides_json}],\"outline_xy\":{outline_xy:?},\"padding\":{padding},\"fairing\":{fairing},\"fit_policy\":{}}}",
                self.diagnostic_json(),
                policy.json()
            );
            std::fs::write(
                std::path::PathBuf::from(path).with_extension("global-inputs.json"),
                data,
            )
            .map_err(|e| e.to_string())?;
        }
        if self.span <= 0.0
            || !self.span.is_finite()
            || self.angle <= 0.0
            || (self.angle > std::f64::consts::FRAC_PI_2
                && self.angle != f64::from(std::f32::consts::FRAC_PI_2))
            || !self.angle.is_finite()
            || self.arc_scale <= 0.0
            || !self.arc_scale.is_finite()
            || padding < 0.0
            || !padding.is_finite()
            || fairing <= 0.0
            || !fairing.is_finite()
            || self
                .seam
                .iter()
                .chain(&self.seam_derivative)
                .any(|v| !v.is_finite())
            || guides.is_empty()
            || guides.iter().any(|g| {
                g.xy.iter().any(|v| !v.is_finite())
                    || !g.target.is_finite()
                    || g.ceiling.is_some_and(|v| !v.is_finite())
            })
        {
            return Err("Invalid global upper loft inputs".into());
        }
        self.fit_radius_policy(guides, outline_xy, policy.minimum_bending_radius)?;
        let neck_index = guides
            .iter()
            .enumerate()
            .filter(|(_, g)| g.xy[1] > self.seam_y + 1e-6)
            .min_by(|(_, a), (_, b)| a.xy[0].abs().total_cmp(&b.xy[0].abs()))
            .map(|(i, _)| i)
            .ok_or("Global upper loft needs a neck-center guide")?;
        let neck = &guides[neck_index];
        // Only anatomical landmarks prescribe desired depth/orientation. The
        // intermediate body samples remain clearance inequalities, not targets
        // that reward fitting triangle fluctuations along an opening.
        let mut landmarks = vec![neck_index];
        landmarks.extend(guides.iter().position(|g| g.normal.is_some()));
        landmarks.extend(guides.iter().rposition(|g| g.normal.is_some()));
        landmarks.sort_unstable();
        landmarks.dedup();
        if policy.dense_guides {
            landmarks = (0..guides.len()).collect();
        }
        let v_neck = (neck.xy[1] - self.seam_y) / self.span;
        self.prior_depth_correction =
            (neck.target - self.rows(0.0, neck.xy[1])[0][0]) / v_neck.powi(2);
        let outline = outline_xy
            .iter()
            .map(|p| {
                if p[1] <= self.seam_y {
                    return Ok([p[0].signum(), p[1]]);
                }
                Ok([self.parameter(*p)? * p[0].signum(), p[1]])
            })
            .collect::<Result<Vec<_>, String>>()?;
        let count = self.control_count();
        let mut h = vec![vec![0.0; count]; count];
        let mut rhs = vec![0.0; count];
        let mut bounds = Vec::new();
        let add_energy =
            |row: &[f64], target: f64, weight: f64, h: &mut [Vec<f64>], rhs: &mut [f64]| {
                for i in 0..count {
                    rhs[i] += weight * row[i + 1] * (target - row[0]);
                    for j in 0..count {
                        h[i][j] += weight * row[i + 1] * row[j + 1];
                    }
                }
            };
        for (index, guide) in guides.iter().enumerate() {
            if guide.xy[1] <= self.seam_y + 1e-6 {
                continue;
            }
            let u = self.parameter(guide.xy)?;
            let row = &self.rows(u, guide.xy[1])[0];
            if landmarks.contains(&index) {
                add_energy(
                    row,
                    guide.target,
                    1.0 / landmarks.len() as f64,
                    &mut h,
                    &mut rhs,
                );
            }
            if let Some(normal) = guide.normal
                && landmarks.contains(&index)
            {
                if normal[2] <= 0.0 || !normal.iter().all(|v| v.is_finite()) {
                    return Err("Invalid global upper crest normal".into());
                }
                let rows = self.rows(u, guide.xy[1]);
                let (a, da) = self.radius(guide.xy[1]);
                let theta = u * self.angle;
                let zu = -normal[0] * a * theta.cos() * self.angle / normal[2];
                let zy = -(normal[0] * da * theta.sin() + normal[1]) / normal[2];
                add_energy(&rows[1], zu, 1.0 / landmarks.len() as f64, &mut h, &mut rhs);
                add_energy(
                    &rows[2],
                    zy,
                    self.span.powi(2) / landmarks.len() as f64,
                    &mut h,
                    &mut rhs,
                );
            }
            if let Some(body) = support(guide.xy[0].abs(), guide.xy[1]) {
                bounds.push(LinearConstraint {
                    coefficients: row[1..].to_vec(),
                    value: body + padding - row[0],
                });
            }
            if let Some(ceiling) = guide.ceiling {
                bounds.push(LinearConstraint {
                    coefficients: row[1..].iter().map(|v| -v).collect(),
                    value: row[0] - ceiling,
                });
            }
        }
        for col in 0..65 {
            let u = col as f64 / 64.0;
            for row in 1..=32 {
                let y = self.seam_y + self.span * row as f64 / 32.0;
                if !contains(&outline, [u, y]) {
                    continue;
                }
                let rows = self.rows(u, y);
                // Independent rounded-shell prior: continue the lower ellipse
                // with a C1 depth meridian aimed at the measured neck stand-off.
                // No old per-meridian yoke depth or derivative is a target.
                let v = (y - self.seam_y) / self.span;
                let prior =
                    rows[0][0] + self.prior_depth_correction * v * v * (u * self.angle).cos();
                add_energy(&rows[0], prior, 1.0 / (65.0 * 32.0), &mut h, &mut rhs);
                let x = self.radius(y).0 * (u * self.angle).sin();
                if let Some(body) = support(x, y) {
                    bounds.push(LinearConstraint {
                        coefficients: rows[0][1..].to_vec(),
                        value: body + padding - rows[0][0],
                    });
                }
                let scale = (0.20 * self.span).powi(4) * fairing / (65.0 * 32.0);
                for (index, weight) in [
                    (3, self.arc_scale.powi(-4)),
                    (4, 1.0),
                    (5, 2.0 / self.arc_scale.powi(2)),
                ] {
                    add_energy(&rows[index], 0.0, scale * weight, &mut h, &mut rhs);
                }
            }
        }
        let mut exact = Vec::new();
        if self.elliptical_depth {
            bounds.extend(elliptical::shape_bounds(
                self.seam[1],
                self.seam_derivative[1],
                self.span,
                v_neck,
            )?);
        } else {
            for j in 0..HEIGHT {
                let mut coefficients = vec![0.0; count];
                coefficients[j] = 1.0;
                coefficients[HEIGHT + j] = -1.0;
                exact.push(LinearConstraint {
                    coefficients,
                    value: 0.0,
                });
            }
        }
        for (i, row) in h.iter_mut().enumerate() {
            row[i] += 1e-8;
        }
        if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
            let rows = |items: &[LinearConstraint]| {
                items
                    .iter()
                    .map(|r| {
                        format!(
                            "{{\"coefficients\":{:?},\"value\":{}}}",
                            r.coefficients, r.value
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let data = format!(
                "{{\"hessian\":{h:?},\"rhs\":{rhs:?},\"equalities\":[{}],\"lower_bounds\":[{}],\"surface\":{},\"fit_policy\":{}}}",
                rows(&exact),
                rows(&bounds),
                self.diagnostic_json(),
                policy.json()
            );
            std::fs::write(
                std::path::PathBuf::from(path).with_extension("global-qp.json"),
                data,
            )
            .map_err(|e| e.to_string())?;
        }
        let result = solve_dense_qp(&h, &rhs, &exact, &bounds, QpOptions::default())
            .map_err(|e| format!("Global upper depth: {e:?}"))?;
        eprintln!(
            "global upper constrained fit: {} bounds {:?}",
            bounds.len(),
            result.diagnostics
        );
        self.depth_controls = result.coefficients;
        Ok(self)
    }

    pub fn diagnostic_json(&self) -> String {
        let kind = if self.elliptical_depth {
            "global-upper-elliptical-v1"
        } else {
            "global-upper-bernstein-v1"
        };
        let angular_count = if self.elliptical_depth { 2 } else { ANGULAR };
        format!(
            "{{\"kind\":\"{kind}\",\"elliptical_depth\":{},\"seam_y\":{},\"span\":{},\"angle\":{},\"arc_scale\":{},\"seam\":{:?},\"seam_derivative\":{:?},\"radius_correction\":{:?},\"prior_depth_correction\":{},\"depth_controls\":{:?},\"angular_count\":{angular_count},\"height_count\":{HEIGHT}}}",
            self.elliptical_depth,
            self.seam_y,
            self.span,
            self.angle,
            self.arc_scale,
            self.seam,
            self.seam_derivative,
            self.radius_correction,
            self.prior_depth_correction,
            self.depth_controls
        )
    }
}
