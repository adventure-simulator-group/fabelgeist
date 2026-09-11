//! Forearm contour correspondence, carrier sampling and authored relief.
use super::{ALONG, AnatomicalSurface, BracerDesign, GenerateError};
use std::{cmp::Ordering, collections::BTreeMap};
#[derive(Clone)]
pub(super) struct Sample(pub(super) Vec<(usize, f32)>);

#[derive(Clone)]
struct ContourPoint {
    sample: Sample,
    position: [f32; 3],
}

pub(super) fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(super) fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn scale(value: [f32; 3], factor: f32) -> [f32; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

pub(super) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(value: [f32; 3]) -> f32 {
    dot(value, value).sqrt()
}

pub(super) fn normalized(value: [f32; 3]) -> Option<[f32; 3]> {
    let magnitude = length(value);
    (magnitude > 1e-8).then(|| scale(value, magnitude.recip()))
}

pub(super) fn face_normal(face: [u32; 3], positions: &[[f32; 3]]) -> [f32; 3] {
    let [a, b, c] = face.map(|index| positions[index as usize]);
    cross(subtract(b, a), subtract(c, a))
}

pub(super) fn generated_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Result<Vec<[f32; 3]>, GenerateError> {
    let mut normals = vec![[0.0; 3]; positions.len()];
    for triangle in indices.as_chunks::<3>().0 {
        let face = face_normal([triangle[0], triangle[1], triangle[2]], positions);
        if dot(face, face) <= 1e-16 || face.iter().any(|value| !value.is_finite()) {
            return Err(GenerateError::Degenerate);
        }
        for vertex in triangle {
            normals[*vertex as usize] = add(normals[*vertex as usize], face);
        }
    }
    normals
        .into_iter()
        .map(|normal| normalized(normal).ok_or(GenerateError::Degenerate))
        .collect()
}

fn merge(weighted: impl IntoIterator<Item = (usize, f32)>) -> Sample {
    let mut merged = BTreeMap::<usize, f32>::new();
    for (vertex, weight) in weighted {
        if weight > 1e-7 {
            *merged.entry(vertex).or_default() += weight;
        }
    }
    let total = merged.values().sum::<f32>();
    Sample(
        merged
            .into_iter()
            .map(|(vertex, weight)| (vertex, weight / total))
            .collect(),
    )
}

fn blend(a: &Sample, b: &Sample, factor: f32) -> Sample {
    merge(
        a.0.iter()
            .map(|(vertex, weight)| (*vertex, *weight * (1.0 - factor)))
            .chain(
                b.0.iter()
                    .map(|(vertex, weight)| (*vertex, *weight * factor)),
            ),
    )
}

pub(super) fn sample_vec3(sample: &Sample, values: &[[f32; 3]]) -> [f32; 3] {
    sample
        .0
        .iter()
        .map(|(vertex, weight)| scale(values[*vertex], *weight))
        .fold([0.0; 3], add)
}

pub(super) fn sample_vec2(sample: &Sample, values: &[[f32; 2]]) -> [f32; 2] {
    sample.0.iter().fold([0.0; 2], |result, (vertex, weight)| {
        [
            result[0] + values[*vertex][0] * weight,
            result[1] + values[*vertex][1] * weight,
        ]
    })
}

fn edge_point(
    a: usize,
    b: usize,
    axial: f32,
    surface: &AnatomicalSurface,
    positions: &[[f32; 3]],
) -> Option<ContourPoint> {
    let start = surface.vertices[a].axial;
    let span = surface.vertices[b].axial - start;
    if span.abs() <= 1e-7 {
        return None;
    }
    let factor = (axial - start) / span;
    if !(-1e-6..=1.0 + 1e-6).contains(&factor) {
        return None;
    }
    let sample = merge([
        (a, 1.0 - factor.clamp(0.0, 1.0)),
        (b, factor.clamp(0.0, 1.0)),
    ]);
    let position = sample_vec3(&sample, positions);
    Some(ContourPoint { sample, position })
}

fn surface_axis(surface: &AnatomicalSurface) -> Result<[f32; 3], GenerateError> {
    let count = surface.vertices.len() as f32;
    let mean_axial = surface.vertices.iter().map(|v| v.axial).sum::<f32>() / count;
    let mean_position = scale(
        surface
            .vertices
            .iter()
            .map(|v| v.position)
            .fold([0.0; 3], add),
        count.recip(),
    );
    normalized(
        surface
            .vertices
            .iter()
            .map(|v| scale(subtract(v.position, mean_position), v.axial - mean_axial))
            .fold([0.0; 3], add),
    )
    .ok_or(GenerateError::Degenerate)
}

fn contour(
    surface: &AnatomicalSurface,
    axial: f32,
    axis: [f32; 3],
    positions: &[[f32; 3]],
    columns: &[f32],
) -> Result<Vec<Sample>, GenerateError> {
    let (minimum, maximum) = positions.iter().copied().fold(
        ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]),
        |(mut minimum, mut maximum), position| {
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(position[axis]);
                maximum[axis] = maximum[axis].max(position[axis]);
            }
            (minimum, maximum)
        },
    );
    let scale_hint = length(subtract(maximum, minimum));
    let weld_distance = (scale_hint * 1e-5).max(1e-7);
    let mut points = Vec::<ContourPoint>::new();
    for face in &surface.faces {
        for (a, b) in [
            (face[0] as usize, face[1] as usize),
            (face[1] as usize, face[2] as usize),
            (face[2] as usize, face[0] as usize),
        ] {
            if let Some(point) = edge_point(a, b, axial, surface, positions)
                && !points
                    .iter()
                    .any(|other| length(subtract(point.position, other.position)) <= weld_distance)
            {
                points.push(point);
            }
        }
    }
    if points.len() < 3 {
        return Err(GenerateError::EmptySelection);
    }

    let center = scale(
        points
            .iter()
            .map(|point| point.position)
            .fold([0.0; 3], add),
        (points.len() as f32).recip(),
    );
    let reference = if axis[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = normalized(cross(axis, reference)).ok_or(GenerateError::Degenerate)?;
    let v = cross(axis, u);
    let angle = |point: &ContourPoint| {
        let radial = subtract(point.position, center);
        dot(radial, v).atan2(dot(radial, u))
    };
    points.sort_by(|a, b| angle(a).partial_cmp(&angle(b)).unwrap_or(Ordering::Equal));

    let angles = points.iter().map(angle).collect::<Vec<_>>();
    let mut result = Vec::with_capacity(columns.len());
    for column in columns {
        let target = -std::f32::consts::PI + std::f32::consts::TAU * column;
        let bracket = angles
            .windows(2)
            .position(|pair| target >= pair[0] && target <= pair[1]);
        let (a, b, start_angle, end_angle) = if let Some(edge) = bracket {
            (edge, edge + 1, angles[edge], angles[edge + 1])
        } else if target < angles[0] {
            (
                points.len() - 1,
                0,
                angles[points.len() - 1] - std::f32::consts::TAU,
                angles[0],
            )
        } else {
            (
                points.len() - 1,
                0,
                angles[points.len() - 1],
                angles[0] + std::f32::consts::TAU,
            )
        };
        let factor = (target - start_angle) / (end_angle - start_angle);
        result.push(blend(&points[a].sample, &points[b].sample, factor));
    }
    Ok(result)
}

