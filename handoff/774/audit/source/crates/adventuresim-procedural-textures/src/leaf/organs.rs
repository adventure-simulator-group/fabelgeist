//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{kernel::Kernel, math::*};
impl Kernel {
    pub(super) fn radial_field(&self, p: Vec2, separate: f32) -> f32 {
        let count = clamp(self.organs.x, 1.0, 11.0);
        let base = self.axis_point(clamp(self.organ_style.x + self.planar.z, 0.0, 0.55));
        let mut field = -10.0;
        for i in 0..11u32 {
            let fi = i as f32;
            let presence = clamp(count - fi, 0.0, 1.0);
            let u = select(0.5, clamp((fi + 0.5) / count, 0.0, 1.0), count > 1.0);
            let angle = mix(
                mix(-self.organs.y, self.organs.y, u),
                mix(-core::f32::consts::PI, core::f32::consts::PI, u),
                clamp(self.planar.z * 2.0, 0.0, 1.0),
            );
            let dir = normalize(Vec2::new(sin(angle), -cos(angle)));
            let center_weight = 1.0 - abs(2.0 * u - 1.0);
            let ordinal_wave = 1.0 + 0.07 * sin(fi * 4.17 + self.render.z * 0.017);
            let len = self.organs.z * mix(0.70, self.organ_style.y, center_weight) * ordinal_wave;
            let blade_origin = base + dir * len * 0.075 * separate;
            let blade_length = len * (1.0 - 0.075 * separate);
            let segment_width =
                self.organs.w * mix(1.8, 1.0, separate) * mix(1.0, 0.42, self.planar.y);

            if segment_width * presence * self.render.y >= 2.5 {
                let growing_origin = mix(base, blade_origin, presence);
                let ef = self.toothed_organ_field(
                    p,
                    growing_origin,
                    dir,
                    blade_length * presence,
                    segment_width * presence,
                    fract(fi * 0.37),
                );
                field = max(field, ef);
            }
        }

        let web = self.simple_field(p) - separate * mix(0.0, 0.62, self.organs.y / 1.5);
        let q = p - base;
        let fan_radius = self.organs.z * 0.34;
        let fan_web = select(
            -1.0,
            fan_radius - length(q),
            self.planar.y > 0.0 && q.y <= 0.02,
        );
        max(max(field, web), fan_web)
    }

    pub(super) fn pinnate_compound_field(&self, p: Vec2) -> f32 {
        let count = clamp(self.organs.x, 1.0, 12.0);
        let separation = clamp(self.topology.y, 0.0, 1.0);
        let mut field = -10.0;
        for i in 0..12u32 {
            let fi = i as f32;
            let presence = clamp(count - fi, 0.0, 1.0);
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
                let growth = presence * enabled * separation;
                if self.organs.w * scale * growth * self.render.y >= 2.5 {
                    let ef = self.toothed_organ_field(
                        p,
                        origin,
                        dir,
                        self.organs.z * scale * growth,
                        self.organs.w * scale * growth,
                        fract(fi * 0.29 + side * 0.21),
                    );
                    field = max(field, ef);
                }
            }
        }
        let terminal_growth = separation;
        if self.organs.w * self.organ_style.y * terminal_growth * self.render.y >= 2.5 {
            let terminal = self.tapered_organ_field(
                p,
                self.axis_point(0.80),
                Vec2::new(0.0, -1.0),
                self.organs.z * self.organ_style.y * terminal_growth,
                self.organs.w * self.organ_style.y * terminal_growth,
            );
            field = max(field, terminal);
        }
        field
    }

    pub(super) fn bipinnate_field(&self, p: Vec2) -> f32 {
        let pinnae = clamp(self.hierarchy.y, 1.0, 6.0);
        let leaflets = clamp(self.hierarchy.z, 1.0, 10.0);
        let mut field = -10.0;
        let small_asset_scale = clamp(128.0 / self.render.y, 1.0, 2.0);
        for i in 0..6u32 {
            let fi = i as f32;
            let pinna_present = clamp(pinnae - fi, 0.0, 1.0);
            let t = self.organ_position(fi, pinnae, 0.16, 0.76);
            let rachis = self.axis_point(t);
            if pinna_present <= 0.0 {
                continue;
            }
            for ps in 0..2u32 {
                let side = select(-1.0, 1.0, ps == 1u32);
                let pdir = normalize(Vec2::new(side * 0.88, -0.48));
                let plen = self.hierarchy.w * self.graded(t);

                let web_width = self.organs.w * 2.2 * (1.0 - self.hierarchy.x) * pinna_present;
                if web_width * self.render.y >= 3.0 {
                    let pinna_web =
                        self.tapered_organ_field(p, rachis, pdir, plen * pinna_present, web_width);
                    field = max(field, pinna_web);
                }
                for j in 0..10u32 {
                    let fj = j as f32;
                    let leaflet_present = clamp(leaflets - fj, 0.0, 1.0);
                    let u = (fj + 0.45) / max(leaflets, 1.0);
                    let origin = mix(rachis, rachis + pdir * plen * u, pinna_present);
                    if leaflet_present <= 0.0 {
                        continue;
                    }
                    let normal = Vec2::new(-pdir.y, pdir.x);
                    for ls in 0..2u32 {
                        let lside = select(-1.0, 1.0, ls == 1u32);
                        let ldir = normalize(pdir * 0.35 + normal * lside);
                        let emergence = smoothstep(0.08, 0.82, self.hierarchy.x);
                        let presence = pinna_present * leaflet_present;
                        if self.organs.w * presence * emergence * small_asset_scale * self.render.y
                            >= 3.0
                        {
                            let ef = self.tapered_organ_field(
                                p,
                                origin,
                                ldir,
                                self.organs.z * presence * emergence,
                                self.organs.w * presence * emergence * small_asset_scale,
                            );
                            field = max(field, ef);
                        }
                    }
                }
            }
        }
        field
    }

    pub(super) fn blade_field(&self, p: Vec2) -> f32 {
        // WGSL select evaluates both branches. On the CPU, skip fields whose
        // results cannot contribute under the already-constrained architecture.
        if self.hierarchy.y > 0.0 {
            return self.bipinnate_field(p);
        }
        let radial_influence = self.topology.x * (1.0 - self.organ_style.z);
        if radial_influence > 0.0001 {
            let separation = self.topology.y * pow(smoothstep(0.65, 1.0, radial_influence), 3.0);
            return self.radial_field(p, separation);
        }
        let simple = self.simple_field(p);
        let fan = if self.topology.z > 0.0 {
            mix(
                simple,
                self.radial_field(p, 0.0),
                clamp(self.topology.z, 0.0, 1.0),
            )
        } else {
            simple
        };
        if self.topology.y == 0.0 {
            return fan;
        }
        let separation = clamp(self.topology.y * (1.0 - self.topology.x), 0.0, 1.0);
        max(fan - separation * 1.25, self.pinnate_compound_field(p))
    }
}
