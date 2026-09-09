//! A rear bowl joined to one continuous chin/neck guard, with an overlapping visor.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::{
    CloseHelmetDesign,
    geometry::{AROUND, Surface, comb},
    shapes::curved_patch,
};
use crate::{GenerateError, PartMesh};

const REAR_START: usize = AROUND / 6;
const BOWL_ROWS: usize = 8;
const GUARD_ROWS: usize = 6;
const GUARD_TOP_HEIGHT: f32 = -0.80;
const GUARD_BOTTOM_HEIGHT: f32 = -1.26;
const GUARD_TOP_RADIUS: f32 = 0.95;
const GUARD_THROAT_RADIUS: f32 = 0.70;
const GUARD_THROAT_ROW: f32 = 0.70;
const LATERAL_FLARE_SCALE: f32 = 0.5;
const FRONT_FLARE_SCALE: f32 = 0.6;
const POSTERIOR_THROAT_FULLNESS: f32 = 0.08;

pub(super) fn generate(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &CloseHelmetDesign,
) -> Result<PartMesh, GenerateError> {
    let mut surface = Surface::default();
    let rim = surface.dome(radii, brow);
    let mut previous = rim[REAR_START..=AROUND - REAR_START].to_vec();
    let mut guard_top = Vec::new();
    for row in 1..=BOWL_ROWS {
        let t = row as f32 / BOWL_ROWS as f32;
        let full = row == BOWL_ROWS;
        let range = if full {
            0..AROUND
        } else {
            REAR_START..AROUND - REAR_START + 1
        };
        let ring = range
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let radius = 1.0 + (GUARD_TOP_RADIUS - 1.0) * t.powi(2);
                let lower = guard_height(half_height, d, angle, 0.0);
                surface.vertex([
                    radii[0] * radius * angle.sin(),
                    brow + (lower - brow) * t,
                    radii[2] * radius * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        surface.connect(
            &previous,
            if full {
                &ring[REAR_START..=AROUND - REAR_START]
            } else {
                &ring
            },
            false,
        );
        if full {
            guard_top = ring
        } else {
            previous = ring
        }
    }
    previous = guard_top;
    for row in 1..=GUARD_ROWS {
        let t = row as f32 / GUARD_ROWS as f32;
        let neck = (t / GUARD_THROAT_ROW).min(1.0);
        let taper = GUARD_TOP_RADIUS + (GUARD_THROAT_RADIUS - GUARD_TOP_RADIUS) * neck;
        let flare =
            d.throat_flare.metres() * ((t - GUARD_THROAT_ROW) / (1.0 - GUARD_THROAT_ROW)).max(0.0);
        let ring = (0..AROUND)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                let depth_taper =
                    taper + POSTERIOR_THROAT_FULLNESS * neck * (-angle.cos()).max(0.0);
                surface.vertex([
                    (radii[0] * taper + flare * LATERAL_FLARE_SCALE) * angle.sin(),
                    guard_height(half_height, d, angle, t),
                    (radii[2] * depth_taper + flare * FRONT_FLARE_SCALE) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &ring, true);
        previous = ring;
    }
    let mut mesh = surface.shell(d.fit.wall_thickness.metres())?;
    mesh.append(curved_patch(
        -FRAC_PI_2,
        FRAC_PI_2,
        |t, angle| {
            let top = brow - d.sight_gap.metres();
            let ridge = d.visor_projection.metres() * (PI * t).sin().max(0.0);
            [
                radii[0] * (1.0 - 0.05 * t.powi(2)) * angle.sin(),
                top + (-half_height * 0.82 - top) * t,
                radii[2] * angle.cos() + ridge * angle.cos().powi(2),
            ]
        },
        d.fit.wall_thickness.metres(),
    )?);
    if d.comb_height.0 > 0 {
        mesh.append(comb(
            radii,
            brow,
            d.comb_height.metres(),
            d.fit.wall_thickness.metres(),
        )?);
    }
    Ok(mesh)
}

fn guard_height(half_height: f32, d: &CloseHelmetDesign, angle: f32, t: f32) -> f32 {
    half_height * (GUARD_TOP_HEIGHT + (GUARD_BOTTOM_HEIGHT - GUARD_TOP_HEIGHT) * t)
        + d.back_edge_lift.metres() * (1.0 - angle.cos()) * 0.5
}
