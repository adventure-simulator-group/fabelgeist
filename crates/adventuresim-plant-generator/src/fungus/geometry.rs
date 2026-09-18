use super::{FertileSurface, FungusParameters};
use crate::{PlantLod, PlantMesh};
use bevy::math::Vec3;
use fabelgeist_determinism::StreamId;
use std::f32::consts::{PI, TAU};
mod attachment;

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

pub(super) fn generate(p: &FungusParameters, seed: u64, detail: PlantLod) -> PlantMesh {
    let mut mesh = PlantMesh::default();
    let profile = Profile {
        p,
        phase: StreamId::new("plant.fungus.phase")
            .rng(seed, &[])
            .unit_f32()
            * TAU,
    };
    let columns = detail.samples(16, 8, 6);
    mesh.surface(
        [detail.samples(3, 2, 1), detail.samples(8, 4, 3)],
        p.stipe,
        |t, v| profile.stipe(t, v * TAU),
    );
    mesh.surface([detail.samples(5, 3, 2), columns], p.cap, |r, v| {
        profile.top(r, v * TAU)
    });
    mesh.surface([detail.samples(3, 2, 1), columns], p.underside, |u, v| {
        let r = 1.0 - u;
        let angle = v * TAU;
        let mut point = profile.bottom(r, angle);
        if detail == PlantLod::High {
            let folds = match p.fertile_surface {
                FertileSurface::Gills => usize::from(p.fold_count).min(8),
                FertileSurface::Ridges => usize::from(p.fold_count).min(4),
                _ => 0,
            };
            if folds > 0 {
                let wave = (0.5 + 0.5 * (angle * folds as f32).cos()).powi(
                    if p.fertile_surface == FertileSurface::Gills {
                        4
                    } else {
                        1
                    },
                );
                point.y -= p.fold_depth_m * wave * (PI * r).sin();
            }
        }
        point
    });
    if detail != PlantLod::Low {
        veil(&mut mesh, &profile, detail.samples(16, 4, 0));
    }
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
    mesh.surface([1, n], p.stipe, |t, v| {
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

fn ornaments(mesh: &mut PlantMesh, profile: &Profile, seed: u64, detail: PlantLod) {
    let p = profile.p;
    let count = usize::from(p.ornament_count).min(detail.samples(24, 6, 0));
    const GOLDEN_ANGLE: f32 = 2.399_963_1;
    for i in 0..count {
        let source_index = i * usize::from(p.ornament_count) / count;
        let r = ((source_index as f32 + 0.5) / f32::from(p.ornament_count)).sqrt() * 0.96;
        let angle = source_index as f32 * GOLDEN_ANGLE
            + profile.phase
            + StreamId::new("plant.fungus.ornament-angle")
                .rng(seed, &[source_index as u64])
                .unit_f32()
                * 0.3;
        let scale = 0.7
            + StreamId::new("plant.fungus.ornament-scale")
                .rng(seed, &[source_index as u64])
                .unit_f32()
                * 0.6;
        let (point, normal) = attachment::cap(profile, detail, r, angle);
        let tip = point + normal * p.ornament_height_m * scale;
        let footprint = p.ornament_radius_m * scale / p.cap_radius_m;
        let bases = std::array::from_fn::<_, 3, _>(|corner| {
            let a = corner as f32 / 3.0 * TAU;
            let x = r * angle.cos() + footprint * a.cos();
            let z = r * angle.sin() + footprint * a.sin();
            let (base, n) = attachment::cap(profile, detail, x.hypot(z).min(0.999), z.atan2(x));
            base - n * p.ornament_height_m * 0.1
        });
        for corner in 0..3 {
            mesh.triangle([bases[corner], tip, bases[(corner + 1) % 3]], p.ornament);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fungus::FungusSpecies;
    #[test]
    fn thin_edited_caps_keep_attachment_inside_the_wall() {
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
