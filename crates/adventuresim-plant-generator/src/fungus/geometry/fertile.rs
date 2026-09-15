use super::*;

pub(super) fn add(mesh: &mut PlantMesh, profile: &Profile, detail: Tessellation) {
    match profile.p.fertile_surface {
        FertileSurface::Gills => folds(mesh, profile, detail, false),
        FertileSurface::Ridges => folds(mesh, profile, detail, true),
        FertileSurface::Pores => pores(mesh, profile, detail),
        FertileSurface::Enclosed => {}
    }
}

fn folds(mesh: &mut PlantMesh, profile: &Profile, detail: Tessellation, rounded: bool) {
    let p = profile.p;
    let count = usize::from(p.fold_count);
    let inner = (p.stipe_radius_m / p.cap_radius_m).min(0.65);
    let sides = if rounded { 6 } else { 2 };
    for i in 0..count {
        let angle = i as f32 / count as f32 * TAU + profile.phase;
        let depth = p.fold_depth_m * if i % 2 == 0 { 1.0 } else { 0.65 };
        let start = if !rounded && i % 3 == 0 {
            inner + 0.3
        } else {
            inner
        };
        mesh.surface([detail.segments() * 2, sides], p.underside, |t, v| {
            let r = start + (0.985 - start) * t;
            let width = if rounded {
                (depth * 1.1 / (r * p.cap_radius_m)).min(TAU / count as f32 * 0.28)
            } else {
                0.0018
            };
            let side = if rounded {
                -(v * PI).cos()
            } else {
                v * 2.0 - 1.0
            };
            let a = angle + side * width;
            let bulge = if rounded {
                (v * PI).sin()
            } else {
                1.0 - (v * 2.0 - 1.0).abs()
            };
            profile.bottom(r, a) - Vec3::Y * depth * bulge * (t * PI).sin().max(0.0).sqrt()
        });
        // Blunt chanterelle folds fork toward the margin; true gills stay blades.
        if rounded && i % 2 == 0 {
            mesh.surface([detail.segments(), 6], p.underside, |t, v| {
                let r = 0.58 + 0.39 * t;
                let width = (depth * 0.8 / (r * p.cap_radius_m)).min(TAU / count as f32 * 0.22);
                let a = angle + t * TAU / count as f32 * 0.48 - (v * PI).cos() * width;
                profile.bottom(r, a) - Vec3::Y * depth * 0.7 * (v * PI).sin() * (t * PI).sin()
            });
        }
    }
}

fn pore_depth(p: &FungusParameters, detail: Tessellation) -> f32 {
    p.cap_radius_m
        / if detail == Tessellation::Close {
            180.0
        } else {
            110.0
        }
}

pub(super) fn pore_inset(profile: &Profile, r: f32, angle: f32, detail: Tessellation) -> f32 {
    pore_depth(profile.p, detail).min(profile.wall(r, angle) * 0.35)
}

fn pores(mesh: &mut PlantMesh, profile: &Profile, detail: Tessellation) {
    let p = profile.p;
    let rings = if detail == Tessellation::Close { 12 } else { 7 };
    let inner = p.stipe_radius_m / p.cap_radius_m;
    let radial_step = (0.94 - inner) / rings as f32;
    // Neighboring polar tiles share their outer boundaries. Each tile slopes
    // into a recessed pore opening; the inset cap sheet closes the cavity.
    for ring in 0..rings {
        let r = inner + radial_step * (ring as f32 + 0.5);
        let count = (TAU * r / radial_step).ceil() as usize;
        let angular_step = TAU / count as f32;
        for i in 0..count {
            let angle = angular_step * (i as f32 + 0.5);
            mesh.surface([1, 8], p.underside, |t, v| {
                let a = v * TAU;
                let square_radius = 1.0 / a.cos().abs().max(a.sin().abs());
                let radius = square_radius * (1.0 - t) + 0.62 * t;
                let sample_r = r + a.cos() * radial_step * 0.5 * radius;
                let sample_a = angle + a.sin() * angular_step * 0.5 * radius;
                profile.bottom(sample_r, sample_a)
                    + Vec3::Y * pore_inset(profile, sample_r, sample_a, detail) * t
            });
        }
    }
    mesh.surface([2, detail.segments() * 6], p.underside, |t, v| {
        profile.bottom(1.0 - 0.06 * t, v * TAU)
    });
}
