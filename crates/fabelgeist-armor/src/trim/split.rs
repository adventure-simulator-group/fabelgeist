//! Cutting the band out of a piece.
//!
//! An outer triangle crossing the band's border is cut where the border
//! meets its edges, found by bisection against the rims themselves, so both
//! triangles sharing an edge share its cut vertex. A new vertex carries every
//! attribute of the piece interpolated along its edge, so skinning and morph
//! targets follow the band.

use std::collections::HashMap;

use super::edges::{Nearest, Rims, lerp};
use super::{ArmorTrim, TrimBand};
use crate::{GeneratedArmor, PlateFace, skin};

/// Bisection steps locating the border on an edge: a millionth of it.
const BORDER_STEPS: u32 = 20;

type Triangle = ([u32; 3], PlateFace);

pub(super) struct Split<'a> {
    armor: GeneratedArmor,
    rims: &'a Rims,
    width: f32,
    /// Per vertex, the nearest rim point within reach.
    near: Vec<Option<Nearest>>,
    /// Per vertex, the component it belongs to, when the piece has any.
    component_of: Vec<usize>,
    /// Border vertices by the edge they cut.
    cuts: HashMap<(u32, u32), u32>,
    /// Copies of vertices one closed rim further along.
    unwrapped: HashMap<u32, u32>,
    /// Per surface, its plate triangles then its band triangles.
    surfaces: Vec<(Vec<Triangle>, Vec<Triangle>)>,
}

impl<'a> Split<'a> {
    pub(super) fn new(armor: GeneratedArmor, rims: &'a Rims, band: TrimBand) -> Self {
        let near = armor.positions.iter().map(|p| rims.nearest(*p)).collect();
        let mut component_of = vec![0; armor.positions.len()];
        for (index, component) in armor.components.iter().enumerate() {
            component_of[component.vertices.clone()].fill(index);
        }
        let layout = armor.surfaces();
        let mut split = Self {
            armor,
            rims,
            width: band.width,
            near,
            component_of,
            cuts: HashMap::new(),
            unwrapped: HashMap::new(),
            surfaces: Vec::with_capacity(layout.len()),
        };
        for surface in layout {
            let component = surface.component.unwrap_or(0);
            let mut plate = Vec::new();
            let mut band = Vec::new();
            for triangle in surface.plate.step_by(3).map(|at| at / 3) {
                let corners = std::array::from_fn(|k| split.armor.indices[triangle * 3 + k]);
                let face = split.armor.faces[triangle];
                split.divide(corners, face, component, &mut plate, &mut band);
            }
            for (corners, _) in &mut band {
                split.unwrap(corners, component);
            }
            split.surfaces.push((plate, band));
        }
        split
    }

    /// Sort one triangle into plate and band, cutting it on the border.
    fn divide(
        &mut self,
        corners: [u32; 3],
        face: PlateFace,
        component: usize,
        plate: &mut Vec<Triangle>,
        band: &mut Vec<Triangle>,
    ) {
        match face {
            PlateFace::Inner => return plate.push((corners, face)),
            PlateFace::Edge => return band.push((corners, face)),
            PlateFace::Outer => {}
        }
        let inside = corners.map(|v| self.inside(v));
        let count = inside.iter().filter(|i| **i).count();
        if count == 3 {
            return band.push((corners, face));
        }
        if count == 0 {
            return plate.push((corners, face));
        }
        // The corner alone on its side of the border, then the other two in
        // winding order.
        let lone = (0..3)
            .find(|k| inside[*k] != inside[(k + 1) % 3] && inside[*k] != inside[(k + 2) % 3])
            .expect("a mixed triangle has a lone corner");
        let [l, a, b] = [0, 1, 2].map(|k| corners[(lone + k) % 3]);
        let p = self.cut(l, a, component);
        let q = self.cut(l, b, component);
        let (tip, base) = if inside[lone] {
            (band, plate)
        } else {
            (plate, band)
        };
        tip.push(([l, p, q], face));
        base.extend([([p, a, b], face), ([p, b, q], face)]);
    }

    fn inside(&self, vertex: u32) -> bool {
        self.near[vertex as usize].is_some_and(|near| near.distance < self.width)
    }

    /// The border vertex on the edge between `a` and `b`, one inside the
    /// band and one out.
    fn cut(&mut self, a: u32, b: u32, component: usize) -> u32 {
        if let Some(cut) = self.cuts.get(&(a.min(b), a.max(b))) {
            return *cut;
        }
        let (inner, outer) = if self.inside(a) { (a, b) } else { (b, a) };
        let [from, to] = [inner, outer].map(|v| self.armor.positions[v as usize]);
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..BORDER_STEPS {
            let middle = 0.5 * (low + high);
            match self.rims.nearest(lerp(from, to, middle)) {
                Some(near) if near.distance < self.width => low = middle,
                _ => high = middle,
            }
        }
        let t = 0.5 * (low + high);
        let near = self
            .rims
            .nearest(lerp(from, to, t))
            .or(self.near[inner as usize])
            .map(|near| Nearest {
                distance: self.width,
                ..near
            });
        let cut = self.push_vertex(inner, outer, t, component, near);
        self.cuts.insert((a.min(b), a.max(b)), cut);
        cut
    }

