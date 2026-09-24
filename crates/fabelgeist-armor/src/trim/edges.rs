//! The rims of a piece, and the nearest rim to any point.
//!
//! Generators duplicate vertices for their own reasons: separate normals on
//! an edge wall, a seam between two charts, a crease. The rims are therefore
//! found on the positions welded together, where the outer face is one
//! surface whose boundary edges are used by a single outer triangle.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::TrimBand;
use crate::PlateFace;

/// Welding resolution: vertices within a micrometre are one point.
const WELD_STEPS_PER_METRE: f32 = 1e6;
/// No plate is thicker, so every edge wall vertex finds its rim however
/// narrow the band.
const THICKEST_PLATE: f32 = 0.025;

/// The rim point nearest a query.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Nearest {
    pub distance: f32,
    /// Metres along the rim, rescaled on a closed rim to whole periods.
    pub along: f32,
    /// The rescaled length of a closed rim; zero on an open one.
    pub closure: f32,
}

struct Segment {
    ends: [[f32; 3]; 2],
    along: [f32; 2],
    closure: f32,
}

/// Every rim of a piece as segments, bucketed on a grid one band wide.
pub(super) struct Rims {
    segments: Vec<Segment>,
    cells: HashMap<[i32; 3], Vec<u32>>,
    reach: f32,
}

impl Rims {
    pub(super) fn new(
        positions: &[[f32; 3]],
        indices: &[u32],
        faces: &[PlateFace],
        band: TrimBand,
    ) -> Self {
        let (welded, points) = weld(positions);
        let mut rims = Self {
            segments: Vec::new(),
            cells: HashMap::new(),
            reach: band.width.max(THICKEST_PLATE),
        };
        for chain in chains(&rim_edges(indices, faces, &welded)) {
            rims.push_chain(
                &chain
                    .iter()
                    .map(|i| points[*i as usize])
                    .collect::<Vec<_>>(),
                band,
            );
        }
        rims
    }

