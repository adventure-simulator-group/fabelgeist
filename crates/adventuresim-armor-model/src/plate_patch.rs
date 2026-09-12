//! Plate charts with feature-aligned flute sampling.
use crate::{GenerateError, PartMesh, PlateFluting, ShellExtrusion};
struct PlateShell {
    thickness: f32,
    extrusion: ShellExtrusion,
}
const AROUND: usize = 40;
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Longitudinal relief stays registered in the full assembly chart across lames.
pub(crate) fn fluted_patch(
    rows: usize,
    cyclic: bool,
    thickness: f32,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    chart(
        rows,
        if cyclic {
            ChartBoundary::Cyclic
        } else {
            ChartBoundary::Open
        },
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::Normal,
        },
        pattern,
        span,
        normal_offset,
        point,
    )
}

/// A mitten or thumb tip shares its carrier with the adjoining dorsal plate.
pub(crate) fn capped_fluted_patch(
    rows: usize,
    thickness: f32,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    point: impl Fn(f32, f32) -> [f32; 3],
    tip_length: f32,
    normal_offset: impl Fn(f32, f32) -> f32,
) -> Result<PartMesh, GenerateError> {
    let origin = [0.0, point(0.0, 0.0)[1], 0.0];
    chart(
        rows,
        ChartBoundary::Capped(tip_length),
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::CappedAxis {
                origin,
                axis: [0.0, 1.0, 0.0],
            },
        },
        pattern,
        span,
        normal_offset,
        point,
    )
}

/// A shoulder crown ends in a shared apex rather than an open medial mouth.
pub(crate) fn fluted_crown_patch(
    rows: usize,
    origin: [f32; 3],
    thickness: f32,
    pattern: Option<&PlateFluting>,
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    chart(
        rows,
        ChartBoundary::Apex,
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::CappedAxis {
                origin,
                axis: [0.0, -1.0, 0.0],
            },
        },
        pattern,
        [0.5, 1.0],
        normal_offset,
        point,
    )
}

/// A front plate remains a graph over its authored X/Y trim domain.
pub(crate) fn fluted_front_plate(
    rows: usize,
    thickness: f32,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    chart(
        rows,
        ChartBoundary::Open,
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::Along {
                direction: [0.0, 0.0, 1.0],
            },
        },
        pattern,
        span,
        normal_offset,
        point,
    )
}

enum ChartBoundary {
    Open,
    Cyclic,
    Capped(f32),
    Apex,
}

/// Foot lames retain their longitudinal lap boundaries above the sole datum.
pub(crate) fn fluted_foot_patch(
    rows: usize,
    sole: f32,
    thickness: f32,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    chart(
        rows,
        ChartBoundary::Open,
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::Radial {
                origin: [0.0, sole, 0.0],
                axis: [0.0, 0.0, 1.0],
            },
        },
        pattern,
        span,
        normal_offset,
        point,
    )
}

/// Cylindrical plates preserve angular and axial correspondence when thickened.
pub(crate) fn fluted_radial_patch(
    rows: usize,
    cyclic: bool,
    thickness: f32,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    chart(
        rows,
        if cyclic {
            ChartBoundary::Cyclic
        } else {
            ChartBoundary::Open
        },
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::Radial {
                origin: [0.0; 3],
                axis: [0.0, 1.0, 0.0],
            },
        },
        pattern,
        span,
        normal_offset,
        point,
    )
}

fn chart(
    rows: usize,
    boundary: ChartBoundary,
    shell: PlateShell,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    let samples = (0..=rows)
        .map(|row| row as f32 / rows as f32)
        .collect::<Vec<_>>();
    sampled_chart(
        &samples,
        boundary,
        shell,
        pattern,
        span,
        normal_offset,
        point,
    )
}

pub(crate) fn fluted_lapped_radial_patch(
    rows: &[f32],
    thickness: f32,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    sampled_chart(
        rows,
        ChartBoundary::Cyclic,
        PlateShell {
            thickness,
            extrusion: ShellExtrusion::Radial {
                origin: [0.0; 3],
                axis: [0.0, 1.0, 0.0],
            },
        },
        pattern,
        span,
        |_, _| 0.0,
        point,
    )
}

