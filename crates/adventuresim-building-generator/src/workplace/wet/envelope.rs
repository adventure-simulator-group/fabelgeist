use super::*;

pub(in super::super) fn build_envelope(a: &mut Assembly<'_>, program: &BuildingProgram) {
    let (width, depth) = program.footprint.dimensions();
    let w = f32::from(width) * crate::CELL_SIZE_METRES;
    let d = f32::from(depth) * crate::CELL_SIZE_METRES;
    let h = program.storey_height_metres;
    let timber = a.plan.kind == WorkplaceKind::Tannery;
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Vec3::new(w * 0.5, 0.08, d * 0.5),
        Vec3::new(w, 0.16, d),
        true,
    );
    // An open front bay admits working light while the rear room remains sheltered.
    let porch = 4.2;
    for x in [0.0, w] {
        for z in [0.0, porch] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(x, (h - 0.24) * 0.5, z),
                Vec3::new(0.26, h - 0.24, 0.26),
                true,
            );
        }
        a.wall(
            Vec2::new(x, 0.0),
            Vec2::new(x, porch),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            h - 0.24,
            0.24,
            true,
        );
        a.wall(
            Vec2::new(x, porch),
            Vec2::new(x, d),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            0.0,
            h,
            timber,
        );
    }
    a.wall(
        Vec2::ZERO,
        Vec2::new(w, 0.0),
        Vec2::NEG_Y,
        h - 0.24,
        0.24,
        true,
    );
    a.wall(Vec2::new(0.0, d), Vec2::new(w, d), Vec2::Y, 0.0, h, timber);
    for (start, end) in [(0.0, w * 0.5 - 1.7), (w * 0.5 + 1.7, w)] {
        a.wall(
            Vec2::new(start, porch),
            Vec2::new(end, porch),
            Vec2::NEG_Y,
            0.0,
            h,
            timber,
        );
    }
    a.wall(
        Vec2::new(w * 0.5 - 1.7, porch),
        Vec2::new(w * 0.5 + 1.7, porch),
        Vec2::NEG_Y,
        2.55,
        h - 2.55,
        timber,
    );
    a.passage(
        Vec3::new(w * 0.5 - 1.5, 0.18, 0.0),
        Vec3::new(w * 0.5 + 1.5, 2.4, d - 0.35),
    );
    a.passage(Vec3::new(w + 0.35, 0.05, 0.0), Vec3::new(w + 1.85, 2.4, d));
}
