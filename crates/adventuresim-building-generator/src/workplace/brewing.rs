//! Small preindustrial brewing yards and ventilated malt-drying houses.
use super::{assembly::Assembly, *};
use crate::{GableProfile, RidgeAxis, RoofKind, RoofPiece};

mod drying;
mod kiln;
#[cfg(test)]
mod tests;
mod vessels;
pub(super) use drying::{drying_wall, malthouse};

/// The service shelter occupies the reserved side yard, leaving its rear hearth in open air.
pub(super) fn service_roof(main: Vec2) -> RoofPiece {
    RoofPiece {
        kind: RoofKind::Gable,
        centre: Vec2::new(main.x + 3.6, (main.y - 2.8) * 0.5),
        size: Vec2::new(3.2, main.y - 4.0),
        base_height_metres: 2.8,
        pitch_degrees: 25.0,
        ridge_axis: RidgeAxis::Z,
        eave_metres: 0.18,
        gable_profile: GableProfile::Plain,
    }
}

pub(super) fn brewery(a: &mut Assembly<'_>, w: f32, d: f32) {
    service_frame(a, w, d);
    for z in [2.1, d * 0.5 - 0.7] {
        vessels::vat(a, Vec2::new(w + 3.6, z), 0.95, 1.25);
    }
    vessels::vat(a, Vec2::new(2.0, 2.2), 1.05, 1.4);
    for z in [d * 0.55, d - 2.2] {
        vessels::vat(a, Vec2::new(2.0, z), 1.05, 1.4);
    }
    brewing_bench(a, Vec2::new(w - 2.1, d - 3.0));
    // Broad masonry shoulders around an open firing mouth, with a continuous rear flue.
    hearth(a, Vec2::new(w + 3.6, d - 1.65), 4.25);
    a.passage(
        Vec3::new(w + 2.2, 0.05, d - 3.0),
        Vec3::new(w + 5.0, 2.3, d - 2.85),
    );
}

fn brewing_bench(a: &mut Assembly<'_>, p: Vec2) {
    for x in [-0.8, 0.8] {
        for z in [-1.2, 1.2] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + x, 0.45, p.y + z),
                Vec3::new(0.18, 0.9, 0.18),
                true,
            );
        }
    }
    a.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 0.98, p.y),
        Vec3::new(2.0, 0.16, 2.8),
        true,
    );
    // A small open rinsing vessel on the working bench gives the surface a clear use.
    for (offset, size) in [
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.3, 0.08, 1.3)),
        (Vec3::new(-0.61, 0.2, 0.0), Vec3::new(0.08, 0.4, 1.3)),
        (Vec3::new(0.61, 0.2, 0.0), Vec3::new(0.08, 0.4, 1.3)),
        (Vec3::new(0.0, 0.2, -0.61), Vec3::new(1.3, 0.4, 0.08)),
        (Vec3::new(0.0, 0.2, 0.61), Vec3::new(1.3, 0.4, 0.08)),
    ] {
        a.part(
            WorkplaceFeature::Trough,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 1.1, p.y) + offset,
            size,
            true,
        );
    }
}

fn service_frame(a: &mut Assembly<'_>, w: f32, d: f32) {
    let front = 0.6;
    let back = d - 3.4;
    let bays = ((back - front) / 3.0).ceil() as u32;
    for x in [w + 2.0, w + 5.2] {
        for bay in 0..=bays {
            let z = front + (back - front) * bay as f32 / bays as f32;
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Vec3::new(x, 1.3, z),
                Vec3::new(0.24, 2.6, 0.24),
                true,
            );
        }
        a.wall(
            Vec2::new(x, front),
            Vec2::new(x, back),
            if x < w + 3.0 { Vec2::NEG_X } else { Vec2::X },
            2.6,
            0.2,
            true,
        );
    }
    for z in [front, back] {
        a.wall(
            Vec2::new(w + 2.0, z),
            Vec2::new(w + 5.2, z),
            if z == front { Vec2::NEG_Y } else { Vec2::Y },
            2.6,
            0.2,
            true,
        );
    }
}

fn hearth(a: &mut Assembly<'_>, p: Vec2, flue_top: f32) {
    for x in [-0.9, 0.9] {
        a.part(
            WorkplaceFeature::Kiln,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x + x, 0.7, p.y),
            Vec3::new(0.5, 1.4, 2.2),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 0.7, p.y + 0.85),
        Vec3::new(1.3, 1.4, 0.5),
        true,
    );
    a.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Iron,
        Vec3::new(p.x, 1.5, p.y - 0.35),
        Vec3::new(2.3, 0.2, 1.5),
        true,
    );
    // Hollow square shaft directly over the rear masonry. Its opening is never capped.
    for x in [-0.34, 0.34] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x + x, (1.4 + flue_top) * 0.5, p.y + 0.75),
            Vec3::new(0.18, flue_top - 1.4, 0.86),
            true,
        );
    }
    for z in [-0.34, 0.34] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x, (1.4 + flue_top) * 0.5, p.y + 0.75 + z),
            Vec3::new(0.5, flue_top - 1.4, 0.18),
            true,
        );
    }
}
