use super::{Corolla, FlowerParameters, LeafArrangement};
use crate::{PlantMesh, Tessellation};
use bevy::math::{Quat, Vec3};
use fabelgeist_determinism::{inclusive_unit_f32, splitmix64};
use std::f32::consts::{PI, TAU};

const GOLDEN_ANGLE: f32 = 2.399_963_1;
const STEM_SEGMENTS: usize = 8;
const CENTER_FLORETS: usize = 61;

pub(super) fn generate(p: &FlowerParameters, seed: u64, detail: Tessellation) -> PlantMesh {
    let mut mesh = PlantMesh::default();
    let phase = inclusive_unit_f32(splitmix64(seed)) * TAU;
    let axis = |t: f32| Vec3::new(p.height_m * p.stem_lean * t * t, p.height_m * t, 0.0);
    for i in 0..STEM_SEGMENTS {
        let t = i as f32 / STEM_SEGMENTS as f32;
        mesh.tube(
            axis(t),
            axis(t + 1.0 / STEM_SEGMENTS as f32),
            [
                p.stem_radius_m * (1.0 - t * 0.4),
                p.stem_radius_m * (1.0 - t * 0.4),
            ],
            p.green,
            detail.segments() / 2,
        );
    }
    leaves(&mut mesh, p, detail, phase, axis);
    for head in 0..p.heads {
        let angle = phase + f32::from(head) * GOLDEN_ANGLE;
        let top = if head == 0 {
            axis(1.0)
        } else {
            let t = 0.58 + 0.32 * f32::from(head) / f32::from(p.heads);
            let end = axis(t) + Vec3::new(angle.cos(), 0.75, angle.sin()) * p.height_m * 0.16;
            mesh.tube(
                axis(t),
                end,
                [p.stem_radius_m * 0.65, p.stem_radius_m * 0.35],
                p.green,
                detail.segments() / 2,
            );
            end
        };
        let mut flower = PlantMesh::default();
        corolla(&mut flower, p, detail, angle);
        let rotation = Quat::from_rotation_y(angle) * Quat::from_rotation_z(p.head_tilt);
        mesh.append(&flower, top, rotation, 1.0);
    }
    mesh
}

fn leaves(
    mesh: &mut PlantMesh,
    p: &FlowerParameters,
    detail: Tessellation,
    phase: f32,
    axis: impl Fn(f32) -> Vec3,
) {
    for leaf in 0..p.leaves {
        let fraction = f32::from(leaf) / f32::from(p.leaves);
        let angle = phase
            + match p.leaf_arrangement {
                LeafArrangement::Alternate => f32::from(leaf) * GOLDEN_ANGLE,
                _ => fraction * TAU,
            };
        let level = match p.leaf_arrangement {
            LeafArrangement::BasalRosette => 0.025,
            LeafArrangement::Whorl => 0.57,
            LeafArrangement::Alternate => 0.1 + fraction * 0.58,
        };
        let length = p.leaf_length_m * (1.0 - fraction * 0.25);
        let base = axis(level);
        let radial = Vec3::new(angle.cos(), 0.1, angle.sin());
        let petiole = length * 0.23;
        mesh.tube(
            base,
            base + radial * petiole,
            [p.stem_radius_m * 0.45, p.stem_radius_m * 0.2],
            p.green,
            detail.segments() / 2,
        );
        for leaflet in 0..p.leaflets {
            let spread = if p.leaflets > 1 {
                f32::from(leaflet) / f32::from(p.leaflets - 1) - 0.5
            } else {
                0.0
            };
            let leaflet_angle = angle + spread * p.leaf_fan_radians;
            let blade_axis = Vec3::new(leaflet_angle.cos(), 0.1, leaflet_angle.sin());
            let side = Vec3::new(-leaflet_angle.sin(), 0.0, leaflet_angle.cos());
            let blade_base = base + radial * petiole;
            mesh.surface(
                [detail.segments(), detail.segments() / 2],
                p.green,
                |t, v| {
                    let across = v * 2.0 - 1.0;
                    let lobes = if p.leaf_lobes == 0 {
                        1.0
                    } else {
                        1.0 - p.leaf_lobe_depth
                            + p.leaf_lobe_depth * (t * PI * f32::from(p.leaf_lobes)).sin().abs()
                    };
                    let width = (PI * t).sin().max(0.0).powf(0.65)
                        * length
                        * p.leaf_width_ratio
                        * 0.5
                        * lobes;
                    blade_base
                        + blade_axis * (t * length)
                        + side * width * across
                        + Vec3::Y
                            * length
                            * (0.13 * (PI * t).sin() - 0.11 * across.abs() * (PI * t).sin())
                },
            );
        }
    }
}