pub(super) fn structured_samples(
    design: &BracerDesign,
    surface: &AnatomicalSurface,
) -> Result<(Vec<Sample>, [f32; 3]), GenerateError> {
    let axis = surface_axis(surface)?;
    let positions = surface
        .vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<_>>();
    let (start, end) = design.axial_interval();
    let columns = design.columns();
    let mut samples = Vec::with_capacity((ALONG + 1) * columns.len());
    for ring in 0..=ALONG {
        let factor = ring as f32 / ALONG as f32;
        // Exact extrema can collapse to one source vertex. This sub-pixel
        // inset retains the intended full span while guaranteeing a loop.
        let axial = (start + (end - start) * factor).clamp(1e-4, 1.0 - 1e-4);
        let mapped = columns
            .iter()
            .map(|u| {
                design
                    .fluting
                    .as_ref()
                    .map_or(*u, |f| f.fan_coordinate(*u, 1.0 - factor))
            })
            .collect::<Vec<_>>();
        samples.extend(contour(surface, axial, axis, &positions, &mapped)?);
    }
    Ok((samples, axis))
}

pub(super) fn displaced(
    samples: &[Sample],
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    design: &BracerDesign,
) -> Result<Vec<[f32; 3]>, GenerateError> {
    let outer = design.clearance.metres() + design.wall_thickness.metres();
    let inner = design.clearance.metres();
    let columns = design.columns();
    let point = |(index, sample): (usize, &Sample), offset: f32| {
        let offset = offset
            + design.relief(
                columns[index % columns.len()],
                (index / columns.len()) as f32 / ALONG as f32,
            );
        let normal = normalized(sample_vec3(sample, normals)).ok_or(GenerateError::Degenerate)?;
        Ok(add(sample_vec3(sample, positions), scale(normal, offset)))
    };
    samples
        .iter()
        .enumerate()
        .map(|sample| point(sample, outer))
        .chain(
            samples
                .iter()
                .enumerate()
                .map(|sample| point(sample, inner)),
        )
        .collect()
}
