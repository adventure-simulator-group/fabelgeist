//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{kernel::Kernel, math::*};
impl Kernel {
    pub(super) fn bipinnate_vein_hit(&self, p: Vec2) -> bool {
        if self.hierarchy.y <= 0.0 {
            return false;
        }
        let pinnae = clamp(self.hierarchy.y, 1.0, 6.0);
        let leaflets = clamp(self.hierarchy.z, 1.0, 10.0);
        let px = 1.05 / self.render.y;
        if self.axis_hit(p, 0.0, 0.84, max(px * 1.5, self.axis.w)) {
            return true;
        }
        for i in 0..6u32 {
            let fi = i as f32;
            let pp = clamp(pinnae - fi, 0.0, 1.0);
            let t = self.organ_position(fi, pinnae, 0.16, 0.76);
            let base = self.axis_point(t);
            if pp <= 0.0 {
                continue;
            }
            for ps in 0..2u32 {
                let side = select(-1.0, 1.0, ps == 1u32);
                let dir = normalize(Vec2::new(side * 0.88, -0.48));
                let plen = self.hierarchy.w * self.graded(t);
                let emergence = smoothstep(0.08, 0.82, self.hierarchy.x);
                let owner_width = max(
                    self.organs.w * 2.2 * (1.0 - self.hierarchy.x),
                    self.organs.w * emergence,
                ) * pp;
                if owner_width * self.render.y < 3.0 {
                    continue;
                }
                let occupied_fraction = clamp((leaflets - 0.45) / max(leaflets, 1.0), 0.25, 0.96);
                let end = base + dir * plen * occupied_fraction;
                if self.line_hit(
                    p,
                    base,
                    mix(base, end, pp),
                    max(px * 1.35, self.vein_style.z * pp),
                ) {
                    return true;
                }
                let normal = Vec2::new(-dir.y, dir.x);
                for j in 0..10u32 {
                    let fj = j as f32;
                    let lp = clamp(leaflets - fj, 0.0, 1.0);
                    let origin = mix(
                        base,
                        base + dir * plen * ((fj + 0.45) / max(leaflets, 1.0)),
                        pp,
                    );
                    if lp <= 0.0 {
                        continue;
                    }
                    let presence = pp * lp * emergence;
                    if self.organs.w * presence * self.render.y < 3.0 {
                        continue;
                    }
                    for ls in 0..2u32 {
                        let lside = select(-1.0, 1.0, ls == 1u32);
                        let ldir = normalize(dir * 0.35 + normal * lside);
                        if self.line_hit(
                            p,
                            origin,
                            origin + ldir * self.organs.z * 0.92 * presence,
                            max(px, self.vein_style.z * 0.6 * presence),
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
