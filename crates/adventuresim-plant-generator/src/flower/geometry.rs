use super::{Corolla, FlowerParameters, LeafArrangement};
use crate::{PlantLod, PlantMesh};
use bevy::math::{Quat, Vec3};
use fabelgeist_determinism::{inclusive_unit_f32, splitmix64};
use std::f32::consts::{PI, TAU};

const GOLDEN_ANGLE: f32 = 2.399_963_1;

// Complexity reduction samples organs across their original arrangement, never
// cuts triangles off an assembled mesh. Every recipe obeys the selected budget.
pub(super) fn generate(p: &FlowerParameters, seed: u64, lod: PlantLod) -> PlantMesh {
    let phase = inclusive_unit_f32(splitmix64(seed)) * TAU;
    for reduction in 1..=40 {
        let mesh = shoot(p, phase, lod, reduction);
        if mesh.indices.len() / 3 <= lod.triangle_budget() {
            return mesh;
        }
    }
    unreachable!("the minimum flowering shoot fits the low detail budget")
}

fn shoot(p: &FlowerParameters, phase: f32, lod: PlantLod, reduction: usize) -> PlantMesh {
    let mut mesh = PlantMesh::default();
    let axis = |t: f32| Vec3::new(p.height_m * p.stem_lean * t * t, p.height_m * t, 0.0);
    let rows = lod.samples(3, 2, 1);
    for i in 0..rows {
        let a = axis(i as f32 / rows as f32);
        let b = axis((i + 1) as f32 / rows as f32);
        if lod == PlantLod::Low {
            ribbon(&mut mesh, a, b, p.stem_radius_m, p.green);
        } else {
            mesh.tube(a, b, [p.stem_radius_m; 2], p.green, 3);
        }
    }
    leaves(&mut mesh, p, lod, phase, reduction, axis);
    let heads = usize::from(p.heads)
        .min(lod.samples(8, 5, 3))
        .div_ceil(reduction);
    for sample in 0..heads {
        let head = sample * usize::from(p.heads) / heads;
        let angle = phase + head as f32 * GOLDEN_ANGLE;
        let top = if head == 0 {
            axis(1.0)
        } else {
            let t = 0.58 + 0.32 * head as f32 / f32::from(p.heads);
            let end = axis(t) + Vec3::new(angle.cos(), 0.75, angle.sin()) * p.height_m * 0.16;
            if lod == PlantLod::High {
                mesh.tube(
                    axis(t),
                    end,
                    [p.stem_radius_m * 0.65, p.stem_radius_m * 0.35],
                    p.green,
                    3,
                );
            } else {
                ribbon(&mut mesh, axis(t), end, p.stem_radius_m * 0.5, p.green);
            }
            end
        };
        let mut flower = PlantMesh::default();
        corolla(&mut flower, p, lod, angle, reduction);
        mesh.append(
            &flower,
            top,
            Quat::from_rotation_y(angle) * Quat::from_rotation_z(p.head_tilt),
            1.0,
        );
    }
    mesh
}

fn ribbon(mesh: &mut PlantMesh, a: Vec3, b: Vec3, radius: f32, pigment: crate::Pigment) {
    let side = (b - a).normalize_or(Vec3::Y).any_orthonormal_vector() * radius;
    mesh.triangle([a - side, a + side, b + side], pigment);
    mesh.triangle([a - side, b + side, b - side], pigment);
}

fn leaves(
    mesh: &mut PlantMesh,
    p: &FlowerParameters,
    lod: PlantLod,
    phase: f32,
    reduction: usize,
    axis: impl Fn(f32) -> Vec3,
) {
    let count = usize::from(p.leaves)
        .min(lod.samples(12, 5, 2))
        .div_ceil(reduction);
    for sample in 0..count {
        let leaf = sample * usize::from(p.leaves) / count;
        let fraction = leaf as f32 / f32::from(p.leaves);
        let angle = phase
            + match p.leaf_arrangement {
                LeafArrangement::Alternate => leaf as f32 * GOLDEN_ANGLE,
                _ => fraction * TAU,
            };
        let level = match p.leaf_arrangement {
            LeafArrangement::BasalRosette => 0.025,
            LeafArrangement::Whorl => 0.57,
            LeafArrangement::Alternate => 0.1 + fraction * 0.58,
        };
        let base = axis(level);
        let length = p.leaf_length_m * (1.0 - fraction * 0.25);
        let leaflets = usize::from(p.leaflets)
            .min(lod.samples(5, 3, 1))
            .div_ceil(reduction);
        for leaflet in 0..leaflets {
            let spread = if leaflets > 1 {
                leaflet as f32 / (leaflets - 1) as f32 - 0.5
            } else {
                0.0
            };
            let angle = angle + spread * p.leaf_fan_radians;
            let forward = Vec3::new(angle.cos(), 0.1, angle.sin());
            let side = Vec3::new(-angle.sin(), 0.0, angle.cos());
            let leaf_rows = lod.samples(3, 2, 2);
            mesh.surface([leaf_rows, 1], p.green, |t, v| {
                let lobes =
                    1.0 - p.leaf_lobe_depth * 0.35 * (t * PI * f32::from(p.leaf_lobes)).cos().abs();
                let width = (PI * t).sin().max(0.0) * length * p.leaf_width_ratio * 0.5 * lobes;
                base + forward * t * length
                    + side * (v * 2.0 - 1.0) * width
                    + Vec3::Y * length * 0.13 * (PI * t).sin()
            });
        }
    }
}

