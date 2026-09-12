//! Smooth axial clearance envelopes for long plates, independent of body topology.
use crate::armor_frames::{FitRegion, Wearer};
use adventuresim_armor_model::{Millimeters, PartFrame, PartMesh};
use anyhow::{Result, ensure};

const STATIONS: usize = 13;
const STATION_HALF_WIDTH_M: f32 = 0.024;
const UPPER_ARM_STATION_HALF_WIDTH_M: f32 = 0.012;
const FIT_MARGIN_M: f32 = 0.004;
/// Section spacing targets one wall gauge of air between successive plates.
pub(super) const LAME_SPACING_GAUGES: f32 = 2.0;

/// Anatomical fit and style allowance are separate from embossed relief.
#[derive(Clone, Copy)]
pub enum PlateFit<'a> {
    Greave(&'a adventuresim_armor_model::GreaveDesign),
    Cuisse(&'a adventuresim_armor_model::CuisseDesign),
    Rerebrace(&'a adventuresim_armor_model::RerebraceDesign),
    Mitten {
        cuff_length: f32,
        cuff_clearance: Millimeters,
    },
}
impl PlateFit<'_> {
    fn axial_span(self, half_height: f32) -> [f32; 2] {
        match self {
            Self::Greave(d) => [-1.0 - d.ankle_extension.metres() / half_height, 1.0],
            Self::Mitten { cuff_length, .. } => [-1.0, 0.94 + 2.0 * cuff_length],
            _ => [-1.0, 1.0],
        }
    }
    fn allowance(self, v: f32, direction: [f32; 2], width: f32) -> f32 {
        match self {
            Self::Greave(d) => {
                let calf = d.calf_height.unit();
                let ankle = (1.0 - v / calf).max(0.0).powi(2);
                let knee = ((v - calf) / (1.0 - calf)).max(0.0).powi(2);
                let belly = (1.0 - ((v - calf) / 0.35).abs()).max(0.0).powi(2);
                width
                    * (0.30 * (d.ankle_taper.unit() - 0.45) * ankle
                        + 0.20 * (d.knee_taper.unit() - 0.70) * knee
                        + 0.06 * belly)
            }
            Self::Cuisse(d) => width * 0.25 * (d.knee_taper.unit() - 0.55) * (1.0 - v).powi(2),
            Self::Rerebrace(d) => {
                width
                    * (0.25 * (d.distal_taper.unit() - 0.60) * (1.0 - v).powi(2)
                        + 0.35 * (d.section_depth.unit() - 0.85) * direction[1].powi(2))
            }
            Self::Mitten { .. } => 0.0,
        }
    }
}

use crate::plate_section::PlateSection as Section;

pub fn fit(
    mesh: &PartMesh,
    wearer: &Wearer<'_>,
    region: FitRegion,
    clearance: Millimeters,
    thickness: Millimeters,
    style: PlateFit<'_>,
) -> Result<PartMesh> {
    let frame = wearer.frame(region)?;
    let mut support = wearer.support_indices(region)?;
    if let FitRegion::Hand(side) = region {
        support.extend(wearer.support_indices(FitRegion::Forearm(side))?);
    }
    if let FitRegion::LowerLeg(side) = region {
        support.extend(
            wearer
                .support_indices(FitRegion::Foot(side))?
                .into_iter()
                .filter(|i| local(&frame, wearer.positions[*i])[2] <= frame.half_extents[2]),
        );
    }
    let points = support
        .iter()
        .map(|i| local(&frame, wearer.positions[*i]))
        .collect::<Vec<_>>();
    ensure!(
        !points.is_empty(),
        "plate has no anatomical section samples"
    );
    let half_width = if matches!(region, FitRegion::UpperArm(_) | FitRegion::LowerLeg(_)) {
        UPPER_ARM_STATION_HALF_WIDTH_M
    } else {
        STATION_HALF_WIDTH_M
    };
    let [low, high] = style.axial_span(frame.half_extents[1]);
    let mut owned = vec![false; wearer.positions.len()];
    for index in support {
        owned[index] = true;
    }
    let triangles: Vec<_> = wearer
        .faces
        .iter()
        .filter(|face| face.iter().any(|index| owned[*index as usize]))
        .map(|face| face.map(|index| local(&frame, wearer.positions[index as usize])))
        .collect();
    let sections: Vec<_> = (0..STATIONS)
        .map(|i| {
            let y = frame.half_extents[1] * (low + (high - low) * i as f32 / (STATIONS - 1) as f32);
            if matches!(style, PlateFit::Rerebrace(_)) {
                Section::measured_surface(&triangles, y, half_width)
                    .ok_or_else(|| anyhow::anyhow!("upper-arm section has no surface support"))
            } else {
                Ok(Section::measured(&points, y, half_width))
            }
        })
        .collect::<Result<_>>()?;
    let gap = clearance.metres() + thickness.metres() + FIT_MARGIN_M;
    let mut carrier = 0;
    Ok(mesh.refit_surfaces(|positions, _| {
        // The gauntlet constructor emits its cuff before the finger lames.
        let cuff_room = match style {
            PlateFit::Mitten { cuff_clearance, .. } if carrier == 0 => cuff_clearance.metres(),
            _ => 0.0,
        };
        fit_carrier(positions, &frame, region, &sections, gap + cuff_room, style);
        carrier += 1;
    })?)
}

fn fit_carrier(
    positions: &mut [[f32; 3]],
    frame: &PartFrame,
    region: FitRegion,
    sections: &[Section],
    gap: f32,
    style: PlateFit<'_>,
) {
    for point in positions {
        let mut p = local(frame, *point);
        let [low, high] = style.axial_span(frame.half_extents[1]);
        let station = ((p[1] / frame.half_extents[1] - low) / (high - low) * (STATIONS - 1) as f32)
            .clamp(0.0, (STATIONS - 1) as f32);
        let first = (station.floor() as usize).min(STATIONS - 2);
        let t = station - first as f32;
        let blend = t * t * (3.0 - 2.0 * t);
        let a = &sections[first];
        let b = &sections[first + 1];
        let center: [f32; 2] =
            std::array::from_fn(|i| a.center[i] + (b.center[i] - a.center[i]) * blend);
        // Keep the authored angular correspondence when the ankle section moves
        // toward the heel. Re-centering input rays can reverse adjacent columns.
        let radial = if matches!(style, PlateFit::Greave(_)) {
            [p[0], p[2]]
        } else {
            [p[0] - center[0], p[2] - center[1]]
        };
        let distance = radial[0].hypot(radial[1]);
        let direction = if distance > f32::EPSILON {
            radial.map(|v| v / distance)
        } else {
            [0.0, 1.0]
        };
        let radius = a.radius(direction) * (1.0 - blend) + b.radius(direction) * blend + gap;
        let rho = distance / radius;
        let v = ((p[1] / frame.half_extents[1] + 1.0) * 0.5).clamp(0.0, 1.0);
        let fitted_radius = match style {
            PlateFit::Mitten { .. } => rho.max(1.0),
            _ => 1.0 + style.allowance(v, direction, frame.half_extents[0]) / radius,
        };
        if rho > f32::EPSILON {
            if matches!(region, FitRegion::Hand(_)) {
                const FINGERTIP_END: f32 = 1.1;
                let strength = ((p[1] / frame.half_extents[1] + FINGERTIP_END)
                    / (FINGERTIP_END - 1.0))
                    .clamp(0.0, 1.0);
                let factor = 1.0 + (fitted_radius / rho - 1.0) * strength;
                p[0] = center[0] + radial[0] * factor;
                p[2] = center[1] + radial[1] * factor;
            } else {
                p[0] = center[0] + radial[0] * fitted_radius / rho;
                p[2] = center[1] + radial[1] * fitted_radius / rho;
            }
        }
        *point = frame.point(p);
    }
}

fn local(frame: &PartFrame, p: [f32; 3]) -> [f32; 3] {
    frame
        .axes
        .map(|axis| (0..3).map(|i| (p[i] - frame.origin[i]) * axis[i]).sum())
}

#[cfg(test)]
#[path = "plate_fit_tests.rs"]
mod tests;
