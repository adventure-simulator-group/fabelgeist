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
//!
//! The hair crate turns a card to face away from the head's centre, right
//! for a card hanging beside the head; a card lying on a shoulder would
//! stand on its edge there, a fin. Near the skin each row is turned to lie
//! flat on it instead, fully within [`LIE_FLAT_REACH`] of touching.

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
/// Rows either side a row's lift spreads to, tapering: near the skin rows
/// lie a centimetre apart, so a lift bends a strip over a few centimetres.
const LIFT_SPREAD: usize = 4;
/// How near the skin a row turns to lie flat on it, metres, fully at the
/// skin and not at all this far off.
const LIE_FLAT_REACH: f32 = 0.03;
/// Shorter vectors than this, metres, have no direction to turn by.
const NO_DIRECTION: f32 = 1e-6;

/// One edge point of a strip's row.
#[derive(Clone, Copy)]
pub(super) struct StripPoint {
    pub(super) position: Vec3,
    pub(super) normal: Vec3,
    /// Along the strands, root to tip.
    pub(super) tangent: Vec3,
    pub(super) texcoord: [f32; 2],
    /// Baked sky visibility, 0 to 1.
    pub(super) occlusion: f32,
    /// The strip's own random value, 0 to 1.
    pub(super) random: f32,
}

impl StripPoint {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            normal: self.normal.lerp(other.normal, t).normalize(),
            tangent: self.tangent.lerp(other.tangent, t).normalize(),
            texcoord: std::array::from_fn(|k| {
                self.texcoord[k] + (other.texcoord[k] - self.texcoord[k]) * t
            }),
            occlusion: self.occlusion + (other.occlusion - self.occlusion) * t,
            random: self.random,
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
            lie_flat(row, body);
        }
        let needs: Vec<Vec3> = rows.iter().map(|row| lift(row, body)).collect();
        for (row, lift) in rows.iter_mut().zip(spread(&needs)) {
            for point in row {
                point.position += lift;
            }
        }
        self.0 = rows;
    }
}

/// Turn `row` about its middle, near `body`, to lie flat on the skin: its
/// width square to both the strands and the skin's normal.
fn lie_flat(row: &mut Row, body: &impl Collider) {
    let centre = middle(row);
    let (distance, out) = body.query(centre);
    let t = (1.0 - distance / LIE_FLAT_REACH).clamp(0.0, 1.0);
    let weight = t * t * (3.0 - 2.0 * t);
    let across = row[1].position - row[0].position;
    let width = across.length();
    let flat = row[0].tangent.cross(out);
    if weight <= 0.0 || width < NO_DIRECTION || flat.length() < NO_DIRECTION {
        return;
    }
    let side = across * (1.0 / width);
    let flat = flat.normalize();
    let flat = if flat.dot(side) < 0.0 {
        flat * -1.0
    } else {
        flat
    };
    let turned = side.lerp(flat, weight).normalize();
    let face = turned.cross(row[0].tangent).normalize();
    let face = if face.dot(row[0].normal) < 0.0 {
        face * -1.0
    } else {
        face
    };
    let half = turned * (0.5 * width);
    row[0].position = centre - half;
    row[1].position = centre + half;
    for point in row {
        point.normal = face;
    }
}

/// How far `row` must move off `body`: whole, as far as its deepest point
/// must -- moving its points apart would twist a thin ribbon into a wide
/// shard -- and by its clearance and its sag across its width.
fn lift(row: &Row, body: &impl Collider) -> Vec3 {
    let width = (row[1].position - row[0].position).length();
    let clearance = CLEARANCE + width * width / (8.0 * TIGHTEST_CURVE);
    let deepest = [middle(row), row[0].position, row[1].position]
        .into_iter()
        .map(|point| (body.distance(point), point))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let Some((_, point)) = deepest.filter(|(depth, _)| *depth < clearance) else {
        return Vec3::new(0.0, 0.0, 0.0);
    };
    push_out(body, point, clearance) - point
}

/// Each row's lift, spread to its neighbours tapering over [`LIFT_SPREAD`]
/// rows: every row moves as far as the most any row near it asks, scaled by
/// how near, so a strip bends smoothly off the skin rather than in a
/// sawtooth of lifted and unlifted rows.
fn spread(needs: &[Vec3]) -> Vec<Vec3> {
    (0..needs.len())
        .map(|row| {
            let first = row.saturating_sub(LIFT_SPREAD);
            let last = (row + LIFT_SPREAD).min(needs.len() - 1);
            (first..=last)
                .map(|other| {
                    let taper = 1.0 - row.abs_diff(other) as f32 / (LIFT_SPREAD + 1) as f32;
                    needs[other] * taper
                })
                .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
                .unwrap_or(needs[row])
        })
        .collect()
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
            randoms: Vec::with_capacity(points),
            tangents: Vec::with_capacity(points),
            faces: Vec::new(),
        };
        for Strip(rows) in strips {
            let base = geometry.positions.len() as u32;
            for point in rows.iter().flatten() {
                geometry.positions.push(point.position.to_array());
                geometry.normals.push(point.normal.to_array());
                geometry.texcoords.push(point.texcoord);
                geometry.occlusion.push(point.occlusion);
                geometry.randoms.push(point.random);
                geometry.tangents.push(point.tangent.to_array());
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
