//! Physical connectivity of cloned body patches.
use super::{CORRESPONDENCE_PRECISION, CUT_VERTEX_SNAP_METRES, SurfaceCut};
use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, BTreeSet};

/// A physically welded patch and its reference points, in canonical metres.
pub(crate) struct WeldedPatch {
    pub cut: SurfaceCut,
    pub positions: Vec<[f32; 3]>,
}

pub(crate) fn weld_patch(cut: SurfaceCut, positions: &[[f32; 3]]) -> WeldedPatch {
    let mut physical = BTreeMap::new();
    let mut points = Vec::new();
    let mut reference = Vec::new();
    let vertices = positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            *physical
                .entry(p.map(|v| (v * CORRESPONDENCE_PRECISION).round() as i64))
                .or_insert_with(|| {
                    let next = points.len() as u32;
                    points.push(cut.points[i]);
                    reference.push(*p);
                    next
                })
        })
        .collect::<Vec<_>>();
    let faces = cut
        .faces
        .iter()
        .map(|face| face.map(|v| vertices[v as usize]))
        .filter(|[a, b, c]| a != b && b != c && c != a)
        .collect::<Vec<_>>();
    WeldedPatch {
        cut: SurfaceCut { points, faces },
        positions: reference,
    }
}

/// Weld physical seams, then retain the component nearest an anatomical anchor.
/// Disconnected limb fragments cannot become detached torso or skirt parts.
pub(crate) fn connected_patch(
    cut: SurfaceCut,
    positions: &[[f32; 3]],
    center: [f32; 3],
) -> Result<SurfaceCut> {
    let WeldedPatch {
        cut: SurfaceCut { points, faces },
        positions: reference,
    } = weld_patch(cut, positions);
    let first = reference
        .iter()
        .enumerate()
        .min_by(|a, b| distance(*a.1, center).total_cmp(&distance(*b.1, center)))
        .map(|(i, _)| i as u32)
        .context("body surface cut is empty")?;
    let mut adjacent = vec![Vec::new(); points.len()];
    for face in &faces {
        for &v in face {
            adjacent[v as usize].extend(face);
        }
    }
    let mut kept = BTreeSet::new();
    let mut pending = vec![first];
    while let Some(vertex) = pending.pop() {
        if kept.insert(vertex) {
            pending.extend(&adjacent[vertex as usize]);
        }
    }
    let mut remap = BTreeMap::new();
    let points = kept
        .iter()
        .map(|&v| {
            remap.insert(v, remap.len() as u32);
            points[v as usize]
        })
        .collect();
    let faces = faces
        .into_iter()
        .filter(|face| kept.contains(&face[0]))
        .map(|face| face.map(|v| remap[&v]))
        .collect();
    Ok(SurfaceCut { points, faces })
}

impl SurfaceCut {
    /// Check all authored panels rather than discarding concealed components.
    pub(crate) fn validate_connected(&self) -> Result<()> {
        ensure!(!self.points.is_empty(), "cloth surface is empty");
        let mut adjacent = vec![Vec::new(); self.points.len()];
        for face in &self.faces {
            for &point in face {
                adjacent[point as usize].extend(face);
            }
        }
        let mut reached = BTreeSet::new();
        let mut pending = vec![0_u32];
        while let Some(point) = pending.pop() {
            if reached.insert(point) {
                pending.extend(&adjacent[point as usize]);
            }
        }
        ensure!(
            reached.len() == self.points.len(),
            "cloth panels or shoulder bridges are disconnected"
        );
        Ok(())
    }

    /// Collapse a numerical aperture smaller than the cutter's two-sided snap
    /// envelope. Intended openings exceed this scale. All incident faces share
    /// one retained source point; no cap or independently fitted vertices are
    /// introduced. Unused points are removed before device normal generation.
    pub(crate) fn collapse_cut_roundoff(self, positions: &[[f32; 3]]) -> Result<Self> {
        let mut aliases = (0..self.points.len() as u32).collect::<Vec<_>>();
        let tolerance_squared = (2. * CUT_VERTEX_SNAP_METRES).powi(2);
        for boundary in self.boundary_loops(positions)? {
            if boundary.iter().all(|&a| {
                boundary.iter().all(|&b| {
                    distance(positions[a as usize], positions[b as usize]) <= tolerance_squared
                })
            }) {
                for &vertex in &boundary {
                    aliases[vertex as usize] = boundary[0];
                }
            }
        }
        let faces = self
            .faces
            .iter()
            .map(|face| face.map(|v| aliases[v as usize]))
            .filter(|[a, b, c]| a != b && b != c && c != a)
            .collect::<Vec<_>>();
        let used = faces.iter().flatten().copied().collect::<BTreeSet<_>>();
        let mut remap = BTreeMap::new();
        let points = used
            .into_iter()
            .map(|old| {
                remap.insert(old, remap.len() as u32);
                self.points[old as usize]
            })
            .collect();
        let faces = faces
            .into_iter()
            .map(|face| face.map(|v| remap[&v]))
            .collect();
        Ok(Self { points, faces })
    }

    /// Ordered opening boundaries after physical welding, using reference
    /// metre positions. Native indices retain the frozen source-point order.
    pub(crate) fn boundary_loops(&self, positions: &[[f32; 3]]) -> Result<Vec<Vec<u32>>> {
        let edges = self.borders(positions);
        let mut next = BTreeMap::new();
        for [a, b] in edges {
            ensure!(
                next.insert(a, b).is_none(),
                "cloned cloth opening is not a simple boundary"
            );
        }
        let mut loops = Vec::new();
        while let Some((&first, _)) = next.first_key_value() {
            let mut boundary = Vec::new();
            let mut at = first;
            loop {
                boundary.push(at);
                at = next
                    .remove(&at)
                    .context("cloned cloth opening is not closed")?;
                if at == first {
                    break;
                }
            }
            loops.push(boundary);
        }
        Ok(loops)
    }
}

fn distance(point: [f32; 3], center: [f32; 3]) -> f32 {
    (0..3)
        .map(|axis| (point[axis] - center[axis]).powi(2))
        .sum()
}