fn sampled_chart(
    samples: &[f32],
    boundary: ChartBoundary,
    shell: PlateShell,
    pattern: Option<&PlateFluting>,
    span: [f32; 2],
    normal_offset: impl Fn(f32, f32) -> f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    let rows = samples.len() - 1;
    let cyclic = matches!(boundary, ChartBoundary::Cyclic);
    let mut columns = pattern.map_or_else(
        || (0..=AROUND).map(|i| i as f32 / AROUND as f32).collect(),
        |pattern| pattern.columns(AROUND),
    );
    if cyclic {
        columns.pop();
    }
    let stride = columns.len();
    let mut positions = Vec::new();
    let mut heights = Vec::new();
    let apex = matches!(boundary, ChartBoundary::Apex);
    let last_row = if apex { rows - 1 } else { rows };
    for &v in &samples[..=last_row] {
        let axial = lerp(span[0], span[1], v);
        for u in &columns {
            positions.push(point(
                pattern.map_or(*u, |pattern| pattern.fan_coordinate(*u, axial)),
                v,
            ));
            heights.push(
                normal_offset(*u, axial) + pattern.map_or(0.0, |pattern| pattern.relief(*u, axial)),
            );
        }
    }
    let mut indices = grid_indices(last_row, stride, cyclic);
    if apex {
        append_apex(
            &mut positions,
            &mut heights,
            &mut indices,
            stride,
            point(0.5, 1.0),
            normal_offset(0.5, span[1]),
        );
    }
    if let ChartBoundary::Capped(length) = boundary {
        const TIP_RINGS: usize = 8;
        let root_y = positions[0][1];
        let mut previous: Vec<u32> = (0..stride as u32).collect();
        for ring in 1..TIP_RINGS {
            let latitude = ring as f32 / TIP_RINGS as f32 * std::f32::consts::FRAC_PI_2;
            let next: Vec<u32> = (0..stride)
                .map(|column| {
                    let base = positions[column];
                    let index = positions.len() as u32;
                    positions.push([
                        base[0] * latitude.cos(),
                        root_y - length * latitude.sin(),
                        base[2] * latitude.cos(),
                    ]);
                    heights.push(heights[column] * latitude.cos().powi(2));
                    index
                })
                .collect();
            for col in 0..stride - 1 {
                let [a, b, c, d] = [previous[col], previous[col + 1], next[col], next[col + 1]];
                indices.extend([a, d, b, a, c, d]);
            }
            previous = next;
        }
        let tip = positions.len() as u32;
        positions.push([0.0, root_y - length, 0.0]);
        heights.push(0.0);
        for col in 0..stride - 1 {
            indices.extend([tip, previous[col + 1], previous[col]]);
        }
    }
    PartMesh::from_relief_surface(
        positions,
        indices,
        shell.thickness,
        if pattern.is_some() {
            crate::BoundaryNormals::Separate
        } else {
            crate::BoundaryNormals::Smooth
        },
        shell.extrusion,
        Some(crate::SurfaceRelief::ShellHeights(heights)),
    )
}

/// Weld the final crown ring to one vertex, avoiding degenerate pole quads.
fn append_apex(
    positions: &mut Vec<[f32; 3]>,
    heights: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    stride: usize,
    point: [f32; 3],
    height: f32,
) {
    let tip = positions.len() as u32;
    let start = positions.len() - stride;
    positions.push(point);
    heights.push(height);
    for col in 0..stride - 1 {
        let a = (start + col) as u32;
        indices.extend([a, a + 1, tip]);
    }
}

fn grid_indices(rows: usize, stride: usize, cyclic: bool) -> Vec<u32> {
    let segments = if cyclic { stride } else { stride - 1 };
    let mut indices = Vec::new();
    for row in 0..rows {
        for col in 0..segments {
            let next = (col + 1) % stride;
            let [a, b, c, d] = [
                row * stride + col,
                row * stride + next,
                (row + 1) * stride + col,
                (row + 1) * stride + next,
            ]
            .map(|i| i as u32);
            indices.extend([a, b, d, a, d, c]);
        }
    }
    indices
}
