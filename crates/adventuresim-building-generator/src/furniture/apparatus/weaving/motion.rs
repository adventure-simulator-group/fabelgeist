//! Roller bearings, locking wheels, and reciprocal heddle suspension.
use super::*;
use std::f32::consts::{PI, TAU};

const ROLLER_SIDES: usize = 6;
const RATCHET_TEETH: usize = 6;
const SUSPENSION_ARC_SEGMENTS: usize = 4;

pub(super) fn seat(builder: &mut Builder, width: f32, depth: f32) {
    let z = -depth * 0.5 - 0.35;
    let span = width * 0.65;
    builder.timber(Vec3::new(0.0, 0.455, z), Vec3::new(span, 0.05, 0.28));
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (span * 0.5 - 0.08), 0.215, z),
            Vec3::new(0.06, 0.43, 0.25),
        );
    }
}

pub(super) fn beams(builder: &mut Builder, width: f32, depth: f32) {
    for z in [-depth * 0.5, depth * 0.5] {
        roller(builder, Vec3::new(0.0, 0.765, z), width, 0.085);
        // Small projecting journals meet the upright bearings. The notched
        // wheel and engaging pawl hold warp tension between advances.
        let x = width * 0.5 + 0.035;
        let outline = (0..RATCHET_TEETH * 2)
            .map(|i| {
                let angle = i as f32 * PI / RATCHET_TEETH as f32;
                let radius = if i % 2 == 0 { 0.075 } else { 0.060 };
                Vec3::new(0.0, angle.cos() * radius, angle.sin() * radius)
            })
            .collect::<Vec<_>>();
        wheel(builder, Vec3::new(x, 0.765, z), 0.04, &outline);
        builder.timber(Vec3::new(x, 0.895, z), Vec3::new(0.035, 0.18, 0.035));
        metal(
            builder,
            Vec3::new(width * 0.5, 1.0, z),
            Vec3::new(0.1, 0.025, 0.025),
        );
    }
}

pub(super) fn suspension(builder: &mut Builder, width: f32, height: f32) {
    let pulley_y = height - 0.2;
    for side in [-1.0, 1.0] {
        let x = side * width * 0.35;
        roller(builder, Vec3::new(x, pulley_y, 0.0), 0.04, 0.067);
        // Fork cheek and axle hang from the independent upper cross member.
        builder.timber(
            Vec3::new(x + side * 0.035, height - 0.14, 0.0),
            Vec3::new(0.025, 0.2, 0.05),
        );
        metal(
            builder,
            Vec3::new(x, pulley_y, 0.0),
            Vec3::new(0.12, 0.018, 0.018),
        );
        for z in [-0.07, 0.07] {
            super::fiber(
                builder,
                BuildingLodMaterial::HempRope,
                Vec3::new(x, (pulley_y + 1.14) * 0.5, z),
                Vec3::new(0.008, pulley_y - 1.14, 0.008),
            );
        }
        for segment in 0..SUSPENSION_ARC_SEGMENTS {
            let angles =
                [segment, segment + 1].map(|i| i as f32 * PI / SUSPENSION_ARC_SEGMENTS as f32);
            let [a, b] =
                angles.map(|angle| Vec3::new(x, pulley_y + angle.sin() * 0.07, angle.cos() * 0.07));
            let side = (b - a).cross(Vec3::X).normalize() * 0.004;
            for half in [Vec3::X * 0.004, side] {
                builder.quad(
                    BuildingLodMaterial::HempRope,
                    [a - half, b - half, b + half, a + half],
                    (b - a).cross(half).normalize(),
                );
            }
        }
    }
}

fn roller(builder: &mut Builder, centre: Vec3, length: f32, radius: f32) {
    let outline = (0..ROLLER_SIDES)
        .map(|i| {
            let angle = i as f32 * TAU / ROLLER_SIDES as f32;
            Vec3::new(0.0, angle.cos() * radius, angle.sin() * radius)
        })
        .collect::<Vec<_>>();
    wheel(builder, centre, length, &outline);
}

fn wheel(builder: &mut Builder, centre: Vec3, length: f32, outline: &[Vec3]) {
    let radius = outline.iter().map(|p| p.length()).fold(0.0, f32::max);
    builder.collider(centre, Vec3::new(length, radius * 2.0, radius * 2.0));
    for index in 0..outline.len() {
        let [a, b] = [outline[index], outline[(index + 1) % outline.len()]];
        let half = Vec3::X * length * 0.5;
        builder.quad(
            BuildingLodMaterial::InteriorTimber,
            [
                centre - half + a,
                centre + half + a,
                centre + half + b,
                centre - half + b,
            ],
            (a + b).normalize(),
        );
        for side in [-1.0, 1.0] {
            let origin = centre + half * side;
            builder.triangle(
                BuildingLodMaterial::TimberEndGrain,
                [origin, origin + a, origin + b],
                Vec3::X * side,
            );
        }
    }
}
