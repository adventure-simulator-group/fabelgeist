//! Coopered vessels assembled from the same oriented timber and iron cuboids as the building.
use super::*;
use std::f32::consts::TAU;

const STAVE_COUNT: u32 = 12;
const STAVE_THICKNESS_METRES: f32 = 0.12;
const HOOP_HEIGHT_METRES: f32 = 0.09;
const HOOP_THICKNESS_METRES: f32 = 0.035;
const BOTTOM_PLANK_COUNT: u32 = 12;
const BOTTOM_THICKNESS_METRES: f32 = 0.16;

pub(super) fn vat(a: &mut Assembly<'_>, p: Vec2, radius: f32, height: f32) {
    let plank_depth = radius * 2.0 / BOTTOM_PLANK_COUNT as f32;
    for index in 0..BOTTOM_PLANK_COUNT {
        let z = -radius + (index as f32 + 0.5) * plank_depth;
        let half_width = (radius * radius - z * z).sqrt();
        a.part(
            WorkplaceFeature::Vat,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, BOTTOM_THICKNESS_METRES * 0.5, p.y + z),
            Vec3::new(half_width * 2.0, BOTTOM_THICKNESS_METRES, plank_depth),
            true,
        );
    }
    let step = TAU / STAVE_COUNT as f32;
    let stave_width = 2.0 * radius * (step * 0.5).tan();
    for index in 0..STAVE_COUNT {
        let angle = index as f32 * step;
        let direction = Vec2::new(angle.sin(), angle.cos());
        let centre = p + direction * radius;
        oriented_part(
            a,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(centre.x, height * 0.5, centre.y),
            Vec3::new(stave_width, height, STAVE_THICKNESS_METRES),
            angle,
        );
    }
    for elevation in [height * 0.18, height * 0.8] {
        for index in 0..STAVE_COUNT {
            let angle = index as f32 * step;
            let hoop_radius = radius + (STAVE_THICKNESS_METRES + HOOP_THICKNESS_METRES) * 0.5;
            let centre = p + Vec2::new(angle.sin(), angle.cos()) * hoop_radius;
            oriented_part(
                a,
                WorkplaceMaterial::Iron,
                Vec3::new(centre.x, elevation, centre.y),
                Vec3::new(
                    2.0 * hoop_radius * (step * 0.5).tan(),
                    HOOP_HEIGHT_METRES,
                    HOOP_THICKNESS_METRES,
                ),
                angle,
            );
        }
    }
}

fn oriented_part(
    a: &mut Assembly<'_>,
    material: WorkplaceMaterial,
    centre: Vec3,
    size: Vec3,
    yaw: f32,
) {
    let id = a.part(WorkplaceFeature::Vat, material, centre, size, true);
    a.orient_part(id, yaw);
}
