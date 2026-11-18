//! Settle each article's cloth clear of the articles under it.
//!
//! An article is fitted over the ones beneath by seating its vertices
//! outside their faces. That samples the surfaces at vertices only. Where
//! cloth leaves an opening in the article beneath, as a full sleeve leaves an
//! armhole, its triangles can still pass through the rim between samples.
//! Settling tests the real surfaces: every edge of the article against the
//! outward faces beneath it, and every edge of those faces against the
//! article. Wherever one passes through the other, the article's cloth
//! there moves a short step outward, and the cloth around it moves with it
//! so that the article's own layers keep their order. Rigid parts do not
//! move. Geometry is canonical unposed metres.
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use fabelgeist_armor::{GeneratedArmor, PlateFace};
use fabelgeist_bvh::{Ray, TriangleBvh};
use fabelgeist_math::Vec3 as BvhPoint;
use std::collections::BTreeMap;

/// Air left between an end that is moved out of a face and that face.
const REST_GAP_M: f32 = 0.002;
/// Most rounds of moving; each moves a vertex by one step at most.
const SETTLE_ROUNDS: usize = 40;
/// Most a vertex moves in one round.
const SETTLE_STEP_M: f32 = 0.002;
/// Distance over which cloth is carried along with a vertex that moves. It
/// spans the layers of one article and a few of its triangles, so a lining
/// moves with the cloth over it and no crease is left at the moved place.
const CARRY_RADIUS_M: f32 = 0.025;

/// What settling could not resolve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerResidue {
    /// Places where an edge of an article or of one beneath it still passes
    /// through the other surface, over the base body and every morph.
    pub crossings: usize,
}

/// The outward faces of the articles beneath one article.
struct Floor {
    tree: TriangleBvh,
    positions: Vec<Vec3>,
    faces: Vec<[u32; 3]>,
    /// Each edge of the faces with the outward direction of the cloth there.
    edges: Vec<([Vec3; 2], Vec3)>,
}

/// The cloth of one article that settling may move.
struct Cloth {
    /// Faces of the article's cloth.
    faces: Vec<[u32; 3]>,
    edges: Vec<[u32; 2]>,
    /// Vertices of every textile component.
    carried: Vec<usize>,
}

/// Settle the cloth of every article clear of the articles fitted before
/// it. `garments` are in fitting order.
pub fn settle_layers(garments: &mut [&mut GeneratedArmor]) -> Result<LayerResidue> {
    let mut residue = LayerResidue::default();
    for index in 1..garments.len() {
        let (earlier, later) = garments.split_at_mut(index);
        let armor = &mut *later[0];
        let Some(cloth) = Cloth::of(armor)? else {
            continue;
        };
        let morphs = armor
            .morphs
            .iter()
            .map(|morph| Some(morph.name.clone()))
            .collect::<Vec<_>>();
        for morph in std::iter::once(None).chain(morphs) {
            let morph = morph.as_deref();
            let Some(floor) = Floor::new(earlier, morph)? else {
                continue;
            };
            residue.crossings += cloth.settle(realization_mut(armor, morph)?, &floor)?;
        }
        let base = &armor.positions;
        for morph in &mut armor.morphs {
            for (delta, (direct, base)) in morph
                .position_deltas
                .iter_mut()
                .zip(morph.direct_positions.iter().zip(base))
            {
                *delta = std::array::from_fn(|axis| direct[axis] - base[axis]);
            }
        }
    }
    Ok(residue)
}

impl Floor {
    /// `None` when nothing beneath has an outward face.
    fn new(earlier: &[&mut GeneratedArmor], morph: Option<&str>) -> Result<Option<Self>> {
        let mut positions = Vec::new();
        let mut faces = Vec::new();
        for armor in earlier {
            let first = u32::try_from(positions.len())?;
            positions.extend(
                realization(armor, morph)?
                    .iter()
                    .map(|p| Vec3::from_array(*p)),
            );
            let (triangles, remainder) = armor.indices.as_chunks::<3>();
            ensure!(remainder.is_empty(), "lower article has invalid triangles");
            for (ordinal, triangle) in triangles.iter().enumerate() {
                // The face against the wearer looks inward; cloth above the
                // article meets its outward face and the walls at its edges.
                let inward = armor.faces.get(ordinal) == Some(&PlateFace::Inner);
                if !inward {
                    faces.push(triangle.map(|vertex| vertex + first));
                }
            }
        }
        if faces.is_empty() {
            return Ok(None);
        }
        let corner = |vertex: u32| positions[vertex as usize];
        let mut outward = BTreeMap::<[u32; 2], Vec3>::new();
        for face in &faces {
            let [a, b, c] = face.map(corner);
            let normal = (b - a).cross(c - a).normalize_or_zero();
            for side in 0..3 {
                let (from, to) = (face[side], face[(side + 1) % 3]);
                *outward.entry([from.min(to), from.max(to)]).or_default() += normal;
            }
        }
        let edges = outward
            .into_iter()
            .filter_map(|(ends, normal)| Some((ends.map(corner), normal.try_normalize()?)))
            .collect();
        let tree = TriangleBvh::new(
            positions.iter().map(|p| point(p.to_array())).collect(),
            faces.clone(),
        );
        Ok(Some(Self {
            tree,
            positions,
            faces,
            edges,
        }))
    }

