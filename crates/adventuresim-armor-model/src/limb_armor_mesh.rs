//! Regular authored patch connectivity and angular shell sampling.

use std::f32::consts::PI;

use crate::{GenerateError, parametric::PartMesh};

pub(super) const AROUND: usize = 40;
pub(super) const ALONG: usize = 16;

pub(super) use crate::plate_patch::fluted_patch;

/// A continuous upper and sole, with the ankle as its only open boundary.
pub(super) fn boot_shell(
    columns: usize,
    rows: usize,
    thickness: f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    let (mut positions, mut indices) = grid(columns, rows, true, point);
    let sole_center = std::array::from_fn(|axis| {
        positions[..columns].iter().map(|p| p[axis]).sum::<f32>() / columns as f32
    });
    let center = positions.len() as u32;
    positions.push(sole_center);
    for index in 0..columns {
        indices.extend([center, ((index + 1) % columns) as u32, index as u32]);
    }
    PartMesh::from_surface(
        positions,
        indices,
        thickness,
        crate::BoundaryNormals::Smooth,
        crate::ShellExtrusion::Normal,
    )
}

fn grid(
    columns: usize,
    rows: usize,
    cyclic: bool,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let stride = if cyclic { columns } else { columns + 1 };
    let mut positions = Vec::with_capacity(stride * (rows + 1));
    for row in 0..=rows {
        for column in 0..stride {
            positions.push(point(
                column as f32 / columns as f32,
                row as f32 / rows as f32,
            ));
        }
    }
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..rows {
        for column in 0..columns {
            let next = (column + 1) % stride;
            let a = (row * stride + column) as u32;
            let b = (row * stride + next) as u32;
            let c = ((row + 1) * stride + column) as u32;
            let d = ((row + 1) * stride + next) as u32;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }
    (positions, indices)
}

pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// A half-ellipsoid shell with an explicit fan tip, rather than a collapsed grid row.
pub(super) fn half_dome(
    width: f32,
    height: f32,
    sole: f32,
    start: f32,
    end: f32,
    gauge: f32,
    roundness: f32,
) -> Result<PartMesh, GenerateError> {
    const RINGS: usize = 8;
    let stride = AROUND + 1;
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for row in 0..RINGS {
        let latitude = row as f32 / RINGS as f32 * PI * 0.5;
        for column in 0..=AROUND {
            let theta = (0.5 - column as f32 / AROUND as f32) * PI;
            positions.push([
                width * theta.sin() * latitude.cos().powf(roundness),
                sole + height * theta.cos() * latitude.cos(),
                lerp(start, end, latitude.sin()),
            ]);
        }
    }
    for row in 0..RINGS - 1 {
        for column in 0..AROUND {
            let a = (row * stride + column) as u32;
            let b = a + 1;
            let c = a + stride as u32;
            let e = c + 1;
            indices.extend_from_slice(&[a, b, e, a, e, c]);
        }
    }
    let tip = positions.len() as u32;
    positions.push([0.0, sole, end]);
    for column in 0..AROUND {
        let a = ((RINGS - 1) * stride + column) as u32;
        indices.extend_from_slice(&[a, a + 1, tip]);
    }
    PartMesh::from_surface(
        positions,
        indices,
        gauge,
        crate::BoundaryNormals::Smooth,
        crate::ShellExtrusion::Normal,
    )
}
