//! A static lifting tackle: strapped timber block, sheave, hemp fall and open iron hook.
use super::*;
use bevy::math::Quat;

const ROPE_DIAMETER_METRES: f32 = 0.025;
const HOOK_SECTION_METRES: f32 = 0.027;
const SHEAVE_RADIUS_METRES: f32 = 0.16;
const SHEAVE_CENTRE_METRES: f32 = 2.64;

pub(super) fn rig(a: &mut Assembly<'_>, centre: Vec2) {
    beam_strap(a, centre);
    pulley_block(a, centre);
    let cleat = Vec3::new(centre.x + 1.7, 1.20, centre.y - 0.22);
    post_cleat(a, cleat);
    rope_crown(a, centre);
    let load = Vec3::new(centre.x - SHEAVE_RADIUS_METRES, 1.42, centre.y);
    member(
        a,
        WorkplaceMaterial::HempRope,
        Vec3::new(load.x, SHEAVE_CENTRE_METRES, load.z),
        load,
        ROPE_DIAMETER_METRES,
    );
    member(
        a,
        WorkplaceMaterial::HempRope,
        Vec3::new(
            centre.x + SHEAVE_RADIUS_METRES,
            SHEAVE_CENTRE_METRES,
            centre.y,
        ),
        cleat - Vec3::X * 0.06,
        ROPE_DIAMETER_METRES,
    );
    open_hook(a, load);
}

fn beam_strap(a: &mut Assembly<'_>, p: Vec2) {
    for z in [-0.174, 0.174] {
        part(
            a,
            WorkplaceMaterial::Iron,
            Vec3::new(p.x, 3.11, p.y + z),
            Vec3::new(0.075, 0.37, 0.045),
        );
    }
    for y in [2.945, 3.275] {
        part(
            a,
            WorkplaceMaterial::Iron,
            Vec3::new(p.x, y, p.y),
            Vec3::new(0.075, 0.045, 0.39),
        );
    }
    // An open eye joins the beam band to the wooden cheeks beneath it.
    for x in [-0.055, 0.055] {
        part(
            a,
            WorkplaceMaterial::Iron,
            Vec3::new(p.x + x, 2.88, p.y),
            Vec3::new(0.032, 0.16, 0.07),
        );
    }
    part(
        a,
        WorkplaceMaterial::Iron,
        Vec3::new(p.x, 2.805, p.y),
        Vec3::new(0.14, 0.035, 0.24),
    );
}

fn pulley_block(a: &mut Assembly<'_>, p: Vec2) {
    for z in [-0.1125, 0.1125] {
        // Stepped shoulders soften the block outline while leaving a central sheave gap.
        for (y, height, width) in [(2.765, 0.10, 0.24), (2.64, 0.15, 0.34), (2.515, 0.10, 0.24)] {
            part(
                a,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x, y, p.y + z),
                Vec3::new(width, height, 0.065),
            );
        }
    }
    part(
        a,
        WorkplaceMaterial::Iron,
        Vec3::new(p.x, SHEAVE_CENTRE_METRES, p.y),
        Vec3::new(0.07, 0.07, 0.34),
    );
    // A faceted wheel bears on the axle between the two visible timber cheeks.
    for size in [Vec3::new(0.23, 0.31, 0.13), Vec3::new(0.31, 0.23, 0.13)] {
        part(
            a,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, SHEAVE_CENTRE_METRES, p.y),
            size,
        );
    }
}

fn rope_crown(a: &mut Assembly<'_>, p: Vec2) {
    let arc_segments = 8;
    for segment in 0..arc_segments {
        let point = |index: u32| {
            let angle = std::f32::consts::PI * index as f32 / arc_segments as f32;
            Vec3::new(
                p.x + SHEAVE_RADIUS_METRES * angle.cos(),
                SHEAVE_CENTRE_METRES + SHEAVE_RADIUS_METRES * angle.sin(),
                p.y,
            )
        };
        member(
            a,
            WorkplaceMaterial::HempRope,
            point(segment),
            point(segment + 1),
            ROPE_DIAMETER_METRES,
        );
    }
}

fn post_cleat(a: &mut Assembly<'_>, p: Vec3) {
    part(
        a,
        WorkplaceMaterial::Iron,
        p + Vec3::new(0.0, -0.055, 0.08),
        Vec3::new(0.14, 0.22, 0.06),
    );
    part(
        a,
        WorkplaceMaterial::UnpaintedTimber,
        p + Vec3::new(0.0, -0.035, 0.035),
        Vec3::new(0.34, 0.055, 0.075),
    );
    for y in [0.0, -0.05] {
        part(
            a,
            WorkplaceMaterial::HempRope,
            p + Vec3::Y * y,
            Vec3::new(0.21, ROPE_DIAMETER_METRES, 0.05),
        );
    }
    member(
        a,
        WorkplaceMaterial::HempRope,
        p - Vec3::X * 0.06,
        p + Vec3::new(-0.085, -0.28, 0.0),
        ROPE_DIAMETER_METRES,
    );
}

fn open_hook(a: &mut Assembly<'_>, eye: Vec3) {
    let bend = [
        Vec3::ZERO,
        Vec3::new(0.0, -0.24, 0.0),
        Vec3::new(0.075, -0.31, 0.0),
        Vec3::new(0.19, -0.31, 0.0),
        Vec3::new(0.22, -0.21, 0.0),
    ];
    for ends in bend.windows(2) {
        member(
            a,
            WorkplaceMaterial::Iron,
            eye + ends[0],
            eye + ends[1],
            HOOK_SECTION_METRES,
        );
    }
}

fn part(a: &mut Assembly<'_>, material: WorkplaceMaterial, centre: Vec3, size: Vec3) {
    a.part(WorkplaceFeature::LoadingHoist, material, centre, size, true);
}

fn member(a: &mut Assembly<'_>, material: WorkplaceMaterial, from: Vec3, to: Vec3, section: f32) {
    let direction = to - from;
    let id = a.part(
        WorkplaceFeature::LoadingHoist,
        material,
        (from + to) * 0.5,
        Vec3::new(section, direction.length(), section),
        true,
    );
    a.orient_part(id, Quat::from_rotation_arc(Vec3::Y, direction.normalize()));
}
