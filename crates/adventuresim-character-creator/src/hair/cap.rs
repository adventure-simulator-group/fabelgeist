//! The scalp under the hair, as a fine cap fitted to the character's skull.
//!
//! Between strands and cards the skin would show bare, so the scalp is
//! painted the hair's colour. The character's body is far too coarse over
//! the skull for that -- a few dozen vertices -- and painting it per vertex
//! smears a parting across centimetres. The bust's own scalp, cut in the
//! hair crate's four-millimetre cells, is carried onto the character
//! instead and laid just over its skin. Over the body's coarse facets a cap
//! triangle laid by its corners alone would dip under the skin wherever it
//! spans a ridge or a corner of the body, and the bare skin would show
//! through it in specks along the body's edges; such triangles are lifted
//! until every point of them clears the skin.
//!
//! The bust's scalp is cut from its cells, so its edge runs in steps a cell
//! high. The cap fades out to nothing at its edge: above the hairline from
//! above the highest of those steps, so the hairline is the hair crate's
//! smooth curve, and over a row of cells along the rest of its edge, as at
//! the nape.

use std::collections::{HashMap, VecDeque};

use fabelgeist_hair::collider::Collider;
use fabelgeist_hair::{Head, MeshCollider, Surface};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::HeadPlacement;
use super::scalp_map::ScalpMap;

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

/// The scalp, fitted to a character's skull, and how much each of its
/// points shows over the skin beneath.
pub struct ScalpCap {
    pub positions: Vec<[f32; 3]>,
    /// The scalp's own smooth normals: the body's facets would show through
    /// the strands painted on it as flat patches.
    pub normals: Vec<[f32; 3]>,
    /// Where each point lies in the scalp's texture.
    pub texcoords: Vec<[f32; 2]>,
    /// Along the texture's `u`, square to the normal, and as the fourth
    /// value which way its `v` runs: `v` runs along the normal crossed
    /// with the tangent, times that.
    pub tangents: Vec<[f32; 4]>,
    pub faces: Vec<[u32; 3]>,
    /// 0 to 1: none at the cap's edge, all a little way in.
    pub opacity: Vec<f32>,
}

impl ScalpCap {
    /// The scalp of the bust `head`, mapped onto its texture by `map`,
    /// carried onto a character by `placement` and laid over its `body`.
    pub(super) fn fit(
        scalp: &Surface,
        head: &Head,
        map: &ScalpMap,
        placement: &HeadPlacement,
        body: &MeshCollider,
    ) -> Self {
        let mut positions: Vec<Vec3> = scalp
            .positions
            .par_iter()
            .map(|&point| {
                let carried = Vec3::from_array(placement.point(point.to_array()));
                let (distance, out) = body.query(carried);
                carried - out * (distance - CAP_LIFT)
            })
            .collect();
        clear_skin(&mut positions, &scalp.triangles, body);
        let normals: Vec<Vec3> = scalp
            .normals
            .iter()
            .map(|normal| Vec3::from_array(placement.normal(normal.to_array())))
            .collect();
        // The map reaches past the scalp's lowest point, so every point of
        // the scalp lies within it.
        let texcoords: Vec<[f32; 2]> = scalp
            .positions
            .iter()
            .map(|&point| map.uv(point).unwrap_or_default())
            .collect();
        let tangents = tangents(&positions, &normals, &texcoords, &scalp.triangles);
        let opacity = edge_fade(positions.len(), &scalp.triangles)
            .into_iter()
            .zip(&scalp.positions)
            .map(|(edge, &point)| {
                let above = head.above_hairline(point) - HAIRLINE_STEPS;
                let t = (above / HAIRLINE_SOFTNESS).clamp(0.0, 1.0);
                edge.min(t * t * (3.0 - 2.0 * t))
            })
            .collect();
        Self {
            positions: positions.iter().map(|point| point.to_array()).collect(),
            normals: normals.iter().map(|normal| normal.to_array()).collect(),
            texcoords,
            tangents,
            faces: scalp.triangles.clone(),
            opacity,
        }
    }
}

/// Each point's direction along the texture's `u` over `triangles`, square
/// to its `normal`, and the sign that turns the normal crossed with it into
/// the direction along `v`.
fn tangents(
    positions: &[Vec3],
    normals: &[Vec3],
    texcoords: &[[f32; 2]],
    triangles: &[[u32; 3]],
) -> Vec<[f32; 4]> {
    let zero = Vec3::new(0.0, 0.0, 0.0);
    let mut along_u = vec![zero; positions.len()];
    let mut along_v = vec![zero; positions.len()];
    for &[a, b, c] in triangles {
        let [a, b, c] = [a, b, c].map(|corner| corner as usize);
        let (first, second) = (positions[b] - positions[a], positions[c] - positions[a]);
        let [du1, dv1] = [
            texcoords[b][0] - texcoords[a][0],
            texcoords[b][1] - texcoords[a][1],
        ];
        let [du2, dv2] = [
            texcoords[c][0] - texcoords[a][0],
            texcoords[c][1] - texcoords[a][1],
        ];
        let area = du1 * dv2 - du2 * dv1;
        if area.abs() <= f32::EPSILON {
            continue;
        }
        let u = (first * dv2 - second * dv1) * (1.0 / area);
        let v = (second * du1 - first * du2) * (1.0 / area);
        for corner in [a, b, c] {
            along_u[corner] += u;
            along_v[corner] += v;
        }
    }
    normals
        .iter()
        .zip(along_u.iter().zip(&along_v))
        .map(|(&normal, (&u, &v))| {
            let square = u - normal * normal.dot(u);
            let length = square.length();
            if length <= f32::EPSILON {
                return [1.0, 0.0, 0.0, 1.0];
            }
            let tangent = square * (1.0 / length);
            let handedness = if normal.cross(tangent).dot(v) < 0.0 {
                -1.0
            } else {
                1.0
            };
            [tangent.x, tangent.y, tangent.z, handedness]
        })
        .collect()
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