fn corolla(mesh: &mut PlantMesh, p: &FlowerParameters, detail: Tessellation, phase: f32) {
    let n = detail.segments();
    let radius = p.center_radius_m;
    // Green receptacle and pointed sepals remain attached to the flower axis.
    mesh.ellipsoid(
        Vec3::new(0.0, -radius * 0.5, 0.0),
        Vec3::new(radius, radius * 0.65, radius),
        p.green,
        n,
    );
    for sepal in 0..p.petals.min(8) {
        let angle = phase + TAU * f32::from(sepal) / f32::from(p.petals.min(8));
        mesh.surface([n / 2, 4], p.green, |t, v| {
            let r = radius * (0.3 + 1.8 * t);
            let a = angle + (v * 2.0 - 1.0) * 0.22 * (PI * t).sin();
            Vec3::new(r * a.cos(), -radius * (0.5 + t * 0.4), r * a.sin())
        });
    }
    if p.corolla == Corolla::FusedBell {
        bell(mesh, p, n, phase);
    } else {
        for petal in 0..p.petals {
            let angle = phase + TAU * f32::from(petal) / f32::from(p.petals);
            petals(mesh, p, n, angle);
        }
        mesh.ellipsoid(
            Vec3::ZERO,
            Vec3::new(radius, radius * p.center_height_ratio, radius),
            p.center,
            n,
        );
        if p.corolla == Corolla::RayAndDisk {
            for floret in 0..CENTER_FLORETS {
                let t = (floret as f32 / CENTER_FLORETS as f32).sqrt();
                let angle = floret as f32 * GOLDEN_ANGLE;
                let center = Vec3::new(
                    angle.cos() * t * radius,
                    radius * p.center_height_ratio * (1.0 - t * t).sqrt(),
                    angle.sin() * t * radius,
                );
                mesh.ellipsoid(center, Vec3::splat(radius * 0.12), p.center, 8);
            }
        }
        stamens(mesh, p, n, phase);
    }
}

fn petals(mesh: &mut PlantMesh, p: &FlowerParameters, n: usize, angle: f32) {
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
        mesh.surface([n / 2, n / 2], p.petal_base, |t, v| {
            evaluate(t * boundary(v), v)
        });
    }
    mesh.surface([n, n / 2], p.petal, |t, v| {
        evaluate(boundary(v) + t * (1.0 - boundary(v)), v)
    });
}

fn stamens(mesh: &mut PlantMesh, p: &FlowerParameters, n: usize, phase: f32) {
    for i in 0..p.stamens {
        let angle = phase + f32::from(i) * GOLDEN_ANGLE;
        let r = p.center_radius_m * (1.1 + 0.4 * f32::from(i % 3) / 2.0);
        let base = Vec3::new(angle.cos() * r, 0.0, angle.sin() * r);
        let tip = base + Vec3::Y * p.center_radius_m * (0.85 + 0.15 * (angle * 3.0).sin());
        mesh.tube(base, tip, [p.center_radius_m * 0.045; 2], p.anther, n / 2);
        mesh.ellipsoid(tip, Vec3::splat(p.center_radius_m * 0.14), p.anther, 8);
    }
}

fn bell(mesh: &mut PlantMesh, p: &FlowerParameters, n: usize, phase: f32) {
    mesh.surface([n, n * 3], p.petal, |t, v| {
        let angle = v * TAU;
        let lobe = (angle * f32::from(p.petals) + phase).cos();
        let radius =
            p.center_radius_m + p.petal_length_m * p.petal_width_ratio * (t * PI * 0.5).sin();
        let length = p.petal_length_m * (t - 0.1 * t.powi(8) * (1.0 - lobe));
        Vec3::new(radius * angle.cos(), length, radius * angle.sin())
    });
    mesh.tube(
        Vec3::ZERO,
        Vec3::Y * p.petal_length_m * 0.86,
        [p.center_radius_m * 0.18; 2],
        p.center,
        n / 2,
    );
}
