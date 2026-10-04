//! Ordinary terrain rendering excludes exactly shared interior prism sides.
//! Closed collision cells and physical support remain untouched. Matching is
//! local to each property and uses vertex bits, with no positional tolerance.

use super::*;
use adventuresim_tactical_core::city_layout::grounding::{
    BoundedSettlementTerrain, PropertyFoundationMesh,
};
use std::collections::HashMap;

pub(super) struct GroundPresentation<'a> {
    surface: &'a BoundedSettlementTerrain,
    foundations: Vec<VisibleFoundation<'a>>,
}

impl<'a> GroundPresentation<'a> {
    #[cfg(test)]
    pub(super) fn new(surface: &'a BoundedSettlementTerrain) -> Self {
        Self::from_foundations(surface, surface.foundations.iter())
    }

    pub(super) fn in_rectangles(
        surface: &'a BoundedSettlementTerrain,
        rectangles: &[[Vec2; 2]],
    ) -> Self {
        let foundations = surface.foundations.iter().filter(|foundation| {
            let (low, high) = foundation
                .positions
                .iter()
                .chain(foundation.cut_faces.iter().flatten())
                .fold(
                    (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                    |(low, high), point| (low.min(point.xz()), high.max(point.xz())),
                );
            rectangles
                .iter()
                .any(|[minimum, maximum]| !low.cmpge(*maximum).any() && !high.cmplt(*minimum).any())
        });
        Self::from_foundations(surface, foundations)
    }

    fn from_foundations(
        surface: &'a BoundedSettlementTerrain,
        foundations: impl Iterator<Item = &'a PropertyFoundationMesh>,
    ) -> Self {
        Self {
            surface,
            foundations: foundations
                .map(|mesh| VisibleFoundation {
                    mesh,
                    internal_sides: internal_sides(mesh),
                })
                .collect(),
        }
    }

    pub(super) fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.surface
            .natural_triangles
            .iter()
            .map(|[a, b, c]| [*a, *c, *b])
            .chain(
                self.foundations
                    .iter()
                    .flat_map(|f| f.mesh.cut_faces.iter().copied()),
            )
            .chain(self.foundations.iter().flat_map(|visible| {
                let foundation = visible.mesh;
                let hidden = &visible.internal_sides;
                foundation
                    .solid_triangles
                    .iter()
                    .enumerate()
                    .filter_map(move |(face, indices)| {
                        // Canonical cells have one top, one bottom and three
                        // consecutive side pairs. Keep buried bottoms too;
                        // visibility never substitutes for support validation.
                        let cell_face = face % 8;
                        let visible =
                            cell_face < 2 || hidden[face / 8] & (1 << ((cell_face - 2) / 2)) == 0;
                        visible.then(|| indices.map(|i| foundation.positions[i as usize]))
                    })
            }))
    }
}

struct VisibleFoundation<'a> {
    mesh: &'a PropertyFoundationMesh,
    internal_sides: Vec<u8>,
}

#[derive(Clone, Copy)]
struct SideRef {
    cell: usize,
    side: usize,
    forward: bool,
}

enum SideOccurrence {
    Single(SideRef),
    Pair(SideRef, SideRef),
    Multiple,
}

impl SideOccurrence {
    fn visit(&mut self, side: SideRef) {
        *self = match *self {
            Self::Single(first) => Self::Pair(first, side),
            Self::Pair(_, _) | Self::Multiple => Self::Multiple,
        };
    }
}

fn internal_sides(foundation: &PropertyFoundationMesh) -> Vec<u8> {
    let cells = foundation.positions.as_chunks::<6>().0;
    let mut hidden = vec![0; cells.len()];
    let mut occurrences = HashMap::new();
    for (cell, points) in cells.iter().enumerate() {
        let bits = points.map(|point| point.to_array().map(f32::to_bits));
        for (side, (a, b)) in [(0, 1), (1, 2), (2, 0)].into_iter().enumerate() {
            let mut vertices = [bits[a], bits[b], bits[a + 3], bits[b + 3]];
            vertices.sort_unstable();
            let reference = SideRef {
                cell,
                side,
                forward: bits[a] < bits[b],
            };
            occurrences
                .entry(vertices)
                .and_modify(|entry: &mut SideOccurrence| entry.visit(reference))
                .or_insert(SideOccurrence::Single(reference));
        }
    }
    for occurrence in occurrences.into_values() {
        if let SideOccurrence::Pair(first, second) = occurrence
            && first.forward != second.forward
        {
            hidden[first.cell] |= 1 << first.side;
            hidden[second.cell] |= 1 << second.side;
        }
    }
    hidden
}

#[cfg(test)]
mod tests;
