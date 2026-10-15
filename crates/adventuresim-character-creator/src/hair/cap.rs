//! The scalp under the hair, as a fine cap fitted to the character's skull.
//!
//! The hair crate shades its own head where roots grow and where hair lies
//! over it, so no bare skin shows between strands; its head is cut in
//! four-millimetre cells, fine enough to keep a parting narrow. The
//! character's body is far coarser over the skull -- a few dozen vertices --
//! and shading it per vertex smears a parting across centimetres of bare
//! skin. The bust's own scalp, at the hair crate's resolution, is carried
//! onto the character instead and laid just over its skin, to be shaded
//! there. Over the body's coarse facets a cap triangle laid by its corners
//! alone would dip under the skin wherever it spans a ridge or a corner of
//! the body, and the bare skin would show through it in specks along the
//! body's edges; such triangles are lifted until every point of them clears
//! the skin.
//!
//! The cap is lit by the scalp's smooth normals, the body by its own, so
//! even bare the cap would show its outline on the skin; and the bust's
//! scalp is cut from its cells, so that outline runs in steps a cell high.
//! The cap fades out instead, to nothing at its edge: above the hairline
//! from above the highest of those steps, so the hairline is the hair
//! crate's smooth curve, and over a row of cells along the rest of its
//! edge, as at the nape.

use std::collections::{HashMap, VecDeque};

use fabelgeist_hair::collider::Collider;
use fabelgeist_hair::cover::ScalpCover;
use fabelgeist_hair::{Head, MeshCollider, Surface};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::{HeadPlacement, SkinCover};

/// How far over the skin the cap lies, metres: under the hair's roots,
/// clear of the skin's own triangles.
const CAP_LIFT: f32 = 0.0004;
/// Rows of cells from the scalp's edge over which the cap fades in: one, so
/// no band of bare skin runs along it where the edge is not the hairline,
/// as at the nape.
const EDGE_FADE_ROWS: f32 = 1.0;
/// How far above the hairline the cap starts to show, metres: above the
/// scalp's stepped edge, which lies up to a cell above it.
const HAIRLINE_STEPS: f32 = 0.005;
/// Over how far above that the cap fades in, metres: a little more than
/// the follicles thin out over.
const HAIRLINE_SOFTNESS: f32 = 0.006;
/// How finely each cap triangle is sampled for dipping under the skin:
/// points at this many steps along each edge, and inside.
const DIP_STEPS: usize = 4;
/// The most passes made lifting triangles off the skin: a lift moves a
/// triangle's neighbours too, so it settles in a few.
const DIP_PASSES: usize = 4;
/// Passes of averaging the cover over the cap's neighbours, as the hair
/// crate smooths its own head's: measured a point at a time, it is grainy
/// at the cap's resolution and steps along the hairline.
const COVER_SMOOTHING: usize = 3;

/// The scalp, fitted to a character's skull, how much hair covers each of
/// its points, and how much each shows over the skin beneath.
pub struct ScalpCap {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub cover: Vec<SkinCover>,
    /// 0 to 1: none at the cap's edge, all a little way in.
    pub opacity: Vec<f32>,
}

impl ScalpCap {
    /// The scalp of the bust `head`, carried onto a character by
    /// `placement` and laid over its `body`, covered as `cover` says.
    pub(super) fn fit(
        scalp: &Surface,
        head: &Head,
        placement: &HeadPlacement,
        body: &MeshCollider,
        cover: &ScalpCover,
    ) -> Self {
        // Laid onto the skin, but shaded with the scalp's own smooth normals:
        // the body's facets would show through as flat patches.
        let laid: Vec<(Vec3, Vec3)> = scalp
            .positions
            .par_iter()
            .zip(&scalp.normals)
            .map(|(&point, &normal)| {
                let carried = Vec3::from_array(placement.point(point.to_array()));
                let (distance, out) = body.query(carried);
                let smooth = Vec3::from_array(placement.normal(normal.to_array()));
                (carried - out * (distance - CAP_LIFT), smooth)
            })
            .collect();
        let mut positions: Vec<Vec3> = laid.iter().map(|&(position, _)| position).collect();
        let normals: Vec<Vec3> = laid.iter().map(|&(_, normal)| normal).collect();
        clear_skin(&mut positions, &scalp.triangles, body);
        let opacity = edge_fade(positions.len(), &scalp.triangles)
            .into_iter()
            .zip(&scalp.positions)
            .map(|(edge, &point)| {
                let above = head.above_hairline(point) - HAIRLINE_STEPS;
                let t = (above / HAIRLINE_SOFTNESS).clamp(0.0, 1.0);
                edge.min(t * t * (3.0 - 2.0 * t))
            })
            .collect();
        let measured = cover.at(&positions, &normals);
        let cover = smooth(measured, &scalp.triangles)
            .into_iter()
            .map(|[roots, over]| SkinCover { roots, over })
            .collect();
        Self {
            positions: positions.iter().map(|point| point.to_array()).collect(),
            normals: normals.iter().map(|normal| normal.to_array()).collect(),
            faces: scalp.triangles.clone(),
            cover,
            opacity,
        }
    }
}

