//! Hangers retain one physical plate attachment across skin and shape changes.
use super::*;

struct Attachment {
    vertices: [usize; 3],
    weights: [f32; 3],
}

pub(crate) fn attach_suspenders(
    plate: &GeneratedArmor,
    hardware: &mut GeneratedArmor,
) -> Result<()> {
    // Opposite walls share an attachment, preserving their leather gauge.
    let mut front = BTreeMap::<[i64; 2], f32>::new();
    let key = |p: &[f32; 3]| [p[0], p[1]].map(|v| (v * 1e6).round() as i64);
    for p in &hardware.positions {
        front
            .entry(key(p))
            .and_modify(|z| *z = z.max(p[2]))
            .or_insert(p[2]);
    }
    let bindings = hardware
        .positions
        .iter()
        .map(|p| Attachment::nearest(plate, [p[0], p[1], front[&key(p)]]))
        .collect::<Result<Vec<_>>>()?;
    for (vertex, binding) in bindings.iter().enumerate() {
        binding.skin(plate, hardware, vertex);
    }
    for (target, source) in hardware.morphs.iter_mut().zip(&plate.morphs) {
        anyhow::ensure!(
            target.name == source.name,
            "suspension morph ordering differs from plate"
        );
        for (vertex, binding) in bindings.iter().enumerate() {
            let delta = std::array::from_fn(|axis| {
                binding
                    .vertices
                    .iter()
                    .zip(binding.weights)
                    .map(|(&i, w)| {
                        w * (source.direct_positions[i][axis] - plate.positions[i][axis])
                    })
                    .sum::<f32>()
            });
            target.position_deltas[vertex] = delta;
            target.direct_positions[vertex] =
                std::array::from_fn(|axis| hardware.positions[vertex][axis] + delta[axis]);
        }
        let mut mesh = adventuresim_armor_model::PartMesh::new();
        mesh.positions = target.direct_positions.clone();
        mesh.indices = hardware.indices.clone();
        target.normal_deltas = mesh
            .normals()?
            .iter()
            .zip(&hardware.normals)
            .map(|(a, b)| std::array::from_fn(|axis| a[axis] - b[axis]))
            .collect();
    }
    Ok(())
}

impl Attachment {
    fn nearest(plate: &GeneratedArmor, point: [f32; 3]) -> Result<Self> {
        plate
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .filter_map(|face| {
                let vertices = face.map(|i| i as usize);
                let points = vertices.map(|i| Vec3::from_array(plate.positions[i]));
                let weights = closest_weights(points, Vec3::from_array(point))?;
                let closest = points
                    .into_iter()
                    .zip(weights)
                    .map(|(p, w)| p * w)
                    .sum::<Vec3>();
                Some((
                    (closest - Vec3::from_array(point)).length_squared(),
                    Self { vertices, weights },
                ))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, binding)| binding)
            .context("suspension requires a nondegenerate attachment plate")
    }

    fn skin(&self, plate: &GeneratedArmor, hardware: &mut GeneratedArmor, vertex: usize) {
        let mut influences = BTreeMap::<u32, f32>::new();
        for (&source, factor) in self.vertices.iter().zip(self.weights) {
            for (&joint, &weight) in plate.joint_indices[source]
                .iter()
                .zip(&plate.joint_weights[source])
            {
                if weight > 0.0 {
                    *influences.entry(joint).or_default() += weight * factor;
                }
            }
        }
        let mut influences = influences.into_iter().collect::<Vec<_>>();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        influences.truncate(4);
        let sum: f32 = influences.iter().map(|(_, weight)| weight).sum();
        hardware.joint_indices[vertex] = [0; 8];
        hardware.joint_weights[vertex] = [0.0; 8];
        for (slot, (joint, weight)) in influences.into_iter().enumerate() {
            hardware.joint_indices[vertex][slot] = joint;
            hardware.joint_weights[vertex][slot] = weight / sum;
        }
    }
}

/// Project onto the triangle plane, then onto its boundary when outside it.
fn closest_weights([a, b, c]: [Vec3; 3], point: Vec3) -> Option<[f32; 3]> {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let determinant = ab.length_squared() * ac.length_squared() - ab.dot(ac).powi(2);
    if determinant <= 0.0 {
        return None;
    }
    let beta = (ac.length_squared() * ap.dot(ab) - ab.dot(ac) * ap.dot(ac)) / determinant;
    let gamma = (ab.length_squared() * ap.dot(ac) - ab.dot(ac) * ap.dot(ab)) / determinant;
    let weights = [1.0 - beta - gamma, beta, gamma];
    if weights.iter().all(|w| *w >= 0.0) {
        return Some(weights);
    }
    let points = [a, b, c];
    (0..3)
        .map(|i| {
            let j = (i + 1) % 3;
            let edge = points[j] - points[i];
            let t = ((point - points[i]).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
            let mut weights = [0.0; 3];
            weights[i] = 1.0 - t;
            weights[j] = t;
            ((point - points[i] - edge * t).length_squared(), weights)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, weights)| weights)
}
