//! Anatomical garment cages: shoulder saddles and joint-following limb sections.
//! Skin samples constrain authored surfaces; body triangles never become armor.
use adventuresim_armor_model::{
    GARMENT_ARMPIT_ROW as ARMPIT_ROW, GARMENT_AXIAL_SEGMENTS, GARMENT_LAME_SPACING_GAUGES,
    GARMENT_PANEL_ACROSS as PANEL_ACROSS, GARMENT_PANEL_ALONG as PANEL_ALONG,
    GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER_DEPTH, GarmentArmorDesign,
    GarmentArmorKind as Kind, PartFrame, PartMesh, generate_garment_armor,
};
use anyhow::{Context, Result};

use crate::armor_frames::{FitRegion, Side, Wearer};
use crate::armor_layer::ArmorLayerSurface;

#[path = "garment_fit_sampling.rs"]
mod sampling;
use sampling::{SectionCage, SurfaceSampler, enclosing_section, section};
#[path = "garment_attachment.rs"]
mod attachment;
use attachment::AttachmentRing;
#[path = "garment_drape_fit.rs"]
pub(crate) mod drape;
#[path = "garment_limb_fit.rs"]
mod limb;
#[path = "garment_torso_fit.rs"]
mod torso;

const SHOULDER_LIFT_M: f32 = 0.045;
const GARMENT_FIT_MARGIN_M: f32 = 0.006;
const ARMPIT_SEAM_DROP_M: f32 = 0.023;
const SHOULDER_SEAM_EASE_M: f32 = 0.005;

pub fn fitted_garment(
    design: &GarmentArmorDesign,
    placement: &str,
    wearer: &Wearer<'_>,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    match design.kind {
        Kind::ArmingDoublet | Kind::Brigandine | Kind::JackOfPlates | Kind::MailShirt => {
            torso::fit(design, wearer)
        }
        Kind::MailSleeve | Kind::QuiltedSleeve | Kind::MailChausses | Kind::PaddedChausses => {
            limb::fit(design, placement, wearer)
        }
        Kind::Gorget => crate::gorget_fit::fit(design, wearer),
        _ => skirt(design, wearer, None, layers),
    }
}

fn joint(wearer: &Wearer<'_>, name: &str) -> Result<[f32; 3]> {
    let index = wearer
        .joint_names
        .iter()
        .position(|n| n == name)
        .with_context(|| format!("missing garment landmark {name}"))?;
    Ok(std::array::from_fn(|axis| wearer.joints[index][axis]))
}

fn upright(
    wearer: &Wearer<'_>,
    top: f32,
    bottom: f32,
    width: f32,
    depth: f32,
    center: [f32; 3],
) -> Result<PartFrame> {
    let head = wearer.frame(FitRegion::Head)?;
    Ok(PartFrame {
        origin: [center[0], (top + bottom) * 0.5, center[2]],
        axes: [head.axes[0], [0.0, 1.0, 0.0], head.axes[2]],
        half_extents: [width, (top - bottom) * 0.5, depth],
    })
}

pub fn suspended_tassets(
    design: &GarmentArmorDesign,
    wearer: &Wearer<'_>,
    top: f32,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    anyhow::ensure!(
        design.kind == Kind::Tassets,
        "suspension requires a tasset panel"
    );
    skirt(design, wearer, Some(top), layers)
}

