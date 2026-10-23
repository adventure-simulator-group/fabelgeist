//! Walk the inserted meridian rather than pulling an arbitrary edge route.
use super::skirt_opening::{
    OpeningChartPoint, SkirtOpeningError, SkirtOpeningSource, VERTICAL_ROUNDOFF_M, angle_delta,
};
use super::skirt_opening_cut::MERIDIAN_ANGLE_TOLERANCE_RAD;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn path(
    chart: &[Option<OpeningChartPoint>],
    indices: &[u32],
    start: usize,
    source: &SkirtOpeningSource,
) -> Result<Vec<usize>, SkirtOpeningError> {
    let mut edges = BTreeMap::<[u32; 2], usize>::new();
    for face in indices.as_chunks::<3>().0 {
        for mut edge in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
            edge.sort_unstable();
            *edges.entry(edge).or_default() += 1;
        }
    }
    let mut adjacent = vec![BTreeSet::new(); chart.len()];
    for ([a, b], count) in edges {
        if count != 2 {
            continue;
        }
        let [Some(pa), Some(pb)] = [chart[a as usize], chart[b as usize]] else {
            continue;
        };
        if [pa, pb]
            .iter()
            .all(|p| angle_delta(p.angle, source.axis_angle).abs() <= MERIDIAN_ANGLE_TOLERANCE_RAD)
        {
            adjacent[a as usize].insert(b as usize);
            adjacent[b as usize].insert(a as usize);
        }
    }
    let mut route = vec![start];
    let mut at = start;
    while chart[at].ok_or(SkirtOpeningError::Source)?.y > source.tip_y {
        let height = chart[at].ok_or(SkirtOpeningError::Source)?.y;
        let next = adjacent[at]
            .iter()
            .filter(|&&v| chart[v].is_some_and(|p| p.y < height - VERTICAL_ROUNDOFF_M))
            .copied()
            .collect::<Vec<_>>();
        let [next] = next.as_slice() else {
            return Err(SkirtOpeningError::Path);
        };
        at = *next;
        route.push(at);
    }
    Ok(route)
}
