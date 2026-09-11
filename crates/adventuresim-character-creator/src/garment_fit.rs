//! Anatomical garment cages: shoulder saddles and joint-following limb sections.
//! Skin samples constrain authored surfaces; body triangles never become armor.
use adventuresim_armor_model::{
    GARMENT_ARMPIT_ROW as ARMPIT_ROW, GARMENT_PANEL_ACROSS as PANEL_ACROSS,
    GARMENT_PANEL_ALONG as PANEL_ALONG, GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER_DEPTH,
    GarmentArmorDesign, GarmentArmorKind as Kind, PartFrame, PartMesh, generate_garment_armor,
};
use anyhow::{Context, Result};

use crate::armor_frames::{FitRegion, Side, Wearer};

#[path = "garment_fit_sampling.rs"]
mod sampling;
use sampling::{SectionCage, SurfaceSampler, enclosing_section, section};
#[path = "garment_attachment.rs"]
mod attachment;
use attachment::AttachmentRing;
#[path = "garment_drape_fit.rs"]
mod drape;
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
) -> Result<PartMesh> {
    match design.kind {
        Kind::ArmingDoublet | Kind::Brigandine | Kind::JackOfPlates | Kind::MailShirt => {
            torso::fit(design, wearer)
        }
        Kind::MailSleeve | Kind::QuiltedSleeve | Kind::MailChausses | Kind::PaddedChausses => {
            limb::fit(design, placement, wearer)
        }
        Kind::Gorget => crate::gorget_fit::fit(design, wearer),
        _ => skirt(design, wearer, None),
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
) -> Result<PartMesh> {
    anyhow::ensure!(
        design.kind == Kind::Tassets,
        "suspension requires a tasset panel"
    );
    skirt(design, wearer, Some(top))
}

fn skirt(
    design: &GarmentArmorDesign,
    wearer: &Wearer<'_>,
    attachment_top: Option<f32>,
) -> Result<PartMesh> {
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
    let mesh = generate_garment_armor(design, &frame)?;
    if design.kind == Kind::Tassets {
        let support = wearer.support_indices(FitRegion::Hips)?;
        let sample = SurfaceSampler::new(wearer, &support, &frame);
        return fit_tassets(mesh, &frame, &sample);
    }
    wrap_skirt(mesh, design, wearer, &frame)
}

/// Move the authored plate above the anatomical depth datum while retaining
/// its flare, transverse crown and separate lame offsets.
fn fit_tassets(mesh: PartMesh, frame: &PartFrame, sample: &SurfaceSampler) -> Result<PartMesh> {
    let depth = sampling::PlateDepth::new(sample, frame.half_extents);
    Ok(mesh.refit_surfaces(|positions| {
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
    let attachment = if flexible {
        Some(skirt_underlayers(design, wearer, frame, &mut envelope)?)
    } else {
        None
    };
    let cage = drape::DrapeCage::new(&envelope, frame);
    let gap = design.clearance.metres() + design.wall_thickness.metres();
    Ok(mesh.refit_surfaces(|positions| {
        let top = positions
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let bottom = positions
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min);
        for point in positions.iter_mut() {
            let local = local_point(frame, *point);
            let angle = local[0].atan2(local[2]);
            let descent = ((frame.half_extents[1] - local[1]) / (2.0 * frame.half_extents[1]))
                .clamp(0.0, 1.0);
            let radius = cage.radius_at_angle(local[1], angle);
            let step = if flexible {
                0.0
            } else {
                design.wall_thickness.metres() * 2.5 * (top - point[1]) / (top - bottom)
            };
            let flare = design.flare.unit() * descent;
            let x = (radius * (1.0 + flare) + gap + step) * angle.sin();
            let z = (radius * (1.0 + flare) + gap + step) * angle.cos();
            let mut fitted = frame.point([x, local[1], z]);
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
    use adventuresim_armor_model::Permille;

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
