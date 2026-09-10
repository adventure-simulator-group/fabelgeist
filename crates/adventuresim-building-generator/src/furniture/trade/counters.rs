use super::{Builder, FurnitureKind, Vec3, legs};

const COUNTER_TOP_THICKNESS_METRES: f32 = 0.065;
const COUNTER_PANEL_THICKNESS_METRES: f32 = 0.035;
const COUNTER_FRAME_THICKNESS_METRES: f32 = 0.075;

pub(super) fn counter(builder: &mut Builder, size: Vec3, kind: FurnitureKind) {
    let top = COUNTER_TOP_THICKNESS_METRES;
    let frame = COUNTER_FRAME_THICKNESS_METRES;
    let panel = COUNTER_PANEL_THICKNESS_METRES;
    // The slab has no overhang: translated modules share exactly one seam plane.
    builder.timber(
        Vec3::Y * (size.y - top * 0.5),
        Vec3::new(size.x, top, size.z),
    );
    legs(builder, size, size.y - top, frame);
    // The merchant side remains open, with a continuous under-counter shelf.
    builder.timber(Vec3::Y * 0.22, Vec3::new(size.x, panel, size.z - panel));
    builder.timber(
        Vec3::new(0.0, (size.y - top) * 0.5, -(size.z - panel) * 0.5),
        Vec3::new(size.x, size.y - top, panel),
    );
    for y in [frame * 0.5, size.y - top - frame * 0.5] {
        builder.timber(
            Vec3::new(0.0, y, -(size.z - frame) * 0.5),
            Vec3::new(size.x, frame, frame),
        );
    }
    match kind {
        FurnitureKind::CounterLeftEnd => end_panel(builder, size, -1.0),
        FurnitureKind::CounterRightEnd => end_panel(builder, size, 1.0),
        FurnitureKind::CounterCorner => {
            // This outside corner turns from its front to its left side; both
            // remaining sides are open for the adjoining straight modules.
            end_panel(builder, size, -1.0);
        }
        _ => {}
    }
}

fn end_panel(builder: &mut Builder, size: Vec3, sign: f32) {
    let panel = COUNTER_PANEL_THICKNESS_METRES;
    let height = size.y - COUNTER_TOP_THICKNESS_METRES;
    builder.timber(
        Vec3::new(sign * (size.x - panel) * 0.5, height * 0.5, 0.0),
        Vec3::new(panel, height, size.z),
    );
}

pub(super) fn display(builder: &mut Builder, size: Vec3) {
    let rim = 0.045;
    let tray_height = size.y - 0.12;
    legs(builder, size, tray_height, 0.085);
    builder.timber(
        Vec3::Y * (tray_height - 0.025),
        Vec3::new(size.x, 0.05, size.z),
    );
    builder.timber(
        Vec3::Y * 0.25,
        Vec3::new(size.x - 0.04, 0.045, size.z - 0.04),
    );
    for sign in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(
                0.0,
                (tray_height + size.y) * 0.5,
                sign * (size.z - rim) * 0.5,
            ),
            Vec3::new(size.x, size.y - tray_height, rim),
        );
        builder.timber(
            Vec3::new(
                sign * (size.x - rim) * 0.5,
                (tray_height + size.y) * 0.5,
                0.0,
            ),
            Vec3::new(rim, size.y - tray_height, size.z),
        );
    }
}
