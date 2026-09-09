use super::*;

pub(in super::super) fn drying_wall(a: &mut Assembly<'_>, x: f32, d: f32, h: f32, outward: Vec2) {
    let bays = (d / 2.5) as u32;
    let length = d / bays as f32;
    for bay in 0..bays {
        let start = bay as f32 * length;
        let left = start + length * 0.5 - 0.7;
        let right = left + 1.4;
        for (from, to) in [(start, left), (right, start + length)] {
            a.wall(Vec2::new(x, from), Vec2::new(x, to), outward, 0.0, h, false);
        }
        for (base, height) in [(0.0, 1.6), (2.6, h - 2.6)] {
            a.wall(
                Vec2::new(x, left),
                Vec2::new(x, right),
                outward,
                base,
                height,
                false,
            );
        }
        // Framed horizontal slats leave real ventilation slots through the masonry wall.
        for z in [left, right] {
            a.part(
                WorkplaceFeature::Louver,
                WorkplaceMaterial::Timber,
                Vec3::new(x, 2.1, z),
                Vec3::new(0.5, 1.04, 0.09),
                true,
            );
        }
        for row in 0..4 {
            a.part(
                WorkplaceFeature::Louver,
                WorkplaceMaterial::Timber,
                Vec3::new(x, 1.71 + row as f32 * 0.25, (left + right) * 0.5),
                Vec3::new(0.5, 0.12, 1.4),
                true,
            );
        }
    }
}

pub(in super::super) fn malthouse(a: &mut Assembly<'_>, w: f32, d: f32) {
    super::kiln::drying_kiln(a, Vec2::new(w + 3.0, d - 2.6));
    for x in [2.2, w - 2.2] {
        for z in [2.4, d * 0.5, d - 2.4] {
            drying_bed(a, Vec2::new(x, z));
        }
    }
}

fn drying_bed(a: &mut Assembly<'_>, p: Vec2) {
    for x in [-0.8, 0.8] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 0.3, p.y),
            Vec3::new(0.16, 0.6, 2.0),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Rack,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 0.68, p.y),
        Vec3::new(2.0, 0.16, 2.0),
        true,
    );
    for x in [-0.98, 0.98] {
        a.part(
            WorkplaceFeature::Rack,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 0.85, p.y),
            Vec3::new(0.08, 0.18, 2.0),
            true,
        );
    }
    for z in [-0.98, 0.98] {
        a.part(
            WorkplaceFeature::Rack,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 0.85, p.y + z),
            Vec3::new(1.88, 0.18, 0.08),
            true,
        );
    }
    a.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::Grain,
        Vec3::new(p.x, 0.79, p.y),
        Vec3::new(1.84, 0.06, 1.84),
        true,
    );
}