fn skirt(
    design: &GarmentArmorDesign,
    wearer: &Wearer<'_>,
    attachment_top: Option<f32>,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    if let adventuresim_armor_model::GarmentPlateShape::WrappedTassets(shape) = design.plate_shape {
        let top = attachment_top.unwrap_or(joint(wearer, "root")?[1] + 0.015);
        return crate::wrapped_tasset_fit::fit(design, &shape, wearer, top, layers);
    }
    let mut frame = wearer.frame(FitRegion::Hips)?;
    if design.kind == Kind::Fauld {
        let adventuresim_armor_model::GarmentPlateShape::Fauld { waist_rise, .. } =
            design.plate_shape
        else {
            anyhow::bail!("fauld shape required")
        };
        let top = joint(wearer, "c_spine1")?[1] + waist_rise.metres();
        let bottom = joint(wearer, "root")?[1] - 0.085;
        let mut support = wearer.support_indices(FitRegion::Hips)?;
        support.extend(wearer.support_indices(FitRegion::Torso)?);
        support.sort_unstable();
        support.dedup();
        frame = upright(
            wearer,
            top,
            bottom,
            frame.half_extents[0],
            frame.half_extents[2],
            frame.origin,
        )?;
        let (lo, hi) = section(
            wearer,
            &support,
            frame.point([0.0, frame.half_extents[1], 0.0]),
            frame.axes,
        );
        frame.half_extents[0] = (hi[0] - lo[0]) * 0.5;
        frame.half_extents[2] = (hi[2] - lo[2]) * 0.5;
    } else {
        let hip = joint(wearer, "root")?;
        let knee = joint(wearer, "l_lowleg")?;
        let top = if design.kind == Kind::Tassets {
            hip[1] + 0.015
        } else {
            joint(wearer, "c_spine1")?[1]
                + if design.kind == Kind::PaddedSkirt {
                    design.wall_thickness.metres() * 3.0
                } else {
                    0.0
                }
        };
        let bottom = hip[1] - (hip[1] - knee[1]) * 0.48;
        frame = upright(
            wearer,
            top,
            bottom,
            frame.half_extents[0],
            frame.half_extents[2],
            frame.origin,
        )?;
    }
    if let Some(top) = attachment_top {
        frame.origin[1] += top - (frame.origin[1] + frame.half_extents[1]);
    }
    frame
        .validate()
        .with_context(|| format!("invalid waist frame {frame:?}"))?;
    let mut carrier_design = design.clone();
    if let adventuresim_armor_model::GarmentPlateShape::Fauld { chevron_slope, .. } =
        &mut carrier_design.plate_shape
    {
        // Apply the rise against the fitted width, not the initial envelope.
        *chevron_slope = adventuresim_armor_model::Permille(0);
    }
    let mesh = generate_garment_armor(&carrier_design, &frame)
        .context("generating waist plate carrier")?;
    if design.kind == Kind::Tassets {
        let support = wearer.support_indices(FitRegion::Hips)?;
        let sample = SurfaceSampler::new(wearer, &support, &frame);
        return fit_tassets(mesh, &frame, &sample);
    }
    wrap_skirt(mesh, design, wearer, &frame, layers).context("fitting waist plate carrier")
}

/// Move the authored plate above the anatomical depth datum while retaining
/// its flare, transverse crown and separate lame offsets.
fn fit_tassets(mesh: PartMesh, frame: &PartFrame, sample: &SurfaceSampler) -> Result<PartMesh> {
    let depth = sampling::PlateDepth::new(sample, frame.half_extents);
    Ok(mesh.refit_surfaces(|positions, _| {
        for point in positions {
            let mut local = local_point(frame, *point);
            let authored_offset = local[2] - frame.half_extents[2];
            local[2] = depth.at(local) + authored_offset;
            *point = frame.point(local);
        }
    })?)
}

