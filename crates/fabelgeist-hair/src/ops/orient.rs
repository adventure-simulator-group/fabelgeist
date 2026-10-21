//! Each strand's root turned the way its own follicle points.
//!
//! Not one of DiffLocks' node groups. Guides are a few centimetres apart
//! and interpolation blends their shapes, so a growth pattern finer than
//! that -- the turn of a crown whorl, a cowlick -- is lost between them, and
//! every strand near a guide leaves the scalp at its guide's angle. This
//! turns the first `memory` metres of every strand, about its root's normal,
//! from the way it leaves to the way the growth pattern points there (and
//! each strand a little off that, and up or down, as follicles are),
//! fading out along the strand so that past the memory it lies as it did.
//! Where the turn is large -- hair combed against its growth -- the root
//! lifts, as a cowlick does.

use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::follicles::GrowthField;
use crate::rng;
use crate::strands::{Strands, rotate, safe_normalize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Orient {
    /// Which way the follicles point.
    pub field: GrowthField,
    /// How far along a strand its follicle's direction holds, metres.
    pub memory: f32,
    /// How far each strand's direction and angle stray, radians.
    pub direction_spread: f32,
    pub angle_spread: f32,
    pub seed: i32,
}

impl Orient {
    pub fn apply(&self, strands: &mut Strands) {
        if self.memory <= 0.0 {
            return;
        }
        let points = strands.points;
        let (ids, normals) = (&strands.id, &strands.root_normal);
        let seed = rng::seed(self.seed, 0x0F0C);
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each(|(s, strand)| {
                let root = strand[0];
                let n = safe_normalize(normals[s], Vec3::up());
                // The way the strand leaves: its first few millimetres, in
                // the scalp's plane.
                let Some(ahead) = strand.iter().find(|p| (**p - root).length() > 0.002) else {
                    return;
                };
                let leaves = *ahead - root;
                let flat = leaves - n * leaves.dot(n);
                let grows = self.field.direction(root, n);
                // Strands standing straight up have no way to turn from.
                let turn = if flat.length() > 0.25 * leaves.length() {
                    let flat = flat.normalize();
                    n.dot(flat.cross(grows)).atan2(flat.dot(grows))
                } else {
                    0.0
                } + self.direction_spread * rng::signed(ids[s], seed);
                let tilt = self.angle_spread * rng::signed(ids[s], seed ^ 0x7117);
                let tilt_axis = safe_normalize(grows.cross(n), crate::strands::orthogonal(n));
                // A root turned right round stands up.
                let lift = (1.0 - turn.cos()) * 0.5 * self.memory * 0.35;
                let mut along = 0.0;
                let mut previous = root;
                for p in strand.iter_mut().skip(1) {
                    along += (*p - previous).length();
                    previous = *p;
                    let share = 1.0 - crate::ops::smoothstep(0.0, self.memory, along);
                    if share <= 0.0 {
                        break;
                    }
                    let mut offset = rotate(*p - root, n, turn * share);
                    offset = rotate(offset, tilt_axis, -tilt * share);
                    let rise = lift * crate::ops::smoothstep(0.0, self.memory, along) * share * 4.0;
                    *p = root + offset + n * rise;
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::follicles::{Follicles, Growth};
    use crate::head::Head;
    use crate::strands::Attachment;

    #[test]
    fn a_short_strand_turns_to_its_follicle_and_a_long_one_keeps_its_course() {
        let head = Head::mannequin();
        let field = Follicles {
            growth: Growth::Forward,
            ..Default::default()
        }
        .field(&head);
        let root = head.crown();
        let n = head.gradient(root);
        // Combed back along the top, 5 cm.
        let mut strands = Strands::new(26);
        let positions: Vec<Vec3> = (0..26)
            .map(|j| root + Vec3::new(0.0, 0.001, -0.002 * j as f32))
            .collect();
        strands.push(
            &positions,
            Attachment {
                normal: n,
                ..Default::default()
            },
        );
        Orient {
            field,
            memory: 0.01,
            direction_spread: 0.0,
            angle_spread: 0.0,
            seed: 1,
        }
        .apply(&mut strands);
        let strand = strands.strand(0);
        // It leaves forward, as the follicle grows; past the memory it is
        // where it was.
        assert!((strand[1] - strand[0]).z > 0.0, "{}", strand[1] - strand[0]);
        assert_eq!(strand[25], positions[25]);
    }
}
