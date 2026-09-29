//! SVG elliptical arcs, following `svgpathtools.path.Arc`.
//!
//! GarmentCode only ever builds *circular* arcs (`rx == ry`, `rotation == 0`),
//! but the endpoint-to-centre parametrisation is kept general so that arcs
//! loaded from SVG files behave the same way.

use super::bezier;
use crate::math::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    pub start: V2,
    pub end: V2,
    /// `(rx, ry)`.
    pub radius: V2,
    /// X-axis rotation, in degrees.
    pub rotation: f64,
    pub large_arc: bool,
    pub sweep: bool,

    // Derived (centre parametrisation).
    pub center: V2,
    /// Start angle, in degrees.
    pub theta: f64,
    /// Swept angle, in degrees.
    pub delta: f64,
}

fn cmul(a: V2, b: V2) -> V2 {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}

fn cdiv(a: V2, b: V2) -> V2 {
    let d = b[0] * b[0] + b[1] * b[1];
    [
        (a[0] * b[0] + a[1] * b[1]) / d,
        (a[1] * b[0] - a[0] * b[1]) / d,
    ]
}

impl Arc {
    pub fn new(
        start: V2,
        radius: V2,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        end: V2,
    ) -> Self {
        let mut arc = Arc {
            start,
            end,
            radius,
            rotation,
            large_arc,
            sweep,
            center: [0.0, 0.0],
            theta: 0.0,
            delta: 0.0,
        };
        arc.parametrize();
        arc
    }

    /// `Arc._parameterize` -- endpoint to centre parametrisation.
    fn parametrize(&mut self) {
        let phi = self.rotation.to_radians();
        let rot_matrix = [phi.cos(), phi.sin()];

        let mut rx = self.radius[0].abs();
        let mut ry = self.radius[1].abs();

        let zp1 = cdiv(scale2(sub2(self.start, self.end), 0.5), rot_matrix);
        let (x1p, y1p) = (zp1[0], zp1[1]);
        let (x1p_sqd, y1p_sqd) = (x1p * x1p, y1p * y1p);

        // Grow out-of-range radii, as `autoscale_radius=True` does.
        let radius_check = x1p_sqd / (rx * rx) + y1p_sqd / (ry * ry);
        if radius_check > 1.0 {
            rx *= radius_check.sqrt();
            ry *= radius_check.sqrt();
            self.radius = [rx, ry];
        }
        let (rx_sqd, ry_sqd) = (rx * rx, ry * ry);

        let tmp = rx_sqd * y1p_sqd + ry_sqd * x1p_sqd;
        let radicand = (rx_sqd * ry_sqd - tmp) / tmp;
        let radical = if radicand.abs() < 1e-8 || radicand < 0.0 {
            0.0
        } else {
            radicand.sqrt()
        };

        let base = [rx * y1p / ry, -ry * x1p / rx];
        let cp = if self.large_arc == self.sweep {
            scale2(base, -radical)
        } else {
            scale2(base, radical)
        };

        self.center = add2(
            cmul(rot_matrix, cp),
            scale2(add2(self.start, self.end), 0.5),
        );

        let u1 = [
            ((x1p - cp[0]) / rx).clamp(-1.0, 1.0),
            ((y1p - cp[1]) / ry).clamp(-1.0, 1.0),
        ];
        let u2 = [
            ((-x1p - cp[0]) / rx).clamp(-1.0, 1.0),
            ((-y1p - cp[1]) / ry).clamp(-1.0, 1.0),
        ];

        self.theta = if u1[1] > 0.0 {
            u1[0].acos().to_degrees()
        } else if u1[1] < 0.0 {
            -u1[0].acos().to_degrees()
        } else if u1[0] > 0.0 {
            0.0
        } else {
            180.0
        };

        let det_uv = u1[0] * u2[1] - u1[1] * u2[0];
        let acosand = (u1[0] * u2[0] + u1[1] * u2[1]).clamp(-1.0, 1.0);

        self.delta = if det_uv > 0.0 {
            acosand.acos().to_degrees()
        } else if det_uv < 0.0 {
            -acosand.acos().to_degrees()
        } else if u1[0] * u2[0] + u1[1] * u2[1] > 0.0 {
            0.0
        } else {
            180.0
        };

        if !self.sweep && self.delta >= 0.0 {
            self.delta -= 360.0;
        } else if self.large_arc && self.delta <= 0.0 {
            self.delta += 360.0;
        }
    }

