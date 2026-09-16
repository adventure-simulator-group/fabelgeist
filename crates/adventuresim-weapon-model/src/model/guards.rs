//! Connected hilt bars, rings, plates and terminal furniture.
use super::*;
use std::f64::consts::{PI, TAU};

pub(super) fn section(value: Option<GuardSection>) -> Section {
    match value {
        None | Some(GuardSection::Round) => Section::Round,
        Some(GuardSection::Oval) => Section::Oval,
        Some(GuardSection::Diamond) => Section::Diamond,
        Some(GuardSection::Flat) => Section::Flat,
        Some(GuardSection::Triangular) => Section::Triangular,
    }
}
fn part(solid: Solid, r: &ResolvedComponent, suffix: &str, material: Material) -> PartSource {
    PartSource::new(solid, material, &format!("{} {suffix}", r.label), &r.id)
}

mod bars;
mod ends;
mod plates;
mod profile;
pub(super) use bars::{figure_eight, knuckle, ring, tube};
use ends::*;
use plates::guard_plate;
fn arm(p: &GuardParameters, side: f64, detail: Detail) -> Vec<Point> {
    let width = p.width.get();
    let sweep = p.sweep.map_or(0.0, Metres::get);
    let independent = p.mirror_mode == Some(GuardMirrorMode::Independent);
    let symmetric = p.mirror_mode == Some(GuardMirrorMode::Symmetric);
    let count = detail.samples(11, 5);
    let length = if independent {
        if side < 0.0 {
            p.left_length
        } else {
            p.right_length
        }
    } else {
        None
    }
    .map_or(width / 2.0, Metres::get);
    let authored_sweep = if independent {
        if side < 0.0 {
            p.left_sweep
        } else {
            p.right_sweep
        }
    } else {
        None
    };
    let set = if independent {
        if side < 0.0 { p.left_set } else { p.right_set }
    } else {
        None
    }
    .map_or(0.0, Metres::get);
    let side_sweep =
        authored_sweep.map_or(if symmetric { sweep } else { sweep * side }, Metres::get);
    (0..=count)
        .map(|i| {
            let t = i as f64 / count as f64;
            [side * length * t, side_sweep * t * t, set * t]
        })
        .collect::<Vec<_>>()
}
pub(super) fn guard(
    r: &ResolvedComponent,
    p: &GuardParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let height = p.height.get();
    let thickness = p.thickness.get();
    let material = r.component.material.unwrap_or(Material::Steel);
    let independent = p.mirror_mode == Some(GuardMirrorMode::Independent);
    let mut centerline = arm(p, -1.0, detail);
    centerline.reverse();
    centerline.extend(arm(p, 1.0, detail).into_iter().skip(1));
    let specification = Sweep {
        section: section(p.section.clone()),
        width: p.section_width.map_or(height * 0.44, Metres::get),
        depth: p.section_depth.map_or(thickness, Metres::get),
        radial_segments: 12,
        twist: p.section_twist.map_or(0.0, Degrees::get),
        tip_scale: p.tip_scale.map_or(1.0, Ratio::get),
        terminal_swell: p.terminal_swell.map_or(0.0, Ratio::get),
        centered_taper: true,
        fit_bends: true,
        ring_scales: None,
    };
    let mut parts = vec![part(
        Solid::sweep(&centerline, &specification, detail)?,
        r,
        "quillons",
        material,
    )];
    for side in [-1.0, 1.0] {
        let selected = if independent {
            if side < 0.0 {
                p.left_terminal.as_ref().and_then(left_terminal)
            } else {
                p.right_terminal.as_ref().and_then(right_terminal)
            }
        } else {
            None
        }
        .or_else(|| p.terminal.clone())
        .unwrap_or(GuardTerminal::None);
        let points = arm(p, side, detail);
        let end = *points.last().unwrap();
        let tangent = sub(end, points[points.len() - 2]);
        let solid = if selected == GuardTerminal::Profile {
            Some(profile::terminal_profile(
                p.terminal_profile
                    .as_ref()
                    .ok_or("profile terminal needs stations")?,
                tangent,
                end,
                &parts[0].solid,
                detail,
            )?)
        } else {
            terminal(
                selected,
                p.terminal_size.map_or(height * 0.3, Metres::get),
                tangent,
                detail,
            )?
        };
        if let Some(solid) = solid {
            parts.push(part(
                solid.transform([0.0; 3], end),
                r,
                "terminal",
                material,
            ));
        }
    }
    let block_width = (height * 1.2).max(r.grip_width.unwrap_or(0.0));
    parts.push(part(
        Solid::rounded_plate(
            &[
                [-block_width / 2.0, -height * 0.28],
                [block_width / 2.0, -height * 0.28],
                [block_width * 0.58, height * 0.28],
                [-block_width * 0.58, height * 0.28],
            ],
            thickness,
            p.block_bevel.map_or(0.14, Ratio::get),
        )?,
        r,
        "block",
        material,
    ));
    Ok(parts)
}

pub(super) fn assembly(
    r: &ResolvedComponent,
    p: &GuardAssemblyParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let mut parts = Vec::new();
    for (index, member) in p.members.iter().enumerate() {
        let anchors: Vec<_> = member
            .path
            .iter()
            .map(|name| {
                p.nodes
                    .get(name)
                    .map(|point| point.map(Metres::get))
                    .ok_or_else(|| format!("missing member node {name}"))
            })
            .collect::<Result<_, _>>()?;
        if anchors.len() < 2 {
            return Err("member needs at least two named nodes".into());
        }
        let samples = detail.samples(5, 3);
        let mut points = Vec::new();
        for row in 0..anchors.len() - 1 {
            for i in 0..samples {
                let t = i as f64 / samples as f64;
                let [a, b, c, d] = [
                    anchors[row.saturating_sub(1)],
                    anchors[row],
                    anchors[row + 1],
                    anchors[(row + 2).min(anchors.len() - 1)],
                ];
                points.push(std::array::from_fn(|axis| {
                    0.5 * (2.0 * b[axis]
                        + (-a[axis] + c[axis]) * t
                        + (2.0 * a[axis] - 5.0 * b[axis] + 4.0 * c[axis] - d[axis]) * t * t
                        + (-a[axis] + 3.0 * b[axis] - 3.0 * c[axis] + d[axis]) * t * t * t)
                }));
            }
        }
        points.push(*anchors.last().unwrap());
        let sweep = Sweep {
            section: section(member.section.clone()),
            width: member.section_width.get(),
            depth: member.section_depth.get(),
            twist: member.section_twist.map_or(0.0, Degrees::get),
            tip_scale: member.tip_scale.map_or(1.0, Ratio::get),
            terminal_swell: member.terminal_swell.map_or(0.0, Ratio::get),
            radial_segments: member.radial_segments.map_or(12, |n| n.0 as usize),
            fit_bends: true,
            ..Sweep::default()
        };
        parts.push(part(
            Solid::sweep(&points, &sweep, detail)?,
            r,
            &member
                .label
                .clone()
                .unwrap_or_else(|| (index + 1).to_string()),
            member
                .material
                .or(r.component.material)
                .unwrap_or(Material::Steel),
        ));
    }
    for (index, plate) in p.plates.iter().flatten().enumerate() {
        parts.extend(guard_plate(r, p, plate, index, detail)?);
    }
    Ok(parts)
}
