use super::{assembly::Assembly, *};
use crate::CELL_SIZE_METRES;

const POST_WIDTH_METRES: f32 = 0.24;
const BEAM_DEPTH_METRES: f32 = 0.28;
const BOARD_WIDTH_METRES: f32 = 0.22;

pub(super) fn build_envelope(a: &mut Assembly<'_>, program: &BuildingProgram) {
    let (width, depth) = program.footprint.dimensions();
    let w = f32::from(width) * CELL_SIZE_METRES;
    let d = f32::from(depth) * CELL_SIZE_METRES;
    let h = program.storey_height_metres;
    let kind = a.plan.kind;
    let timber = matches!(
        kind,
        WorkplaceKind::Barn | WorkplaceKind::Stable | WorkplaceKind::MarketHall
    );
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Vec3::new(w * 0.5, 0.08, d * 0.5),
        Vec3::new(w, 0.16, d),
        true,
    );
    if kind == WorkplaceKind::MarketHall {
        market_frame(a, w, d, h);
    } else {
        let portal_width = if kind == WorkplaceKind::Barn {
            3.8
        } else {
            2.4
        };
        let portal_height = if kind == WorkplaceKind::Barn {
            3.2
        } else {
            2.5
        };
        for (z, outward) in [(0.0, Vec2::NEG_Y), (d, Vec2::Y)] {
            if z == d && kind != WorkplaceKind::Barn {
                a.wall(Vec2::new(0.0, z), Vec2::new(w, z), outward, 0.0, h, timber);
            } else {
                let left = (w - portal_width) * 0.5;
                let right = w - left;
                a.wall(
                    Vec2::new(0.0, z),
                    Vec2::new(left, z),
                    outward,
                    0.0,
                    h,
                    timber,
                );
                a.wall(
                    Vec2::new(right, z),
                    Vec2::new(w, z),
                    outward,
                    0.0,
                    h,
                    timber,
                );
                if kind == WorkplaceKind::Granary && z == 0.0 {
                    a.wall(
                        Vec2::new(left, z),
                        Vec2::new(right, z),
                        outward,
                        portal_height,
                        3.5 - portal_height,
                        false,
                    );
                    a.wall(
                        Vec2::new(left, z),
                        Vec2::new(right, z),
                        outward,
                        5.3,
                        h - 5.3,
                        false,
                    );
                } else {
                    a.wall(
                        Vec2::new(left, z),
                        Vec2::new(right, z),
                        outward,
                        portal_height,
                        h - portal_height,
                        timber,
                    );
                }
            }
        }
        if timber {
            a.wall(Vec2::ZERO, Vec2::new(0.0, d), Vec2::NEG_X, 0.0, h, true);
        } else {
            ventilated_side(a, 0.0, d, h, Vec2::NEG_X);
        }
        if kind == WorkplaceKind::Stable {
            open_side(a, w, d, h);
        } else if timber {
            a.wall(Vec2::new(w, 0.0), Vec2::new(w, d), Vec2::X, 0.0, h, true);
        } else {
            ventilated_side(a, w, d, h, Vec2::X);
        }
        if timber {
            boarding(a, w, d, h, kind);
        }
        let half = portal_width * 0.5 - 0.12;
        a.passage(
            Vec3::new(w * 0.5 - half, 0.18, 0.0),
            Vec3::new(
                w * 0.5 + half,
                portal_height - 0.1,
                if kind == WorkplaceKind::Barn {
                    d
                } else {
                    d - 0.35
                },
            ),
        );
    }
    working_yard(a, w, d);
    if matches!(kind, WorkplaceKind::Barn | WorkplaceKind::Stable) {
        yard_shed(a, w, d);
    }
}

fn yard_shed(a: &mut Assembly<'_>, w: f32, d: f32) {
    let west = w + 1.8;
    let east = w + 4.2;
    let front = d - 4.2;
    let back = d - 0.6;
    a.wall(
        Vec2::new(west, front),
        Vec2::new(west, back),
        Vec2::NEG_X,
        0.0,
        2.2,
        true,
    );
    a.wall(
        Vec2::new(east, front),
        Vec2::new(east, back),
        Vec2::X,
        0.0,
        2.2,
        true,
    );
    a.wall(
        Vec2::new(west, back),
        Vec2::new(east, back),
        Vec2::Y,
        0.0,
        2.2,
        true,
    );
    a.wall(
        Vec2::new(west, front),
        Vec2::new(east, front),
        Vec2::NEG_Y,
        1.95,
        0.25,
        true,
    );
}