fn corolla(
    mesh: &mut PlantMesh,
    p: &FlowerParameters,
    lod: PlantLod,
    phase: f32,
    reduction: usize,
) {
    let radius = p.center_radius_m;
    if p.corolla == Corolla::FusedBell {
        bell(mesh, p, lod, phase);
        return;
    }
    if lod == PlantLod::Low {
        // At subpixel petal width a single lobed corolla preserves the head's
        // physical envelope and pigment, without dozens of isolated rays.
        let columns = if p.corolla == Corolla::RayAndDisk {
            8
        } else {
            usize::from(p.petals).clamp(3, 6)
        };
        let outer = radius * 0.75 + p.petal_length_m;
        for i in 0..columns {
            let a = phase + i as f32 / columns as f32 * TAU;
            let b = phase + (i + 1) as f32 / columns as f32 * TAU;
            mesh.triangle(
                [
                    Vec3::ZERO,
                    Vec3::new(
                        outer * a.cos(),
                        p.petal_length_m * p.petal_cup,
                        outer * a.sin(),
                    ),
                    Vec3::new(
                        outer * b.cos(),
                        p.petal_length_m * p.petal_cup,
                        outer * b.sin(),
                    ),
                ],
                p.petal,
            );
        }
    } else {
        let petals_count = usize::from(p.petals)
            .min(lod.samples(40, 12, 6))
            .div_ceil(reduction)
            .max(3);
        for i in 0..petals_count {
            petals(
                mesh,
                p,
                lod.samples(3, 2, 1),
                phase + i as f32 / petals_count as f32 * TAU,
            );
        }
    }
    let center_y = if lod == PlantLod::Low {
        radius * 0.2
    } else {
        0.0
    };
    mesh.ellipsoid(
        Vec3::Y * center_y,
        Vec3::new(radius, radius * p.center_height_ratio, radius),
        p.center,
        lod.samples(6, 4, 4),
    );
    if lod == PlantLod::High {
        let stamens = usize::from(p.stamens).min(8).div_ceil(reduction);
        for i in 0..stamens {
            let angle = phase + i as f32 * GOLDEN_ANGLE;
            let r = radius * 1.25;
            let base = Vec3::new(angle.cos() * r, 0.0, angle.sin() * r);
            mesh.triangle(
                [
                    base,
                    base + Vec3::Y * radius,
                    base + Vec3::new(radius * 0.2, radius * 0.8, 0.0),
                ],
                p.anther,
            );
        }
    }
}

fn petals(mesh: &mut PlantMesh, p: &FlowerParameters, rows: usize, angle: f32) {
    let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
    let side = Vec3::new(-angle.sin(), 0.0, angle.cos());
    let evaluate = |t: f32, v: f32| {
        let across = v * 2.0 - 1.0;
        let width = p.petal_length_m * p.petal_width_ratio * (PI * t).sin().max(0.0).powf(0.55);
        let notch = p.petal_notch * t.powi(6) * (PI * t).sin().max(0.0) * (1.0 - across.abs());
        let r = p.center_radius_m * 0.75 + p.petal_length_m * (t - notch);
        let cup = p.petal_cup * t * t + 0.12 * across * across * (PI * t).sin();
        let ripple = p.petal_ripple * (across * PI * 3.0).sin() * t * t * (PI * t).sin().max(0.0);
        radial * r + side * across * width + Vec3::Y * p.petal_length_m * (cup + ripple)
    };
    // Split the surface at the pigment boundary: no interpolated albedo gradient.
    let boundary = |v: f32| p.base_fraction * (1.0 - 0.75 * (v * 2.0 - 1.0).powi(2));
    if p.base_fraction > 0.0 {
        mesh.surface([1, 2], p.petal_base, |t, v| evaluate(t * boundary(v), v));
    }
    mesh.surface([rows, 2], p.petal, |t, v| {
        evaluate(boundary(v) + t * (1.0 - boundary(v)), v)
    });
}

fn bell(mesh: &mut PlantMesh, p: &FlowerParameters, lod: PlantLod, phase: f32) {
    let radius = p.center_radius_m;
    let columns = (usize::from(p.petals) * lod.samples(2, 1, 1)).clamp(3, lod.samples(12, 6, 3));
    mesh.surface([lod.samples(3, 2, 1), columns], p.petal, |t, v| {
        let angle = v * TAU;
        let lobe = (angle * f32::from(p.petals) + phase).cos();
        let radius = radius + p.petal_length_m * p.petal_width_ratio * (t * PI * 0.5).sin();
        Vec3::new(
            radius * angle.cos(),
            p.petal_length_m * (t - 0.1 * t.powi(8) * (1.0 - lobe)),
            radius * angle.sin(),
        )
    });
    if lod == PlantLod::High {
        mesh.tube(
            Vec3::ZERO,
            Vec3::Y * p.petal_length_m * 0.86,
            [radius * 0.18; 2],
            p.center,
            3,
        );
    }
}
