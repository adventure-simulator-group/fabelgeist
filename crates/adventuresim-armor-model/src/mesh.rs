use std::{cmp::Ordering, collections::BTreeMap};

use thiserror::Error;

use crate::{
    AnatomicalSurface, ArmorMorph, BracerDesign, DesignError, GeneratedArmor, design_hash, validate,
};

const AROUND: usize = 32;
const ALONG: usize = 16;

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("invalid bracer design: {0}")]
    Design(#[from] DesignError),
    #[error("anatomical surface domain cannot be empty")]
    EmptyDomain,
    #[error("anatomical surface arrays or topology are inconsistent")]
    InvalidSurface,
    #[error("anatomical surface has no closed forearm contour at the requested placement")]
    EmptySelection,
    #[error("generated bracer geometry is degenerate")]
    Degenerate,
}

#[derive(Clone)]
struct Sample(Vec<(usize, f32)>);

#[derive(Clone)]
struct ContourPoint {
    sample: Sample,
    position: [f32; 3],
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(value: [f32; 3], factor: f32) -> [f32; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(value: [f32; 3]) -> f32 {
    dot(value, value).sqrt()
}

fn normalized(value: [f32; 3]) -> Option<[f32; 3]> {
    let magnitude = length(value);
    (magnitude > 1e-8).then(|| scale(value, magnitude.recip()))
}

fn face_normal(face: [u32; 3], positions: &[[f32; 3]]) -> [f32; 3] {
    let [a, b, c] = face.map(|index| positions[index as usize]);
    cross(subtract(b, a), subtract(c, a))
}

fn generated_normals(
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

fn sample_vec3(sample: &Sample, values: &[[f32; 3]]) -> [f32; 3] {
    sample
        .0
        .iter()
        .map(|(vertex, weight)| scale(values[*vertex], *weight))
        .fold([0.0; 3], add)
}

fn sample_vec2(sample: &Sample, values: &[[f32; 2]]) -> [f32; 2] {
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
    points.sort_by(|a, b| {
        let angle = |point: &ContourPoint| {
            let radial = subtract(point.position, center);
            dot(radial, v).atan2(dot(radial, u))
        };
        angle(a).partial_cmp(&angle(b)).unwrap_or(Ordering::Equal)
    });

    let mut cumulative = vec![0.0];
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        cumulative.push(
            cumulative[index] + length(subtract(points[next].position, points[index].position)),
        );
    }
    let perimeter = *cumulative.last().unwrap();
    if perimeter <= 1e-7 {
        return Err(GenerateError::Degenerate);
    }
    let mut result = Vec::with_capacity(AROUND);
    for segment in 0..AROUND {
        let distance = perimeter * segment as f32 / AROUND as f32;
        let edge = cumulative
            .windows(2)
            .position(|interval| distance >= interval[0] && distance <= interval[1])
            .unwrap_or(points.len() - 1);
        let edge_length = cumulative[edge + 1] - cumulative[edge];
        let factor = if edge_length <= 1e-8 {
            0.0
        } else {
            (distance - cumulative[edge]) / edge_length
        };
        result.push(blend(
            &points[edge].sample,
            &points[(edge + 1) % points.len()].sample,
            factor,
        ));
    }
    Ok(result)
}

fn structured_samples(
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
    let mut samples = Vec::with_capacity((ALONG + 1) * AROUND);
    for ring in 0..=ALONG {
        let factor = ring as f32 / ALONG as f32;
        // Exact extrema can collapse to one source vertex. This sub-pixel
        // inset retains the intended full span while guaranteeing a loop.
        let axial = (start + (end - start) * factor).clamp(1e-4, 1.0 - 1e-4);
        samples.extend(contour(surface, axial, axis, &positions)?);
    }
    Ok((samples, axis))
}

fn displaced(
    samples: &[Sample],
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    design: &BracerDesign,
) -> Result<Vec<[f32; 3]>, GenerateError> {
    let outer = design.clearance.metres() + design.wall_thickness.metres();
    let inner = design.clearance.metres();
    let point = |sample: &Sample, offset: f32| {
        let normal = normalized(sample_vec3(sample, normals)).ok_or(GenerateError::Degenerate)?;
        Ok(add(sample_vec3(sample, positions), scale(normal, offset)))
    };
    samples
        .iter()
        .map(|sample| point(sample, outer))
        .chain(samples.iter().map(|sample| point(sample, inner)))
        .collect()
}

fn push_oriented(
    indices: &mut Vec<u32>,
    mut face: [u32; 3],
    reference: [f32; 3],
    positions: &[[f32; 3]],
) {
    if dot(face_normal(face, positions), reference) < 0.0 {
        face.swap(1, 2);
    }
    indices.extend(face);
}

fn structured_indices(
    positions: &[[f32; 3]],
    body_normals: &[[f32; 3]],
    axis: [f32; 3],
) -> Vec<u32> {
    let layer = ((ALONG + 1) * AROUND) as u32;
    let vertex = |ring: usize, segment: usize| (ring * AROUND + segment % AROUND) as u32;
    let mut indices = Vec::new();
    for ring in 0..ALONG {
        for segment in 0..AROUND {
            let a = vertex(ring, segment);
            let b = vertex(ring, segment + 1);
            let c = vertex(ring + 1, segment + 1);
            let d = vertex(ring + 1, segment);
            let reference = normalized(add(body_normals[a as usize], body_normals[c as usize]))
                .unwrap_or(body_normals[a as usize]);
            push_oriented(&mut indices, [a, d, c], reference, positions);
            push_oriented(&mut indices, [a, c, b], reference, positions);
            push_oriented(
                &mut indices,
                [a + layer, c + layer, d + layer],
                scale(reference, -1.0),
                positions,
            );
            push_oriented(
                &mut indices,
                [a + layer, b + layer, c + layer],
                scale(reference, -1.0),
                positions,
            );
        }
    }
    for (ring, reference) in [(0, scale(axis, -1.0)), (ALONG, axis)] {
        for segment in 0..AROUND {
            let a = vertex(ring, segment);
            let b = vertex(ring, segment + 1);
            push_oriented(&mut indices, [a, b, b + layer], reference, positions);
            push_oriented(
                &mut indices,
                [a, b + layer, a + layer],
                reference,
                positions,
            );
        }
    }
    indices
}

fn sample_skin(sample: &Sample, surface: &AnatomicalSurface) -> ([u32; 8], [f32; 8]) {
    let mut weights = BTreeMap::<u32, f32>::new();
    for (vertex, sample_weight) in &sample.0 {
        let source = &surface.vertices[*vertex];
        for (joint, weight) in source.joint_indices.iter().zip(source.joint_weights) {
            *weights.entry(*joint).or_default() += weight * sample_weight;
        }
    }
    let mut weights = weights.into_iter().collect::<Vec<_>>();
    weights.sort_by(|(joint_a, weight_a), (joint_b, weight_b)| {
        weight_b
            .partial_cmp(weight_a)
            .unwrap_or(Ordering::Equal)
            .then_with(|| joint_a.cmp(joint_b))
    });
    weights.truncate(8);
    let total = weights.iter().map(|(_, weight)| weight).sum::<f32>();
    let mut joints = [0; 8];
    let mut result = [0.0; 8];
    for (index, (joint, weight)) in weights.into_iter().enumerate() {
        joints[index] = joint;
        result[index] = weight / total;
    }
    (joints, result)
}

fn validate_surface(surface: &AnatomicalSurface) -> Result<(), GenerateError> {
    let vertices = surface.vertices.len();
    if surface.domain.trim().is_empty() {
        return Err(GenerateError::EmptyDomain);
    }
    if vertices == 0
        || surface.faces.is_empty()
        || surface
            .faces
            .iter()
            .flatten()
            .any(|vertex| *vertex as usize >= vertices)
        || surface.vertices.iter().any(|vertex| {
            vertex
                .position
                .iter()
                .chain(&vertex.normal)
                .chain(&vertex.uv)
                .chain(std::iter::once(&vertex.axial))
                .any(|value| !value.is_finite())
                || (vertex.joint_weights.iter().sum::<f32>() - 1.0).abs() > 1e-4
        })
        || surface.morphs.iter().any(|morph| {
            morph.name.trim().is_empty()
                || morph.positions.len() != vertices
                || morph.normals.len() != vertices
                || morph
                    .positions
                    .iter()
                    .flatten()
                    .chain(morph.normals.iter().flatten())
                    .any(|value| !value.is_finite())
        })
    {
        return Err(GenerateError::InvalidSurface);
    }
    Ok(())
}

pub fn generate_bracer(
    design: &BracerDesign,
    surface: &AnatomicalSurface,
) -> Result<GeneratedArmor, GenerateError> {
    validate(design)?;
    validate_surface(surface)?;
    let (samples, axis) = structured_samples(design, surface)?;
    let base_positions = surface
        .vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<_>>();
    let base_normals = surface
        .vertices
        .iter()
        .map(|vertex| vertex.normal)
        .collect::<Vec<_>>();
    let positions = displaced(&samples, &base_positions, &base_normals, design)?;
    let body_normals = samples
        .iter()
        .map(|sample| {
            normalized(sample_vec3(sample, &base_normals)).ok_or(GenerateError::Degenerate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let indices = structured_indices(&positions, &body_normals, axis);
    let normals = generated_normals(&positions, &indices)?;
    let source_uvs = surface
        .vertices
        .iter()
        .map(|vertex| vertex.uv)
        .collect::<Vec<_>>();
    let texcoords = samples
        .iter()
        .map(|sample| sample_vec2(sample, &source_uvs))
        .chain(
            samples
                .iter()
                .map(|sample| sample_vec2(sample, &source_uvs)),
        )
        .collect();
    let skin = samples
        .iter()
        .map(|sample| sample_skin(sample, surface))
        .collect::<Vec<_>>();
    let joint_indices = skin
        .iter()
        .map(|(joints, _)| *joints)
        .chain(skin.iter().map(|(joints, _)| *joints))
        .collect();
    let joint_weights = skin
        .iter()
        .map(|(_, weights)| *weights)
        .chain(skin.iter().map(|(_, weights)| *weights))
        .collect();
    let morphs = surface
        .morphs
        .iter()
        .map(|morph| {
            let target_positions = displaced(&samples, &morph.positions, &morph.normals, design)?;
            let target_normals = generated_normals(&target_positions, &indices)?;
            Ok(ArmorMorph {
                name: morph.name.clone(),
                position_deltas: target_positions
                    .iter()
                    .zip(&positions)
                    .map(|(target, base)| subtract(*target, *base))
                    .collect(),
                normal_deltas: target_normals
                    .iter()
                    .zip(&normals)
                    .map(|(target, base)| subtract(*target, *base))
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, GenerateError>>()?;
    Ok(GeneratedArmor {
        design_hash: design_hash(design)?,
        surface_domain: surface.domain.clone(),
        positions,
        normals,
        texcoords,
        joint_indices,
        joint_weights,
        indices,
        morphs,
    })
}
