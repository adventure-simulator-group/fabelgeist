//! Each helmet's carriers and design floats, family by family.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::codes::{CheekRow, PostMap, Slot, VertexKind};
use super::surface::{AROUND, CoordSurface};
use crate::gpu::coord::{CoordExtrusion, CoordKernel, CoordShell};
use crate::gpu::recipe::PartRecipe;
use crate::{
    ArmorComponentRole, BarbuteDesign, GenerateError, HelmetCrown, HelmetDesign, HelmetFit,
    KettleHatDesign, MorionDesign, SalletDesign, VisoredSalletDesign,
};

/// The arming cap's dome keeps its width high up the skull.
const ARMING_CAP_FULLNESS: f32 = 0.72;
/// The kettle hat seats higher on the forehead than its brow line.
const KETTLE_FOREHEAD_SEATING_RISE: f32 = 0.12;
/// The kettle hat's brim reaches further fore and aft than at the sides,
/// and sweeps evenly all round.
const KETTLE_BRIM_REACH: f32 = 1.25;
const KETTLE_BRIM_BACK_SWEEP: f32 = 1.0;
const BRIM_RINGS: usize = 5;
/// Rim samples on each side of the front left open by a barbute's face.
const BARBUTE_OPENING_START: usize = AROUND / 8;
/// Extra barbute cheek columns between mouth and eye on each side.
const BARBUTE_CHEEK_COLUMNS: usize = 12;
const BARBUTE_CORNER_ROWS: usize = 4;
const BARBUTE_LOWER_ROWS: usize = 8;
/// A rounded corner smaller than this is square.
const BARBUTE_MINIMUM_CORNER_M: f32 = 1e-6;
const BARBUTE_CORNER_SHARE: f32 = 0.45;
const SALLET_SKIRT_ROWS: usize = 12;
const SALLET_OPENING_START: usize = AROUND / 6;
const VISOR_COLUMNS: usize = 64;
const VISOR_ROWS: usize = 10;

/// A helmet's design floats: named slots, then the bowl remap table.
pub(super) struct DesignFloats(Vec<f32>);

impl DesignFloats {
    fn new(fit: HelmetFit, crown: Option<&HelmetCrown>) -> Self {
        let mut floats = Self(vec![0.0; Slot::COUNT + AROUND]);
        floats.set(
            Slot::Gap,
            fit.clearance.metres() + fit.wall_thickness.metres(),
        );
        floats.set(Slot::CrownHeight, fit.crown_height.unit());
        floats.set(Slot::Gauge, fit.wall_thickness.metres());
        if let Some(crown) = crown {
            floats.set(Slot::Fullness, crown.fullness.unit());
            floats.set(Slot::Ridge, crown.ridge_height.metres());
            if let Some(f) = &crown.fluting {
                for (slot, value) in [
                    (Slot::Fluted, 1.0),
                    (Slot::FluteCount, f32::from(f.count.0)),
                    (Slot::FluteWidth, f.width.unit()),
                    (Slot::FluteDepth, f.depth.metres()),
                    (Slot::FluteSpread, f.spread.unit()),
                    (Slot::FluteLowerSpread, f.lower_spread.unit()),
                    (Slot::FluteStart, f.start.unit()),
                    (Slot::FluteEnd, f.end.unit()),
                    (Slot::FluteFade, f.fade.unit()),
                ] {
                    floats.set(slot, value);
                }
            }
        }
        floats
    }

    pub(super) fn set(&mut self, slot: Slot, value: f32) {
        self.0[slot as usize] = value;
    }

    pub(super) fn into_vec(self) -> Vec<f32> {
        self.0
    }
}

/// Shells in part order, each with the component it closes, if any.
pub(super) struct HelmetParts {
    pub shells: Vec<(CoordShell, Option<(ArmorComponentRole, bool)>)>,
    pub design: DesignFloats,
}

impl HelmetParts {
    pub(super) fn single(shell: CoordShell, design: DesignFloats) -> Self {
        Self {
            shells: vec![(shell, None)],
            design,
        }
    }

    /// Push every shell into a recipe evaluated by `kernel`.
    pub(super) fn recipe(
        self,
        kernel: &CoordKernel,
    ) -> Result<(PartRecipe, Vec<f32>), GenerateError> {
        let mut recipe = PartRecipe::new();
        for (mut shell, component) in self.shells {
            let hinge = component
                .as_ref()
                .is_some_and(|(_, hinged)| *hinged)
                .then(|| recipe.hinge());
            shell.hinge = hinge;
            recipe.push_coord(shell, kernel.clone())?;
            if let Some((role, _)) = component {
                recipe.component(role, hinge);
            }
        }
        Ok((recipe, self.design.into_vec()))
    }
}

