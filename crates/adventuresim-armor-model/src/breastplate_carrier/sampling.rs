//! Transfer anatomical surface UVs and skin weights to the fitted shell.

use super::*;

/// Detail vertices share displacement from the coarse carrier that defines
/// their surface. Sampling the body independently at each flute ridge can
/// introduce high-frequency displacement and fold narrow relief channels.
pub(super) fn carrier_samples(
    mesh: &MidMesh,
    wearer: Wearer<'_>,
    eligible_faces: &[usize],
) -> Vec<MorphSample> {
    if let Some(samples) = &mesh.morph_samples {
        return samples.clone();
    }
    if let Some(carrier) = &mesh.morph_carrier {
        let coarse = carrier
            .positions
            .iter()
            .map(|point| source_sample(*point, wearer, eligible_faces))
            .collect::<Vec<_>>();
        carrier
            .samples
            .iter()
            .map(|(index, blend)| MorphSample {
                endpoints: [
                    coarse[*index],
                    coarse[*index + 1],
                    coarse[*index],
                    coarse[*index],
                ],
                weights: [1.0 - blend, *blend, 0.0, 0.0],
            })
            .collect()
    } else {
        mesh.positions
            .iter()
            .map(|point| MorphSample {
                endpoints: [source_sample(*point, wearer, eligible_faces); 4],
                weights: [1.0, 0.0, 0.0, 0.0],
            })
            .collect()
    }
}

pub(super) fn source_sample(
    point: [f32; 3],
    wearer: Wearer<'_>,
    eligible_faces: &[usize],
) -> SourceSample {
    let target = local(point, wearer.frame);
    let mut best = None::<(f32, SourceSample)>;
    for face_index in eligible_faces.iter().copied() {
        let face = wearer.source_faces[face_index];
        let indices = face.map(|index| index as usize);
        let triangle = indices.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index].position,
                wearer.frame,
            )
        });
        let weights = closest_triangle_weights(target, triangle);
        let closest = triangle
            .into_iter()
            .zip(weights)
            .fold([0.0; 3], |sum, (vertex, weight)| {
                add(sum, scale(vertex, weight))
            });
        let distance = dot(sub(closest, target), sub(closest, target));
        if best.is_none_or(|current| distance < current.0) {
            best = Some((
                distance,
                SourceSample {
                    face: face_index,
                    weights,
                },
            ));
        }
    }
    best.expect("validated torso has faces").1
}

pub(super) fn closest_triangle_weights(point: [f32; 3], triangle: [[f32; 3]; 3]) -> [f32; 3] {
    let [a, b, c] = triangle;
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(point, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = sub(point, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = sub(point, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let denominator = (va + vb + vc).recip();
    let v = vb * denominator;
    let w = vc * denominator;
    [1.0 - v - w, v, w]
}

pub(super) fn sampled_uv(sample: SourceSample, surface: &TorsoSurface) -> [f32; 2] {
    surface.clearance_mesh.enclosure_texcoord_faces[sample.face]
        .into_iter()
        .zip(sample.weights)
        .fold([0.0; 2], |sum, (index, weight)| {
            let uv = surface.clearance_mesh.enclosure_texcoords[index as usize];
            [sum[0] + uv[0] * weight, sum[1] + uv[1] * weight]
        })
}

pub(super) fn sampled_skin(sample: SourceSample, surface: &TorsoSurface) -> ([u32; 8], [f32; 8]) {
    let mut merged = BTreeMap::<u32, f32>::new();
    for (vertex, source_weight) in surface.clearance_mesh.enclosure_faces[sample.face]
        .into_iter()
        .zip(sample.weights)
    {
        let vertex = vertex as usize;
        for (joint, weight) in surface.clearance_mesh.enclosure_joint_indices[vertex]
            .into_iter()
            .zip(surface.clearance_mesh.enclosure_joint_weights[vertex])
        {
            if weight > 1e-7 {
                *merged.entry(joint).or_default() += source_weight * weight;
            }
        }
    }
    let mut weights = merged.into_iter().collect::<Vec<_>>();
    weights.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    // Bevy currently consumes only JOINTS_0/WEIGHTS_0. Keep the strongest
    // four and renormalize them rather than exporting a silently lost tail.
    weights.truncate(4);
    let total = weights.iter().map(|v| v.1).sum::<f32>().max(1e-8);
    let mut joints = [0; 8];
    let mut result = [0.0; 8];
    for (slot, (joint, weight)) in weights.into_iter().enumerate() {
        joints[slot] = joint;
        result[slot] = weight / total;
    }
    (joints, result)
}

/// Map the torso subset to full enclosure face indices for surface attributes.
pub(super) fn eligible_torso_faces(surface: &TorsoSurface) -> Result<Vec<usize>, GenerateError> {
    let source_face_indices = surface
        .clearance_mesh
        .enclosure_faces
        .iter()
        .copied()
        .enumerate()
        .map(|(index, face)| (face, index))
        .collect::<BTreeMap<_, _>>();
    surface
        .clearance_mesh
        .enclosure_torso_faces
        .iter()
        .map(|face| {
            source_face_indices
                .get(face)
                .copied()
                .ok_or(GenerateError::InvalidSurface)
        })
        .collect()
}