/// Lift every triangle of the cap whose `positions` dip under `body`'s skin
/// between its corners, until each point of it lies [`CAP_LIFT`] off the
/// skin.
fn clear_skin(positions: &mut [Vec3], triangles: &[[u32; 3]], body: &MeshCollider) {
    let samples: Vec<[f32; 3]> = (0..=DIP_STEPS)
        .flat_map(|i| (0..=DIP_STEPS - i).map(move |j| [i, j, DIP_STEPS - i - j]))
        .filter(|steps| steps.iter().all(|&step| step < DIP_STEPS))
        .map(|steps| steps.map(|step| step as f32 / DIP_STEPS as f32))
        .collect();
    for _ in 0..DIP_PASSES {
        let dips: Vec<f32> = triangles
            .par_iter()
            .map(|&[a, b, c]| {
                let corners = [a, b, c].map(|corner| positions[corner as usize]);
                samples
                    .iter()
                    .map(|weights| {
                        let point = corners[0] * weights[0]
                            + corners[1] * weights[1]
                            + corners[2] * weights[2];
                        CAP_LIFT - body.distance(point)
                    })
                    .fold(0.0, f32::max)
            })
            .collect();
        let mut lifts = vec![0.0f32; positions.len()];
        for (&dip, triangle) in dips.iter().zip(triangles) {
            for &corner in triangle {
                let lift = &mut lifts[corner as usize];
                *lift = lift.max(dip);
            }
        }
        if lifts.iter().all(|&lift| lift <= 0.0) {
            break;
        }
        positions
            .par_iter_mut()
            .zip(&lifts)
            .filter(|(_, lift)| **lift > 0.0)
            .for_each(|(position, &lift)| {
                let (_, out) = body.query(*position);
                *position += out * lift;
            });
    }
}

/// `cover` averaged over each point's neighbours in `triangles`,
/// [`COVER_SMOOTHING`] times.
fn smooth(mut cover: Vec<[f32; 2]>, triangles: &[[u32; 3]]) -> Vec<[f32; 2]> {
    for _ in 0..COVER_SMOOTHING {
        let mut sums = vec![[0.0f32; 2]; cover.len()];
        let mut counts = vec![0.0f32; cover.len()];
        for triangle in triangles {
            for &a in triangle {
                for &b in triangle {
                    let (a, b) = (a as usize, b as usize);
                    sums[a][0] += cover[b][0];
                    sums[a][1] += cover[b][1];
                    counts[a] += 1.0;
                }
            }
        }
        for ((value, sum), count) in cover.iter_mut().zip(sums).zip(counts) {
            if count > 0.0 {
                *value = sum.map(|total| total / count);
            }
        }
    }
    cover
}

/// For each of `points` vertices, 0 on the edge of `triangles` rising to 1
/// [`EDGE_FADE_ROWS`] edges in.
fn edge_fade(points: usize, triangles: &[[u32; 3]]) -> Vec<f32> {
    let mut uses: HashMap<(u32, u32), u32> = HashMap::new();
    let mut neighbours = vec![Vec::new(); points];
    for &[a, b, c] in triangles {
        for (p, q) in [(a, b), (b, c), (c, a)] {
            *uses.entry((p.min(q), p.max(q))).or_default() += 1;
            neighbours[p as usize].push(q);
        }
    }
    // A breadth-first walk in from the edge, where an edge bounds one
    // triangle only.
    let mut rows = vec![u32::MAX; points];
    let mut queue = VecDeque::new();
    for (&(p, q), &count) in &uses {
        if count == 1 {
            for point in [p, q] {
                rows[point as usize] = 0;
                queue.push_back(point);
            }
        }
    }
    while let Some(point) = queue.pop_front() {
        let next = rows[point as usize] + 1;
        for &neighbour in &neighbours[point as usize] {
            if rows[neighbour as usize] > next {
                rows[neighbour as usize] = next;
                queue.push_back(neighbour);
            }
        }
    }
    rows.into_iter()
        .map(|row| (row as f32 / EDGE_FADE_ROWS).min(1.0))
        .collect()
}
