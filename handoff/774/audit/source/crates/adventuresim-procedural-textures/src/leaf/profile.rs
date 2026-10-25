//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{kernel::Kernel, math::*};
impl Kernel {
    pub(super) fn axis_x(&self, t: f32) -> f32 {
        self.axis.x * sin(core::f32::consts::PI * t)
    }
    pub(super) fn axis_point(&self, t: f32) -> Vec2 {
        Vec2::new(self.axis_x(t), self.profile.x * (1.0 - 2.0 * t))
    }
    pub(super) fn end_fade(&self, t: f32) -> f32 {
        smoothstep(0.025, 0.13, t) * smoothstep(0.025, 0.13, 1.0 - t)
    }

    pub(super) fn profile_width(&self, t: f32) -> f32 {
        let w = clamp(self.profile.z, 0.03, 0.97);
        if t < w {
            let u = clamp(t / w, 0.0, 1.0);
            return mix(
                self.shape.y,
                1.0,
                pow(sin(core::f32::consts::FRAC_PI_2 * u), self.profile.w),
            );
        }
        let u = clamp((1.0 - t) / (1.0 - w), 0.0, 1.0);
        mix(
            self.shape.z,
            1.0,
            pow(sin(core::f32::consts::FRAC_PI_2 * u), self.shape.x),
        )
    }

    pub(super) fn graded(&self, t: f32) -> f32 {
        if t < 0.5 {
            return mix(
                self.lobe_grade.x,
                self.lobe_grade.y,
                smoothstep(0.0, 0.5, t),
            );
        }
        mix(
            self.lobe_grade.y,
            self.lobe_grade.z,
            smoothstep(0.5, 1.0, t),
        )
    }

    pub(super) fn organ_position(&self, index: f32, count: f32, lo: f32, hi: f32) -> f32 {
        let organic_offset = 0.065 * sin(index * 12.9898 + self.render.z * 0.071) / max(count, 1.0);
        mix(
            lo,
            hi,
            clamp((index + 0.5) / max(count, 1.0) + organic_offset, 0.0, 1.0),
        )
    }

    pub(super) fn lobe_signal(&self, t: f32, side: f32) -> f32 {
        let count = clamp(self.lobes.x, 0.0, 16.0);
        let mut signal = 0.0;
        for i in 0..16u32 {
            let fi = i as f32;
            let presence = clamp(count - fi, 0.0, 1.0);
            let c =
                self.organ_position(fi, count, 0.10, 0.91) + side * self.lobes.w / max(count, 1.0);
            let spacing = 0.81 / max(count, 1.0);
            let d = abs(t - c) / max(spacing * 0.62, 0.001);
            let rounded = pow(max(0.0, 1.0 - d * d), max(0.08, self.lobes.z));
            signal = max(signal, presence * rounded * self.graded(c));
        }
        signal
    }

    pub(super) fn skew_triangle(&self, x: f32, peak: f32) -> f32 {
        let p = clamp(peak, 0.05, 0.95);
        select((1.0 - x) / (1.0 - p), x / p, x < p)
    }

    pub(super) fn tooth_signal(&self, t: f32, side: f32) -> f32 {
        let count = self.teeth.x;
        if count <= 0.0 {
            return 0.0;
        }
        let hierarchy = max(1.0, self.margin_style.x);
        let base_phase = fract(count * t + select(0.0, 0.5, side > 0.0));
        let primary = pow(
            max(0.0, self.skew_triangle(base_phase, self.teeth.w)),
            max(0.05, self.teeth.z),
        );
        let fine_phase = fract(count * hierarchy * t + 0.37 + select(0.0, 0.5, side < 0.0));
        let fine = pow(
            max(0.0, self.skew_triangle(fine_phase, self.teeth.w)),
            max(0.05, self.teeth.z),
        );
        mix(
            primary,
            max(primary, 0.48 * fine),
            clamp(self.margin_style.y, 0.0, 1.0),
        )
    }

