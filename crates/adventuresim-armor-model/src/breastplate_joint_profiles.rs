//! Diagnostic whole-torso ellipse family. The lower field is fitted jointly
//! with its upper continuation instead of prescribing an arbitrary C1 seam.
//! Graph containment is not a certificate for the solidified inner layer.

use crate::{
    breastplate_global_upper::{GlobalUpperLoft, UpperGuide},
    breastplate_joint_boundary::{JointReturn, ResolvedJointReturn},
    breastplate_loft_profiles::LoftProfiles,
    breastplate_qp::{LinearConstraint, QpOptions, solve_dense_qp},
};

const N: usize = 16;
type Row = [f64; N];
#[path = "breastplate_whole_front_field.rs"]
mod whole_front_field;

fn valid_crest_normal(normal: [f64; 3]) -> bool {
    let magnitude = normal.iter().map(|v| v.abs()).fold(0.0, f64::max);
    // Slope targets divide by the anterior normal component. Reject a graph
    // chart that would lose more than half of f64's significant digits; this
    // is scale-invariant and does not replace the body's measured normal.
    normal.iter().all(|v| v.is_finite()) && normal[2] > magnitude * f64::EPSILON.sqrt()
}

pub(super) fn baseline_policy_values() -> [String; 3] {
    [
        ("BREASTPLATE_DIAGNOSTIC_UPPER_RADIUS_MODE", "grounded"),
        ("BREASTPLATE_DIAGNOSTIC_UPPER_GUIDE_MODE", "landmarks"),
        ("BREASTPLATE_DIAGNOSTIC_UPPER_DEPTH_MODE", "bernstein"),
    ]
    .map(|(name, default)| std::env::var(name).unwrap_or_else(|_| default.to_owned()))
}

#[derive(Clone, Debug, serde::Serialize)]
struct SupportSample {
    xy: [f64; 2],
    floor: Option<f64>,
    ceiling: Option<f64>,
    origin: &'static str,
}

#[derive(Clone, Debug)]
pub(super) struct JointProfiles {
    pub waist_y: f64,
    pub seam_y: f64,
    pub lower_a: [f64; 4],
    pub lower_b: [f64; 4],
    pub lower_c: [f64; 4],
    pub upper: GlobalUpperLoft,
    pub side_trim: f64,
    pub trim_start_y: f64,
    pub boundary_domain: Vec<[f32; 2]>,
    pub return_curve: JointReturn,
    pub resolved_return: Option<ResolvedJointReturn>,
    pub fit_iteration: usize,
    pub whole_front: Option<crate::breastplate_whole_front::WholeFront>,
    pub front_support: Vec<crate::breastplate_front_envelope::FrontSupport>,
    pub physical_support: Vec<([f64; 2], f64)>,
}