    fn is_circular(&self) -> bool {
        (self.radius[0] - self.radius[1]).abs() < 1e-12
    }

    pub fn point(&self, t: f64) -> V2 {
        let angle = (self.theta + t * self.delta).to_radians();
        let phi = self.rotation.to_radians();
        let (sinphi, cosphi) = phi.sin_cos();
        let (rx, ry) = (self.radius[0], self.radius[1]);
        let (sina, cosa) = angle.sin_cos();
        [
            rx * cosphi * cosa - ry * sinphi * sina + self.center[0],
            rx * sinphi * cosa + ry * cosphi * sina + self.center[1],
        ]
    }

    pub fn derivative(&self, t: f64) -> V2 {
        let angle = (self.theta + t * self.delta).to_radians();
        let phi = self.rotation.to_radians();
        let (sinphi, cosphi) = phi.sin_cos();
        let (rx, ry) = (self.radius[0], self.radius[1]);
        let k = self.delta.to_radians();
        let (sina, cosa) = angle.sin_cos();
        [
            k * (-rx * cosphi * sina - ry * sinphi * cosa),
            k * (-rx * sinphi * sina + ry * cosphi * cosa),
        ]
    }

    pub fn second_derivative(&self, t: f64) -> V2 {
        let angle = (self.theta + t * self.delta).to_radians();
        let phi = self.rotation.to_radians();
        let (sinphi, cosphi) = phi.sin_cos();
        let (rx, ry) = (self.radius[0], self.radius[1]);
        let k = self.delta.to_radians().powi(2);
        let (sina, cosa) = angle.sin_cos();
        [
            k * (-rx * cosphi * cosa + ry * sinphi * sina),
            k * (-rx * sinphi * cosa - ry * cosphi * sina),
        ]
    }

    pub fn length_range(&self, t0: f64, t1: f64) -> f64 {
        if self.is_circular() {
            // Constant speed -- the numerical integral in the reference lands
            // on exactly this value.
            return self.radius[0] * self.delta.to_radians().abs() * (t1 - t0);
        }
        bezier::adaptive_length(|t| self.derivative(t), t0, t1)
    }

    pub fn cropped(&self, t0: f64, t1: f64) -> Arc {
        let new_large_arc = (self.delta * (t1 - t0)).abs() > 180.0;
        Arc::new(
            self.point(t0),
            self.radius,
            self.rotation,
            new_large_arc,
            self.sweep,
            self.point(t1),
        )
    }

    pub fn reversed(&self) -> Arc {
        Arc::new(
            self.end,
            self.radius,
            self.rotation,
            self.large_arc,
            !self.sweep,
            self.start,
        )
    }

    pub fn map_points<F: Fn(V2) -> V2>(&self, f: F) -> Arc {
        // Only rigid transforms and uniform scaling are used on arcs here, so
        // the radii transform by the same scale factor.
        let s0 = f(self.start);
        let s1 = f(self.end);
        let scale = dist2(s1, s0) / dist2(self.end, self.start).max(1e-300);
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        Arc::new(
            s0,
            scale2(self.radius, scale),
            self.rotation,
            self.large_arc,
            self.sweep,
            s1,
        )
    }