    pub(super) fn half_width(&self, t: f32, side: f32) -> f32 {
        let base = self.profile.y * self.profile_width(t);
        let asym = self.shape.w * sin(core::f32::consts::PI * t)
            + self.margin_style.z * (2.0 * t - 1.0) * 0.15;
        let lobe = (1.0 - self.lobes.y * (1.0 - clamp(self.lobe_signal(t, side), 0.0, 1.5)))
            * self.end_fade(t)
            + (1.0 - self.end_fade(t));
        let teeth = 1.0 + self.teeth.y * self.tooth_signal(t, side) * self.end_fade(t);
        let bristle = self.margin_style.w * pow(self.lobe_signal(t, side), 8.0);
        let basal_oblique = self.landmarks.x * side * pow(max(0.0, 1.0 - t), 3.0);
        let landmark_a = pow(max(0.0, 1.0 - abs(t - 0.43) / 0.13), 1.4);
        let landmark_b = pow(max(0.0, 1.0 - abs(t - 0.68) / 0.12), 1.4);
        let side_weight = clamp(1.0 + side * self.landmarks.z, 0.0, 2.0);
        let use_two_landmarks = select(0.0, 1.0, self.landmarks.w > 0.0);
        let landmark = base
            * self.landmarks.y
            * side_weight
            * mix(landmark_a, max(landmark_a, landmark_b), use_two_landmarks);
        max(
            0.0,
            base * (1.0 - side * asym) * lobe * teeth + bristle + landmark + basal_oblique,
        )
    }

    pub(super) fn simple_field(&self, p: Vec2) -> f32 {
        let t = (self.profile.x - p.y) / (2.0 * self.profile.x);
        if t < 0.0 || t > 1.0 {
            return -1.0;
        }
        let x = p.x - self.axis_x(t);
        let side = select(-1.0, 1.0, x >= 0.0);
        let mut field = self.half_width(t, side) - abs(x);
        if self.landmarks.w > 0.0 && t > 1.0 - self.landmarks.w {
            let shoulder = self.half_width(1.0 - self.landmarks.w, side);
            let distal_u = (t - (1.0 - self.landmarks.w)) / self.landmarks.w;
            let plateau = shoulder * mix(1.0, 0.86, smoothstep(0.0, 1.0, distal_u));
            field = plateau - abs(x);
        }
        if self.notches.x > 0.0 && t < self.notches.x {
            let curved_opening = self.notches.y * pow(max(0.0, 1.0 - t / self.notches.x), 0.58);
            field = min(field, abs(x) - curved_opening);
        }
        if self.notches.z > 0.0 && t > 1.0 - self.notches.z {
            field = min(
                field,
                abs(x) - self.notches.w * (1.0 - (1.0 - t) / self.notches.z),
            );
        }
        field
    }

    pub(super) fn tapered_organ_field(
        &self,
        p: Vec2,
        origin: Vec2,
        dir: Vec2,
        length_: f32,
        width_: f32,
    ) -> f32 {
        let q = p - origin;
        let along = dot(q, dir);
        let across = abs(dot(q, Vec2::new(-dir.y, dir.x)));
        let u = along / max(length_, 0.001);
        if u < 0.0 || u > 1.0 {
            return -1.0;
        }
        let taper = pow(max(0.0, sin(core::f32::consts::PI * u)), 0.58);
        let basal_width = mix(0.10, 0.0, clamp(self.topology.y, 0.0, 1.0));
        let separated_base = smoothstep(0.035, 0.28, u);
        let local_width =
            width_ * mix(basal_width, 1.0, taper) * mix(1.0, separated_base, self.topology.y);
        local_width - across
    }

    pub(super) fn toothed_organ_field(
        &self,
        p: Vec2,
        origin: Vec2,
        dir: Vec2,
        length_: f32,
        width_: f32,
        phase: f32,
    ) -> f32 {
        let q = p - origin;
        let along = dot(q, dir);
        let across = abs(dot(q, Vec2::new(-dir.y, dir.x)));
        let u = along / max(length_, 0.001);
        if u < 0.0 || u > 1.0 {
            return -1.0;
        }
        let taper = pow(max(0.0, sin(core::f32::consts::PI * u)), 0.58);
        let margin_tooth = pow(
            max(
                0.0,
                self.skew_triangle(fract(self.teeth.x * u + phase), self.teeth.w),
            ),
            max(0.05, self.teeth.z),
        );
        let basal_width = mix(0.10, 0.0, clamp(self.topology.y, 0.0, 1.0));
        let separated_base = smoothstep(0.035, 0.28, u);
        let local_width = width_
            * mix(basal_width, 1.0, taper)
            * mix(1.0, separated_base, self.topology.y)
            * (1.0 + self.teeth.y * margin_tooth);
        local_width - across
    }
}
