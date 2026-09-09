use super::*;

const POST_SECTION_METRES: f32 = 0.28;
const WALL_PLATE_DEPTH_METRES: f32 = 0.3;
const STRUCTURAL_BAY_METRES: f32 = 3.0;
pub(super) const JOINERY_PORCH_DEPTH_METRES: f32 = 4.5;
const HANDLING_LANE_HALF_WIDTH_METRES: f32 = 1.6;

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
    perimeter_frame(a, w, d, h);
    if a.plan.kind == WorkplaceKind::Carpenter {
        joinery_walls(a, w, d, h);
    } else {
        // The low rear windbreak leaves both long sides open for air-drying.
        a.wall(Vec2::new(0.0, d), Vec2::new(w, d), Vec2::Y, 0.0, 1.1, true);
    }
    a.passage(
        Vec3::new(w * 0.5 - HANDLING_LANE_HALF_WIDTH_METRES, 0.18, 0.0),
        Vec3::new(w * 0.5 + HANDLING_LANE_HALF_WIDTH_METRES, 2.7, d - 0.35),
    );
}

fn perimeter_frame(a: &mut Assembly<'_>, w: f32, d: f32, h: f32) {
    let bays = (d / STRUCTURAL_BAY_METRES).ceil() as u32;
    for x in [0.0, w] {
        for bay in 0..=bays {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Vec3::new(
                    x,
                    (h - WALL_PLATE_DEPTH_METRES) * 0.5,
                    d * bay as f32 / bays as f32,
                ),
                Vec3::new(
                    POST_SECTION_METRES,
                    h - WALL_PLATE_DEPTH_METRES,
                    POST_SECTION_METRES,
                ),
                true,
            );
        }
        a.wall(
            Vec2::new(x, 0.0),
            Vec2::new(x, d),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            h - WALL_PLATE_DEPTH_METRES,
            WALL_PLATE_DEPTH_METRES,
            true,
        );
    }
    for z in [0.0, d] {
        a.wall(
            Vec2::new(0.0, z),
            Vec2::new(w, z),
            if z == 0.0 { Vec2::NEG_Y } else { Vec2::Y },
            h - WALL_PLATE_DEPTH_METRES,
            WALL_PLATE_DEPTH_METRES,
            true,
        );
    }
}

fn joinery_walls(a: &mut Assembly<'_>, w: f32, d: f32, h: f32) {
    let porch = JOINERY_PORCH_DEPTH_METRES;
    for x in [0.0, w] {
        a.wall(
            Vec2::new(x, porch),
            Vec2::new(x, d),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            0.0,
            h,
            true,
        );
        // Boarding seams express the enclosed shop while the front working bay stays open.
        for board in 0..((d - porch) / 0.24) as u32 {
            a.part(
                WorkplaceFeature::Boarding,
                WorkplaceMaterial::Timber,
                Vec3::new(
                    x + if x == 0.0 { -0.1 } else { 0.1 },
                    h * 0.5,
                    porch + (board as f32 + 0.5) * 0.24,
                ),
                Vec3::new(0.035, h, 0.21),
                false,
            );
        }
    }
    a.wall(Vec2::new(0.0, d), Vec2::new(w, d), Vec2::Y, 0.0, h, true);
    let left = w * 0.5 - 1.85;
    let right = w - left;
    for (start, end) in [(0.0, left), (right, w)] {
        a.wall(
            Vec2::new(start, porch),
            Vec2::new(end, porch),
            Vec2::NEG_Y,
            0.0,
            h,
            true,
        );
    }
    a.wall(
        Vec2::new(left, porch),
        Vec2::new(right, porch),
        Vec2::NEG_Y,
        2.9,
        h - 2.9,
        true,
    );
    // An aisle-aligned doorway connects the covered work bay to the enclosed rear shop.
    for x in [left, right] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Vec3::new(x, 1.45, porch),
            Vec3::new(0.24, 2.9, 0.28),
            true,
        );
    }
}