    /// `(xmin, xmax, ymin, ymax)`.
    pub fn bbox(&self) -> [f64; 4] {
        let mut xs = vec![self.start[0], self.end[0]];
        let mut ys = vec![self.start[1], self.end[1]];

        // Axis-aligned extrema of the (possibly rotated) ellipse, kept only
        // when they fall inside the swept angular range.
        let phi = self.rotation.to_radians();
        let (rx, ry) = (self.radius[0], self.radius[1]);

        let t_x = (-ry * phi.tan() / rx).atan();
        let t_y = if phi.tan().abs() < 1e-12 {
            std::f64::consts::FRAC_PI_2
        } else {
            (ry / (rx * phi.tan())).atan()
        };

        for base in [t_x, t_x + std::f64::consts::PI] {
            if let Some(t) = self.angle_to_t(base.to_degrees()) {
                xs.push(self.point(t)[0]);
            }
        }
        for base in [t_y, t_y + std::f64::consts::PI] {
            if let Some(t) = self.angle_to_t(base.to_degrees()) {
                ys.push(self.point(t)[1]);
            }
        }

        [
            xs.iter().cloned().fold(f64::INFINITY, f64::min),
            xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            ys.iter().cloned().fold(f64::INFINITY, f64::min),
            ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ]
    }

    /// Map an ellipse angle (degrees) to the arc parameter `t`, if that angle
    /// is actually swept.
    fn angle_to_t(&self, angle_deg: f64) -> Option<f64> {
        if self.delta == 0.0 {
            return None;
        }
        // Try every 360-degree shift that could land in range.
        for k in -2..=2 {
            let a = angle_deg + 360.0 * k as f64;
            let t = (a - self.theta) / self.delta;
            if (0.0..=1.0).contains(&t) {
                return Some(t);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semicircle_geometry() {
        // Unit semicircle from (-1, 0) to (1, 0), bulging upwards.
        let a = Arc::new([-1.0, 0.0], [1.0, 1.0], 0.0, false, false, [1.0, 0.0]);
        assert!((a.center[0]).abs() < 1e-12, "{:?}", a.center);
        assert!((a.center[1]).abs() < 1e-12, "{:?}", a.center);
        assert!(
            (a.length_range(0.0, 1.0) - std::f64::consts::PI).abs() < 1e-12,
            "{}",
            a.length_range(0.0, 1.0)
        );
        let mid = a.point(0.5);
        assert!(
            (mid[0]).abs() < 1e-12 && (mid[1] - 1.0).abs() < 1e-12,
            "{mid:?}"
        );
    }

    #[test]
    fn cropped_stays_on_the_circle() {
        let a = Arc::new([-1.0, 0.0], [1.0, 1.0], 0.0, false, false, [1.0, 0.0]);
        let c = a.cropped(0.25, 0.75);
        assert!(dist2(c.start, a.point(0.25)) < 1e-12);
        for i in 0..=10 {
            let p = c.point(i as f64 / 10.0);
            assert!((norm2(sub2(p, a.center)) - 1.0).abs() < 1e-9, "{p:?}");
        }
    }

    #[test]
    fn reversed_swaps_ends() {
        let a = Arc::new([-1.0, 0.0], [1.0, 1.0], 0.0, false, false, [1.0, 0.0]);
        let r = a.reversed();
        assert!(dist2(r.start, a.end) < 1e-12);
        assert!(dist2(r.point(0.5), a.point(0.5)) < 1e-9);
    }

    #[test]
    fn bbox_of_semicircle() {
        let a = Arc::new([-1.0, 0.0], [1.0, 1.0], 0.0, false, false, [1.0, 0.0]);
        let b = a.bbox();
        assert!((b[0] + 1.0).abs() < 1e-9, "{b:?}");
        assert!((b[1] - 1.0).abs() < 1e-9, "{b:?}");
        assert!((b[2] - 0.0).abs() < 1e-9, "{b:?}");
        assert!((b[3] - 1.0).abs() < 1e-9, "{b:?}");
    }
}
