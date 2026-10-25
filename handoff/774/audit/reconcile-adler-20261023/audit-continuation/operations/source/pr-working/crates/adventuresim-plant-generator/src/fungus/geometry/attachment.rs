use super::*;

/// Barycentric interpolation of the actual cap triangles, rather than the
/// unsampled analytic profile. Ornaments must touch coarse meshes too.
pub(super) fn cap(profile: &Profile, lod: PlantLod, r: f32, angle: f32) -> (Vec3, Vec3) {
    let rows = lod.samples(5, 3, 2);
    let columns = lod.samples(16, 8, 6);
    let u = r.clamp(0.0, 0.999999) * rows as f32;
    let v = angle.rem_euclid(TAU) / TAU * columns as f32;
    let row = u.floor();
    let col = v.floor();
    let du = u - row;
    let dv = v - col;
    let at = |i: f32, j: f32| profile.top(i / rows as f32, j / columns as f32 * TAU);
    let a = at(row, col);
    let b = at(row, col + 1.0);
    let c = at(row + 1.0, col);
    let d = at(row + 1.0, col + 1.0);
    if row == 0.0 {
        return (
            a * (1.0 - du) + c * du * (1.0 - dv) + d * du * dv,
            (d - a).cross(c - a).normalize_or(Vec3::Y),
        );
    }
    if du + dv <= 1.0 {
        (
            a * (1.0 - du - dv) + b * dv + c * du,
            (b - a).cross(c - a).normalize_or(Vec3::Y),
        )
    } else {
        (
            b * (1.0 - du) + c * (1.0 - dv) + d * (du + dv - 1.0),
            (d - b).cross(c - b).normalize_or(Vec3::Y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attachment_samples_the_rendered_triangle_plane() {
        let p = super::super::super::FungusSpecies::FlyAgaric.parameters();
        let profile = Profile { p: &p, phase: 0.7 };
        for lod in PlantLod::ALL {
            let rows = lod.samples(5, 3, 2);
            let columns = lod.samples(16, 8, 6);
            for row in 1..rows {
                for col in 0..columns {
                    let a =
                        profile.top(row as f32 / rows as f32, col as f32 / columns as f32 * TAU);
                    let b = profile.top(
                        row as f32 / rows as f32,
                        (col + 1) as f32 / columns as f32 * TAU,
                    );
                    let c = profile.top(
                        (row + 1) as f32 / rows as f32,
                        col as f32 / columns as f32 * TAU,
                    );
                    let (point, normal) = cap(
                        &profile,
                        lod,
                        (row as f32 + 0.2) / rows as f32,
                        (col as f32 + 0.3) / columns as f32 * TAU,
                    );
                    assert!(point.distance(a * 0.5 + b * 0.3 + c * 0.2) < 1e-6);
                    assert!((point - a).dot(normal).abs() < 1e-6);
                }
            }
        }
    }
}