fn wrap_skirt(
    mesh: PartMesh,
    design: &GarmentArmorDesign,
    wearer: &Wearer<'_>,
    frame: &PartFrame,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let mut support = wearer.support_indices(FitRegion::Hips)?;
    support.extend(wearer.support_indices(FitRegion::Torso)?);
    support.sort_unstable();
    support.dedup();
    let mut envelope = support
        .iter()
        .map(|index| wearer.positions[*index])
        .collect::<Vec<_>>();
    let flexible = matches!(design.kind, Kind::MailSkirt | Kind::PaddedSkirt);
    for layer in layers {
        envelope.extend_from_slice(layer.positions);
    }
    let attachment = if flexible {
        Some(skirt_underlayers(design, wearer, frame, &mut envelope)?)
    } else {
        None
    };
    let chevron = match design.plate_shape {
        adventuresim_armor_model::GarmentPlateShape::Fauld { chevron_slope, .. } => {
            chevron_slope.unit()
        }
        _ => 0.0,
    };
    let relief = layers
        .iter()
        .map(|layer| layer.relief.metres())
        .fold(0.0, f32::max);
    let gap = design.clearance.metres() + design.wall_thickness.metres() + relief;
    let cage = drape::DrapeCage::for_skirt(&envelope, frame, &mesh, design, gap, chevron);
    let charts = if design.kind == Kind::Fauld {
        (0..usize::from(design.lame_count))
            .map(|lame| {
                adventuresim_armor_model::FauldLameChart::new(lame, usize::from(design.lame_count))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let mut course = 0;
    Ok(mesh.refit_surfaces(|positions, _| {
        // The authored chart's row retains the lap position even where the
        // groin arch raises the hem. Each course seats on the same enclosure
        // and lifts only toward its lower edge, without accumulating radial
        // clearance from every course beneath the waist attachment.
        let rows = charts.get(course).map(|chart| chart.rows.as_slice());
        course += 1;
        let columns = positions.len() / rows.map_or(GARMENT_AXIAL_SEGMENTS + 1, <[f32]>::len);
        for (index, point) in positions.iter_mut().enumerate() {
            let row = index / columns;
            let along = rows.map_or(row as f32 / GARMENT_AXIAL_SEGMENTS as f32, |rows| rows[row]);
            let step = if flexible || design.lame_count <= 1 {
                0.0
            } else {
                design.wall_thickness.metres() * GARMENT_LAME_SPACING_GAUGES * (1.0 - along)
            };
            let local = local_point(frame, *point);
            let angle = charts.get(course - 1).map_or_else(
                || local[0].atan2(local[2]),
                |chart| fauld_chart_angle(local, frame, design, chart, along, step),
            );
            let span = if design.kind == Kind::Fauld {
                design.length.unit()
            } else {
                1.0
            };
            let descent = ((frame.half_extents[1] - local[1])
                / (2.0 * frame.half_extents[1] * span))
                .clamp(0.0, 1.0);
            let flare = design.flare.unit() * descent;
            let mut radius =
                cage.radius_at_slope(local[1], angle, chevron, 1.0 + flare, gap + step);
            if design.kind == Kind::Fauld {
                let top = cage.radius_at_slope(frame.half_extents[1], angle, chevron, 1.0, gap);
                radius = radius.max((top - gap) * (1.0 + flare) + gap + step);
            }
            let x = radius * angle.sin();
            let z = radius * angle.cos();
            let mut fitted = frame.point([x, local[1] + chevron * x.abs(), z]);
            if let Some(attachment) = &attachment {
                let transition = (descent / 0.45).clamp(0.0, 1.0);
                let blend = 1.0 - transition * transition * (3.0 - 2.0 * transition);
                let mut seam = attachment.at(angle);
                seam[1] = fitted[1];
                let inset = if design.kind == Kind::PaddedSkirt {
                    design.wall_thickness.metres() * 1.5
                } else {
                    0.0
                };
                seam[0] -= frame.axes[0][0] * angle.sin() * inset;
                seam[2] -= frame.axes[2][2] * angle.cos() * inset;
                fitted = lerp(fitted, seam, blend);
            }
            *point = fitted;
        }
    })?)
}

/// An ellipse's physical polar angle changes with lap padding. Recover the
/// authored angular coordinate so overlapping flute columns stay registered.
fn fauld_chart_angle(
    local: [f32; 3],
    frame: &PartFrame,
    design: &GarmentArmorDesign,
    chart: &adventuresim_armor_model::FauldLameChart,
    along: f32,
    step: f32,
) -> f32 {
    let axial = chart.span[0] + (chart.span[1] - chart.span[0]) * along;
    let radius = 1.0 + design.flare.unit() * (1.0 - axial);
    let padding = design.clearance.metres() + design.wall_thickness.metres() + step;
    (local[0] / (frame.half_extents[0] * radius + padding))
        .atan2(local[2] / (frame.half_extents[2] * radius + padding))
}

/// Shared sewn waist boundary and the leg garments enclosed beneath it.
fn skirt_underlayers(
    design: &GarmentArmorDesign,
    wearer: &Wearer<'_>,
    frame: &PartFrame,
    envelope: &mut Vec<[f32; 3]>,
) -> Result<AttachmentRing> {
    let leg_kind = if design.kind == Kind::PaddedSkirt {
        Kind::PaddedChausses
    } else {
        Kind::MailChausses
    };
    let mut leg_design = GarmentArmorDesign::new(leg_kind);
    leg_design.clearance = design.clearance;
    leg_design.wall_thickness = design.wall_thickness;
    for placement in ["left", "right"] {
        envelope.extend(limb::fit(&leg_design, placement, wearer)?.positions);
    }
    let mut top_design = GarmentArmorDesign::new(if design.kind == Kind::PaddedSkirt {
        Kind::ArmingDoublet
    } else {
        Kind::MailShirt
    });
    top_design.clearance = design.clearance;
    top_design.wall_thickness = design.wall_thickness;
    Ok(AttachmentRing::hem(
        &torso::fit(&top_design, wearer)?,
        frame,
    ))
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    a.map(|v| v * s)
}
fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn normalized(a: [f32; 3]) -> [f32; 3] {
    scale(a, 1.0 / length(a).max(f32::EPSILON))
}
fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] * (1.0 - t) + b[i] * t)
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn local_point(frame: &PartFrame, point: [f32; 3]) -> [f32; 3] {
    frame
        .axes
        .map(|axis| dot(axis, subtract(point, frame.origin)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fauld_flare_widens_from_attachment_over_a_narrower_midriff() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.18, 0.15, 0.18],
        };
        let points = (0..=60)
            .flat_map(|row| {
                let y = -0.3 + row as f32 * 0.01;
                let radius = 0.12 + 0.06 * (y / 0.15).clamp(0.0, 1.0).powi(4);
                (0..48).map(move |column| {
                    let angle = std::f32::consts::TAU * column as f32 / 48.0;
                    [radius * angle.sin(), y, radius * angle.cos()]
                })
            })
            .collect::<Vec<_>>();
        let names = ["root", "c_spine0", "c_spine1", "c_neck"].map(str::to_owned);
        let joints = vec![[0; 8]; points.len()];
        let weights = vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; points.len()];
        let wearer = Wearer {
            positions: &points,
            normals: &[],
            faces: &[],
            joint_indices: &joints,
            joint_weights: &weights,
            joint_names: &names,
            joints: &[[0.0; 8]; 4],
        };
        let mut attachments = Vec::new();
        let mut hems = Vec::new();
        for (flare, length) in [(0, 750), (0, 1000), (500, 750), (500, 1000)] {
            let mut design = GarmentArmorDesign::new(Kind::Fauld);
            design.flare = Permille(flare);
            design.length = Permille(length);
            if let adventuresim_armor_model::GarmentPlateShape::Fauld { front_arch, .. } =
                &mut design.plate_shape
            {
                *front_arch = Permille(0);
            }
            let generated = generate_garment_armor(&design, &frame).unwrap();
            let fitted = wrap_skirt(generated, &design, &wearer, &frame, &[]).unwrap();
            let mut carrier = Vec::new();
            fitted
                .refit_surfaces(|positions, _| carrier.extend_from_slice(positions))
                .unwrap();
            let top = carrier
                .iter()
                .filter(|p| (p[1] - 0.15).abs() < 1e-6)
                .map(|p| p[0].hypot(p[2]))
                .fold(f32::INFINITY, f32::min);
            let mut lowest = f32::INFINITY;
            for point in &carrier {
                let radius = point[0].hypot(point[2]);
                assert!(radius >= top - 1e-5, "fauld tightened around the midriff");
                if point[1] < 0.15 - 0.3 * design.length.unit() + 1e-5 {
                    lowest = lowest.min(radius);
                }
            }
            assert!(lowest.is_finite(), "fixture must include the lower hem");
            if flare > 0 {
                assert!(lowest > top + 0.03, "authored flare must widen the hem");
            }
            attachments.push(top);
            hems.push(lowest);
        }
        assert!(
            attachments
                .iter()
                .all(|top| (top - attachments[0]).abs() < 1e-6),
            "flare must preserve top attachment"
        );
        assert!(
            (hems[2] - hems[3]).abs() < 0.001,
            "shortening the fauld must retain its authored hem flare"
        );
    }

    #[test]
    fn elliptical_fauld_courses_recover_the_same_authored_angle() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.22, 0.15, 0.10],
        };
        for count in [2, 6, 8] {
            let mut design = GarmentArmorDesign::new(Kind::Fauld);
            design.lame_count = count;
            design.flare = Permille(450);
            let mesh = generate_garment_armor(&design, &frame).unwrap();
            let mut course = 0;
            mesh.refit_surfaces(|positions, _| {
                let chart =
                    adventuresim_armor_model::FauldLameChart::new(course, usize::from(count))
                        .unwrap();
                course += 1;
                let columns = positions.len() / chart.rows.len();
                for (index, point) in positions.iter().enumerate() {
                    let along = chart.rows[index / columns];
                    let step = design.wall_thickness.metres()
                        * GARMENT_LAME_SPACING_GAUGES
                        * (1.0 - along);
                    let actual = fauld_chart_angle(*point, &frame, &design, &chart, along, step);
                    let expected = (index % columns) as f32 / columns as f32
                        * std::f32::consts::TAU
                        - std::f32::consts::PI;
                    assert!((actual.sin() - expected.sin()).abs() < 1e-6);
                    assert!((actual.cos() - expected.cos()).abs() < 1e-6);
                }
            })
            .unwrap();
        }
    }
    use adventuresim_armor_model::Permille;

    #[test]
    fn fauld_attachment_does_not_accumulate_clearance_from_lower_courses() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.15, 0.15, 0.15],
        };
        let points = (0..=60)
            .flat_map(|row| {
                (0..48).map(move |column| {
                    let angle = std::f32::consts::TAU * column as f32 / 48.0;
                    [
                        0.15 * angle.sin(),
                        -0.3 + row as f32 * 0.01,
                        0.15 * angle.cos(),
                    ]
                })
            })
            .collect::<Vec<_>>();
        let joint_names = ["root", "c_spine0", "c_spine1", "c_neck"].map(str::to_owned);
        let indices = vec![[0; 8]; points.len()];
        let weights = vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; points.len()];
        let wearer = Wearer {
            positions: &points,
            normals: &[],
            faces: &[],
            joint_indices: &indices,
            joint_weights: &weights,
            joint_names: &joint_names,
            joints: &[[0.0; 8]; 4],
        };
        let mut widths = Vec::new();
        for count in [1, 2, 6, 8] {
            let mut design = GarmentArmorDesign::new(Kind::Fauld);
            design.lame_count = count;
            design.fluting = Some(adventuresim_armor_model::PlateFluting::default());
            let mesh = generate_garment_armor(&design, &frame).unwrap();
            let mesh = wrap_skirt(mesh, &design, &wearer, &frame, &[]).unwrap();
            mesh.normals().unwrap();
            widths.push(
                mesh.positions
                    .iter()
                    .filter(|point| (point[1] - frame.half_extents[1]).abs() < 1e-6)
                    .map(|point| point[0].hypot(point[2]))
                    .fold(0.0_f32, f32::max),
            );
        }
        for radius in &widths[1..] {
            assert!(
                (radius - widths[0]).abs() < 1e-6,
                "additional courses must not inflate the waist attachment"
            );
        }
    }

    #[test]
    fn anatomical_tasset_fit_preserves_lower_flare_and_upper_attachment() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.18, 0.12, 0.10],
        };
        let points = vec![[0.0, 0.0, 0.08]];
        let normals = vec![[0.0, 0.0, 1.0]];
        let wearer = Wearer {
            faces: &[],
            positions: &points,
            normals: &normals,
            joints: &[],
            joint_indices: &[],
            joint_weights: &[],
            joint_names: &[],
        };
        let sampler = SurfaceSampler::new(&wearer, &[0], &frame);
        let fitted = |flare| {
            let mut design = GarmentArmorDesign::new(Kind::Tassets);
            design.flare = Permille(flare);
            let mesh = generate_garment_armor(&design, &frame).unwrap();
            fit_tassets(mesh, &frame, &sampler).unwrap()
        };
        let straight = fitted(0);
        let flared = fitted(500);
        assert_eq!(straight.indices, flared.indices);
        flared.normals().unwrap();
        let lower_gain = straight
            .positions
            .iter()
            .zip(&flared.positions)
            .filter(|(p, _)| p[1] < -0.10)
            .map(|(a, b)| b[2] - a[2])
            .fold(f32::INFINITY, f32::min);
        assert!(lower_gain > 0.04, "fit must preserve visible lower flare");
        let top_shift = straight
            .positions
            .iter()
            .zip(&flared.positions)
            .filter(|(p, _)| p[1] > 0.119)
            .map(|(a, b)| (b[2] - a[2]).abs())
            .fold(0.0, f32::max);
        assert!(
            top_shift < 0.002,
            "flare must leave upper attachment seated"
        );
    }
}
