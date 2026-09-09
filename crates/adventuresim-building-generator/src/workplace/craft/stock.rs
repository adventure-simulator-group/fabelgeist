use super::*;

const STACK_BAY_LENGTH_METRES: f32 = 4.5;
const PLANK_THICKNESS_METRES: f32 = 0.12;
const STICKER_THICKNESS_METRES: f32 = 0.1;
const MAX_RACK_SUPPORT_SPAN_METRES: f32 = 3.0;

pub(super) fn timber_yard(a: &mut Assembly<'_>, w: f32, d: f32) {
    let bays = ((d - 3.0) / STACK_BAY_LENGTH_METRES) as u32;
    for x in [2.1, w - 2.1] {
        for bay in 0..bays {
            let z = 0.8 + STACK_BAY_LENGTH_METRES * (bay as f32 + 0.5);
            timber_stack(a, Vec2::new(x, z), 2.8, 3.8, 5 + bay % 3);
        }
    }
    // Cross-cutting trestles occupy a rear side bay, never the long handling lane.
    saw_bench(a, Vec2::new(2.1, d - 1.25), Vec2::new(2.7, 1.0));
}

pub(super) fn joinery(a: &mut Assembly<'_>, w: f32, d: f32) {
    workpiece_on_trestles(a, Vec2::new(1.7, 2.2));
    for x in [1.7, w - 1.7] {
        if x > w * 0.5 {
            saw_bench(a, Vec2::new(x, 2.0), Vec2::new(2.2, 2.8));
        }
        timber_stack(a, Vec2::new(x, d - 2.7), 2.2, 3.5, 4);
    }
    // Wide shallow shelves on grounded standards hold shorter joinery stock.
    let rack_bays = ((d - 7.0) / MAX_RACK_SUPPORT_SPAN_METRES).ceil() as u32;
    for bay in 0..=rack_bays {
        let z = 6.0 + (d - 7.0) * bay as f32 / rack_bays as f32;
        for x in [0.45, w - 0.45] {
            a.part(
                WorkplaceFeature::Rack,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(x, 1.4, z),
                Vec3::new(0.18, 2.8, 0.18),
                true,
            );
        }
    }
    for x in [0.65, w - 0.65] {
        for height in [1.8, 2.5] {
            a.part(
                WorkplaceFeature::Rack,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(x, height, (6.0 + d - 1.0) * 0.5),
                Vec3::new(0.6, 0.12, d - 7.0),
                true,
            );
        }
    }
}

fn workpiece_on_trestles(a: &mut Assembly<'_>, p: Vec2) {
    // A long squared beam is raised across two separate trestles for marking and joinery.
    for z in [-1.1, 1.1] {
        for x in [-0.65, 0.65] {
            a.part(
                WorkplaceFeature::SawBench,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + x, 0.55, p.y + z),
                Vec3::new(0.18, 0.78, 0.4),
                true,
            );
        }
        a.part(
            WorkplaceFeature::SawBench,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 1.02, p.y + z),
            Vec3::new(1.6, 0.16, 0.3),
            true,
        );
    }
    a.part(
        WorkplaceFeature::SawBench,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.26, p.y),
        Vec3::new(0.42, 0.32, 3.9),
        true,
    );
}

fn timber_stack(a: &mut Assembly<'_>, p: Vec2, width: f32, length: f32, tiers: u32) {
    for z in [-0.35, 0.35] {
        a.part(
            WorkplaceFeature::TimberStack,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 0.24, p.y + z * length),
            Vec3::new(width, 0.24, 0.22),
            true,
        );
    }
    for tier in 0..tiers {
        let bottom = 0.36 + tier as f32 * (PLANK_THICKNESS_METRES + STICKER_THICKNESS_METRES);
        for plank in 0..4 {
            let x = p.x - width * 0.5 + (plank as f32 + 0.5) * width * 0.25;
            a.part(
                WorkplaceFeature::TimberStack,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(x, bottom + PLANK_THICKNESS_METRES * 0.5, p.y),
                Vec3::new(width * 0.25 - 0.04, PLANK_THICKNESS_METRES, length),
                true,
            );
        }
        if tier + 1 < tiers {
            for z in [-0.35, 0.35] {
                a.part(
                    WorkplaceFeature::TimberStack,
                    WorkplaceMaterial::UnpaintedTimber,
                    Vec3::new(
                        p.x,
                        bottom + PLANK_THICKNESS_METRES + STICKER_THICKNESS_METRES * 0.5,
                        p.y + z * length,
                    ),
                    Vec3::new(width, STICKER_THICKNESS_METRES, 0.1),
                    true,
                );
            }
        }
    }
}

fn saw_bench(a: &mut Assembly<'_>, p: Vec2, size: Vec2) {
    for x in [-0.35, 0.35] {
        for z in [-0.35, 0.35] {
            a.part(
                WorkplaceFeature::SawBench,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + size.x * x, 0.57, p.y + size.y * z),
                Vec3::new(0.18, 0.82, 0.18),
                true,
            );
        }
    }
    a.part(
        WorkplaceFeature::SawBench,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.06, p.y),
        Vec3::new(size.x, 0.16, size.y),
        true,
    );
    a.part(
        WorkplaceFeature::SawBench,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.21, p.y),
        Vec3::new(size.x * 0.65, 0.14, 0.3),
        true,
    );
}
