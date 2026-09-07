//! Periodic lime binder with discrete aggregate, application strokes and stone contact.

use super::{cell_id, hash_unit, noise, smoothstep};

const TILE_METRES: f32 = super::super::DRESSED_STONE_TILE_METRES;
const AGGREGATE_CELLS: i32 = 600;
const STROKE_CELLS: i32 = 40;
const STROKE_DENSITY: f32 = 0.65;
const CONTACT_WIDTH_METRES: f32 = 0.012;
const BINDER_HEIGHT: f32 = 0.12;
const AGGREGATE_RELIEF: f32 = 0.040;
const STROKE_RELIEF: f32 = 0.024;
const RECESS_VARIATION: f32 = 0.024;
const CONTACT_BUILDUP: f32 = 0.020;
const POCKET_DEPTH: f32 = 0.055;
const AGGREGATE_RADIUS_CELLS: [f32; 2] = [0.23, 0.38];
const STROKE_ANGLE_SPREAD: f32 = 0.65;
const STROKE_TAPER_CELLS: [f32; 2] = [0.35, 0.80];
const STROKE_CURVATURE: f32 = 0.12;
const STROKE_HALF_WIDTH_CELLS: f32 = 0.09;
const POCKET_RADII_CELLS: [f32; 2] = [0.16, 0.23];
const RECESS_NOISE_SCALE: f32 = 0.12;

pub(in super::super) struct MortarDetail {
    pub(in super::super) height: f32,
}

// Cell centers are evaluated in world coordinates and IDs wrap independently;
// particles crossing a repeat boundary have exactly the same shape on both sides.
fn aggregate(x: f32, y: f32) -> f32 {
    let spacing = TILE_METRES / AGGREGATE_CELLS as f32;
    let px = x / spacing;
    let py = y / spacing;
    let mut field = 0.0_f32;
    for iy in (py.floor() as i32 - 1)..=(py.floor() as i32 + 1) {
        for ix in (px.floor() as i32 - 1)..=(px.floor() as i32 + 1) {
            let id = cell_id(
                ix.rem_euclid(AGGREGATE_CELLS),
                iy.rem_euclid(AGGREGATE_CELLS),
                0x6c31,
            );
            let dx = px - ix as f32 - hash_unit(id);
            let dy = py - iy as f32 - hash_unit(id ^ 0x731a);
            let radius = super::between(AGGREGATE_RADIUS_CELLS, hash_unit(id ^ 0x27ca));
            let dome = (1.0 - (dx * dx + dy * dy) / (radius * radius)).max(0.0);
            field = field.max(dome);
        }
    }
    field
}

fn strokes(x: f32, y: f32) -> (f32, f32) {
    let spacing = TILE_METRES / STROKE_CELLS as f32;
    let px = x / spacing;
    let py = y / spacing;
    let mut stroke = 0.0_f32;
    let mut pocket = 0.0_f32;
    for iy in (py.floor() as i32 - 1)..=(py.floor() as i32 + 1) {
        for ix in (px.floor() as i32 - 1)..=(px.floor() as i32 + 1) {
            let id = cell_id(
                ix.rem_euclid(STROKE_CELLS),
                iy.rem_euclid(STROKE_CELLS),
                0x314b,
            );
            if hash_unit(id) > STROKE_DENSITY {
                continue;
            }
            let dx = px - ix as f32 - hash_unit(id ^ 0x84d1);
            let dy = py - iy as f32 - hash_unit(id ^ 0x3a17);
            let angle = (hash_unit(id ^ 0x374a) - 0.5) * STROKE_ANGLE_SPREAD;
            let (sin, cos) = angle.sin_cos();
            let along = dx * cos + dy * sin;
            let across = -dx * sin + dy * cos;
            let taper = 1.0 - smoothstep(STROKE_TAPER_CELLS[0], STROKE_TAPER_CELLS[1], along.abs());
            let edge = (1.0
                - ((across - along * along * STROKE_CURVATURE) / STROKE_HALF_WIDTH_CELLS).abs())
            .max(0.0);
            stroke = stroke.max(edge * taper);
            // Small lost pockets share the application region but not the stroke ridge.
            let cavity =
                (1.0 - (dx / POCKET_RADII_CELLS[0]).powi(2) - (dy / POCKET_RADII_CELLS[1]).powi(2))
                    .max(0.0);
            pocket = pocket.max(cavity * cavity);
        }
    }
    (stroke, pocket)
}

pub(in super::super) fn mortar_detail(x: f32, y: f32, stone_distance: f32) -> MortarDetail {
    let (stroke, pocket) = strokes(x, y);
    let tau = std::f32::consts::TAU;
    let (sx, cx) = (x / TILE_METRES * tau).sin_cos();
    let (sy, cy) = (y / TILE_METRES * tau).sin_cos();
    let recession = noise(sx + cy, sy + cx, RECESS_NOISE_SCALE, 0x781a);
    let contact = 1.0 - smoothstep(0.0, CONTACT_WIDTH_METRES, stone_distance.max(0.0));
    MortarDetail {
        height: BINDER_HEIGHT
            + aggregate(x, y) * AGGREGATE_RELIEF
            + stroke * STROKE_RELIEF
            + recession * RECESS_VARIATION
            + contact * CONTACT_BUILDUP
            - pocket * POCKET_DEPTH,
    }
}
