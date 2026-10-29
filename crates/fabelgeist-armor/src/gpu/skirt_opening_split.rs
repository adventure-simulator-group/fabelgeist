//! Split each local cloth vertex fan into two lips; retain one shared slit tip.
use super::coord::CoordShell;
use super::skirt_opening::VERTICAL_ROUNDOFF_M;
use super::skirt_opening::{
    ORIGINAL_COORD_VARIANTS, OpeningChartPoint, OpeningEdgeSource, SkirtOpeningError,
    SkirtOpeningSource,
};
use super::skirt_opening_cut::CUT_TIP_COORDINATE;
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, SkirtOpeningError>;
pub(super) fn split(
    shell: &mut CoordShell,
    path: &mut [usize],
    source: &SkirtOpeningSource,
    chart: &[Option<OpeningChartPoint>],
    sources: &mut BTreeMap<usize, OpeningEdgeSource>,
) -> Result<BTreeMap<usize, usize>> {
    let last = path
        .len()
        .checked_sub(1)
        .filter(|&i| i > 0)
        .ok_or(SkirtOpeningError::Path)?;
    let above = path[last - 1];
    let below = path[last];
    let plain = |vertex: usize| {
        if let Some(edge) = sources.get(&vertex) {
            edge.vertices.iter().all(|&v| shell.coords[v][3] == 0.)
        } else {
            shell.coords[vertex][3] == 0.
        }
    };
    if !plain(above) || !plain(below) {
        return Err(SkirtOpeningError::Tip);
    }
    let above_y = chart[above].ok_or(SkirtOpeningError::Source)?.y;
    let below_y = chart[below].ok_or(SkirtOpeningError::Source)?.y;
    let tip_on_row = (below_y - source.tip_y).abs() <= VERTICAL_ROUNDOFF_M;
    let (tip, mut faces) = if tip_on_row {
        (below, shell.indices.as_chunks::<3>().0.to_vec())
    } else {
        let tip = shell.coords.len();
        shell.coords.push([
            above as f32,
            below as f32,
            (above_y - source.tip_y) / (above_y - below_y),
            CUT_TIP_COORDINATE,
        ]);
        let mut faces = Vec::new();
        let mut incident = 0;
        for face in shell.indices.as_chunks::<3>().0 {
            if let Some(edge) = (0..3).find(|&i| {
                [face[i] as usize, face[(i + 1) % 3] as usize]
                    .iter()
                    .all(|&v| v == above || v == below)
            }) {
                let [a, b, c] = [face[edge], face[(edge + 1) % 3], face[(edge + 2) % 3]];
                faces.extend([[a, tip as u32, c], [tip as u32, b, c]]);
                incident += 1;
            } else {
                faces.push(*face);
            }
        }
        if incident != 2 {
            return Err(SkirtOpeningError::Tip);
        }
        (tip, faces)
    };
    path[last] = tip;
    let assignments = (0..last)
        .map(|i| fan(&faces, path, i))
        .collect::<Result<Vec<_>>>()?;
    let mut aliases = BTreeMap::new();
    if !tip_on_row {
        sources.insert(
            tip,
            OpeningEdgeSource {
                vertices: [above, below],
                blend: (above_y - source.tip_y) / (above_y - below_y),
            },
        );
    }
    for (i, assigned) in assignments.into_iter().enumerate() {
        let original = path[i];
        let duplicate = shell.coords.len();
        let coord = shell.coords[original];
        shell.coords[original][3] = coord[3] + ORIGINAL_COORD_VARIANTS;
        let mut right = coord;
        right[3] += 2. * ORIGINAL_COORD_VARIANTS;
        shell.coords.push(right);
        aliases.insert(duplicate, original);
        for face in assigned {
            for vertex in &mut faces[face] {
                if *vertex == original as u32 {
                    *vertex = duplicate as u32;
                }
            }
        }
    }
    shell.coords[tip][3] += ORIGINAL_COORD_VARIANTS;
    shell.indices = faces.into_iter().flatten().collect();
    Ok(aliases)
}
fn fan(faces: &[[u32; 3]], path: &[usize], at: usize) -> Result<BTreeSet<usize>> {
    let vertex = path[at] as u32;
    let next = path[at + 1] as u32;
    let previous = at.checked_sub(1).map(|i| path[i] as u32);
    let incident = faces
        .iter()
        .enumerate()
        .filter(|(_, f)| f.contains(&vertex))
        .map(|(i, _)| i)
        .collect::<BTreeSet<_>>();
    let seed = incident
        .iter()
        .copied()
        .find(|&i| (0..3).any(|e| faces[i][e] == vertex && faces[i][(e + 1) % 3] == next))
        .ok_or(SkirtOpeningError::Fan)?;
    let neighbors = |face: usize| {
        faces[face]
            .iter()
            .copied()
            .filter(|&v| v != vertex && v != next && Some(v) != previous)
            .flat_map(|v| {
                incident
                    .iter()
                    .copied()
                    .filter(move |&i| i != face && faces[i].contains(&v))
            })
            .collect::<Vec<_>>()
    };
    let connected = |seed: usize| {
        let mut reached = BTreeSet::from([seed]);
        let mut queue = vec![seed];
        while let Some(face) = queue.pop() {
            for other in neighbors(face) {
                if reached.insert(other) {
                    queue.push(other);
                }
            }
        }
        reached
    };
    let right = connected(seed);
    let other = incident
        .difference(&right)
        .copied()
        .next()
        .ok_or(SkirtOpeningError::Fan)?;
    let left = connected(other);
    if !left.is_disjoint(&right) || left.len() + right.len() != incident.len() {
        return Err(SkirtOpeningError::Fan);
    }
    Ok(right)
}
