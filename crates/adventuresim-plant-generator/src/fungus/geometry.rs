use super::{FertileSurface, FungusParameters};
use crate::{PlantMesh, Tessellation};
use bevy::math::{Quat, Vec3};
use fabelgeist_determinism::splitmix64;
use std::f32::consts::{PI, TAU};
mod fertile;

fn unit(seed: u64) -> f32 {
    (splitmix64(seed) >> 40) as f32 / (1_u32 << 24) as f32
}

struct Profile<'a> {
    p: &'a FungusParameters,
    phase: f32,
}
impl Profile<'_> {
    fn center(&self) -> Vec3 {
        Vec3::new(
            self.p.cap_elevation_m * self.p.stipe_bend,
            self.p.cap_elevation_m,
            0.0,
        )
    }
    fn radius(&self, r: f32, angle: f32) -> f32 {
        self.p.cap_radius_m
            * r
            * (1.0
                + self.p.asymmetry * (angle + self.phase).cos()
                + self.p.rim_wave
                    * (angle * f32::from(self.p.rim_lobes) + self.phase).sin()
                    * r
                    * r)
    }
    fn rim_height(&self, r: f32, angle: f32) -> f32 {
        self.p.cap_radius_m
            * self.p.rim_wave
            * (angle * f32::from(self.p.rim_lobes) + self.phase).sin()
            * r
            * r
    }
    fn top(&self, r: f32, angle: f32) -> Vec3 {
        let radius = self.radius(r, angle);
        self.center()
            + Vec3::new(
                radius * angle.cos(),
                self.p.cap_radius_m
                    * (self.p.cap_rise_ratio * (1.0 - r * r).max(0.0).sqrt()
                        - self.p.cap_depression_ratio * (1.0 - r).powi(2))
                    + self.rim_height(r, angle),
                radius * angle.sin(),
            )
    }
    fn bottom(&self, r: f32, angle: f32) -> Vec3 {
        let radius = self.radius(r, angle);
        self.center()
            + Vec3::new(
                radius * angle.cos(),
                -self.p.cap_radius_m * self.p.cap_thickness_ratio * (1.0 - r * r).max(0.0).sqrt()
                    - self.p.decurrent_m * (1.0 - r).powi(2)
                    + self.rim_height(r, angle),
                radius * angle.sin(),
            )
    }
    fn stipe(&self, t: f32, angle: f32) -> Vec3 {
        let inner = self.p.stipe_radius_m / self.p.cap_radius_m;
        let end = self.bottom(inner, angle) + Vec3::Y * self.wall(inner, angle) * 0.2;
        let radius =
            self.p.stipe_radius_m * (1.0 + (self.p.stipe_base_ratio - 1.0) * (1.0 - t).powi(3));
        let outline = self.radius(inner, angle) / self.p.stipe_radius_m;
        let radius = radius * (1.0 + (outline - 1.0) * t * t);
        Vec3::new(
            self.center().x * t * t + radius * angle.cos(),
            t * end.y,
            radius * angle.sin(),
        )
    }
    fn wall(&self, r: f32, angle: f32) -> f32 {
        (self.top(r, angle).y - self.bottom(r, angle).y).max(0.0)
    }
}

pub(super) fn generate(p: &FungusParameters, seed: u64, detail: Tessellation) -> PlantMesh {
    let mut mesh = PlantMesh::default();
    let profile = Profile {
        p,
        phase: unit(seed) * TAU,
    };
    let n = detail.segments();
    // A revolved curved axis with an independently controlled basal swelling.
    mesh.surface([n, n * 2], p.stipe, |t, v| profile.stipe(t, v * TAU));
    mesh.surface([n * 2, n * 6], p.cap, |r, v| profile.top(r, v * TAU));
    mesh.surface([n * 2, n * 6], p.underside, |u, v| {
        let r = 1.0 - u;
        let inset = if p.fertile_surface == FertileSurface::Pores {
            fertile::pore_inset(&profile, r, v * TAU, detail) * ((1.0 - r) / 0.06).min(1.0)
        } else {
            0.0
        };
        profile.bottom(r, v * TAU) + Vec3::Y * inset
    });
    fertile::add(&mut mesh, &profile, detail);
    veil(&mut mesh, &profile, n);
    ornaments(&mut mesh, &profile, seed, detail);
    mesh
}

