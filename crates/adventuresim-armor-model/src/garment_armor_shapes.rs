use std::f32::consts::{PI, TAU};

use crate::{GenerateError, PartFrame, PartMesh};

use super::{GarmentArmorDesign, GarmentArmorKind, shell::Pattern};

use super::{
    GARMENT_ARMPIT_ROW as ARMPIT_ROW, GARMENT_AXIAL_SEGMENTS as ALONG,
    GARMENT_PANEL_ACROSS as PANEL_ACROSS, GARMENT_PANEL_ALONG as PANEL_ALONG,
    GARMENT_RING_SEGMENTS as AROUND, GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER_DEPTH_SEGMENTS,
};
const QUILT_RELIEF_METRES: f32 = 0.0015;

fn padded(kind: GarmentArmorKind) -> bool {
    matches!(
        kind,
        GarmentArmorKind::ArmingDoublet
            | GarmentArmorKind::PaddedChausses
            | GarmentArmorKind::PaddedSkirt
            | GarmentArmorKind::QuiltedSleeve
    )
}

fn quilt(kind: GarmentArmorKind, coordinate: f32) -> f32 {
    if padded(kind) {
        QUILT_RELIEF_METRES * (coordinate * PI * 12.0).cos().powi(2)
    } else {
        0.0
    }
}

/// Rectangular front/back cutting patterns, sewn at the flanks below the
/// armscyes and across the shoulders outside the neckline. The topology has
/// exactly four deliberate openings: neck, two armscyes, and lower hem.
pub(super) fn torso(
    design: &GarmentArmorDesign,
    fit: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    let [width, height, depth] = fit.half_extents;
    let padding = design.clearance.metres() + design.wall_thickness.metres();
    let mut pattern = Pattern::default();
    let stride = PANEL_ACROSS + 1;
    let panel_size = stride * (PANEL_ALONG + 1);
    for front in [true, false] {
        for row in 0..=PANEL_ALONG {
            let v = row as f32 / PANEL_ALONG as f32;
            for col in 0..=PANEL_ACROSS {
                let u = 2.0 * col as f32 / PANEL_ACROSS as f32 - 1.0;
                let neck_scoop = (1.0 - (u.abs() / 0.5).min(1.0).powi(2)).max(0.0);
                let top = height * (1.0 - if front { 0.36 } else { 0.10 } * neck_scoop);
                let bottom = -height * design.length.unit();
                let shoulder_inset = ((v - 0.58) / 0.42).max(0.0).powi(2);
                let waist = design.waist.unit() + (1.0 - design.waist.unit()) * (v * PI).sin();
                let x = u * (width * waist + padding) * (1.0 - 0.19 * shoulder_inset);
                let y = bottom + (top - bottom) * v;
                let shoulder_depth = 1.0 - 0.42 * shoulder_inset;
                let z = (depth + padding + quilt(design.kind, (u + 1.0) * 0.5))
                    * (1.0 - 0.72 * u * u).sqrt()
                    * shoulder_depth;
                pattern.vertex([x, y, if front { z } else { -z }]);
            }
        }
        let offset = if front { 0 } else { panel_size as u32 };
        for row in 0..PANEL_ALONG {
            for col in 0..PANEL_ACROSS {
                let a = offset + (row * stride + col) as u32;
                let b = a + 1;
                let d = a + stride as u32;
                if front {
                    pattern.quad(a, b, d + 1, d);
                } else {
                    pattern.quad(b, a, d, d + 1);
                }
            }
        }
    }
    shoulder_seams(&mut pattern);
    flank_seams(&mut pattern);
    pattern.lined(design.wall_thickness.metres())
}

