//! Leather blends between construction anchors; mounted metal has one owner.
use super::*;
use adventuresim_armor_model::ArmorComponentRole;
use std::ops::Range;

struct Attachment {
    vertices: [usize; 3],
    weights: [f32; 3],
}

struct Hanger {
    upper: Vec3,
    lower: Vec3,
    anchors: [Attachment; 2],
    seat_top: f32,
}

pub(crate) fn attach_suspenders(
    plate: &GeneratedArmor,
    hardware: &mut GeneratedArmor,
    leather_sheets: &[Range<usize>],
) -> Result<()> {
    let mut hangers = leather_sheets
        .iter()
        .map(|range| Hanger::new(plate, &hardware.positions[range.clone()]))
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(!hangers.is_empty(), "suspension requires leather sheets");
    // Connected metal islands share one binding, including opposite walls.
    // Choosing an endpoint once avoids a joint transition through a buckle.
    let metal = hardware
        .components
        .iter()
        .filter(|c| c.role == ArmorComponentRole::Buckles)
        .flat_map(|c| {
            hardware.indices[c.indices.clone()]
                .as_chunks::<3>()
                .0
                .iter()
        })
        .collect::<Vec<_>>();
    let mut neighbors = BTreeMap::<usize, Vec<usize>>::new();
    for face in metal {
        for &a in face {
            neighbors
                .entry(a as usize)
                .or_default()
                .extend(face.iter().map(|&b| b as usize));
        }
    }
    while let Some((&first, _)) = neighbors.first_key_value() {
        let mut pending = vec![first];
        let mut island = Vec::new();
        while let Some(vertex) = pending.pop() {
            if let Some(adjacent) = neighbors.remove(&vertex) {
                pending.extend(adjacent);
                island.push(vertex);
            }
        }
        let center = island
            .iter()
            .map(|&i| Vec3::from_array(hardware.positions[i]))
            .sum::<Vec3>()
            / island.len() as f32;
        let (hanger_index, end) = hangers
            .iter()
            .enumerate()
            .flat_map(|(i, _)| [(i, 0), (i, 1)])
            .min_by(|(a, ae), (b, be)| {
                center
                    .distance_squared([hangers[*a].upper, hangers[*a].lower][*ae])
                    .total_cmp(
                        &center.distance_squared([hangers[*b].upper, hangers[*b].lower][*be]),
                    )
            })
            .expect("nonempty hangers");
        let hanger = &mut hangers[hanger_index];
        if end == 1 {
            hanger.seat_top = island
                .iter()
                .map(|&i| hardware.positions[i][1])
                .fold(hanger.seat_top, f32::max);
        }
        for vertex in island {
            hanger.skin(plate, hardware, vertex, end as f32);
        }
    }
    for (hanger, range) in hangers.iter().zip(leather_sheets) {
        anyhow::ensure!(
            hanger.upper.y > hanger.seat_top,
            "suspension has no flexible span above its buckle seat"
        );
        for vertex in range.clone() {
            let t = ((hanger.upper.y - hardware.positions[vertex][1])
                / (hanger.upper.y - hanger.seat_top))
                .clamp(0.0, 1.0);
            hanger.skin(plate, hardware, vertex, t * t * (3.0 - 2.0 * t));
        }
    }
    // Preserve the independently refitted fastener morph endpoints. The caller
    // computes their skeletal residuals after assigning these skin weights.
    Ok(())
}

impl Hanger {
    fn new(plate: &GeneratedArmor, points: &[[f32; 3]]) -> Result<Self> {
        let bounds = points
            .iter()
            .fold([f32::INFINITY, f32::NEG_INFINITY], |[lo, hi], p| {
                [lo.min(p[1]), hi.max(p[1])]
            });
        anyhow::ensure!(bounds[1] > bounds[0], "hanger has no attachment span");
        let endpoint = |height: f32| {
            let ring = points
                .iter()
                .filter(|p| (p[1] - height).abs() < f32::EPSILON)
                .map(|p| Vec3::from_array(*p))
                .collect::<Vec<_>>();
            ring.iter().sum::<Vec3>() / ring.len() as f32
        };
        let upper = endpoint(bounds[1]);
        let lower = endpoint(bounds[0]);
        Ok(Self {
            upper,
            lower,
            seat_top: lower.y,
            anchors: [
                Attachment::nearest(plate, upper, ArmorComponentRole::Fauld)?,
                Attachment::nearest(plate, lower, ArmorComponentRole::Tassets)?,
            ],
        })
    }

    fn skin(&self, plate: &GeneratedArmor, hardware: &mut GeneratedArmor, vertex: usize, t: f32) {
        let mut influences = BTreeMap::<u32, f32>::new();
        for (binding, blend) in self.anchors.iter().zip([1.0 - t, t]) {
            for (&source, factor) in binding.vertices.iter().zip(binding.weights) {
                for (&joint, &weight) in plate.joint_indices[source]
                    .iter()
                    .zip(&plate.joint_weights[source])
                {
                    if weight * factor * blend > 0.0 {
                        *influences.entry(joint).or_default() += weight * factor * blend;
                    }
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

impl Attachment {
    fn nearest(plate: &GeneratedArmor, point: Vec3, role: ArmorComponentRole) -> Result<Self> {
        plate
            .components
            .iter()
            .filter(|c| c.role == role)
            .flat_map(|c| plate.indices[c.indices.clone()].as_chunks::<3>().0.iter())
            .filter_map(|face| {
                let vertices = [face[0], face[1], face[2]].map(|i| i as usize);
                let points = vertices.map(|i| Vec3::from_array(plate.positions[i]));
                let weights = closest_weights(points, point)?;
                let closest = points
                    .into_iter()
                    .zip(weights)
                    .map(|(p, w)| p * w)
                    .sum::<Vec3>();
                Some((
                    (closest - point).length_squared(),
                    Self { vertices, weights },
                ))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, binding)| binding)
            .context("suspension requires nondegenerate fauld and tasset attachment plates")
    }
}

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
