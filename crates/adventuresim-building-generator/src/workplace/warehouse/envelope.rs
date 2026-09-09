use super::*;

const LOADING_PORTAL_WIDTH_METRES: f32 = 2.8;
const LOADING_PORTAL_HEIGHT_METRES: f32 = 2.8;
const VENTILATION_BAY_METRES: f32 = 3.0;

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
    for (start, end) in [(0.0, w * 0.5 - 1.9), (w * 0.5 + 1.9, w)] {
        a.wall(
            Vec2::new(start, 0.0),
            Vec2::new(end, 0.0),
            Vec2::NEG_Y,
            0.0,
            h,
            false,
        );
    }
    a.wall(
        Vec2::new(w * 0.5 - 1.9, 0.0),
        Vec2::new(w * 0.5 + 1.9, 0.0),
        Vec2::NEG_Y,
        LOADING_PORTAL_HEIGHT_METRES,
        h - LOADING_PORTAL_HEIGHT_METRES,
        false,
    );
    a.wall(Vec2::new(0.0, d), Vec2::new(w, d), Vec2::Y, 0.0, h, false);
    long_wall(a, 0.0, d, h, Vec2::NEG_X, &[]);
    let loading = [d * 0.25, d * 0.5];
    long_wall(a, w, d, h, Vec2::X, &loading);
    a.passage(
        Vec3::new(w * 0.5 - 1.5, 0.18, 0.0),
        Vec3::new(w * 0.5 + 1.5, 2.7, d - 0.35),
    );
    // The apron is part of the reserved lot, with a cart lane beyond the hood's outer posts.
    a.passage(Vec3::new(w + 5.0, 0.05, 0.0), Vec3::new(w + 7.8, 2.7, d));
    for z in loading {
        a.passage(
            Vec3::new(w * 0.5 - 0.1, 0.18, z - 1.1),
            Vec3::new(w + 7.8, 2.7, z + 1.1),
        );
    }
}

fn long_wall(a: &mut Assembly<'_>, x: f32, d: f32, h: f32, outward: Vec2, loading: &[f32]) {
    let half = LOADING_PORTAL_WIDTH_METRES * 0.5;
    let mut boundaries = vec![0.0, d];
    for &centre in loading {
        boundaries.extend([centre - half, centre + half]);
    }
    boundaries.sort_by(f32::total_cmp);
    for pair in boundaries.windows(2) {
        let centre = (pair[0] + pair[1]) * 0.5;
        if loading.iter().any(|z| (z - centre).abs() < half) {
            a.wall(
                Vec2::new(x, pair[0]),
                Vec2::new(x, pair[1]),
                outward,
                LOADING_PORTAL_HEIGHT_METRES,
                h - LOADING_PORTAL_HEIGHT_METRES,
                false,
            );
        } else {
            ventilated_bays(a, x, pair[0], pair[1], h, outward);
        }
    }
}

fn ventilated_bays(a: &mut Assembly<'_>, x: f32, start: f32, end: f32, h: f32, outward: Vec2) {
    let bays = ((end - start) / VENTILATION_BAY_METRES).ceil() as u32;
    let length = (end - start) / bays as f32;
    for bay in 0..bays {
        let from = start + bay as f32 * length;
        let centre = from + length * 0.5;
        for (left, right) in [(from, centre - 0.4), (centre + 0.4, from + length)] {
            a.wall(
                Vec2::new(x, left),
                Vec2::new(x, right),
                outward,
                0.0,
                h,
                false,
            );
        }
        // Both rows open into real storage floors, with masonry sills and lintels.
        for (base, height) in [(0.0, 1.8), (2.5, 2.0), (5.2, h - 5.2)] {
            a.wall(
                Vec2::new(x, centre - 0.4),
                Vec2::new(x, centre + 0.4),
                outward,
                base,
                height,
                false,
            );
        }
    }
}
