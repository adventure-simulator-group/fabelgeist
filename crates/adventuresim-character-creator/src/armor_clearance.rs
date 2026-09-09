//! Smooth axial clearance envelopes for long plates, independent of body topology.
use crate::armor_frames::{FitRegion, Wearer};
use adventuresim_armor_model::{Millimeters, PartFrame, PartMesh};
use anyhow::{Result, ensure};

const STATIONS: usize = 13;
const STATION_HALF_WIDTH_M: f32 = 0.024;
const UPPER_ARM_STATION_HALF_WIDTH_M: f32 = 0.012;
const MINIMUM_RADIUS_M: f32 = 0.012;
const FIT_MARGIN_M: f32 = 0.004;
const MINIMUM_SECTION_SAMPLES: usize = 16;
/// Section spacing targets one wall gauge of air between successive plates.
pub(super) const LAME_SPACING_GAUGES: f32 = 2.0;

/// Preserve the physical ordering of independently thickened carrier surfaces.
#[derive(Clone, Copy)]
pub enum PlateLayering {
    Single,
    Mitten { finger_lames: u8 },
}

impl PlateLayering {
    fn clearance(self, carrier: usize, thickness: f32) -> Option<f32> {
        match self {
            Self::Single => Some(0.0),
            Self::Mitten { .. } if carrier == 0 => None,
            Self::Mitten { finger_lames } if carrier <= usize::from(finger_lames) => {
                Some(carrier as f32 * thickness * LAME_SPACING_GAUGES)
            }
            Self::Mitten { .. } => Some(0.0),
        }
    }
}

#[derive(Clone, Copy)]
struct Section {
    center: [f32; 2],
    radii: [f32; 2],
}

impl Section {
    fn measured(points: &[[f32; 3]], y: f32, half_width: f32) -> Self {
        let mut nearest = points.iter().collect::<Vec<_>>();
        nearest.sort_by(|a, b| (a[1] - y).abs().total_cmp(&(b[1] - y).abs()));
        let count = nearest
            .iter()
            .take_while(|p| (p[1] - y).abs() < half_width)
            .count()
            .max(MINIMUM_SECTION_SAMPLES)
            .min(nearest.len());
        let section = &nearest[..count];
        let lo: [f32; 2] = std::array::from_fn(|i| {
            section
                .iter()
                .map(|p| p[i * 2])
                .fold(f32::INFINITY, f32::min)
        });
        let hi: [f32; 2] = std::array::from_fn(|i| {
            section
                .iter()
                .map(|p| p[i * 2])
                .fold(f32::NEG_INFINITY, f32::max)
        });
        let center = std::array::from_fn(|i| (lo[i] + hi[i]) * 0.5);
        let mut radii = std::array::from_fn(|i| ((hi[i] - lo[i]) * 0.5).max(MINIMUM_RADIUS_M));
        let enclosure = section
            .iter()
            .map(|p| {
                (((p[0] - center[0]) / radii[0]).powi(2) + ((p[2] - center[1]) / radii[1]).powi(2))
                    .sqrt()
            })
            .fold(1.0, f32::max);
        for radius in &mut radii {
            *radius *= enclosure;
        }
        Self { center, radii }
    }
}

pub fn fit(
    mesh: &PartMesh,
    wearer: &Wearer<'_>,
    region: FitRegion,
    clearance: Millimeters,
    thickness: Millimeters,
    layering: PlateLayering,
) -> Result<PartMesh> {
    let frame = wearer.frame(region)?;
    let points = wearer
        .support_indices(region)?
        .into_iter()
        .map(|i| local(&frame, wearer.positions[i]))
        .collect::<Vec<_>>();
    ensure!(
        !points.is_empty(),
        "plate has no anatomical section samples"
    );
    let half_width = if matches!(region, FitRegion::UpperArm(_)) {
        UPPER_ARM_STATION_HALF_WIDTH_M
    } else {
        STATION_HALF_WIDTH_M
    };
    let sections: Vec<_> = (0..STATIONS)
        .map(|i| {
            Section::measured(
                &points,
                frame.half_extents[1] * (2.0 * i as f32 / (STATIONS - 1) as f32 - 1.0),
                half_width,
            )
        })
        .collect();
    let gap = clearance.metres() + thickness.metres() + FIT_MARGIN_M;
    let mut carrier_index = 0;
    Ok(mesh.refit_surfaces(|positions| {
        let layer = layering.clearance(carrier_index, thickness.metres());
        carrier_index += 1;
        let Some(layer) = layer else {
            // The cuff is one carrier, so its wrist rim cannot split between
            // fitted and unfitted vertices through floating-point comparisons.
            return;
        };
        fit_carrier(positions, &frame, region, &sections, gap, layer);
    })?)
}

fn fit_carrier(
    positions: &mut [[f32; 3]],
    frame: &PartFrame,
    region: FitRegion,
    sections: &[Section],
    gap: f32,
    layer: f32,
) {
    for point in positions {
        let mut p = local(frame, *point);
        let station = ((p[1] / frame.half_extents[1] + 1.0) * 0.5 * (STATIONS - 1) as f32)
            .clamp(0.0, (STATIONS - 1) as f32);
        let first = (station.floor() as usize).min(STATIONS - 2);
        let t = station - first as f32;
        let blend = t * t * (3.0 - 2.0 * t);
        let a = sections[first];
        let b = sections[first + 1];
        let center: [f32; 2] =
            std::array::from_fn(|i| a.center[i] + (b.center[i] - a.center[i]) * blend);
        let radii: [f32; 2] =
            std::array::from_fn(|i| a.radii[i] + (b.radii[i] - a.radii[i]) * blend + gap + layer);
        let radial = [p[0] - center[0], p[2] - center[1]];
        let rho = ((radial[0] / radii[0]).powi(2) + (radial[1] / radii[1]).powi(2)).sqrt();
        let upper_plate = matches!(region, FitRegion::Thigh(_) | FitRegion::UpperArm(_));
        let fitted_radius = if upper_plate {
            rho.clamp(1.0, 1.06)
        } else {
            rho.max(1.0)
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