    /// The nearest rim point within reach of `point`: one band width, or
    /// one plate thickness when that is more.
    pub(super) fn nearest(&self, point: [f32; 3]) -> Option<Nearest> {
        let mut best: Option<Nearest> = None;
        for &index in self.cells.get(&self.cell(point))?.iter() {
            let segment = &self.segments[index as usize];
            let [a, b] = segment.ends;
            let ab = sub(b, a);
            let length = dot(ab, ab);
            let t = if length > 0.0 {
                (dot(sub(point, a), ab) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let offset = sub(point, lerp(a, b, t));
            let distance = dot(offset, offset).sqrt();
            if distance <= self.reach && best.is_none_or(|best| distance < best.distance) {
                best = Some(Nearest {
                    distance,
                    along: segment.along[0] + (segment.along[1] - segment.along[0]) * t,
                    closure: segment.closure,
                });
            }
        }
        best
    }

    fn cell(&self, point: [f32; 3]) -> [i32; 3] {
        point.map(|x| (x / self.reach).floor() as i32)
    }

    /// Add one rim, a polyline that is closed when its ends meet.
    fn push_chain(&mut self, points: &[[f32; 3]], band: TrimBand) {
        let mut along = vec![0.0f32];
        for pair in points.windows(2) {
            let step = sub(pair[1], pair[0]);
            along.push(along[along.len() - 1] + dot(step, step).sqrt());
        }
        let length = along[along.len() - 1];
        let closed = points.len() > 2 && points[0] == points[points.len() - 1];
        let (scale, closure) = if closed && length > 0.0 {
            let periods = (length / band.period).round().max(1.0);
            (periods * band.period / length, periods * band.period)
        } else {
            (1.0, 0.0)
        };
        for (pair, span) in points.windows(2).zip(along.windows(2)) {
            let index = self.segments.len() as u32;
            let low = self.cell(std::array::from_fn(|k| {
                pair[0][k].min(pair[1][k]) - self.reach
            }));
            let high = self.cell(std::array::from_fn(|k| {
                pair[0][k].max(pair[1][k]) + self.reach
            }));
            for x in low[0]..=high[0] {
                for y in low[1]..=high[1] {
                    for z in low[2]..=high[2] {
                        self.cells.entry([x, y, z]).or_default().push(index);
                    }
                }
            }
            self.segments.push(Segment {
                ends: [pair[0], pair[1]],
                along: [span[0] * scale, span[1] * scale],
                closure,
            });
        }
    }
}

/// Each vertex's welded point, and the welded points.
fn weld(positions: &[[f32; 3]]) -> (Vec<u32>, Vec<[f32; 3]>) {
    let mut ids = HashMap::new();
    let mut points = Vec::new();
    let welded = positions
        .iter()
        .map(|p| {
            *ids.entry(p.map(|x| (x * WELD_STEPS_PER_METRE).round() as i64))
                .or_insert_with(|| {
                    points.push(*p);
                    (points.len() - 1) as u32
                })
        })
        .collect();
    (welded, points)
}

/// The outer face's boundary edges between welded points, directed as the
/// outer triangle using each is wound.
fn rim_edges(indices: &[u32], faces: &[PlateFace], welded: &[u32]) -> Vec<(u32, u32)> {
    let mut edges = BTreeMap::<(u32, u32), (u32, (u32, u32))>::new();
    for (triangle, face) in indices.as_chunks::<3>().0.iter().zip(faces) {
        if *face != PlateFace::Outer {
            continue;
        }
        let [a, b, c] = triangle.map(|i| welded[i as usize]);
        for (start, end) in [(a, b), (b, c), (c, a)] {
            if start != end {
                let entry = edges
                    .entry((start.min(end), start.max(end)))
                    .or_insert((0, (start, end)));
                entry.0 += 1;
            }
        }
    }
    edges
        .into_values()
        .filter(|(uses, _)| *uses == 1)
        .map(|(_, edge)| edge)
        .collect()
}

/// Join directed edges into polylines of welded points. A closed chain
/// repeats its first point last; open chains start where no edge arrives.
fn chains(edges: &[(u32, u32)]) -> Vec<Vec<u32>> {
    let mut outgoing = BTreeMap::<u32, Vec<usize>>::new();
    let arriving = edges.iter().map(|(_, end)| *end).collect::<BTreeSet<_>>();
    for (index, (start, _)) in edges.iter().enumerate() {
        outgoing.entry(*start).or_default().push(index);
    }
    let mut used = vec![false; edges.len()];
    let open_starts = (0..edges.len()).filter(|e| !arriving.contains(&edges[*e].0));
    let mut chains = Vec::new();
    for first in open_starts.chain(0..edges.len()).collect::<Vec<_>>() {
        if used[first] {
            continue;
        }
        let mut chain = vec![edges[first].0];
        let mut at = first;
        loop {
            used[at] = true;
            let end = edges[at].1;
            chain.push(end);
            if end == chain[0] {
                break;
            }
            match outgoing
                .get(&end)
                .and_then(|next| next.iter().find(|e| !used[**e]))
            {
                Some(next) => at = *next,
                None => break,
            }
        }
        chains.push(chain);
    }
    chains
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| a[k] - b[k])
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closed_rim_holds_whole_periods() {
        // A unit square outline, 4 m round, with a 0.3 m period: 13 periods
        // stretch it to 3.9 m.
        let square = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let mut rims = Rims {
            segments: Vec::new(),
            cells: HashMap::new(),
            reach: 0.1,
        };
        let band = TrimBand {
            width: 0.1,
            period: 0.3,
        };
        rims.push_chain(&[square.as_slice(), &square[..1]].concat(), band);
        // On the last side, running from (0, 1) back to the origin, 3.05 m
        // round before stretching.
        let near = rims.nearest([0.02, 0.95, 0.0]).unwrap();
        assert!((near.closure - 3.9).abs() < 1e-5);
        assert!((near.distance - 0.02).abs() < 1e-6);
        assert!((near.along - 3.05 * 3.9 / 4.0).abs() < 1e-5);
        assert!(rims.nearest([0.5, 0.5, 0.0]).is_none());
    }

    #[test]
    fn chains_close_loops_and_follow_open_runs_from_their_start() {
        let closed = chains(&[(0, 1), (1, 2), (2, 0)]);
        assert_eq!(closed, [vec![0, 1, 2, 0]]);
        let open = chains(&[(5, 6), (4, 5)]);
        assert_eq!(open, [vec![4, 5, 6]]);
    }
}
