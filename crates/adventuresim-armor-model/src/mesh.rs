use std::{cmp::Ordering, collections::BTreeMap};

use thiserror::Error;

use crate::{
    AnatomicalSurface, ArmorMorph, BracerDesign, DesignError, GeneratedArmor, design_hash, validate,
};

const ALONG: usize = 16;

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error(
        "sabaton ankle cutaway {cutaway_m} m must be smaller than the available instep span {available_span_m} m"
    )]
    SabatonTrimExceedsFoot {
        cutaway_m: f32,
        available_span_m: f32,
    },
    #[error("invalid bracer design: {0}")]
    Design(#[from] DesignError),
    #[error("anatomical surface domain cannot be empty")]
    EmptyDomain,
    #[error("anatomical surface arrays or topology are inconsistent")]
    InvalidSurface,
    #[error("anatomical surface has no closed forearm contour at the requested placement")]
    EmptySelection,
    #[error("generated armor geometry is degenerate")]
    Degenerate,
}

#[path = "bracer_surface.rs"]
mod surface;
use surface::{
    Sample, add, displaced, dot, face_normal, generated_normals, sample_vec2, sample_vec3,
    structured_samples, subtract,
};
use surface::{normalized, scale};

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
    along: usize,
) -> Vec<u32> {
    let around = body_normals.len() / (along + 1);
    let layer = ((along + 1) * around) as u32;
    let vertex = |ring: usize, segment: usize| (ring * around + segment % around) as u32;
    let mut indices = Vec::new();
    for ring in 0..along {
        for segment in 0..around {
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
    for (ring, reference) in [(0, scale(axis, -1.0)), (along, axis)] {
        for segment in 0..around {
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

fn rim_edges(sample_count: usize, along: usize) -> Vec<[u32; 2]> {
    let around = sample_count / (along + 1);
    [0, along]
        .into_iter()
        .flat_map(|ring| {
            (0..around).map(move |segment| {
                [
                    (ring * around + segment) as u32,
                    (ring * around + (segment + 1) % around) as u32,
                ]
            })
        })
        .collect()
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
    let source_hash = design_hash(design)?;
    let runtime_fluting = matches!(surface.detail, crate::ArmorDetail::Runtime(_))
        .then_some(design.fluting)
        .flatten();
    let mut carrier_design = design.clone();
    if matches!(surface.detail, crate::ArmorDetail::Runtime(_)) {
        carrier_design.fluting = None;
    }
    let design = &carrier_design;
    let along = surface.detail.segments(ALONG, 2);
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
    let positions = displaced(
        &samples,
        &base_positions,
        &base_normals,
        design,
        surface.detail,
    )?;
    let body_normals = samples
        .iter()
        .map(|sample| {
            normalized(sample_vec3(sample, &base_normals)).ok_or(GenerateError::Degenerate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let indices = structured_indices(&positions, &body_normals, axis, along);
    let normals = generated_normals(&positions, &indices)?;
    let source_uvs = surface
        .vertices
        .iter()
        .map(|vertex| vertex.uv)
        .collect::<Vec<_>>();
    let mut texcoords = samples
        .iter()
        .map(|sample| sample_vec2(sample, &source_uvs))
        .chain(
            samples
                .iter()
                .map(|sample| sample_vec2(sample, &source_uvs)),
        )
        .collect();
    let normal_map = bracer_normal_map(
        runtime_fluting,
        design,
        surface.detail,
        along,
        &mut texcoords,
    );
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
    let morphs = sample_morphs(design, surface, &samples, &positions, &normals, &indices)?;
    Ok(GeneratedArmor {
        construction_faces: (0..along * samples.len() / (along + 1))
            .map(|quad| quad * 12 + 6..quad * 12 + 12)
            .chain(std::iter::once(
                along * samples.len() / (along + 1) * 12..indices.len(),
            ))
            .collect(),
        plate_edges: rim_edges(samples.len(), along),
        components: Vec::new(),
        design_hash: source_hash,
        surface_domain: surface.domain.clone(),
        positions,
        normals,
        texcoords,
        normal_map,
        joint_indices,
        joint_weights,
        indices,
        morphs,
    })
}

fn bracer_normal_map(
    pattern: Option<crate::PlateFluting>,
    design: &BracerDesign,
    detail: crate::ArmorDetail,
    along: usize,
    texcoords: &mut Vec<[f32; 2]>,
) -> Option<crate::GeneratedNormalMap> {
    pattern.map(|pattern| {
        let columns = design.columns(detail);
        let chart = (0..=along)
            .flat_map(|row| {
                columns
                    .iter()
                    .map(move |u| (0, [*u, 1.0 - row as f32 / along as f32]))
            })
            .collect::<Vec<_>>();
        let chart = chart
            .iter()
            .copied()
            .chain(chart.iter().copied())
            .collect::<Vec<_>>();
        let tiles = [crate::fluting_texture::FlutingTile::new(
            pattern, 0.25, 0.30,
        )];
        let (normal_map, chart_texcoords) = crate::fluting_texture::atlas(&tiles, &chart);
        *texcoords = chart_texcoords;
        normal_map
    })
}

fn sample_morphs(
    design: &BracerDesign,
    surface: &AnatomicalSurface,
    samples: &[Sample],
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
) -> Result<Vec<ArmorMorph>, GenerateError> {
    surface
        .morphs
        .iter()
        .map(|morph| {
            let target_positions = displaced(
                samples,
                &morph.positions,
                &morph.normals,
                design,
                surface.detail,
            )?;
            let target_normals = generated_normals(&target_positions, indices)?;
            Ok(ArmorMorph {
                name: morph.name.clone(),
                direct_positions: target_positions.clone(),
                position_deltas: target_positions
                    .iter()
                    .zip(positions)
                    .map(|(target, base)| subtract(*target, *base))
                    .collect(),
                normal_deltas: target_normals
                    .iter()
                    .zip(normals)
                    .map(|(target, base)| subtract(*target, *base))
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, GenerateError>>()
}