fn market_frame(a: &mut Assembly<'_>, w: f32, d: f32, h: f32) {
    // Each side is a continuous wall-plate on independent grounded posts.
    for x in [0.0, w] {
        for bay in 0..=(d / 3.0) as u32 {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Vec3::new(x, (h - BEAM_DEPTH_METRES) * 0.5, bay as f32 * 3.0),
                Vec3::new(POST_WIDTH_METRES, h - BEAM_DEPTH_METRES, POST_WIDTH_METRES),
                true,
            );
        }
        a.wall(
            Vec2::new(x, 0.0),
            Vec2::new(x, d),
            if x == 0.0 { Vec2::NEG_X } else { Vec2::X },
            h - BEAM_DEPTH_METRES,
            BEAM_DEPTH_METRES,
            true,
        );
    }
    for z in [0.0, d] {
        a.wall(
            Vec2::new(0.0, z),
            Vec2::new(w, z),
            if z == 0.0 { Vec2::NEG_Y } else { Vec2::Y },
            h - BEAM_DEPTH_METRES,
            BEAM_DEPTH_METRES,
            true,
        );
    }
    a.passage(
        Vec3::new(w * 0.5 - 1.4, 0.18, 0.0),
        Vec3::new(w * 0.5 + 1.4, 2.8, d),
    );
}

fn open_side(a: &mut Assembly<'_>, w: f32, d: f32, h: f32) {
    for bay in 0..=(d / 3.0) as u32 {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Vec3::new(w, 1.25, bay as f32 * 3.0),
            Vec3::new(POST_WIDTH_METRES, 2.5, POST_WIDTH_METRES),
            true,
        );
    }
    a.wall(
        Vec2::new(w, 0.0),
        Vec2::new(w, d),
        Vec2::X,
        2.5,
        h - 2.5,
        true,
    );
}

fn boarding(a: &mut Assembly<'_>, w: f32, d: f32, h: f32, kind: WorkplaceKind) {
    // Raised board edges catch light at close range; the continuous panel remains at distance.
    for x in [0.0, w] {
        if x == w && kind == WorkplaceKind::Stable {
            continue;
        }
        for board in 0..(d / BOARD_WIDTH_METRES) as u32 {
            a.part(
                WorkplaceFeature::Boarding,
                WorkplaceMaterial::Timber,
                Vec3::new(
                    x + if x == 0.0 { -0.10 } else { 0.10 },
                    h * 0.5,
                    (board as f32 + 0.5) * BOARD_WIDTH_METRES,
                ),
                Vec3::new(0.035, h, BOARD_WIDTH_METRES - 0.025),
                false,
            );
        }
    }
    for x in [0.0, w] {
        for z in [0.0, d * 0.5, d] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Vec3::new(x, h * 0.5, z),
                Vec3::new(0.3, h, 0.3),
                true,
            );
        }
    }
}

fn working_yard(a: &mut Assembly<'_>, w: f32, d: f32) {
    let yard = a.plan.kind.yard_width_metres();
    if yard == 0.0 {
        return;
    }
    let edge = w + yard - 0.3;
    for bay in 0..=(d / 3.0) as u32 {
        a.part(
            WorkplaceFeature::Fence,
            WorkplaceMaterial::Timber,
            Vec3::new(edge, 0.7, bay as f32 * 3.0),
            Vec3::new(0.18, 1.4, 0.18),
            true,
        );
    }
    for y in [0.55, 1.05] {
        // Rails bear laterally against the posts; the support ledger records that connection.
        a.part(
            WorkplaceFeature::Fence,
            WorkplaceMaterial::Timber,
            Vec3::new(edge, y, d * 0.5),
            Vec3::new(0.14, 0.12, d),
            true,
        );
    }
    a.passage(Vec3::new(w + 0.4, 0.05, 0.0), Vec3::new(w + 1.7, 2.3, d));
}

fn ventilated_side(a: &mut Assembly<'_>, x: f32, d: f32, h: f32, outward: Vec2) {
    let bays = (d / 3.0) as u32;
    let bay_length = d / bays as f32;
    for bay in 0..bays {
        let start = bay as f32 * bay_length;
        let left = start + bay_length * 0.5 - 0.4;
        let right = left + 0.8;
        a.wall(
            Vec2::new(x, start),
            Vec2::new(x, left),
            outward,
            0.0,
            h,
            false,
        );
        a.wall(
            Vec2::new(x, right),
            Vec2::new(x, start + bay_length),
            outward,
            0.0,
            h,
            false,
        );
        a.wall(
            Vec2::new(x, left),
            Vec2::new(x, right),
            outward,
            0.0,
            2.0,
            false,
        );
        if a.plan.kind == WorkplaceKind::Granary {
            a.wall(
                Vec2::new(x, left),
                Vec2::new(x, right),
                outward,
                2.5,
                2.2,
                false,
            );
            a.wall(
                Vec2::new(x, left),
                Vec2::new(x, right),
                outward,
                5.2,
                h - 5.2,
                false,
            );
        } else {
            a.wall(
                Vec2::new(x, left),
                Vec2::new(x, right),
                outward,
                2.5,
                h - 2.5,
                false,
            );
        }
    }
}
