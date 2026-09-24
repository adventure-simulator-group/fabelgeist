//! Worn plate the cloth lies under.
//!
//! Garments under the plate layer -- clothing, padding and mail -- are pressed
//! in towards the wearer wherever a plate covers them, so the plate shows on
//! top however loosely the cloth settled. The plate is met by its lining: the
//! faces of every worn rigid piece that turn towards the wearer. A cloth point
//! is pressed only where a plate's lining lies over it, not past the plate's
//! edge, and the press spreads to the cloth around it, so cloth tucks under a
//! plate's rim instead of folding sharply at it.

use std::collections::HashMap;

use fabelgeist_armor::{GeneratedArmor, PlateFace};

use super::conform::SewnCloth;
use super::*;
use crate::item_catalog_schema::EquipmentChannel;

/// How far from its lining a cloth point is still found under a plate,
/// metres: cloth that settled through a plate lies well within it.
const REACH_M: f32 = 0.05;
/// A lining point this close to its rim is the plate's edge, metres.
const RIM_TOLERANCE_M: f32 = 1e-4;
/// Rounds of spreading the press to cloth the plate does not cover.
const SPREAD_ROUNDS: usize = 8;
/// Share of the neighbours' average press a vertex takes when the plate does
/// not cover it.
const SPREAD: f32 = 0.5;
/// Rounds of pressing: cloth moved in can come under another plate.
const PRESS_ROUNDS: usize = 2;

/// The wearer-facing faces of every worn rigid piece.
pub struct PlateLining {
    surface: fabelgeist_bvh::TriangleBvh,
    /// Per lining triangle, its edges that are the lining's rim.
    rims: Vec<[bool; 3]>,
}

/// A garment's place under the plate.
#[derive(Clone)]
pub struct UnderPlate {
    pub lining: std::sync::Arc<PlateLining>,
    /// How far in from the lining the garment's mid-surface lies: its own
    /// half thickness and the garments pressed over it, metres.
    pub standoff: f32,
}

impl PlateLining {
    /// The lining of `pieces`; `None` when none of them is a thickened plate.
    pub fn new<'a>(pieces: impl IntoIterator<Item = &'a GeneratedArmor>) -> Option<Self> {
        let mut positions = Vec::new();
        let mut triangles = Vec::new();
        for piece in pieces {
            let offset = positions.len() as u32;
            positions.extend(piece.positions.iter().copied().map(Vec3::from_array));
            for (triangle, face) in piece.indices.as_chunks::<3>().0.iter().zip(&piece.faces) {
                if *face == PlateFace::Inner {
                    triangles.push(triangle.map(|i| i + offset));
                }
            }
        }
        if triangles.is_empty() {
            return None;
        }
        let key = |i: u32| positions[i as usize].to_array().map(f32::to_bits);
        let mut uses = HashMap::<([u32; 3], [u32; 3]), u32>::new();
        let edge = |a: u32, b: u32| {
            let (a, b) = (key(a), key(b));
            (a.min(b), a.max(b))
        };
        for &[a, b, c] in &triangles {
            for (from, to) in [(a, b), (b, c), (c, a)] {
                *uses.entry(edge(from, to)).or_default() += 1;
            }
        }
        let rims = triangles
            .iter()
            .map(|&[a, b, c]| [(a, b), (b, c), (c, a)].map(|(from, to)| uses[&edge(from, to)] == 1))
            .collect();
        Some(Self {
            surface: fabelgeist_bvh::TriangleBvh::new(positions, triangles),
            rims,
        })
    }

    /// The press that puts `point` `standoff` in from the lining over it:
    /// its direction, towards the wearer, and depth. `None` where no plate
    /// covers the point or it already lies deep enough.
    fn press(&self, point: Vec3, standoff: f32) -> Option<Vec3> {
        let (index, closest, _) = self.surface.closest_point(point, REACH_M)?;
        let (a, b, c) = self.surface.triangle(index);
        let rims = self.rims[index as usize];
        let on_rim = [(a, b), (b, c), (c, a)]
            .into_iter()
            .zip(rims)
            .any(|((from, to), rim)| {
                rim && distance_to_segment(closest, from, to) < RIM_TOLERANCE_M
            });
        if on_rim {
            return None;
        }
        let normal = (b - a).cross(c - a).normalize();
        let depth = standoff - (point - closest).dot(normal);
        (depth > 0.0).then_some(normal * depth)
    }
}

