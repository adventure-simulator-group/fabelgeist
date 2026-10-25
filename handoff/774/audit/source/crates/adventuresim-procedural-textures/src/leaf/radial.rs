//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{kernel::Kernel, math::*};
impl Kernel {
    pub(super) fn radial_vein_hit(&self, p: Vec2) -> bool {
        let count = clamp(self.organs.x, 1.0, 11.0);
        let base = self.axis_point(clamp(self.organ_style.x + self.planar.z, 0.0, 0.55));
        let px = 1.05 / self.render.y;
        if self.line_hit(p, self.axis_point(0.0), base, max(px, self.axis.w)) {
            return true;
        }
        for i in 0..11u32 {
            let fi = i as f32;
            let present = clamp(count - fi, 0.0, 1.0);
            let u = select(0.5, clamp((fi + 0.5) / count, 0.0, 1.0), count > 1.0);
            let angle = mix(
                mix(-self.organs.y, self.organs.y, u),
                mix(-core::f32::consts::PI, core::f32::consts::PI, u),
                clamp(self.planar.z * 2.0, 0.0, 1.0),
            );
            let dir = normalize(Vec2::new(sin(angle), -cos(angle)));
            let center_weight = 1.0 - abs(2.0 * u - 1.0);
            let len = self.organs.z
                * mix(0.70, self.organ_style.y, center_weight)
                * (1.0 + 0.07 * sin(fi * 4.17 + self.render.z * 0.017));
            let owner_width = self.organs.w
                * mix(1.8, 1.0, self.topology.y)
                * mix(1.0, 0.42, self.planar.y)
                * present;
            if owner_width * self.render.y < 2.5 {
                continue;
            }
            let dichotomy = self.organ_style.w;
            let first_fork = mix(1.0, 0.36 + 0.16 * fract(fi * 0.618 + 0.21), dichotomy);
            let trunk_end = base + dir * len * first_fork * present;
            let radial_presence = clamp(self.topology.x, 0.0, 1.0);
            let peltate_scale = select(1.0, 0.28, self.planar.z > 0.0 || self.organ_style.z > 0.5);
            if radial_presence * self.vein_style.z * peltate_scale * self.render.y >= 0.50
                && self.line_hit(
                    p,
                    base,
                    trunk_end,
                    max(
                        px * 0.72,
                        self.vein_style.z * radial_presence * peltate_scale,
                    ) * present,
                )
            {
                return true;
            }
            if self.organ_style.w > 0.0 {
                let normal = Vec2::new(-dir.y, dir.x);
                for branch in 0..2u32 {
                    let bs = select(-1.0, 1.0, branch == 1u32);
                    let first_end =
                        base + (dir * len * 0.72 + normal * bs * len * 0.11 * dichotomy) * present;
                    if self.line_hit(
                        p,
                        trunk_end,
                        first_end,
                        max(px, self.vein_style.z * 0.72) * present,
                    ) {
                        return true;
                    }
                    let second_fraction = 0.67 + 0.12 * fract(fi * 0.414 + (branch as f32) * 0.33);
                    let second_origin = mix(trunk_end, first_end, second_fraction);
                    for twig in 0..2u32 {
                        let ts = select(-1.0, 1.0, twig == 1u32);
                        let second_end = first_end
                            + (dir * len * 0.20
                                + normal * (bs * 0.055 + ts * 0.045) * len * dichotomy)
                                * present;
                        if self.line_hit(
                            p,
                            second_origin,
                            second_end,
                            max(px, self.vein_style.z * 0.46) * present,
                        ) {
                            return true;
                        }
                    }
                }
            } else {
                if self.line_hit(
                    p,
                    trunk_end,
                    base + dir * len * present,
                    max(px, self.vein_style.z) * present,
                ) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn pinnate_organ_vein_hit(&self, p: Vec2) -> bool {
        let count = clamp(self.organs.x, 1.0, 12.0);
        let px = 1.05 / self.render.y;
        for i in 0..12u32 {
            let fi = i as f32;
            let present = clamp(count - fi, 0.0, 1.0);
            let t = self.organ_position(fi, count, 0.13, 0.78);
            let origin = self.axis_point(t);
            for s in 0..2u32 {
                let side = select(-1.0, 1.0, s == 1u32);
                let enabled = select(
                    1.0,
                    clamp(count - fi - 0.5, 0.0, 1.0),
                    self.organ_style.z > 0.5 && s == 1u32,
                );
                let dir = normalize(mix(
                    Vec2::new(side * 0.72, -0.69 - self.lobe_grade.w),
                    Vec2::new(side * 0.96, -0.28 - self.lobe_grade.w),
                    self.planar.x,
                ));
                let scale = self.graded(t)
                    * (1.0 + 0.055 * sin(fi * 5.13 + side * 1.7 + self.render.z * 0.013));
                let organ_endpoint = origin + dir * self.organs.z * scale;
                let simple_t = clamp(t + self.vein_style.x, 0.02, 0.98);
                let simple_endpoint = self.axis_point(simple_t)
                    + Vec2::new(side * self.half_width(simple_t, side) * self.veins.w, 0.0);
                let mature_endpoint = mix(simple_endpoint, organ_endpoint, self.topology.y);
                let growth = present * enabled;
                let endpoint = mix(origin, mature_endpoint, growth);
                if self.organs.w * scale * growth * self.render.y < 2.5 {
                    continue;
                }
                if self.line_hit(p, origin, endpoint, max(px, self.vein_style.z * growth)) {
                    return true;
                }
                if self.planar.w > 0.0 {
                    let owner = self.toothed_organ_field(
                        p,
                        origin,
                        dir,
                        self.organs.z * scale * growth,
                        self.organs.w * scale * growth,
                        fract(fi * 0.29 + side * 0.21),
                    );
                    let normal = Vec2::new(-dir.y, dir.x);
                    let offset = self.organs.w * scale * 0.28;
                    if owner >= px * 1.5
                        && self.line_hit(
                            p,
                            origin + normal * offset * growth,
                            endpoint + normal * offset * growth,
                            max(px, self.vein_style.z * 0.45 * growth) * self.planar.w,
                        )
                    {
                        return true;
                    }
                    if owner >= px * 1.5
                        && self.line_hit(
                            p,
                            origin - normal * offset * growth,
                            endpoint - normal * offset * growth,
                            max(px, self.vein_style.z * 0.45 * growth) * self.planar.w,
                        )
                    {
                        return true;
                    }
                }
            }
        }
        let terminal_origin = self.axis_point(0.80);
        let terminal_end =
            terminal_origin + Vec2::new(0.0, -self.organs.z * self.organ_style.y * self.topology.y);
        self.line_hit(p, terminal_origin, terminal_end, max(px, self.vein_style.z))
    }
}