pub(super) fn parts(design: &HelmetDesign) -> Result<HelmetParts, GenerateError> {
    let fit = design.fit();
    let mut floats = DesignFloats::new(fit, design.crown());
    let gauge = fit.wall_thickness.metres();
    Ok(match design {
        HelmetDesign::ArmingCap(_) => {
            floats.set(Slot::Fullness, ARMING_CAP_FULLNESS);
            let mut surface = CoordSurface::default();
            surface.full_dome();
            HelmetParts::single(surface.shell(gauge, CoordExtrusion::Normal), floats)
        }
        HelmetDesign::Morion(d) => morion(d, floats, gauge),
        HelmetDesign::KettleHat(d) => kettle_hat(d, floats, gauge),
        HelmetDesign::Barbute(d) => barbute(d, floats, gauge),
        HelmetDesign::Burgonet(d) => super::burgonet::burgonet(d, floats, gauge)?,
        HelmetDesign::Sallet(d) => {
            let skull = sallet_skull(d, &mut floats).shell(gauge, CoordExtrusion::Normal);
            HelmetParts::single(skull, floats)
        }
        HelmetDesign::VisoredSallet(d) => visored_sallet(d, floats, gauge),
        HelmetDesign::CloseHelmet(_) | HelmetDesign::MailCoif(_) => {
            return Err(GenerateError::InvalidSurface);
        }
    })
}

fn morion(d: &MorionDesign, mut floats: DesignFloats, gauge: f32) -> HelmetParts {
    for (slot, value) in [
        (Slot::BrimWidth, d.brim_width.metres()),
        (Slot::BrimSweep, d.brim_sweep.metres()),
        (Slot::FrontReach, d.front_reach.unit()),
        (Slot::BackReach, d.back_reach.unit()),
        (Slot::BackSweep, d.back_sweep.unit()),
    ] {
        floats.set(slot, value);
    }
    brimmed(&d.crown, d.comb_height.metres(), floats, gauge)
}

fn kettle_hat(d: &KettleHatDesign, mut floats: DesignFloats, gauge: f32) -> HelmetParts {
    for (slot, value) in [
        (Slot::BrowRise, KETTLE_FOREHEAD_SEATING_RISE),
        (Slot::BrimWidth, d.brim_width.metres()),
        (Slot::BrimDrop, d.brim_drop.metres()),
        (Slot::FrontReach, KETTLE_BRIM_REACH),
        (Slot::BackReach, KETTLE_BRIM_REACH),
        (Slot::BackSweep, KETTLE_BRIM_BACK_SWEEP),
    ] {
        floats.set(slot, value);
    }
    brimmed(&d.crown, 0.0, floats, gauge)
}

fn brimmed(crown: &HelmetCrown, crest: f32, mut floats: DesignFloats, gauge: f32) -> HelmetParts {
    floats.set(Slot::Crest, crest);
    let mut surface = CoordSurface::default();
    let mut ring = surface.styled_dome(crown, crest);
    for row in 1..=BRIM_RINGS {
        let t = row as f32 / BRIM_RINGS as f32;
        let next = (0..AROUND)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * TAU;
                surface.vertex(VertexKind::Brim, [t, angle, 0.0])
            })
            .collect::<Vec<_>>();
        surface.connect(&ring, &next, true);
        ring = next;
    }
    let extrusion = if crest > 0.0 {
        CoordExtrusion::AngleWeightedNormal
    } else {
        CoordExtrusion::Normal
    };
    HelmetParts::single(surface.shell(gauge, extrusion), floats)
}

