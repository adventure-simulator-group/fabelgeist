//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{kernel::Kernel, math::*};
impl Kernel {
    pub(super) fn distance_segment(&self, p: Vec2, a: Vec2, b: Vec2) -> f32 {
        let d = b - a;
        length(p - (a + d * clamp(dot(p - a, d) / max(dot(d, d), 1e-7), 0.0, 1.0)))
    }
    pub(super) fn bezier(&self, a: Vec2, b: Vec2, c: Vec2, t: f32) -> Vec2 {
        mix(mix(a, b, t), mix(b, c, t), t)
    }
    pub(super) fn line_hit(&self, p: Vec2, a: Vec2, b: Vec2, width_: f32) -> bool {
        self.distance_segment(p, a, b) < width_
    }
    pub(super) fn axis_hit(&self, p: Vec2, start_t: f32, end_t: f32, width_: f32) -> bool {
        for segment in 0..16u32 {
            let u0 = (segment as f32) / 16.0;
            let u1 = ((segment + 1u32) as f32) / 16.0;
            if self.line_hit(
                p,
                self.axis_point(mix(start_t, end_t, u0)),
                self.axis_point(mix(start_t, end_t, u1)),
                width_,
            ) {
                return true;
            }
        }
        false
    }

    pub(super) fn secondary_hit(&self, p: Vec2) -> bool {
        let count = clamp(self.veins.x, 0.0, 16.0);
        let px = 1.05 / self.render.y;
        for i in 0..16u32 {
            let fi = i as f32;
            let present = clamp(count - fi, 0.0, 1.0);
            let base_t = self.organ_position(fi, count, self.veins.y, self.veins.z);
            for s in 0..2u32 {
                let side = select(-1.0, 1.0, s == 1u32);
                let alternating = self.venation.x * 0.45 / max(count, 1.0)
                    * select(-1.0, 1.0, (i + s) % 2u32 == 0u32);
                let t = clamp(base_t + alternating, 0.02, 0.97);
                let origin = mix(self.axis_point(t), self.axis_point(0.04), self.topology.x);
                let coupled = abs(self.teeth.x - count) < 0.75 && self.teeth.y > 0.0;
                let tooth_target = (fi + self.teeth.w) / max(self.teeth.x, 1.0);
                let target_t = select(
                    clamp(
                        t + self.vein_style.x + self.lobe_grade.w * (t - 0.5),
                        0.02,
                        0.98,
                    ),
                    clamp(tooth_target, 0.02, 0.98),
                    coupled,
                );
                let scale = 1.0 + self.venation.y * (self.graded(t) - 1.0);
                let endpoint = self.axis_point(target_t)
                    + Vec2::new(
                        side * self.half_width(target_t, side) * self.veins.w * scale,
                        0.0,
                    );
                let control = mix(origin, endpoint, 0.5)
                    + Vec2::new(0.0, -self.vein_style.y * self.profile.x);
                for j in 0..9u32 {
                    let u0 = (j as f32) / 9.0;
                    let u1 = ((j + 1u32) as f32) / 9.0;
                    let a = self.bezier(origin, control, endpoint, u0);
                    let b = self.bezier(origin, control, endpoint, u1);
                    let secondary_presence = 1.0 - clamp(self.topology.x, 0.0, 1.0);
                    let width = max(
                        px,
                        self.vein_style.z
                            * mix(1.0, self.vein_style.w, (u0 + u1) * 0.5)
                            * secondary_presence,
                    ) * present;
                    if self.line_hit(p, a, b, width) {
                        return true;
                    }
                    if self.venation.z > 0.0 && j > 4u32 {
                        let branch_end = b + Vec2::new(side * 0.055, -0.035) * self.venation.z;
                        if self.line_hit(p, b, branch_end, width * 0.62) {
                            return true;
                        }
                    }
                    if self.venation.w > 0.0 && j == 8u32 {
                        let loop_end = endpoint + Vec2::new(0.0, -0.09) * self.venation.w;
                        if self.line_hit(p, endpoint, loop_end, width * 0.7) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