impl UnderPlate {
    /// Whether a garment in `channel` is worn under the plate.
    pub fn covers(channel: EquipmentChannel) -> bool {
        channel.order() < EquipmentChannel::RigidArmor.order()
    }
}

impl SewnCloth {
    /// Press the cloth in under the plate, spreading each press to the cloth
    /// around it.
    pub(super) fn tuck_under(&self, positions: &[[f32; 3]], under: &UnderPlate) -> Vec<[f32; 3]> {
        let mut sewn = self.sewn(positions);
        for _ in 0..PRESS_ROUNDS {
            let presses = sewn
                .iter()
                .map(|point| under.lining.press(*point, under.standoff))
                .collect::<Vec<_>>();
            if presses.iter().all(Option::is_none) {
                break;
            }
            let mut spread = presses
                .iter()
                .map(|press| press.unwrap_or_default())
                .collect::<Vec<_>>();
            for _ in 0..SPREAD_ROUNDS {
                let previous = spread.clone();
                for (vertex, press) in presses.iter().enumerate() {
                    if press.is_some() {
                        continue;
                    }
                    let neighbours = self.neighbours(vertex);
                    if neighbours.is_empty() {
                        continue;
                    }
                    let sum = neighbours
                        .iter()
                        .fold(Vec3::default(), |sum, n| sum + previous[*n]);
                    spread[vertex] = sum * (SPREAD / neighbours.len() as f32);
                }
            }
            for (point, press) in sewn.iter_mut().zip(spread) {
                *point = *point + press;
            }
        }
        self.expand(sewn)
    }
}

fn distance_to_segment(point: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let length = ab.dot(ab);
    let t = if length > 0.0 {
        ((point - a).dot(ab) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (point - (a + ab * t)).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square plate a metre across in the plane z = 0.1, facing the wearer
    /// at the origin side, thickened to 2 mm.
    fn plate() -> GeneratedArmor {
        let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
        let mut positions = Vec::new();
        for z in [0.1, 0.102] {
            positions.extend(corners.map(|[x, y]| [x, y, z]));
        }
        // Inner face wound to face -z, outer face +z.
        let indices = vec![0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7];
        GeneratedArmor {
            components: Vec::new(),
            design_hash: [0; 32],
            surface_domain: String::new(),
            normals: vec![[0.0; 3]; 8],
            texcoords: vec![[0.0; 2]; 8],
            joint_indices: vec![[0; 8]; 8],
            joint_weights: vec![[0.0; 8]; 8],
            faces: vec![
                PlateFace::Inner,
                PlateFace::Inner,
                PlateFace::Outer,
                PlateFace::Outer,
            ],
            indices,
            trim: None,
            grids: Vec::new(),
            morphs: Vec::new(),
            positions,
        }
    }

    #[test]
    fn cloth_through_a_plate_is_pressed_under_it() {
        let lining = PlateLining::new([&plate()]).unwrap();
        let press = lining.press(Vec3::new(0.1, 0.0, 0.12), 0.005).unwrap();
        assert!((press - Vec3::new(0.0, 0.0, -0.025)).length() < 1e-5);
        // Already deep enough, beyond the plate's edge, or out of reach.
        assert!(lining.press(Vec3::new(0.1, 0.0, 0.05), 0.005).is_none());
        assert!(lining.press(Vec3::new(0.6, 0.0, 0.12), 0.005).is_none());
        assert!(lining.press(Vec3::new(0.0, 0.0, -0.2), 0.005).is_none());
    }

    #[test]
    fn a_piece_without_a_lining_is_no_plate() {
        let mut open = plate();
        open.faces = Vec::new();
        open.indices = Vec::new();
        assert!(PlateLining::new([&open]).is_none());
        assert!(UnderPlate::covers(EquipmentChannel::FlexibleArmor));
        assert!(!UnderPlate::covers(EquipmentChannel::Outerwear));
    }
}
