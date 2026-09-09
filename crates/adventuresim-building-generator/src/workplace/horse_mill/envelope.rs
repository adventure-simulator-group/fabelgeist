use super::*;

pub(in super::super) fn build_envelope(a: &mut Assembly<'_>, program: &BuildingProgram) {
    let (width, depth) = program.footprint.dimensions();
    let w = f32::from(width) * crate::CELL_SIZE_METRES;
    let d = f32::from(depth) * crate::CELL_SIZE_METRES;
    let h = program.storey_height_metres;
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Vec3::new(w * 0.5, 0.08, d * 0.5),
        Vec3::new(w, 0.16, d),
        true,
    );
    let bays = (d / 3.0).ceil() as u32;
    for x in [0.0, w] {
        for bay in 0..=bays {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(x, (h - 0.25) * 0.5, d * bay as f32 / bays as f32),
                Vec3::new(0.28, h - 0.25, 0.28),
                true,
            );
        }
        a.wall(
            Vec2::new(x, 0.0),
            Vec2::new(x, d),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            h - 0.25,
            0.25,
            true,
        );
        // The front two bays expose the drive; rear weatherboarding shelters grain.
        a.wall(
            Vec2::new(x, 8.5),
            Vec2::new(x, d),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            0.0,
            h,
            true,
        );
    }
    for x in [4.0, 8.0] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(x, (h - 0.25) * 0.5, 0.0),
            Vec3::new(0.3, h - 0.25, 0.3),
            true,
        );
    }
    a.wall(
        Vec2::ZERO,
        Vec2::new(w, 0.0),
        Vec2::NEG_Y,
        h - 0.25,
        0.25,
        true,
    );
    a.wall(Vec2::new(0.0, d), Vec2::new(w, d), Vec2::Y, 0.0, h, true);
    a.passage(Vec3::new(0.2, 0.18, 0.0), Vec3::new(1.7, 2.4, d - 0.3));
    a.passage(Vec3::new(1.7, 0.18, 11.0), Vec3::new(10.8, 2.4, 12.0));
    a.passage(Vec3::new(5.2, 0.18, 11.0), Vec3::new(7.6, 2.4, d - 0.3));
}