    /// Give a band triangle straddling a closed rim's start copies of its
    /// low corners one rim further along, so its coordinates run on.
    fn unwrap(&mut self, corners: &mut [u32; 3], component: usize) {
        let near = corners.map(|v| self.near[v as usize]);
        let Some(closure) = near.iter().flatten().map(|n| n.closure).find(|c| *c > 0.0) else {
            return;
        };
        let highest = near
            .iter()
            .flatten()
            .map(|n| n.along)
            .fold(f32::MIN, f32::max);
        for (corner, near) in corners.iter_mut().zip(near) {
            let Some(near) = near.filter(|n| n.along < highest - 0.5 * closure) else {
                continue;
            };
            *corner = match self.unwrapped.get(corner) {
                Some(copy) => *copy,
                None => {
                    let moved = Nearest {
                        along: near.along + closure,
                        ..near
                    };
                    let copy = self.push_vertex(*corner, *corner, 0.0, component, Some(moved));
                    self.unwrapped.insert(*corner, copy);
                    copy
                }
            };
        }
    }

    /// Append a vertex `t` of the way from `a` to `b`.
    fn push_vertex(
        &mut self,
        a: u32,
        b: u32,
        t: f32,
        component: usize,
        near: Option<Nearest>,
    ) -> u32 {
        let (a, b) = (a as usize, b as usize);
        let armor = &mut self.armor;
        let index = armor.positions.len() as u32;
        armor
            .positions
            .push(lerp(armor.positions[a], armor.positions[b], t));
        let normal = lerp(armor.normals[a], armor.normals[b], t);
        let length = normal.iter().map(|x| x * x).sum::<f32>().sqrt();
        armor.normals.push(if length > 0.0 {
            normal.map(|x| x / length)
        } else {
            armor.normals[a]
        });
        let [ua, ub] = [armor.texcoords[a], armor.texcoords[b]];
        armor
            .texcoords
            .push(std::array::from_fn(|k| ua[k] + (ub[k] - ua[k]) * t));
        let (joints, weights) = skin::blend(&[
            ((armor.joint_indices[a], armor.joint_weights[a]), 1.0 - t),
            ((armor.joint_indices[b], armor.joint_weights[b]), t),
        ]);
        armor.joint_indices.push(joints);
        armor.joint_weights.push(weights);
        for morph in &mut armor.morphs {
            for values in [
                &mut morph.position_deltas,
                &mut morph.normal_deltas,
                &mut morph.direct_positions,
            ] {
                values.push(lerp(values[a], values[b], t));
            }
        }
        self.near.push(near);
        self.component_of.push(component);
        index
    }

    /// The trimmed piece, its vertices grouped by component again.
    pub(super) fn finish(self) -> GeneratedArmor {
        let Self {
            mut armor,
            near,
            component_of,
            surfaces,
            ..
        } = self;
        let mut indices = Vec::with_capacity(armor.indices.len());
        let mut faces = Vec::with_capacity(armor.faces.len());
        let mut bands = Vec::with_capacity(surfaces.len());
        let mut push = |indices: &mut Vec<u32>, triangles: Vec<Triangle>| {
            for (corners, face) in triangles {
                indices.extend(corners);
                faces.push(face);
            }
        };
        for (surface, (plate, band)) in surfaces.into_iter().enumerate() {
            let start = indices.len();
            push(&mut indices, plate);
            let band_start = indices.len();
            push(&mut indices, band);
            bands.push(band_start..indices.len());
            if let Some(component) = armor.components.get_mut(surface) {
                component.indices = start..indices.len();
            }
        }
        let coordinates = near
            .iter()
            .map(|near| near.map_or([0.0, 0.0], |n| [n.along, n.distance]))
            .collect();
        armor.indices = indices;
        armor.faces = faces;
        armor.trim = Some(ArmorTrim { coordinates, bands });
        armor.grids.clear();
        if !armor.components.is_empty() {
            group_by_component(&mut armor, &component_of);
        }
        armor
    }
}

/// Reorder the vertices so each component's are contiguous again.
fn group_by_component(armor: &mut GeneratedArmor, component_of: &[usize]) {
    let mut order = (0..component_of.len() as u32).collect::<Vec<_>>();
    order.sort_by_key(|v| component_of[*v as usize]);
    let mut position = vec![0u32; order.len()];
    for (new, old) in order.iter().enumerate() {
        position[*old as usize] = new as u32;
    }
    fn permute<T: Copy>(values: &mut Vec<T>, order: &[u32]) {
        *values = order.iter().map(|old| values[*old as usize]).collect();
    }
    permute(&mut armor.positions, &order);
    permute(&mut armor.normals, &order);
    permute(&mut armor.texcoords, &order);
    permute(&mut armor.joint_indices, &order);
    permute(&mut armor.joint_weights, &order);
    for morph in &mut armor.morphs {
        permute(&mut morph.position_deltas, &order);
        permute(&mut morph.normal_deltas, &order);
        permute(&mut morph.direct_positions, &order);
    }
    if let Some(trim) = &mut armor.trim {
        permute(&mut trim.coordinates, &order);
    }
    for index in &mut armor.indices {
        *index = position[*index as usize];
    }
    let mut start = 0;
    for (index, component) in armor.components.iter_mut().enumerate() {
        let count = component_of.iter().filter(|c| **c == index).count();
        component.vertices = start..start + count;
        start += count;
    }
}
