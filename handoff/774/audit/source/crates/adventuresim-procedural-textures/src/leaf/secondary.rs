//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{kernel::Kernel, math::*};
impl Kernel {
    pub(super) fn compound_local_secondary_hit(&self, p: Vec2) -> bool {
        let separation = self.topology.y;
        if separation < 0.30 {
            return false;
        }
        let px = 1.05 / self.render.y;
        if self.topology.x > 0.5 {
            self.radial_secondary_hit(p, separation, px)
        } else {
            self.pinnate_secondary_hit(p, separation, px)
        }
    }
    fn radial_secondary_hit(&self, p: Vec2, separation: f32, px: f32) -> bool {
        let count = clamp(self.organs.x, 1.0, 11.0);
        let base = self.axis_point(clamp(self.organ_style.x, 0.0, 0.35));
        for i in 0..11u32 {
            let fi = i as f32;
            let present = clamp(count - fi, 0.0, 1.0);
            let u = select(0.5, (fi + 0.5) / count, count > 1.0);
            let angle = mix(-self.organs.y, self.organs.y, u);
            let dir = normalize(Vec2::new(sin(angle), -cos(angle)));
            let center_weight = 1.0 - abs(2.0 * u - 1.0);
            let len = self.organs.z
                * mix(0.70, self.organ_style.y, center_weight)
                * (1.0 + 0.07 * sin(fi * 4.17 + self.render.z * 0.017));
            let width = self.organs.w;
            let blade_origin = base + dir * len * 0.075 * separation;
            let blade_length = len * (1.0 - 0.075 * separation);
            let organ_field = self.toothed_organ_field(
                p,
                blade_origin,
                dir,
                blade_length,
                width,
                fract(fi * 0.37),
            );
            if organ_field >= 0.0 && present > 0.0 {
                let normal = Vec2::new(-dir.y, dir.x);
                for j in 0..3u32 {
                    let along = (0.30 + (j as f32) * 0.18) * len;
                    let origin = base + dir * along;
                    let local_half = width
                        * pow(max(0.0, sin(core::f32::consts::PI * along / len)), 0.58)
                        * 0.78;
                    for s in 0..2u32 {
                        let side = select(-1.0, 1.0, s == 1u32);
                        let endpoint = origin + dir * len * 0.07 + normal * side * local_half;
                        if self.line_hit(
                            p,
                            origin,
                            endpoint,
                            max(px, self.vein_style.z * 0.50 * separation) * present,
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    fn pinnate_secondary_hit(&self, p: Vec2, separation: f32, px: f32) -> bool {
        let count = clamp(self.organs.x, 1.0, 12.0);
        for i in 0..12u32 {
            let fi = i as f32;
            let present = clamp(count - fi, 0.0, 1.0);
            let t = self.organ_position(fi, count, 0.13, 0.78);
            let base = self.axis_point(t);
            for side_index in 0..2u32 {
                let side = select(-1.0, 1.0, side_index == 1u32);
                let dir = normalize(Vec2::new(side * 0.72, -0.69 - self.lobe_grade.w));
                let normal = Vec2::new(-dir.y, dir.x);
                let scale = self.graded(t);
                let len = self.organs.z * scale;
                let width = self.organs.w * scale;
                let organ_field = self.toothed_organ_field(
                    p,
                    base,
                    dir,
                    len,
                    width,
                    fract(fi * 0.29 + side * 0.21),
                );
                if organ_field >= 0.0 && present > 0.0 {
                    for j in 0..2u32 {
                        let along = (0.36 + (j as f32) * 0.23) * len;
                        let origin = base + dir * along;
                        let local_half = width
                            * pow(max(0.0, sin(core::f32::consts::PI * along / len)), 0.58)
                            * 0.76;
                        for s in 0..2u32 {
                            let branch_side = select(-1.0, 1.0, s == 1u32);
                            let endpoint =
                                origin + dir * len * 0.065 + normal * branch_side * local_half;
                            if self.line_hit(
                                p,
                                origin,
                                endpoint,
                                max(px, self.vein_style.z * 0.48 * separation) * present,
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub(super) fn basal_primary_hit(&self, p: Vec2) -> bool {
        if self.notches.x <= 0.0 && self.landmarks.y <= 0.0 {
            return false;
        }
        let base = self.axis_point(0.03);
        let px = 1.05 / self.render.y;
        for level in 0..2u32 {
            let landmark_level = select(0.0, 1.0, level == 1u32 && self.landmarks.w > 0.0);
            if level == 1u32 && landmark_level <= 0.0 {
                continue;
            }
            for s in 0..2u32 {
                let side = select(-1.0, 1.0, s == 1u32);
                let lobe_presence =
                    clamp(self.landmarks.y * (1.0 + side * self.landmarks.z), 0.0, 1.0);
                let cordate_presence = select(0.0, 1.0, self.notches.x > 0.0 && level == 0u32);
                let presence = max(cordate_presence, lobe_presence);
                let landmark_t = mix(0.43, 0.68, landmark_level);
                let target_t = mix(0.22, landmark_t, select(0.0, 1.0, self.landmarks.y > 0.0));
                let endpoint = self.axis_point(target_t)
                    + Vec2::new(side * self.half_width(target_t, side) * 0.90, 0.0);
                let growth = smoothstep(0.0, 0.38, presence);
                if presence * self.vein_style.z * self.render.y >= 0.75
                    && self.line_hit(
                        p,
                        base,
                        mix(base, endpoint, growth),
                        max(px, self.vein_style.z * presence),
                    )
                {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn parallel_hit(&self, p: Vec2) -> bool {
        let amount = self.topology.w;
        if amount <= 0.0 {
            return false;
        }
        let count = clamp(self.veins.x, 1.0, 16.0);
        let px = 1.05 / self.render.y;
        for i in 0..16u32 {
            let fi = i as f32;
            let present = clamp(count - fi, 0.0, 1.0);
            let u = (fi + 0.5) / count;
            let offset = (u * 2.0 - 1.0) * self.profile.y * 0.72;
            let a = self.axis_point(0.08) + Vec2::new(offset * 0.25, 0.0);
            let b = self.axis_point(0.93) + Vec2::new(offset, 0.0);
            if self.line_hit(p, a, b, max(px, self.vein_style.z * 0.7) * present * amount) {
                return true;
            }
        }
        false
    }
}
