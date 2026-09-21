//! The burgonet's skull with its nape and peak, the separate neck guard,
//! and the two cheek plates.

use std::f32::consts::{FRAC_PI_3, TAU};

use super::codes::{Slot, VertexKind};
use super::parts::{DesignFloats, HelmetParts};
use super::surface::{AROUND, CoordSurface};
use crate::BurgonetDesign;
use crate::gpu::coord::{CoordExtrusion, CoordShell};

const BURGONET_SKIRT_ROWS: usize = 8;
const BURGONET_NAPE_START: usize = AROUND * 7 / 24;
const BURGONET_PEAK_HALF_COLUMNS: usize = AROUND / 6;
const BURGONET_PEAK_ROWS: usize = 4;
/// The neck guard starts this fraction of the nape above the skull's edge.
const NECK_LAP_FRACTION: f32 = 0.05;
const CHEEK_RADIAL_ROWS: usize = 16;
const CHEEK_COLUMNS: usize = 48;
/// The chin tab's falloff around the cheek's lower front.
const CHIN_TAB_SHARPNESS: i32 = 10;

/// Which shell a helmet kernel dispatch evaluates, as its first value.
pub(super) const CHEEK_SHELL: f32 = 1.0;

pub(super) fn burgonet(d: &BurgonetDesign, mut floats: DesignFloats, gauge: f32) -> HelmetParts {
    for (slot, value) in [
        (Slot::Crest, d.comb_height.metres()),
        (Slot::NapeDepth, d.nape_depth.unit()),
        (Slot::NapeTaper, d.nape_taper.unit()),
        (Slot::NapeRecession, d.nape_recession.metres()),
        (Slot::NeckFlare, d.neck_flare.metres()),
        (Slot::PeakLength, d.peak_length.metres()),
        (Slot::PeakDrop, d.peak_drop.metres()),
        (Slot::CheekWidth, d.cheek_width.unit()),
        (Slot::CheekTaper, d.cheek_taper.unit()),
        (Slot::CheekDepth, d.cheek_depth.unit()),
    ] {
        floats.set(slot, value);
    }
    let rows = BURGONET_SKIRT_ROWS;
    let nape = BURGONET_NAPE_START..=AROUND - BURGONET_NAPE_START;
    let meridian = |i: usize| i as f32 / AROUND as f32 * TAU;
    let mut skull = CoordSurface::default();
    let rim = skull.styled_dome(&d.crown, d.comb_height.metres());
    let mut previous = rim[nape.clone()].to_vec();
    let join = 1.0 - d.neck_guard_fraction.unit();
    for row in 1..=rows {
        let v = row as f32 / rows as f32;
        let ring = nape
            .clone()
            .map(|i| skull.vertex(VertexKind::Nape, [join * v, v, meridian(i)]))
            .collect::<Vec<_>>();
        skull.connect(&previous, &ring, false);
        previous = ring;
    }
    let half = BURGONET_PEAK_HALF_COLUMNS;
    let mut previous = (0..=2 * half)
        .map(|i| rim[(AROUND - half + i) % AROUND])
        .collect::<Vec<_>>();
    for row in 1..=BURGONET_PEAK_ROWS {
        let t = row as f32 / BURGONET_PEAK_ROWS as f32;
        let next = (0..=2 * half)
            .map(|i| {
                let angle = -FRAC_PI_3 + 2.0 * FRAC_PI_3 * i as f32 / (2 * half) as f32;
                skull.vertex(VertexKind::Peak, [t, angle, 0.0])
            })
            .collect::<Vec<_>>();
        skull.connect(&previous, &next, false);
        previous = next;
    }
    let mut guard = CoordSurface::default();
    let mut previous = Vec::new();
    for row in 0..=rows {
        let t =
            join - NECK_LAP_FRACTION + (1.0 - join + NECK_LAP_FRACTION) * row as f32 / rows as f32;
        let ring = nape
            .clone()
            .map(|i| guard.vertex(VertexKind::Guard, [t, meridian(i), 0.0]))
            .collect::<Vec<_>>();
        if row > 0 {
            guard.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    let cheek = cheek(d, gauge);
    let mut shells = vec![
        (skull.shell(gauge, CoordExtrusion::Normal), None),
        (
            guard.shell(
                gauge,
                CoordExtrusion::Radial {
                    origin: [0.0; 3],
                    axis: [0.0, 1.0, 0.0],
                },
            ),
            None,
        ),
    ];
    // The left cheek is the right one reflected.
    for mirrored in [true, false] {
        let mut shell = cheek.clone();
        shell.mirrored = mirrored;
        shells.push((shell, None));
    }
    HelmetParts {
        shells,
        design: floats,
    }
}

/// A cheek plate fanned from its root: the pole, then rings of columns.
/// Its coordinates are the ring fraction, the fan angle, the chin tab and
/// the relief, all of which the design alone decides.
fn cheek(d: &BurgonetDesign, gauge: f32) -> CoordShell {
    let pattern = d.cheek_fluting.as_ref();
    let mut columns = pattern.map_or_else(
        || {
            (0..=CHEEK_COLUMNS)
                .map(|i| i as f32 / CHEEK_COLUMNS as f32)
                .collect()
        },
        |p| p.columns(CHEEK_COLUMNS),
    );
    columns.pop();
    // The cheek laps outside the rear skull sheet by one gauge of air.
    let separation = gauge * 2.0;
    let mut surface = CoordSurface::default();
    let pole = surface.coord([0.0, 0.0, 0.0, separation]);
    let mut previous: Vec<u32> = Vec::new();
    for row in 1..=CHEEK_RADIAL_ROWS {
        let v = row as f32 / CHEEK_RADIAL_ROWS as f32;
        let mut ring = Vec::new();
        for u in &columns {
            let mapped = pattern.map_or(*u, |p| p.fan_coordinate(*u, v));
            let angle = (mapped - 0.5) * TAU;
            let tab = d.chin_tab.metres()
                * (angle + 3.0 * TAU / 8.0)
                    .cos()
                    .max(0.0)
                    .powi(CHIN_TAB_SHARPNESS);
            let relief = separation + pattern.map_or(0.0, |p| p.relief(*u, v));
            ring.push(surface.coord([v, angle, tab, relief]));
        }
        if previous.is_empty() {
            for i in 0..ring.len() {
                surface
                    .indices
                    .extend([pole, ring[(i + 1) % ring.len()], ring[i]]);
            }
        } else {
            for i in 0..ring.len() {
                let next = (i + 1) % ring.len();
                surface.indices.extend([
                    previous[i],
                    previous[next],
                    ring[next],
                    previous[i],
                    ring[next],
                    ring[i],
                ]);
            }
        }
        previous = ring;
    }
    let mut shell = surface.shell(gauge, CoordExtrusion::Normal);
    shell.values[0] = CHEEK_SHELL;
    shell
}