fn basis3(t: f64, k: usize) -> [f64; 4] {
    match k {
        0 => [
            (1.0 - t).powi(3),
            3.0 * t * (1.0 - t).powi(2),
            3.0 * t * t * (1.0 - t),
            t.powi(3),
        ],
        1 => [
            -3.0 * (1.0 - t).powi(2),
            3.0 - 12.0 * t + 9.0 * t * t,
            6.0 * t - 9.0 * t * t,
            3.0 * t * t,
        ],
        2 => [6.0 * (1.0 - t), -12.0 + 18.0 * t, 6.0 - 18.0 * t, 6.0 * t],
        _ => unreachable!(),
    }
}
fn correction(t: f64, k: usize) -> [f64; 4] {
    let b = basis3(t, 0);
    match k {
        0 => b.map(|b| t * t * b),
        1 => {
            let d = basis3(t, 1);
            std::array::from_fn(|i| 2.0 * t * b[i] + t * t * d[i])
        }
        2 => {
            let d = basis3(t, 1);
            let dd = basis3(t, 2);
            std::array::from_fn(|i| 2.0 * b[i] + 4.0 * t * d[i] + t * t * dd[i])
        }
        _ => unreachable!(),
    }
}
fn dot<const K: usize>(a: &[f64; K], b: &[f64; K]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn combine(a: Row, b: Row, weight: f64) -> Row {
    std::array::from_fn(|i| a[i] + weight * b[i])
}
fn constraint<const K: usize>(a: [f64; K], value: f64) -> LinearConstraint {
    LinearConstraint {
        coefficients: a.to_vec(),
        value,
    }
}
fn contains(poly: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut inside = false;
    for (a, b) in poly
        .iter()
        .zip(poly.iter().cycle().skip(1))
        .take(poly.len())
    {
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}
fn energy<const K: usize>(h: &mut [Vec<f64>], rhs: &mut [f64], r: [f64; K], target: f64, w: f64) {
    for i in 0..K {
        rhs[i] += w * r[i] * target;
        for j in 0..K {
            h[i][j] += w * r[i] * r[j];
        }
    }
}
fn dump(label: &str, value: serde_json::Value) -> Result<(), String> {
    if let Some(path) = std::env::var_os("BREASTPLATE_ANGULAR_DUMP") {
        let path = std::path::PathBuf::from(path).with_extension(format!("{label}.json"));
        std::fs::write(path, serde_json::to_vec(&value).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn qp_dump(
    label: &str,
    h: &[Vec<f64>],
    rhs: &[f64],
    eq: &[LinearConstraint],
    bounds: &[LinearConstraint],
    context: serde_json::Value,
) -> Result<(), String> {
    let rows = |r: &[LinearConstraint]| {
        r.iter()
            .map(|r| serde_json::json!({"coefficients":r.coefficients,"value":r.value}))
            .collect::<Vec<_>>()
    };
    dump(
        label,
        serde_json::json!({"hessian":h,"rhs":rhs,"equalities":rows(eq),"lower_bounds":rows(bounds),"context":context}),
    )
}

impl JointProfiles {
    fn lower_span(&self) -> f64 {
        self.seam_y - self.waist_y
    }
    pub fn lower_eval(&self, y: f64) -> [f64; 3] {
        let b = basis3(((y - self.waist_y) / self.lower_span()).clamp(0.0, 1.0), 0);
        let mut values = [
            dot(&b, &self.lower_a),
            dot(&b, &self.lower_b),
            dot(&b, &self.lower_c),
        ];
        if let Some(front) = self.whole_front.as_ref().filter(|front| front.active(y)) {
            values[1] = front.jet(y, self.base_front_jet(y))[0] - values[2];
        }
        values
    }
    fn lower_derivative(&self, y: f64) -> [f64; 3] {
        let b = basis3(((y - self.waist_y) / self.lower_span()).clamp(0.0, 1.0), 1)
            .map(|v| v / self.lower_span());
        let mut values = [
            dot(&b, &self.lower_a),
            dot(&b, &self.lower_b),
            dot(&b, &self.lower_c),
        ];
        if let Some(front) = self.whole_front.as_ref().filter(|front| front.active(y)) {
            values[1] = front.jet(y, self.base_front_jet(y))[1] - values[2];
        }
        values
    }
    pub fn coverage(&self, y: f64) -> [f64; 3] {
        if y > self.seam_y {
            return [self.upper.angle, 0.0, 0.0];
        }
        let width = self.seam_y - self.trim_start_y;
        let t = ((y - self.trim_start_y) / width.max(1e-6)).clamp(0.0, 1.0);
        [
            std::f32::consts::FRAC_PI_2 as f64 - self.side_trim * t * t * (3.0 - 2.0 * t),
            -self.side_trim * 6.0 * t * (1.0 - t) / width.max(1e-6),
            if t > 0.0 {
                -self.side_trim * (6.0 - 12.0 * t) / width.max(1e-6).powi(2)
            } else {
                0.0
            },
        ]
    }
    pub fn evaluate(&self, u: f64, y: f64) -> ([f64; 3], [f64; 3]) {
        if let Some(value) = self.composed_evaluate(u, y) {
            return value;
        }
        if y > self.seam_y {
            return self.upper.evaluate(u, y);
        }
        let [a, b, c] = self.lower_eval(y);
        let [da, db, dc] = self.lower_derivative(y);
        let theta = u * self.coverage(y)[0];
        let n = [
            b * theta.sin(),
            -(b * da * theta.sin().powi(2) + a * theta.cos() * (dc + db * theta.cos())),
            a * theta.cos(),
        ];
        let norm = dot(&n, &n).sqrt();
        (
            [a * theta.sin(), y, c + b * theta.cos()],
            n.map(|v| v / norm),
        )
    }
    fn component(&self, y: f64, b: bool, k: usize) -> Row {
        let mut r = [0.0; N];
        let start = if b { 4 } else { 0 };
        if y <= self.seam_y {
            let t = (y - self.waist_y) / self.lower_span();
            let basis = basis3(t, k);
            for j in 0..4 {
                r[start + j] = basis[j] / self.lower_span().powi(k as i32);
            }
        } else {
            let dy = y - self.seam_y;
            let v = dy / self.upper.span;
            if k == 0 {
                r[start + 3] = 1.0 + 3.0 * dy / self.lower_span();
                r[start + 2] = -3.0 * dy / self.lower_span();
            } else if k == 1 {
                r[start + 3] = 3.0 / self.lower_span();
                r[start + 2] = -3.0 / self.lower_span();
            }
            let basis = correction(v, k);
            for j in 0..4 {
                r[8 + start + j] = basis[j] / self.upper.span.powi(k as i32);
            }
        }
        r
    }
    fn rows(&self, u: f64, y: f64) -> [Row; 6] {
        let [angle, ay, ayy] = self.coverage(y);
        let theta = u * angle;
        let co = theta.cos();
        let si = theta.sin();
        let ty = u * ay;
        let tyy = u * ayy;
        let c = self.component(y, false, 0);
        let b = self.component(y, true, 0);
        let cy = self.component(y, false, 1);
        let by = self.component(y, true, 1);
        [
            combine(c, b, co),
            b.map(|v| -v * si * angle),
            combine(combine(cy, by, co), b, -si * ty),
            b.map(|v| -v * co * angle * angle),
            combine(
                combine(
                    combine(self.component(y, false, 2), self.component(y, true, 2), co),
                    by,
                    -2.0 * si * ty,
                ),
                b,
                -co * ty * ty - si * tyy,
            ),
            combine(by.map(|v| -v * si * angle), b, -co * ty * angle - si * ay),
        ]
    }
    fn radius_row(&self, y: f64, k: usize) -> [f64; 6] {
        let mut r = [0.0; 6];
        if y <= self.seam_y {
            let b = basis3((y - self.waist_y) / self.lower_span(), k);
            for j in 0..4 {
                r[j] = b[j] / self.lower_span().powi(k as i32);
            }
        } else {
            let dy = y - self.seam_y;
            let v = dy / self.upper.span;
            match k {
                0 => {
                    r[3] = 1.0 + 3.0 * dy / self.lower_span();
                    r[2] = -3.0 * dy / self.lower_span();
                    r[4] = v * v;
                    r[5] = v * v * v;
                }
                1 => {
                    r[3] = 3.0 / self.lower_span();
                    r[2] = -3.0 / self.lower_span();
                    r[4] = 2.0 * v / self.upper.span;
                    r[5] = 3.0 * v * v / self.upper.span;
                }
                2 => {
                    r[4] = 2.0 / self.upper.span.powi(2);
                    r[5] = 6.0 * v / self.upper.span.powi(2);
                }
                _ => unreachable!(),
            }
        }
        r
    }
    fn radius_coefficients(&self) -> [f64; 6] {
        std::array::from_fn(|j| {
            if j < 4 {
                self.lower_a[j]
            } else {
                self.upper.radius_correction[j - 4]
            }
        })
    }
    fn radius(&self, y: f64) -> f64 {
        dot(&self.radius_row(y, 0), &self.radius_coefficients())
    }
    fn parameter(&self, xy: [f64; 2]) -> Result<f64, String> {
        let ratio = xy[0].abs() / self.radius(xy[1]);
        let angle = self.coverage(xy[1])[0];
        if !ratio.is_finite() || ratio > angle.sin() + 1e-7 {
            return Err(format!(
                "joint inverse query outside radius: xy={xy:?} ratio={ratio} angle={angle}"
            ));
        }
        Ok(if ratio >= angle.sin() {
            1.0
        } else {
            ratio.asin() / angle
        })
    }
    fn update_upper_seam(&mut self) {
        self.upper.seam = self.lower_eval(self.seam_y);
        self.upper.seam_derivative = self.lower_derivative(self.seam_y);
    }
    pub fn diagnostic_json(&self) -> serde_json::Value {
        let mut data = serde_json::json!({"kind":"joint-torso-elliptical-v1","fit_iteration":self.fit_iteration,"waist_y":self.waist_y,"seam_y":self.seam_y,"span":self.upper.span,"arc_scale":self.upper.arc_scale,"angle":self.upper.angle,"side_trim":self.side_trim,"trim_start_y":self.trim_start_y,"lower_a":self.lower_a,"lower_b":self.lower_b,"lower_c":self.lower_c,"upper_radius_correction":self.upper.radius_correction,"upper_depth_controls":self.upper.depth_controls,"return_curve":self.return_curve,"resolved_angular_return":self.resolved_return,"boundary_domain":self.boundary_domain,"baseline_policy_radius_guide_depth":baseline_policy_values()});
        if let Some(front) = &self.whole_front {
            data["whole_front"] =
                serde_json::to_value(front).expect("finite whole-front representation");
        }
        data
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fit(
        lower: &LoftProfiles,
        baseline: &GlobalUpperLoft,
        guides: &[UpperGuide],
        old_outline: &[[f64; 2]],
        old_domain: &[[f32; 2]],
        return_curve: JointReturn,
        padding: f64,
        fairing: f64,
        side_trim: f64,
        trim_start_y: f64,
        iteration: usize,
        support: impl Fn(f64, f64) -> Option<f64>,
    ) -> Result<Self, String> {
        return_curve.validate()?;
        if !baseline.elliptical_depth || old_outline.len() != old_domain.len() {
            return Err(
                "joint diagnostic requires elliptical baseline and paired boundary samples".into(),
            );
        }
        let [waist_y, seam_y] = lower.height_range();
        let l = seam_y - waist_y;
        let c0 = lower.eval(waist_y)[2];
        let dc = lower.derivative(waist_y)[2];
        let mut result = Self {
            waist_y,
            seam_y,
            lower_a: lower.width_controls,
            lower_b: lower.depth_controls,
            lower_c: std::array::from_fn(|j| c0 + dc * l * j as f64 / 3.0),
            upper: baseline.clone(),
            side_trim,
            trim_start_y,
            boundary_domain: Vec::new(),
            return_curve,
            resolved_return: None,
            fit_iteration: iteration,
            whole_front: None,
            front_support: Vec::new(),
            physical_support: Vec::new(),
        };
        result.upper.angle = result.return_curve.angle;
        let old_a = result.radius_coefficients();
        let old_depth: Row = std::array::from_fn(|j| match j {
            0..=3 => result.lower_c[j],
            4..=7 => result.lower_b[j - 4],
            _ => baseline.depth_controls[j - 8],
        });
        let old_c = result.lower_c;
        let old_b = result.lower_b;
        let old_polygon = old_outline
            .iter()
            .zip(old_domain)
            .map(|(xy, q)| {
                if xy[1] <= seam_y {
                    Ok([q[0] as f64 / baseline.arc_scale, xy[1]])
                } else {
                    baseline.parameter(*xy).map(|u| [u.copysign(xy[0]), xy[1]])
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut inherited = guides
            .iter()
            .map(|g| SupportSample {
                xy: g.xy,
                floor: support(g.xy[0], g.xy[1]).map(|z| z + padding),
                ceiling: g.ceiling,
                origin: "inherited-guide",
            })
            .collect::<Vec<_>>();
        for col in 0..65 {
            for row in 1..=32 {
                let u = col as f64 / 64.0;
                let y = seam_y + baseline.span * row as f64 / 32.0;
                if contains(&old_polygon, [u, y]) {
                    let p = baseline.evaluate(u, y).0;
                    inherited.push(SupportSample {
                        xy: [p[0], y],
                        floor: support(p[0], y).map(|z| z + padding),
                        ceiling: None,
                        origin: "inherited-material",
                    });
                }
            }
        }
        dump(
            &format!("joint-inputs-{iteration}"),
            serde_json::json!({"field":result.diagnostic_json(),"baseline_upper":serde_json::from_str::<serde_json::Value>(&baseline.diagnostic_json()).map_err(|e|e.to_string())?,"inherited_support":inherited,"old_outline":old_outline,"old_domain":old_domain,"guides":guides.iter().map(|g|serde_json::json!({"xy":g.xy,"target":g.target,"ceiling":g.ceiling,"normal":g.normal})).collect::<Vec<_>>(),"padding":padding,"fairing":fairing,"baseline_contact_dependency":"baseline28 fit computed once per wearer/design"}),
        )?;
        // Fit genuine upper anchors and inherited physical support. The
        // derived tail is rebuilt in angle after this solve; retaining its
        // old XY gap constraints would force a full-wrap 3D tangent kink.
        let mut h = vec![vec![0.0; 6]; 6];
        let mut rhs = vec![0.0; 6];
        let mut eq = Vec::new();
        let mut bounds = Vec::new();
        for j in 0..6 {
            h[j][j] = 1e-8;
            rhs[j] = 1e-8 * old_a[j];
        }
        for j in 0..2 {
            let mut r = [0.0; 6];
            r[j] = 1.0;
            eq.push(constraint(r, old_a[j]));
        }
        let mut r = [0.0; 6];
        r[4] = 2.0 / result.upper.span.powi(2);
        r[1] = -6.0 / l.powi(2);
        r[2] = 12.0 / l.powi(2);
        r[3] = -6.0 / l.powi(2);
        eq.push(constraint(r, 0.0));
        for j in 0..4 {
            let mut r = [0.0; 6];
            r[j] = 1.0;
            bounds.push(constraint(r, old_a[j]));
        }
        for j in 0..4 {
            let mut r = [0.0; 6];
            r[3] = 1.0 + result.upper.span * j as f64 / l;
            r[2] = -result.upper.span * j as f64 / l;
            if j == 2 {
                r[4] = 1.0 / 3.0;
            }
            if j == 3 {
                r[4] = 1.0;
                r[5] = 1.0;
            }
            bounds.push(constraint(r, 1e-6));
        }
        for xy in guides
            .iter()
            .take(result.return_curve.guide_start + 1)
            .map(|g| g.xy)
            .chain(
                old_outline
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !result.return_curve.boundary_stations.contains(i))
                    .map(|(_, xy)| *xy),
            )
        {
            if xy[1] <= seam_y + 1e-6 {
                continue;
            }
            let v = (xy[1] - seam_y) / result.upper.span;
            let row = result.radius_row(xy[1], 0);
            let need = xy[0].abs() / result.upper.angle.sin() + 0.005 * v * v;
            bounds.push(constraint(row, need));
        }
        for sample in &inherited {
            if sample.xy[1] > seam_y && (sample.floor.is_some() || sample.ceiling.is_some()) {
                bounds.push(constraint(
                    result.radius_row(sample.xy[1], 0),
                    sample.xy[0].abs() / result.upper.angle.sin(),
                ));
            }
        }
        for i in 0..129 {
            let y = waist_y + (l + result.upper.span) * i as f64 / 128.0;
            let row = result.radius_row(y, 0);
            energy(&mut h, &mut rhs, row, dot(&row, &old_a), 1.0 / 129.0);
            energy(
                &mut h,
                &mut rhs,
                result.radius_row(y, 2),
                0.0,
                (0.2 * (l + result.upper.span)).powi(4) / 129.0,
            );
        }
        qp_dump(
            &format!("joint-radius-{iteration}-qp"),
            &h,
            &rhs,
            &eq,
            &bounds,
            result.diagnostic_json(),
        )?;
        let fitted = solve_dense_qp(&h, &rhs, &eq, &bounds, QpOptions::default())
            .map_err(|e| format!("joint radius QP: {e:?}"))?;
        result.lower_a.copy_from_slice(&fitted.coefficients[..4]);
        result
            .upper
            .radius_correction
            .copy_from_slice(&fitted.coefficients[4..]);
        result.update_upper_seam();
        eprintln!("joint radius {iteration}: {:?}", fitted.diagnostics);
        let radius = result.radius_coefficients();
        let return_validation = result.return_curve.resolve(
            result.radius(result.return_curve.upper_xy[1]),
            dot(
                &result.radius_row(result.return_curve.upper_xy[1], 1),
                &radius,
            ),
        );
        dump(
            &format!("joint-radius-{iteration}-result"),
            serde_json::json!({"coefficients":radius,"diagnostics":format!("{:?}",fitted.diagnostics),"return_validation":return_validation.as_ref().map_err(|e|e.as_str())}),
        )?;
        let angular_return = return_validation?;
        let mut outline = Vec::with_capacity(old_outline.len());
        for (index, (xy, domain)) in old_outline.iter().zip(old_domain).enumerate() {
            let y = xy[1];
            let u = domain[0] as f64 / result.upper.arc_scale;
            let x = if y <= seam_y {
                result.radius(y) * (u * result.coverage(y)[0]).sin()
            } else if result.return_curve.boundary_stations.contains(&index) {
                angular_return.x(y, result.radius(y)).copysign(xy[0])
            } else {
                xy[0]
            };
            let p = if y <= seam_y {
                [domain[0], domain[1]]
            } else if result.return_curve.boundary_stations.contains(&index) {
                [
                    (angular_return.parameter(y).copysign(x) * result.upper.arc_scale) as f32,
                    y as f32,
                ]
            } else {
                [
                    (result.parameter([x, y])?.copysign(x) * result.upper.arc_scale) as f32,
                    y as f32,
                ]
            };
            result.boundary_domain.push(p);
            outline.push([x, y]);
        }
        result.resolved_return = Some(angular_return);
        let polygon = result
            .boundary_domain
            .iter()
            .map(|p| [p[0] as f64 / result.upper.arc_scale, p[1] as f64])
            .collect::<Vec<_>>();
        let mut samples = inherited;
        for xy in &outline {
            samples.push(SupportSample {
                xy: *xy,
                floor: support(xy[0], xy[1]).map(|z| z + padding),
                ceiling: None,
                origin: "new-boundary",
            });
        }
        let mut material = Vec::new();
        for upper in [false, true] {
            let (start, span) = if upper {
                (seam_y, result.upper.span)
            } else {
                (waist_y, l)
            };
            for col in 0..65 {
                for row in 1..=32 {
                    let u = col as f64 / 64.0;
                    let y = start + span * row as f64 / 32.0;
                    if !contains(&polygon, [u, y]) {
                        continue;
                    }
                    let x = result.radius(y) * (u * result.coverage(y)[0]).sin();
                    samples.push(SupportSample {
                        xy: [x, y],
                        floor: support(x, y).map(|z| z + padding),
                        ceiling: None,
                        origin: if upper {
                            "new-upper-material"
                        } else {
                            "new-lower-material"
                        },
                    });
                    material.push((u, y, upper));
                }
            }
        }
        // The baseline solve is used only for this diagnostic's no-worse
        // contact-error bounds; actual support is freshly sampled above.
        let neck = guides
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.xy[0].abs().total_cmp(&b.xy[0].abs()))
            .map(|(i, _)| i)
            .ok_or("No joint neck guide")?;
        let mut landmarks = vec![neck];
        landmarks.extend(guides.iter().position(|g| g.normal.is_some()));
        landmarks.extend(guides.iter().rposition(|g| g.normal.is_some()));
        landmarks.sort_unstable();
        landmarks.dedup();
        let mut h = vec![vec![0.0; N]; N];
        let mut rhs = vec![0.0; N];
        let mut eq = Vec::new();
        let mut bounds = Vec::new();
        for j in 0..N {
            h[j][j] = 1e-8;
            rhs[j] = 1e-8 * old_depth[j];
        }
        for sample in &samples {
            let u = result.parameter(sample.xy)?;
            let row = result.rows(u, sample.xy[1])[0];
            if let Some(floor) = sample.floor {
                bounds.push(constraint(row, floor));
            }
            if let Some(ceiling) = sample.ceiling {
                bounds.push(constraint(row.map(|v| -v), -ceiling));
            }
        }
        let physical_bound_count = bounds.len();
        for (start, old) in [(0, old_c), (4, old_b)] {
            for j in 0..2 {
                let mut r = [0.0; N];
                r[start + j] = 1.0;
                eq.push(constraint(r, old[j]));
            }
            let mut r = [0.0; N];
            r[8 + start] = 2.0 / result.upper.span.powi(2);
            r[start + 1] = -6.0 / l.powi(2);
            r[start + 2] = 12.0 / l.powi(2);
            r[start + 3] = -6.0 / l.powi(2);
            eq.push(constraint(r, 0.0));
        }
        for j in 0..4 {
            let mut r = [0.0; N];
            r[j] = 1.0;
            bounds.push(constraint(r, old_c[j]));
            r[j + 4] = 1.0;
            bounds.push(constraint(r, old_c[j] + old_b[j]));
            let mut r = [0.0; N];
            r[j + 4] = 1.0;
            bounds.push(constraint(r, 1e-6));
        }
        let mut upper_controls = Vec::new();
        for j in 0..6 {
            let mut r = [0.0; N];
            r[7] = 1.0 + 3.0 * result.upper.span * j as f64 / (5.0 * l);
            r[6] = -3.0 * result.upper.span * j as f64 / (5.0 * l);
            if j >= 2 {
                r[12 + j - 2] = [0.1, 0.3, 0.6, 1.0][j - 2];
            }
            bounds.push(constraint(r, 1e-6));
            for i in 0..4 {
                r[i] = r[4 + i];
                r[8 + i] = r[12 + i];
            }
            upper_controls.push(r);
        }
        for j in 0..2 {
            let mut r = [0.0; N];
            for start in [0, 4] {
                r[start + j] = -1.0;
                r[start + j + 1] = 2.0;
                r[start + j + 2] = -1.0;
            }
            bounds.push(constraint(r, 0.0));
        }
        let mut second = (0..4)
            .map(|j| {
                std::array::from_fn::<_, N, _>(|i| {
                    20.0 * (upper_controls[j + 2][i] - 2.0 * upper_controls[j + 1][i]
                        + upper_controls[j][i])
                        / result.upper.span.powi(2)
                })
            })
            .collect::<Vec<_>>();
        let vn = (guides[neck].xy[1] - seam_y) / result.upper.span;
        loop {
            bounds.push(constraint(second[0].map(|v| -v), 0.0));
            if second.len() == 1 {
                break;
            }
            second = second
                .windows(2)
                .map(|w| std::array::from_fn(|j| (1.0 - vn) * w[0][j] + vn * w[1][j]))
                .collect();
        }
        for index in &landmarks {
            let g = &guides[*index];
            let u = result.parameter(g.xy)?;
            let rows = result.rows(u, g.xy[1]);
            let old_z = baseline.evaluate(baseline.parameter(g.xy)?, g.xy[1]).0[2];
            let tolerance = (old_z - g.target).abs();
            for sign in [-1.0, 1.0] {
                bounds.push(constraint(
                    rows[0].map(|v| sign * v),
                    sign * g.target - tolerance,
                ));
            }
            energy(
                &mut h,
                &mut rhs,
                rows[0],
                g.target,
                1.0 / landmarks.len() as f64,
            );
            if let Some([nx, ny, nz]) = g.normal {
                if !valid_crest_normal([nx, ny, nz]) {
                    return Err("Invalid joint crest normal".into());
                }
                let a = result.radius(g.xy[1]);
                let da = dot(&result.radius_row(g.xy[1], 1), &radius);
                let theta = u * result.upper.angle;
                energy(
                    &mut h,
                    &mut rhs,
                    rows[1],
                    -nx * a * theta.cos() * result.upper.angle / nz,
                    1.0 / landmarks.len() as f64,
                );
                energy(
                    &mut h,
                    &mut rhs,
                    rows[2],
                    -(nx * da * theta.sin() + ny) / nz,
                    result.upper.span.powi(2) / landmarks.len() as f64,
                );
            }
        }
        for (u, y, upper) in material {
            let rows = result.rows(u, y);
            let scale = if upper { result.upper.span } else { l };
            let target = if upper {
                let v = (y - seam_y) / result.upper.span;
                baseline.seam[2]
                    + baseline.seam_derivative[2] * (y - seam_y)
                    + (baseline.seam[1] + baseline.seam_derivative[1] * (y - seam_y))
                        * (u * result.upper.angle).cos()
                    + baseline.prior_depth_correction * v * v * (u * result.upper.angle).cos()
            } else {
                dot(&rows[0], &old_depth)
            };
            energy(&mut h, &mut rhs, rows[0], target, 1.0 / (65.0 * 32.0));
            for (j, w) in [
                (3, result.upper.arc_scale.powi(-4)),
                (4, 1.0),
                (5, 2.0 / result.upper.arc_scale.powi(2)),
            ] {
                energy(
                    &mut h,
                    &mut rhs,
                    rows[j],
                    0.0,
                    (0.2 * scale).powi(4) * fairing * w / (65.0 * 32.0),
                );
            }
        }
        qp_dump(
            &format!("joint-depth-{iteration}-qp"),
            &h,
            &rhs,
            &eq,
            &bounds,
            serde_json::json!({"field":result.diagnostic_json(),"physical_samples":samples,"physical_bound_count":physical_bound_count,"outline_xy":outline,"landmarks":landmarks}),
        )?;
        let fitted = solve_dense_qp(&h, &rhs, &eq, &bounds, QpOptions::default())
            .map_err(|e| format!("joint depth QP: {e:?}"))?;
        result.lower_c.copy_from_slice(&fitted.coefficients[..4]);
        result.lower_b.copy_from_slice(&fitted.coefficients[4..8]);
        result.upper.depth_controls = fitted.coefficients[8..].to_vec();
        result.update_upper_seam();
        result.front_support = samples
            .iter()
            .filter_map(|s| {
                s.floor
                    .map(|floor| crate::breastplate_front_envelope::FrontSupport {
                        height_m: s.xy[1],
                        floor_m: floor,
                        origin: s.origin,
                    })
            })
            .collect();
        result.physical_support = samples
            .iter()
            .filter_map(|s| s.floor.map(|floor| (s.xy, floor)))
            .collect();
        eprintln!(
            "joint depth {iteration}: {:?}; fresh+inherited physical samples {}",
            fitted.diagnostics,
            samples.len()
        );
        dump(
            &format!("joint-result-{iteration}"),
            result.diagnostic_json(),
        )?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_horizontal_crest_graph_normal_is_rejected_before_division() {
        assert!(valid_crest_normal([0.2, 0.9, 0.3]));
        assert!(!valid_crest_normal([1.0, 0.0, 1e-12]));
        assert!(!valid_crest_normal([0.0, 0.0, 0.0]));
        assert!(!valid_crest_normal([f64::NAN, 0.0, 1.0]));
    }

    pub(super) fn field() -> JointProfiles {
        let mut field = JointProfiles {
            waist_y: 1.0,
            seam_y: 1.3,
            lower_a: [0.2, 0.21, 0.22, 0.23],
            lower_b: [0.15, 0.17, 0.18, 0.18],
            lower_c: [-0.03, -0.025, -0.02, -0.015],
            upper: GlobalUpperLoft {
                elliptical_depth: true,
                seam_y: 1.3,
                span: 0.2,
                angle: 1.48,
                arc_scale: 0.3,
                seam: [0.0; 3],
                seam_derivative: [0.0; 3],
                radius_correction: [0.0; 2],
                prior_depth_correction: 0.0,
                depth_controls: vec![0.0; 8],
            },
            side_trim: std::f32::consts::FRAC_PI_2 as f64 - 1.48,
            trim_start_y: 1.1,
            boundary_domain: Vec::new(),
            return_curve: JointReturn {
                seam_y: 1.3,
                upper_xy: [0.235, 1.42],
                upper_dxdy: -0.02,
                angle: 1.48,
                guide_start: 0,
                boundary_stations: Vec::new(),
            },
            resolved_return: None,
            fit_iteration: 0,
            whole_front: None,
            front_support: Vec::new(),
            physical_support: Vec::new(),
        };
        field.update_upper_seam();
        for (start, controls) in [(0, field.lower_c), (4, field.lower_b)] {
            field.upper.depth_controls[start] = 3.0
                * (controls[3] - 2.0 * controls[2] + controls[1])
                * (field.upper.span / field.lower_span()).powi(2);
        }
        field
    }

    #[test]
    fn joint_rows_are_actual_lower_and_upper_positions_with_cubic_coronal_curve() {
        let f = field();
        let coefficients: Row = std::array::from_fn(|j| match j {
            0..=3 => f.lower_c[j],
            4..=7 => f.lower_b[j - 4],
            _ => f.upper.depth_controls[j - 8],
        });
        for y in [1.0, 1.15, 1.299, 1.3, 1.301, 1.44] {
            for u in [0.0, 0.3, 0.8] {
                assert!(
                    (dot(&f.rows(u, y)[0], &coefficients) - f.evaluate(u, y).0[2]).abs() < 1e-13
                );
                // At the seam, angular second derivatives can jump purely
                // tangentially. A centered first difference is then O(h).
                let h = 1e-7;
                let numeric = (f.evaluate(u, y + h).0[2] - f.evaluate(u, y - h).0[2]) / (2.0 * h);
                if y > 1.0 {
                    assert!(
                        (numeric - dot(&f.rows(u, y)[2], &coefficients)).abs() < 1e-6,
                        "u={u} y={y} finite={numeric} analytic={}",
                        dot(&f.rows(u, y)[2], &coefficients)
                    );
                }
            }
        }
    }

    #[test]
    fn joint_normal_is_invariant_to_angular_trim_redistribution() {
        let f = field();
        let u = 0.73;
        let y = 1.21;
        let h = 1e-5;
        let du = std::array::from_fn::<_, 3, _>(|i| {
            (f.evaluate(u + h, y).0[i] - f.evaluate(u - h, y).0[i]) / (2.0 * h)
        });
        let dy = std::array::from_fn::<_, 3, _>(|i| {
            (f.evaluate(u, y + h).0[i] - f.evaluate(u, y - h).0[i]) / (2.0 * h)
        });
        let n = [
            du[1] * dy[2] - du[2] * dy[1],
            du[2] * dy[0] - du[0] * dy[2],
            du[0] * dy[1] - du[1] * dy[0],
        ];
        let norm = dot(&n, &n).sqrt();
        let actual = f.evaluate(u, y).1;
        for j in 0..3 {
            assert!((actual[j] - n[j] / norm).abs() < 1e-7);
        }
    }

    #[test]
    fn joint_c2_is_physical_graph_continuity_not_constant_angle_chart_second_derivative() {
        let f = field();
        let x = 0.14;
        let z = |y| {
            let a = if y <= f.seam_y {
                f.lower_eval(y)[0]
            } else {
                f.upper.radius(y).0
            };
            let angle = f.coverage(y)[0];
            f.evaluate((x / a).asin() / angle, y).0[2]
        };
        let h = 1e-4;
        let left = (z(f.seam_y) - 2.0 * z(f.seam_y - h) + z(f.seam_y - 2.0 * h)) / h.powi(2);
        let right = (z(f.seam_y + 2.0 * h) - 2.0 * z(f.seam_y + h) + z(f.seam_y)) / h.powi(2);
        assert!(
            (left - right).abs() < 0.03,
            "physical graph second derivatives {left} vs {right}"
        );
        let lo = f.lower_derivative(f.seam_y);
        for j in 0..3 {
            assert!((lo[j] - f.upper.seam_derivative[j]).abs() < 1e-14);
        }
    }

    #[test]
    fn centered_radius_ridge_preserves_an_already_affine_baseline() {
        let f = field();
        let old = f.radius_coefficients();
        let mut h = vec![vec![0.0; 6]; 6];
        let mut rhs = vec![0.0; 6];
        for j in 0..6 {
            h[j][j] = 1e-8;
            rhs[j] = 1e-8 * old[j];
        }
        for i in 0..129 {
            let y = f.waist_y + (f.lower_span() + f.upper.span) * i as f64 / 128.0;
            let row = f.radius_row(y, 0);
            energy(&mut h, &mut rhs, row, dot(&row, &old), 1.0 / 129.0);
            energy(
                &mut h,
                &mut rhs,
                f.radius_row(y, 2),
                0.0,
                (0.2 * (f.lower_span() + f.upper.span)).powi(4) / 129.0,
            );
        }
        let fit = solve_dense_qp(&h, &rhs, &[], &[], QpOptions::default()).unwrap();
        for (actual, expected) in fit.coefficients.iter().zip(old) {
            assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
        }
    }
}
