//! Hair draped on the character's own body.
//!
//! A hairstyle is groomed, and draped, on the hair crate's mannequin bust.
//! Carried onto a character whose shoulders stand higher or wider, long hair
//! would hang through them. What a body does to a strand is how its
//! equilibrium under gravity against the body differs from its equilibrium
//! against nothing. That is found for each strand on the bust, as groomed,
//! and on the character, as carried, and the character's minus the bust's is
//! added to it: where the two bodies hold the hair alike, or a bun or a quiff
//! touches neither, it is nothing and the style keeps its shape; where the
//! character's shoulders stand in the way, it is how the hair falls over
//! them. A body acts on a tight coil -- a bun's, a braid's -- unsteadily, so
//! the drape takes hold only below the head, fading in from the ears down to
//! the jaw, where the bust and the character really differ: at the neck and
//! the shoulders. And the solve knows nothing of what ties a lock: a
//! ponytail's tail, resting against a neck a little thicker than the bust's,
//! would slide off it strand by strand, round both sides. So a strand is
//! draped only as far as it sinks deep into the character's body as
//! carried -- long hair through a shoulder fully, a tail resting against the
//! neck not at all, the shrinkwrap after pushing it out. Draping every strand twice would take long, so a few thousand
//! guides spread over the head are draped, and every strand follows the
//! guide whose root is nearest its own.

use fabelgeist_hair::collider::{Collider, NoCollider};
use fabelgeist_hair::ops::{STANDARD_GRAVITY, Settle, restore_lengths};
use fabelgeist_hair::{Head, MeshCollider, Strands};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::HeadPlacement;

/// Strands draped as guides for the rest.
const GUIDES: usize = 2048;
/// Guides each strand follows: its nearest only. Over a shoulder the
/// guides either side of a strand may fall in front and behind, and a blend
/// of them would run through the shoulder; following one parts the hair
/// cleanly, as hair parts over a shoulder.
const NEIGHBOURS: usize = 1;
/// Root distance below which a guide counts as the strand's own, metres.
const SAME_ROOT: f32 = 1e-5;
/// How deep inside the character's body, metres, a strand starts to be
/// draped, and how deep it is draped fully. A tail resting a couple of
/// centimetres into a thicker neck is left to the shrinkwrap, which pushes
/// it straight out; long hair through a shoulder runs several centimetres
/// deep.
const SINK_START: f32 = 0.03;
const SINK_FULL: f32 = 0.05;

/// How far below the cranium's centre the drape starts to take hold, and
/// where it takes hold fully, in the cranium's vertical radii.
const DRAPE_START: f32 = 0.5;
const DRAPE_FULL: f32 = 1.0;

/// Where the drape takes hold: heights in model space, metres.
#[derive(Clone, Copy)]
struct Hold {
    /// Above this, none of it.
    start: f32,
    /// Below this, all of it.
    full: f32,
}

impl Hold {
    /// How much of the drape moves a point at `height`, 0 to 1, smoothly.
    fn at(self, height: f32) -> f32 {
        let t = ((self.start - height) / (self.start - self.full)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// A guide's root, and how much more the character's body moves each of its
/// points than the bust's.
struct Guide {
    root: Vec3,
    moves: Vec<Vec3>,
}

/// Drape `strands`, carried by `placement` from `groomed` on the `bust`, on
/// the character's `body`.
pub(super) fn drape(
    strands: &mut Strands,
    groomed: &Strands,
    bust: &Head,
    body: &MeshCollider,
    placement: &HeadPlacement,
) {
    let points = strands.points;
    let count = strands.positions.len() / points.max(1);
    if count == 0 {
        return;
    }
    let settle = Settle::default();
    let load = Vec3::new(0.0, -STANDARD_GRAVITY, 0.0);
    let carry = |point: Vec3| Vec3::from_array(placement.point(point.to_array()));
    let cranium = &bust.cranium;
    let height_below =
        |radii: f32| carry(cranium.center - Vec3::new(0.0, cranium.radii.y * radii, 0.0)).y;
    let hold = Hold {
        start: height_below(DRAPE_START),
        full: height_below(DRAPE_FULL),
    };
    let step = (count / GUIDES).max(1);
    let guides: Vec<Guide> = (0..count)
        .step_by(step)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|strand| {
            let placed = &strands.positions[strand * points..][..points];
            let original = &groomed.positions[strand * points..][..points];
            let held = |strand: &[Vec3], by: &dyn Collider| {
                let against = settle.equilibrium(strand, load, by);
                let free = settle.equilibrium(strand, load, &NoCollider);
                against
                    .into_iter()
                    .zip(free)
                    .map(|(held, free)| held - free)
            };
            let moves = held(placed, body)
                .zip(held(original, bust))
                .zip(original)
                .map(|((by_body, by_bust), &point)| {
                    // The bust's push, carried as the strand was.
                    let pushed = carry(point + by_bust) - carry(point);
                    by_body - pushed
                })
                .collect();
            Guide {
                root: placed[0],
                moves,
            }
        })
        .collect();
    strands
        .positions
        .par_chunks_mut(points)
        .for_each(|strand| follow(strand, &guides, hold, body));
}

/// Move `strand` as the guides nearest its root move, as far as the drape
/// `hold`s at each point and as far as the strand sinks into `body`,
/// keeping its length.
fn follow(strand: &mut [Vec3], guides: &[Guide], hold: Hold, body: &MeshCollider) {
    let sunk = strand
        .iter()
        .map(|&point| -body.distance(point))
        .fold(0.0_f32, f32::max);
    let t = ((sunk - SINK_START) / (SINK_FULL - SINK_START)).clamp(0.0, 1.0);
    let sinking = t * t * (3.0 - 2.0 * t);
    if sinking <= 0.0 {
        return;
    }
    let root = strand[0];
    let mut nearest: Vec<(f32, &Guide)> = guides
        .iter()
        .map(|guide| ((guide.root - root).length_squared(), guide))
        .collect();
    let neighbours = NEIGHBOURS.min(nearest.len());
    nearest.select_nth_unstable_by(neighbours - 1, |a, b| a.0.total_cmp(&b.0));
    nearest.truncate(neighbours);
    let weights: Vec<f32> = nearest
        .iter()
        .map(|&(squared, _)| 1.0 / squared.sqrt().max(SAME_ROOT))
        .collect();
    let total: f32 = weights.iter().sum();
    let before = strand.to_vec();
    for (index, point) in strand.iter_mut().enumerate().skip(1) {
        let moved = nearest
            .iter()
            .zip(&weights)
            .fold(Vec3::new(0.0, 0.0, 0.0), |sum, (&(_, guide), &weight)| {
                sum + guide.moves[index] * weight
            });
        *point += moved * (sinking * hold.at(before[index].y) / total);
    }
    restore_lengths(strand, &before, 1.0);
}
