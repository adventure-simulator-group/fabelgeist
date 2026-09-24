//! Torso garments on the device: the panels' topology, placed by the
//! fitter.
//!
//! The front and back panels are sewn by shoulder saddles and flank patches;
//! which vertices exist and how they join depends on the design alone. Where
//! they go is the fit's business, so the fitter supplies the WGSL.

use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

use super::coord::CoordKernel;
use super::garment::{constants, coord_shell};
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::garment_armor::{
    GARMENT_ARMPIT_ROW as ARMPIT_ROW, GARMENT_PANEL_ACROSS as PANEL_ACROSS,
    GARMENT_PANEL_ALONG as PANEL_ALONG, GARMENT_SHOULDER_DEPTH_SEGMENTS as SHOULDER_DEPTH,
};
use crate::{GarmentArmorDesign, GenerateError};

/// What a torso vertex is, for the fitter's `torso_vertex`: a panel vertex
/// `[0, front, row, column]`; a shoulder seam vertex `[1, band, depth row,
/// band column]`; a flank seam vertex `[2, side, row, depth]`. Seam vertices
/// are placed in a second pass, from the panels either side of them.
fn torso_topology() -> (Vec<[f32; 4]>, Vec<u32>) {
    let stride = PANEL_ACROSS + 1;
    let panel_size = stride * (PANEL_ALONG + 1);
    let mut coords = Vec::new();
    let mut indices = Vec::new();
    let quad = |indices: &mut Vec<u32>, a: u32, b: u32, c: u32, d: u32| {
        indices.extend([a, b, c, a, c, d]);
    };
    for front in [true, false] {
        for row in 0..=PANEL_ALONG {
            for col in 0..=PANEL_ACROSS {
                coords.push([0.0, if front { 1.0 } else { 0.0 }, row as f32, col as f32]);
            }
        }
        let offset = if front { 0 } else { panel_size as u32 };
        for row in 0..PANEL_ALONG {
            for col in 0..PANEL_ACROSS {
                let a = offset + (row * stride + col) as u32;
                let b = a + 1;
                let d = a + stride as u32;
                if front {
                    quad(&mut indices, a, b, d + 1, d);
                } else {
                    quad(&mut indices, b, a, d, d + 1);
                }
            }
        }
    }
    // Shoulder saddles, sewn between the panels' top rows.
    let band_width = PANEL_ACROSS / 4 + 1;
    for (band, columns) in [0..=PANEL_ACROSS / 4, 3 * PANEL_ACROSS / 4..=PANEL_ACROSS]
        .into_iter()
        .enumerate()
    {
        let mut rows = Vec::new();
        for depth_row in 0..=SHOULDER_DEPTH {
            let mut row = Vec::new();
            for (band_col, col) in columns.clone().enumerate() {
                let front = (PANEL_ALONG * stride + col) as u32;
                row.push(if depth_row == 0 {
                    front
                } else if depth_row == SHOULDER_DEPTH {
                    front + panel_size as u32
                } else {
                    coords.push([1.0, band as f32, depth_row as f32, band_col as f32]);
                    coords.len() as u32 - 1
                });
            }
            rows.push(row);
        }
        for row in 0..SHOULDER_DEPTH {
            for col in 0..band_width - 1 {
                quad(
                    &mut indices,
                    rows[row][col],
                    rows[row][col + 1],
                    rows[row + 1][col + 1],
                    rows[row + 1][col],
                );
            }
        }
    }
    // Flank patches, sewn between the panels' side columns below the armpit.
    for (side, col) in [0, PANEL_ACROSS].into_iter().enumerate() {
        let mut rows = Vec::new();
        for row in 0..=ARMPIT_ROW {
            let front = (row * stride + col) as u32;
            let mut across = vec![front];
            for depth in 1..SHOULDER_DEPTH {
                coords.push([2.0, side as f32, row as f32, depth as f32]);
                across.push(coords.len() as u32 - 1);
            }
            across.push(front + panel_size as u32);
            rows.push(across);
        }
        for row in 0..ARMPIT_ROW {
            for depth in 0..SHOULDER_DEPTH {
                let [a, b, c, d] = [
                    rows[row][depth],
                    rows[row + 1][depth],
                    rows[row + 1][depth + 1],
                    rows[row][depth + 1],
                ];
                if col == 0 {
                    quad(&mut indices, a, b, c, d);
                } else {
                    quad(&mut indices, d, c, b, a);
                }
            }
        }
    }
    (coords, indices)
}

/// Seam vertices follow the fitted panels they join.
const TORSO: &str = r#"
fn shell_pass(index: u32, coord: vec4<f32>) -> u32 {
    return select(0u, 1u, coord.x > 0.5);
}

fn shell_origin() -> vec3<f32> {
    return vec3<f32>(params.origin_x, params.origin_y, params.origin_z);
}

fn shell_hinge() -> array<vec3<f32>, 2> {
    return array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0));
}

fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    return ShellVertex(torso_vertex(index, coord), 0.0);
}
"#;

/// Record a torso garment's panels and seams, placed by `cage`: WGSL that
/// defines `torso_vertex(index, coord) -> vec3<f32>`, the local point of a
/// vertex (see [`torso_topology`]), reading the fit from `frames`, whose
/// first fifteen floats are the part frame.
pub fn record_garment_torso(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &GarmentArmorDesign,
    fit: &Buffer,
    cage: &str,
) -> Result<DevicePart, GenerateError> {
    design.validate()?;
    let (coords, indices) = torso_topology();
    let kernel = CoordKernel::new(gpu, &format!("{}{cage}{TORSO}", constants()))?;
    let mut part = PartRecipe::new();
    part.push_coord(
        coord_shell(coords, indices, design.wall_thickness.metres(), 2),
        kernel,
    )?;
    part.record(gpu, batch, &[0.0], &[fit])
}
