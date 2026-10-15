//! Hair strips kept off the skin.
//!
//! Cards and strand ribbons are both strips: rows of two points, one either
//! side of the strip's middle, joined by straight segments. The hair crate
//! keeps a strand's points and a card's spine outside the body, but over a
//! rounded shoulder or skull a straight segment between two rows just
//! outside the skin passes under it in the middle -- the longer the segment,
//! the deeper: the coarse levels most. Near the skin, rows are added along
//! each segment longer than [`LONGEST_SEGMENT`], and every row there is
//! moved off the skin by [`CLEARANCE`] and by as much as the strip sags
//! across its width over the tightest curve hair lies on.

use fabelgeist_hair::collider::{Collider, push_out};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::HairGeometry;

/// How far off the skin hair is kept, metres: as close as the hair crate
/// keeps a root, over the scalp cap, so roots leave the scalp rather than
/// hover over it.
const CLEARANCE: f32 = 0.0006;
/// The longest segment left near the skin, metres: a straight segment this
/// long over the tightest curve dips under its ends by a quarter of a
/// millimetre, inside the clearance.
const LONGEST_SEGMENT: f32 = 0.01;
/// The tightest curve hair lies over, metres of radius: the neck's.
const TIGHTEST_CURVE: f32 = 0.05;

/// One edge point of a strip's row.
#[derive(Clone, Copy)]
pub(super) struct StripPoint {
    pub(super) position: Vec3,
    pub(super) normal: Vec3,
    pub(super) texcoord: [f32; 2],
    /// Baked sky visibility, 0 to 1.
    pub(super) occlusion: f32,
}

impl StripPoint {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            normal: self.normal.lerp(other.normal, t).normalize(),
            texcoord: std::array::from_fn(|k| {
                self.texcoord[k] + (other.texcoord[k] - self.texcoord[k]) * t
            }),
            occlusion: self.occlusion + (other.occlusion - self.occlusion) * t,
        }
    }
}

/// One row across a strip: its two edge points.
pub(super) type Row = [StripPoint; 2];

/// A card or a strand's ribbon: its rows, root to tip.
pub(super) struct Strip(pub(super) Vec<Row>);

impl Strip {
    /// Keep this strip off `body`: add rows along its long segments near
    /// the skin, then move its rows near the skin out.
    fn wrap(&mut self, body: &impl Collider) {
        let distance = |row: &Row| body.distance(middle(row));
        let mut rows = Vec::with_capacity(self.0.len());
        let mut previous: Option<(Row, f32)> = None;
        for &row in &self.0 {
            let here = distance(&row);
            if let Some((last, there)) = previous {
                let length = (middle(&row) - middle(&last)).length();
                // A straight segment dips under its ends by less than half
                // its length.
                let near = here.min(there) < length * 0.5;
                if near && length > LONGEST_SEGMENT {
                    let pieces = (length / LONGEST_SEGMENT).ceil() as usize;
                    for piece in 1..pieces {
                        let t = piece as f32 / pieces as f32;
                        rows.push([last[0].lerp(row[0], t), last[1].lerp(row[1], t)]);
                    }
                }
            }
            rows.push(row);
            previous = Some((row, here));
        }
        for row in &mut rows {
            let width = (row[1].position - row[0].position).length();
            let clearance = CLEARANCE + width * width / (8.0 * TIGHTEST_CURVE);
            // The row moves whole, as far as its deepest point must: moving
            // its points apart would twist a thin ribbon into a wide shard.
            let deepest = [middle(row), row[0].position, row[1].position]
                .into_iter()
                .map(|point| (body.distance(point), point))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((_, point)) = deepest.filter(|(depth, _)| *depth < clearance) else {
                continue;
            };
            let lift = push_out(body, point, clearance) - point;
            for point in row {
                point.position += lift;
            }
        }
        self.0 = rows;
    }
}

fn middle([left, right]: &Row) -> Vec3 {
    (left.position + right.position) * 0.5
}

impl HairGeometry {
    /// `strips` kept off `body`, as triangles.
    pub(super) fn from_strips(mut strips: Vec<Strip>, body: &impl Collider) -> Self {
        strips.par_iter_mut().for_each(|strip| strip.wrap(body));
        let points: usize = strips.iter().map(|strip| 2 * strip.0.len()).sum();
        let mut geometry = HairGeometry {
            positions: Vec::with_capacity(points),
            normals: Vec::with_capacity(points),
            texcoords: Vec::with_capacity(points),
            occlusion: Vec::with_capacity(points),
            faces: Vec::new(),
        };
        for Strip(rows) in strips {
            let base = geometry.positions.len() as u32;
            for point in rows.iter().flatten() {
                geometry.positions.push(point.position.to_array());
                geometry.normals.push(point.normal.to_array());
                geometry.texcoords.push(point.texcoord);
                geometry.occlusion.push(point.occlusion);
            }
            for segment in 0..rows.len().saturating_sub(1) as u32 {
                let [a, b] = [base + 2 * segment, base + 2 * segment + 1];
                let [c, d] = [a + 2, b + 2];
                geometry.faces.extend([[a, c, b], [b, c, d]]);
            }
        }
        geometry
    }
}