    /// A corner of one face and the face's outward direction.
    fn plane(&self, face: usize) -> Result<(Vec3, Vec3)> {
        let [a, b, c] = self
            .faces
            .get(face)
            .context("lower article has no such face")?
            .map(|vertex| self.positions[vertex as usize]);
        let normal = (b - a)
            .cross(c - a)
            .try_normalize()
            .context("lower article has a degenerate triangle")?;
        Ok((a, normal))
    }
}

impl Cloth {
    /// `None` for an article without cloth of its own.
    fn of(armor: &GeneratedArmor) -> Result<Option<Self>> {
        let mut faces = Vec::new();
        let mut carried = Vec::new();
        for component in &armor.components {
            if component.textile.is_none() {
                continue;
            }
            carried.extend(component.vertices.clone());
            let (triangles, remainder) = armor.indices[component.indices.clone()].as_chunks::<3>();
            ensure!(remainder.is_empty(), "article has invalid triangles");
            faces.extend_from_slice(triangles);
        }
        if faces.is_empty() {
            return Ok(None);
        }
        let mut edges = faces
            .iter()
            .flat_map(|face| {
                [0, 1, 2].map(|side| {
                    let (from, to) = (face[side], face[(side + 1) % 3]);
                    [from.min(to), from.max(to)]
                })
            })
            .collect::<Vec<_>>();
        edges.sort_unstable();
        edges.dedup();
        Ok(Some(Self {
            faces,
            edges,
            carried,
        }))
    }

    /// Move the cloth at `positions` until it no longer passes through
    /// `floor`. Returns the crossings left when the rounds run out.
    fn settle(&self, positions: &mut [[f32; 3]], floor: &Floor) -> Result<usize> {
        let at = |positions: &[[f32; 3]], vertex: u32| Vec3::from_array(positions[vertex as usize]);
        let mut crossings = 0;
        for _ in 0..SETTLE_ROUNDS {
            let mut push = BTreeMap::<u32, Vec3>::new();
            let mut ask = |vertex: u32, step: Vec3| {
                let asked = push.entry(vertex).or_default();
                if step.length_squared() > asked.length_squared() {
                    *asked = step;
                }
            };
            crossings = 0;
            for &[from, to] in &self.edges {
                let ends = [at(positions, from), at(positions, to)];
                let Some(face) = segment_hit(&floor.tree, ends[0], ends[1]) else {
                    continue;
                };
                crossings += 1;
                let (corner, normal) = floor.plane(face)?;
                // The end behind the face comes out in front of it.
                for (vertex, end) in [from, to].into_iter().zip(ends) {
                    let depth = (corner - end).dot(normal);
                    if depth > 0. {
                        ask(vertex, normal * (depth + REST_GAP_M).min(SETTLE_STEP_M));
                    }
                }
            }
            let tree = TriangleBvh::new(
                positions.iter().map(|p| point(*p)).collect(),
                self.faces.clone(),
            );
            for &([from, to], outward) in &floor.edges {
                let Some(face) = segment_hit(&tree, from, to) else {
                    continue;
                };
                crossings += 1;
                for vertex in self.faces[face] {
                    ask(vertex, outward * SETTLE_STEP_M);
                }
            }
            if crossings == 0 {
                break;
            }
            let moved = push
                .into_iter()
                .map(|(vertex, step)| (at(positions, vertex), step))
                .collect::<Vec<_>>();
            for &vertex in &self.carried {
                let place = Vec3::from_array(positions[vertex]);
                // The nearest moved vertex carries this one, less with
                // distance, so a moved patch eases into the cloth around it.
                let carry = moved
                    .iter()
                    .filter_map(|&(from, step)| {
                        let near = 1. - place.distance(from) / CARRY_RADIUS_M;
                        (near > 0.).then(|| step * (near * near * (3. - 2. * near)))
                    })
                    .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));
                if let Some(carry) = carry {
                    positions[vertex] = (place + carry).to_array();
                }
            }
        }
        Ok(crossings)
    }
}

fn realization<'a>(armor: &'a GeneratedArmor, morph: Option<&str>) -> Result<&'a [[f32; 3]]> {
    Ok(match morph {
        None => &armor.positions,
        Some(name) => {
            &armor
                .morphs
                .iter()
                .find(|morph| morph.name == name)
                .with_context(|| format!("article has no morph {name}"))?
                .direct_positions
        }
    })
}

fn realization_mut<'a>(
    armor: &'a mut GeneratedArmor,
    morph: Option<&str>,
) -> Result<&'a mut Vec<[f32; 3]>> {
    Ok(match morph {
        None => &mut armor.positions,
        Some(name) => {
            &mut armor
                .morphs
                .iter_mut()
                .find(|morph| morph.name == name)
                .with_context(|| format!("article has no morph {name}"))?
                .direct_positions
        }
    })
}

fn point(position: [f32; 3]) -> BvhPoint {
    BvhPoint::new(position[0], position[1], position[2])
}

/// The face of `tree` that the segment from `from` to `to` passes through
/// first, if any.
fn segment_hit(tree: &TriangleBvh, from: Vec3, to: Vec3) -> Option<usize> {
    let span = to - from;
    let length = span.length();
    let direction = span.try_normalize()?;
    let ray = Ray::new(point(from.to_array()), point(direction.to_array()));
    tree.raycast(&ray, length).map(|(face, _)| face as usize)
}
