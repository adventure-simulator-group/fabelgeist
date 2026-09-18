//! Transfer anatomical surface UVs and skin weights to the fitted shell.

use super::*;

const SOURCE_INDEX_LEAF_FACES: usize = 8;

pub(super) struct SourceSampler {
    triangles: Vec<[[f32; 3]; 3]>,
    source_faces: Vec<usize>,
    order: Vec<usize>,
    nodes: Vec<SourceNode>,
}

#[derive(Clone, Copy)]
struct SourceNode {
    lower: [f32; 3],
    upper: [f32; 3],
    children: Option<[usize; 2]>,
    range: [usize; 2],
}

impl SourceSampler {
    pub(super) fn new(wearer: Wearer<'_>, eligible_faces: &[usize]) -> Self {
        let triangles = eligible_faces
            .iter()
            .map(|&face_index| {
                wearer.source_faces[face_index].map(|index| {
                    local(
                        wearer.clearance.enclosure_vertices[index as usize].position,
                        wearer.frame,
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut sampler = Self {
            source_faces: eligible_faces.to_vec(),
            order: (0..triangles.len()).collect(),
            triangles,
            nodes: Vec::new(),
        };
        if !sampler.triangles.is_empty() {
            sampler.build(0, sampler.triangles.len());
        }
        sampler
    }

    pub(super) fn sample(&self, point: [f32; 3], frame: Frame) -> SourceSample {
        let target = local(point, frame);
        let mut best = None;
        self.nearest(0, target, &mut best);
        let (_, rank, weights) = best.expect("validated torso has faces");
        SourceSample {
            face: self.source_faces[rank],
            weights,
        }
    }

    fn build(&mut self, start: usize, end: usize) -> usize {
        let (lower, upper) = source_bounds(&self.triangles, &self.order[start..end]);
        let node = self.nodes.len();
        self.nodes.push(SourceNode {
            lower,
            upper,
            children: None,
            range: [start, end],
        });
        if end - start > SOURCE_INDEX_LEAF_FACES {
            let axis = (0..3)
                .max_by(|&a, &b| (upper[a] - lower[a]).total_cmp(&(upper[b] - lower[b])))
                .expect("three spatial axes");
            let middle = start + (end - start) / 2;
            self.order[start..end].select_nth_unstable_by(middle - start, |&a, &b| {
                source_centroid(self.triangles[a], axis)
                    .total_cmp(&source_centroid(self.triangles[b], axis))
                    .then_with(|| a.cmp(&b))
            });
            let left = self.build(start, middle);
            let right = self.build(middle, end);
            self.nodes[node].children = Some([left, right]);
        }
        node
    }

    fn nearest(
        &self,
        node_index: usize,
        target: [f32; 3],
        best: &mut Option<(f32, usize, [f32; 3])>,
    ) {
        let node = self.nodes[node_index];
        if best.is_some_and(|current| source_bounds_distance(target, node) > current.0) {
            return;
        }
        if let Some(mut children) = node.children {
            if source_bounds_distance(target, self.nodes[children[1]])
                < source_bounds_distance(target, self.nodes[children[0]])
            {
                children.reverse();
            }
            self.nearest(children[0], target, best);
            self.nearest(children[1], target, best);
            return;
        }
        for &rank in &self.order[node.range[0]..node.range[1]] {
            let triangle = self.triangles[rank];
            let weights = closest_triangle_weights(target, triangle);
            let closest = triangle
                .into_iter()
                .zip(weights)
                .fold([0.0; 3], |sum, (vertex, weight)| {
                    add(sum, scale(vertex, weight))
                });
            let delta = sub(closest, target);
            let distance = dot(delta, delta);
            if best.is_none_or(|current| {
                distance < current.0 || (distance == current.0 && rank < current.1)
            }) {
                *best = Some((distance, rank, weights));
            }
        }
    }
}

fn source_bounds(triangles: &[[[f32; 3]; 3]], order: &[usize]) -> ([f32; 3], [f32; 3]) {
    let mut lower = [f32::INFINITY; 3];
    let mut upper = [f32::NEG_INFINITY; 3];
    for &triangle in order {
        for point in triangles[triangle] {
            for axis in 0..3 {
                lower[axis] = lower[axis].min(point[axis]);
                upper[axis] = upper[axis].max(point[axis]);
            }
        }
    }
    (lower, upper)
}

fn source_centroid(triangle: [[f32; 3]; 3], axis: usize) -> f32 {
    triangle.iter().map(|point| point[axis]).sum::<f32>() / 3.0
}

fn source_bounds_distance(point: [f32; 3], node: SourceNode) -> f32 {
    (0..3)
        .map(|axis| {
            if point[axis] < node.lower[axis] {
                node.lower[axis] - point[axis]
            } else if point[axis] > node.upper[axis] {
                point[axis] - node.upper[axis]
            } else {
                0.0
            }
        })
        .map(|distance| distance * distance)
        .sum()
}

/// Detail vertices share displacement from the coarse carrier that defines
/// their surface. Sampling the body independently at each flute ridge can
/// introduce high-frequency displacement and fold narrow relief channels.
pub(super) fn carrier_samples(
    mesh: &MidMesh,
    sampler: &SourceSampler,
    frame: Frame,
) -> Vec<MorphSample> {
    if let Some(samples) = &mesh.morph_samples {
        return samples.clone();
    }
    if let Some(carrier) = &mesh.morph_carrier {
        let coarse = carrier
            .positions
            .iter()
            .map(|point| sampler.sample(*point, frame))
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
                endpoints: [sampler.sample(*point, frame); 4],
                weights: [1.0, 0.0, 0.0, 0.0],
            })
            .collect()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_source_sampling_matches_linear_closest_triangle_search() {
        let triangles = (0..24)
            .map(|index| {
                let x = (index % 6) as f32 * 0.3 - 0.8;
                let y = (index / 6) as f32 * 0.25 - 0.4;
                let z = (index as f32 * 0.37).sin() * 0.2;
                [
                    [x, y, z],
                    [x + 0.21, y + 0.03, z + 0.04],
                    [x + 0.02, y + 0.18, z - 0.03],
                ]
            })
            .collect::<Vec<_>>();
        let mut sampler = SourceSampler {
            source_faces: (100..100 + triangles.len()).collect(),
            order: (0..triangles.len()).collect(),
            triangles,
            nodes: Vec::new(),
        };
        sampler.build(0, sampler.triangles.len());

        for target in [
            [-0.73, -0.21, 0.31],
            [0.04, 0.17, -0.14],
            [0.91, 0.53, 0.27],
            [-1.4, 0.8, -0.5],
        ] {
            let mut indexed = None;
            sampler.nearest(0, target, &mut indexed);
            let indexed = indexed.unwrap();
            let linear = sampler
                .triangles
                .iter()
                .enumerate()
                .map(|(rank, &triangle)| {
                    let weights = closest_triangle_weights(target, triangle);
                    let closest = triangle
                        .into_iter()
                        .zip(weights)
                        .fold([0.0; 3], |sum, (point, weight)| {
                            add(sum, scale(point, weight))
                        });
                    let delta = sub(closest, target);
                    (dot(delta, delta), rank, weights)
                })
                .min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
                .unwrap();
            assert_eq!(indexed.1, linear.1);
            for axis in 0..3 {
                assert!((indexed.2[axis] - linear.2[axis]).abs() < 1e-6);
            }
        }
    }
}
