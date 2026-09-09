//! Visby-style hood and pendant breast/back flaps, with open shoulders.

use super::{
    CoifDesign, CoifDrapeProfile, CoifFlapDrape,
    drape::FLAP_EDGE,
    geometry::{AROUND, Surface},
};
use crate::{GenerateError, parametric::PartMesh};
use std::f32::consts::TAU;

const HOOD_ROWS: usize = 8;
const NECK_ROWS: usize = 8;
const FLAP_ROWS: usize = 10;
const FACE_EDGE: usize = AROUND / 8;
const DRAPE_CONTACT_ROUNDING_M: f32 = 0.002;
// Fitted morph deltas interpolate a nonlinear torso envelope. Preserve a small
// rear-panel reserve between its fixed attachment, side edges and lower hem.
const BACK_FLAP_MORPH_RESERVE_M: f32 = 0.005;
const UNDER_CHIN_MORPH_RESERVE_M: f32 = 0.002;

pub(super) fn generate(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &CoifDesign,
) -> Result<PartMesh, GenerateError> {
    let drape = CoifDrapeProfile::authored(d, half_height, radii);
    generate_fitted(radii, brow, half_height, d, &drape)
}

pub(super) fn generate_fitted(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &CoifDesign,
    drape: &CoifDrapeProfile,
) -> Result<PartMesh, GenerateError> {
    let mut surface = Surface::default();
    let rim = surface.full_dome(radii, brow, 0.72);
    let mut previous = rim[FACE_EDGE..=AROUND - FACE_EDGE].to_vec();
    let mut neck = Vec::new();
    for row in 1..=HOOD_ROWS {
        let t = row as f32 / HOOD_ROWS as f32;
        let full = row == HOOD_ROWS;
        let range = if full {
            0..AROUND
        } else {
            FACE_EDGE..AROUND - FACE_EDGE + 1
        };
        let ring = range
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let end_y = half_height * (-1.12 + 0.16 * (1.0 - angle.cos()));
                let front = angle.cos().max(0.0);
                let back = (-angle.cos()).max(0.0);
                surface.vertex([
                    radii[0] * (1.0 - 0.13 * t.powi(2)) * angle.sin(),
                    brow + (end_y - brow) * t
                        - UNDER_CHIN_MORPH_RESERVE_M * front.powi(4) * t.powi(2),
                    radii[2]
                        * (1.0 - 0.35 * t.powi(2) * front + 0.05 * t.powi(2) * back)
                        * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        let strip = if full {
            &ring[FACE_EDGE..=AROUND - FACE_EDGE]
        } else {
            &ring
        };
        surface.connect(&previous, strip, false);
        if full {
            neck = ring;
        } else {
            previous = ring;
        }
    }
    // The face opening ends below the chin; the continuous neck enclosure
    // continues to the anatomical neck/shoulder junction before flaps split.
    neck = neck_tube(&mut surface, &neck, drape);
    let front_ids = (AROUND - FLAP_EDGE..AROUND)
        .chain(0..=FLAP_EDGE)
        .map(|i| neck[i])
        .collect::<Vec<_>>();
    let back_ids = neck[AROUND / 2 - FLAP_EDGE..=AROUND / 2 + FLAP_EDGE].to_vec();
    flap(
        &mut surface,
        &front_ids,
        d.front_flap_length.metres(),
        d.flap_width.unit(),
        &drape.front,
        FlapFacing::Front,
    );
    flap(
        &mut surface,
        &back_ids,
        d.back_flap_length.metres(),
        d.flap_width.unit(),
        &drape.back,
        FlapFacing::Back,
    );
    surface.shell(d.fit.wall_thickness.metres())
}

fn neck_tube(surface: &mut Surface, upper: &[u32], drape: &CoifDrapeProfile) -> Vec<u32> {
    let tops = upper
        .iter()
        .map(|i| surface.positions[*i as usize])
        .collect::<Vec<_>>();
    let mut previous = upper.to_vec();
    for row in 1..=NECK_ROWS {
        let t = row as f32 / NECK_ROWS as f32;
        let ring = tops
            .iter()
            .enumerate()
            .map(|(i, top)| {
                let bottom = drape.neck.point(i as f32 / AROUND as f32 * TAU);
                surface.vertex(std::array::from_fn(|axis| {
                    top[axis] + (bottom[axis] - top[axis]) * t
                }))
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &ring, true);
        previous = ring;
    }
    previous
}

#[derive(Clone, Copy)]
enum FlapFacing {
    Front,
    Back,
}

impl FlapFacing {
    fn sign(self) -> f32 {
        match self {
            Self::Front => 1.0,
            Self::Back => -1.0,
        }
    }
}

fn flap(
    surface: &mut Surface,
    attachment: &[u32],
    length: f32,
    width: f32,
    drape: &CoifFlapDrape,
    facing: FlapFacing,
) {
    let tops = attachment
        .iter()
        .map(|i| surface.positions[*i as usize])
        .collect::<Vec<_>>();
    let edge_x = tops.first().unwrap()[0].abs();
    let mut previous = attachment.to_vec();
    for row in 1..=FLAP_ROWS {
        let t = row as f32 / FLAP_ROWS as f32;
        let ring = tops
            .iter()
            .map(|p| {
                let y = p[1] - length * t;
                let fitted = drape.depth(y, p[0] / edge_x);
                let transition = (t * 3.0).min(1.0);
                let blend = transition.powi(2) * (3.0 - 2.0 * transition);
                let body_profile = p[2] + (fitted - p[2]) * blend;
                // Hanging mail bridges a neck hollow. Its support chord runs
                // from the hood attachment to the lower chest/back contact;
                // a body bulge can push it outward, but cannot pull it inward.
                let lower = drape.depth(p[1] - length, p[0] / edge_x);
                let chord = p[2] + (lower - p[2]) * t;
                let sign = facing.sign();
                let a = sign * body_profile;
                let b = sign * chord;
                let rounding = DRAPE_CONTACT_ROUNDING_M * (std::f32::consts::PI * t).sin();
                let reserve = match facing {
                    FlapFacing::Front => 0.0,
                    FlapFacing::Back => {
                        BACK_FLAP_MORPH_RESERVE_M
                            * (std::f32::consts::PI * t).sin().powi(2)
                            * (1.0 - (p[0] / edge_x).powi(2)).max(0.0)
                    }
                };
                let z =
                    sign * (a + b + ((a - b).powi(2) + rounding.powi(2)).sqrt()) * 0.5 - reserve;
                surface.vertex([p[0] * (1.0 + (width - 1.0) * t), y, z])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &ring, false);
        previous = ring;
    }
}