fn veil(mesh: &mut PlantMesh, profile: &Profile, n: usize) {
    let p = profile.p;
    if p.ring_radius_ratio <= 1.0 {
        return;
    }
    let h = profile.stipe(p.ring_height_fraction, 0.0).y;
    let center = Vec3::new(
        p.cap_elevation_m * p.stipe_bend * p.ring_height_fraction.powi(2),
        h,
        0.0,
    );
    mesh.surface([4, n * 3], p.stipe, |t, v| {
        let angle = v * TAU;
        let radius = p.stipe_radius_m * (1.0 + (p.ring_radius_ratio - 1.0) * t);
        center
            + Vec3::new(
                radius * angle.cos(),
                -p.stipe_radius_m * t * t * (0.8 + 0.13 * (angle * 13.0).sin()),
                radius * angle.sin(),
            )
    });
}

fn ornaments(mesh: &mut PlantMesh, profile: &Profile, seed: u64, detail: Tessellation) {
    let p = profile.p;
    let count = if detail == Tessellation::Field {
        usize::from(p.ornament_count) / 2
    } else {
        usize::from(p.ornament_count)
    };
    const GOLDEN_ANGLE: f32 = 2.399_963_1;
    for i in 0..count {
        let r = ((i as f32 + 0.5) / count as f32).sqrt() * 0.96;
        let angle = i as f32 * GOLDEN_ANGLE + profile.phase + unit(seed ^ i as u64) * 0.3;
        let point = profile.top(r, angle);
        let tangent_r =
            profile.top((r + 0.001).min(0.999), angle) - profile.top((r - 0.001).max(0.0), angle);
        let tangent_a = profile.top(r, angle + 0.001) - profile.top(r, angle - 0.001);
        let normal = tangent_a.cross(tangent_r).normalize_or(Vec3::Y);
        let scale = 0.7 + unit(seed ^ i as u64 ^ 0x7761_7274) * 0.6;
        let rotation = Quat::from_rotation_arc(Vec3::Y, normal);
        let mut ornament = PlantMesh::default();
        if p.fertile_surface == FertileSurface::Enclosed {
            ornament.tube(
                Vec3::ZERO,
                Vec3::Y * p.ornament_height_m * scale,
                [p.ornament_radius_m * scale, 0.0],
                p.ornament,
                5,
            );
        } else {
            ornament.ellipsoid(
                Vec3::ZERO,
                Vec3::new(
                    p.ornament_radius_m,
                    p.ornament_height_m,
                    p.ornament_radius_m * 0.8,
                ) * scale,
                p.ornament,
                8,
            );
        }
        mesh.append(&ornament, point, rotation, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fungus::FungusSpecies;
    #[test]
    fn thin_edited_caps_keep_attachment_and_pores_inside_the_wall() {
        let mut p = FungusSpecies::FlyAgaric.parameters();
        p.cap_radius_m = 0.15;
        p.stipe_radius_m = 0.002;
        p.cap_rise_ratio = 0.0;
        p.cap_thickness_ratio = 0.4;
        p.cap_depression_ratio = 0.39;
        p.validate().unwrap();
        let profile = Profile { p: &p, phase: 0.0 };
        let inner = p.stipe_radius_m / p.cap_radius_m;
        assert!(profile.stipe(1.0, 0.0).y < profile.top(inner, 0.0).y);
        for i in 1..100 {
            let r = i as f32 / 100.0;
            for detail in [Tessellation::Close, Tessellation::Field] {
                assert!(fertile::pore_inset(&profile, r, 0.0, detail) < profile.wall(r, 0.0));
            }
        }
    }
    #[test]
    fn stipe_joins_below_cap_top_even_in_depressed_funnels() {
        for species in FungusSpecies::ALL {
            let p = species.parameters();
            let profile = Profile { p: &p, phase: 1.2 };
            let inner = p.stipe_radius_m / p.cap_radius_m;
            for i in 0..64 {
                let angle = i as f32 / 64.0 * TAU;
                let end = profile.stipe(1.0, angle);
                assert!(end.y < profile.top(inner, angle).y, "{species:?}");
                assert!(end.y >= profile.bottom(inner, angle).y, "{species:?}");
                assert!(
                    (Vec3::new(end.x, 0.0, end.z)
                        - Vec3::new(
                            profile.bottom(inner, angle).x,
                            0.0,
                            profile.bottom(inner, angle).z
                        ))
                    .length()
                        < 1e-6
                );
            }
        }
    }
}
