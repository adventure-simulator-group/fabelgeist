//! The scalp under the hair, as a fine cap fitted to the character's skull.
//!
//! The hair crate shades its own head where roots grow and where hair lies
//! over it, so no bare skin shows between strands; its head is cut in
//! four-millimetre cells, fine enough to keep a parting narrow. The
//! character's body is far coarser over the skull -- a few dozen vertices --
//! and shading it per vertex smears a parting across centimetres of bare
//! skin. The bust's own scalp, at the hair crate's resolution, is carried
//! onto the character instead and laid just over its skin, to be shaded
//! there. Towards the scalp's edge the root tint fades out, so the cap
//! meets the body without a seam.

use std::collections::{HashMap, VecDeque};

use fabelgeist_hair::collider::Collider;
use fabelgeist_hair::cover::ScalpCover;
use fabelgeist_hair::{MeshCollider, Surface};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::{HeadPlacement, SkinCover};

/// How far over the skin the cap lies, metres: under the hair's roots,
/// clear of the skin's own triangles.
const CAP_LIFT: f32 = 0.0004;
/// Rows of cells from the scalp's edge over which the root tint fades in:
/// one, so the hairline is soft but no band of bare skin runs along it.
const EDGE_FADE_ROWS: f32 = 1.0;

/// The scalp, fitted to a character's skull, and how much hair covers each
/// of its points.
pub struct ScalpCap {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub cover: Vec<SkinCover>,
}

impl ScalpCap {
    /// The bust's `scalp`, carried onto a character by `placement` and laid
    /// over its `body`, covered as `cover` says.
    pub(super) fn fit(
        scalp: &Surface,
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
        let positions: Vec<Vec3> = laid.iter().map(|&(position, _)| position).collect();
        let normals: Vec<Vec3> = laid.iter().map(|&(_, normal)| normal).collect();
        let fade = edge_fade(positions.len(), &scalp.triangles);
        let cover = cover
            .at(&positions, &normals)
            .into_iter()
            .zip(fade)
            .map(|([roots, over], fade)| SkinCover {
                roots: roots * fade,
                over,
            })
            .collect();
        Self {
            positions: positions.iter().map(|point| point.to_array()).collect(),
            normals: normals.iter().map(|normal| normal.to_array()).collect(),
            faces: scalp.triangles.clone(),
            cover,
        }
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