fn barbute(d: &BarbuteDesign, mut floats: DesignFloats, gauge: f32) -> HelmetParts {
    let eye = d.eye_opening.radians();
    let mouth = d.mouth_opening.radians();
    for (slot, value) in [
        (Slot::PostMap, PostMap::BowlTable as u32 as f32),
        (Slot::EyeOpening, eye),
        (Slot::MouthOpening, mouth),
        (Slot::EyeHeight, d.eye_height.metres()),
        (Slot::OpeningRoundness, d.opening_roundness.unit()),
        (Slot::RearEdgeLift, d.rear_edge_lift.metres()),
        (Slot::NapeFlare, d.nape_flare.metres()),
        (Slot::ChinTaper, d.chin_taper.unit()),
        (Slot::CheekDepth, d.cheek_depth.unit()),
    ] {
        floats.set(slot, value);
    }
    let start = BARBUTE_OPENING_START;
    let angles = (0..AROUND)
        .map(|i| {
            let half = i.min(AROUND - i);
            let angle = if half <= start {
                eye * half as f32 / start as f32
            } else {
                eye + (PI - eye) * (half - start) as f32 / (AROUND / 2 - start) as f32
            };
            if i > AROUND / 2 { TAU - angle } else { angle }
        })
        .collect::<Vec<_>>();
    floats.0[Slot::COUNT..].copy_from_slice(&angles);
    let mut surface = CoordSurface::default();
    let rim = surface.styled_dome(&d.crown, 0.0);
    let columns = BARBUTE_CHEEK_COLUMNS;
    let mut lower_angles = (0..columns)
        .map(|i| mouth + (eye - mouth) * i as f32 / columns as f32)
        .collect::<Vec<_>>();
    lower_angles.extend(&angles[start..=AROUND - start]);
    lower_angles
        .extend((1..=columns).map(|i| TAU - eye + (eye - mouth) * i as f32 / columns as f32));
    let corner = d.eye_height.metres() * BARBUTE_CORNER_SHARE * d.opening_roundness.unit();
    let mut rows = vec![(CheekRow::Shelf, 0)];
    if corner > BARBUTE_MINIMUM_CORNER_M {
        rows.extend((1..=BARBUTE_CORNER_ROWS).map(|i| (CheekRow::Corner, i)));
    }
    rows.extend((1..=BARBUTE_LOWER_ROWS).map(|i| (CheekRow::Lower, i)));
    let mut previous = Vec::new();
    for (row, step) in rows {
        let ring = lower_angles
            .iter()
            .map(|&angle| {
                surface.vertex(
                    VertexKind::BarbuteCheek,
                    [row as u32 as f32, step as f32, angle],
                )
            })
            .collect::<Vec<_>>();
        if previous.is_empty() {
            surface.connect(
                &rim[start..=AROUND - start],
                &ring[columns..ring.len() - columns],
                false,
            );
        } else {
            surface.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    HelmetParts::single(surface.shell(gauge, CoordExtrusion::Normal), floats)
}

fn sallet_skull(d: &SalletDesign, floats: &mut DesignFloats) -> CoordSurface {
    for (slot, value) in [
        (Slot::ZExtension, d.brow_projection.metres()),
        (Slot::PostMap, PostMap::FrontArc as u32 as f32),
        (Slot::OpeningWidth, d.opening_width.radians()),
        (Slot::RearEdgeLift, d.rear_edge_lift.metres()),
        (Slot::CheekDepth, d.cheek_depth.unit()),
        (Slot::TailWidth, d.tail_width.unit()),
        (Slot::TailLength, d.tail_length.metres()),
        (Slot::TailDrop, d.tail_drop.metres()),
    ] {
        floats.set(slot, value);
    }
    let mut surface = CoordSurface::default();
    let rim = surface.styled_dome(&d.crown, 0.0);
    let mut previous = rim[SALLET_OPENING_START..=AROUND - SALLET_OPENING_START].to_vec();
    let width = d.opening_width.radians();
    for row in 1..=SALLET_SKIRT_ROWS {
        let t = row as f32 / SALLET_SKIRT_ROWS as f32;
        let next = (0..previous.len())
            .map(|i| {
                let opening = width + (FRAC_PI_2 - width) * d.opening_sweep.unit() * t.powi(2);
                let angle =
                    opening + (TAU - 2.0 * opening) * i as f32 / (previous.len() - 1) as f32;
                surface.vertex(VertexKind::SalletSkirt, [t, angle, 0.0])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &next, false);
        previous = next;
    }
    surface
}

fn visored_sallet(d: &VisoredSalletDesign, mut floats: DesignFloats, gauge: f32) -> HelmetParts {
    let skull = sallet_skull(&d.skull, &mut floats).shell(gauge, CoordExtrusion::Normal);
    let skull_relief = d
        .skull
        .crown
        .fluting
        .map_or(0.0, |pattern| pattern.depth.metres());
    for (slot, value) in [
        (Slot::SightGap, d.sight_gap.metres()),
        (Slot::PivotRise, d.pivot_rise.metres()),
        (Slot::VisorHeight, d.visor_height.metres()),
        (Slot::SidePanel, d.side_panel.unit()),
        (Slot::VisorProjection, d.visor_projection.metres()),
        (Slot::VisorSpacing, gauge * 2.0 + skull_relief),
    ] {
        floats.set(slot, value);
    }
    let mut surface = CoordSurface::default();
    let mut previous = Vec::new();
    for row in 0..=VISOR_ROWS {
        let v = row as f32 / VISOR_ROWS as f32;
        let ring = (0..=VISOR_COLUMNS)
            .map(|column| {
                let angle = (2.0 * column as f32 / VISOR_COLUMNS as f32 - 1.0) * FRAC_PI_2;
                surface.vertex(VertexKind::Visor, [v, angle, 0.0])
            })
            .collect::<Vec<_>>();
        if row > 0 {
            surface.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    let visor = surface.shell(
        gauge,
        CoordExtrusion::Radial {
            origin: [0.0; 3],
            axis: [0.0, 1.0, 0.0],
        },
    );
    HelmetParts {
        shells: vec![
            (skull, Some((ArmorComponentRole::Skull, false))),
            (visor, Some((ArmorComponentRole::Visor, true))),
        ],
        design: floats,
    }
}