fn shoulder_seams(pattern: &mut Pattern) {
    let stride = PANEL_ACROSS + 1;
    let panel_size = stride * (PANEL_ALONG + 1);
    // A shoulder is a curved saddle, not a planar bridge between chest/back.
    // Independent depth sampling lets anatomical fitting seat the whole band.
    for columns in [0..=PANEL_ACROSS / 4, 3 * PANEL_ACROSS / 4..=PANEL_ACROSS] {
        let mut rows = Vec::new();
        for depth_row in 0..=SHOULDER_DEPTH_SEGMENTS {
            let t = depth_row as f32 / SHOULDER_DEPTH_SEGMENTS as f32;
            let mut row = Vec::new();
            for col in columns.clone() {
                let front = (PANEL_ALONG * stride + col) as u32;
                let back = front + panel_size as u32;
                row.push(if depth_row == 0 {
                    front
                } else if depth_row == SHOULDER_DEPTH_SEGMENTS {
                    back
                } else {
                    let a = pattern.positions[front as usize];
                    let b = pattern.positions[back as usize];
                    pattern.vertex(std::array::from_fn(|axis| {
                        a[axis] * (1.0 - t) + b[axis] * t
                    }))
                });
            }
            rows.push(row);
        }
        for row in 0..SHOULDER_DEPTH_SEGMENTS {
            for col in 0..rows[row].len() - 1 {
                pattern.quad(
                    rows[row][col],
                    rows[row][col + 1],
                    rows[row + 1][col + 1],
                    rows[row + 1][col],
                );
            }
        }
    }
}

fn flank_seams(pattern: &mut Pattern) {
    let stride = PANEL_ACROSS + 1;
    let panel_size = stride * (PANEL_ALONG + 1);
    // Front and back are sewn through a sampled flank patch. Its upper edge
    // can dip beneath the armpit without distorting the chest or sleeve bulk.
    for col in [0, PANEL_ACROSS] {
        let mut rows = Vec::new();
        for row in 0..=ARMPIT_ROW {
            let front = (row * stride + col) as u32;
            let back = front + panel_size as u32;
            let mut across = vec![front];
            for depth in 1..SHOULDER_DEPTH_SEGMENTS {
                let t = depth as f32 / SHOULDER_DEPTH_SEGMENTS as f32;
                let a = pattern.positions[front as usize];
                let b = pattern.positions[back as usize];
                across.push(pattern.vertex(std::array::from_fn(|axis| {
                    a[axis] * (1.0 - t) + b[axis] * t
                })));
            }
            across.push(back);
            rows.push(across);
        }
        for row in 0..ARMPIT_ROW {
            for depth in 0..SHOULDER_DEPTH_SEGMENTS {
                let [a, b, c, d] = [
                    rows[row][depth],
                    rows[row + 1][depth],
                    rows[row + 1][depth + 1],
                    rows[row][depth + 1],
                ];
                if col == 0 {
                    pattern.quad(a, b, c, d);
                } else {
                    pattern.quad(d, c, b, a);
                }
            }
        }
    }
}

pub(super) fn tube(
    design: &GarmentArmorDesign,
    fit: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    let skirt = matches!(
        design.kind,
        GarmentArmorKind::MailSkirt | GarmentArmorKind::PaddedSkirt
    );
    let mut pattern = Pattern::default();
    let [width, height, depth] = fit.half_extents;
    for row in 0..=ALONG {
        let axial = row as f32 / ALONG as f32;
        let radius = if skirt {
            1.0 + design.flare.unit() * (1.0 - axial)
        } else {
            0.70 + 0.30 * axial
        };
        for col in 0..AROUND {
            let u = col as f32 / AROUND as f32;
            let angle = u * TAU;
            let padding =
                design.clearance.metres() + design.wall_thickness.metres() + quilt(design.kind, u);
            pattern.vertex([
                (width * radius + padding) * angle.sin(),
                height - 2.0 * height * design.length.unit() * (1.0 - axial),
                (depth * radius + padding) * angle.cos(),
            ]);
        }
    }
    connect_rings(&mut pattern, ALONG);
    pattern.lined(design.wall_thickness.metres())
}

fn connect_rings(pattern: &mut Pattern, rows: usize) {
    for row in 0..rows {
        for col in 0..AROUND {
            let a = (row * AROUND + col) as u32;
            let b = (row * AROUND + (col + 1) % AROUND) as u32;
            pattern.quad(a, b, b + AROUND as u32, a + AROUND as u32);
        }
    }
}
