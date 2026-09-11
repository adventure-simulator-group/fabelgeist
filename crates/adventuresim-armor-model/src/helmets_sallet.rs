//! Swept sallet tails and a separate visor with narrow rising pivot arms.
use super::{
    SalletDesign, VisoredSalletDesign,
    geometry::{AROUND, Surface},
};
use crate::{ArmorComponentRole, ArmorHinge, GenerateError, PartMesh};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, TAU};

pub(super) fn skull(
    mut radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &SalletDesign,
) -> Result<PartMesh, GenerateError> {
    const SKIRT_ROWS: usize = 12;
    const OPENING_START: usize = AROUND / 6;
    radii[2] += d.brow_projection.metres();
    let mut surface = Surface::default();
    let rim = surface.styled_dome(radii, brow, &d.crown, 0.0);
    surface.reshape_front_arc(radii, FRAC_PI_3, d.opening_width.radians());
    let mut previous = rim[OPENING_START..=AROUND - OPENING_START].to_vec();
    for row in 1..=SKIRT_ROWS {
        let t = row as f32 / SKIRT_ROWS as f32;
        let next = (0..previous.len())
            .map(|i| {
                let opening = d.opening_width.radians()
                    + (FRAC_PI_2 - d.opening_width.radians()) * d.opening_sweep.unit() * t.powi(2);
                let angle =
                    opening + (TAU - 2.0 * opening) * i as f32 / (previous.len() - 1) as f32;
                let rear = (-angle.cos()).max(0.0);
                let extension = d.tail_length.metres() * t.powi(3);
                let tail = rear.powf(2.0 / d.tail_width.unit());
                surface.vertex([
                    (radii[0] + extension * tail * 0.12)
                        * angle.sin()
                        * (1.0 - 0.35 * tail * t * t),
                    brow - half_height * d.cheek_depth.unit() * t - d.tail_drop.metres() * tail * t
                        + d.rear_edge_lift.metres()
                            * ((1.0 - angle.cos()) * 0.5).powi(2)
                            * t.powi(2),
                    (radii[2] + extension * tail) * angle.cos(),
                ])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &next, false);
        previous = next;
    }
    surface.shell(d.fit.wall_thickness.metres(), crate::ShellExtrusion::Normal)
}

pub(super) fn visored(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    d: &VisoredSalletDesign,
) -> Result<PartMesh, GenerateError> {
    const COLUMNS: usize = 64;
    const ROWS: usize = 10;
    const ARM_WIDTH_M: f32 = 0.012;
    let mut mesh =
        skull(radii, brow, half_height, &d.skull)?.with_component(ArmorComponentRole::Skull, None);
    let gauge = d.skull.fit.wall_thickness.metres();
    let skull_relief = d
        .skull
        .crown
        .fluting
        .map_or(0.0, |pattern| pattern.depth.metres());
    let visor_spacing = gauge * 2.0 + skull_relief;
    let top = brow - d.sight_gap.metres();
    let rise = d.pivot_rise.metres();
    let mut surface = Surface::default();
    let mut previous = Vec::new();
    for row in 0..=ROWS {
        let v = row as f32 / ROWS as f32;
        let ring = (0..=COLUMNS)
            .map(|column| {
                let angle = (2.0 * column as f32 / COLUMNS as f32 - 1.0) * FRAC_PI_2;
                let arm = (angle.abs() / FRAC_PI_2).powi(5);
                let upper = top + rise * arm;
                let tip = ((angle.abs() / FRAC_PI_2 - 0.88) / 0.12).max(0.0);
                let panel = d.visor_height.metres() * (1.0 - (1.0 - d.side_panel.unit()) * arm)
                    + rise * arm;
                let height = panel + (ARM_WIDTH_M - panel) * tip * tip;
                let projection =
                    d.visor_projection.metres() * (1.0 - v).powi(2) * angle.cos().powi(2);
                let y = upper - height * v;
                let latitude = ((y - brow) / (radii[1] - brow)).clamp(0.0, 0.99);
                let bowl = (1.0 - latitude * latitude).powf(d.skull.crown.fullness.unit() * 0.5);
                surface.vertex([
                    (radii[0] * bowl + visor_spacing) * angle.sin(),
                    y,
                    ((radii[2] + d.skull.brow_projection.metres()) * bowl + visor_spacing)
                        * angle.cos()
                        + projection,
                ])
            })
            .collect::<Vec<_>>();
        if row > 0 {
            surface.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    let hinge = ArmorHinge {
        origin: [0.0, top + rise - ARM_WIDTH_M * 0.5, 0.0],
        axis: [1.0, 0.0, 0.0],
    };
    mesh.append(
        PartMesh::from_relief_surface(
            surface.positions,
            surface.indices,
            gauge,
            crate::BoundaryNormals::Smooth,
            crate::ShellExtrusion::Radial {
                origin: [0.0; 3],
                axis: [0.0, 1.0, 0.0],
            },
            None,
        )?
        .with_component(ArmorComponentRole::Visor, Some(hinge)),
    );
    Ok(mesh)
}
