//! Length variation: `Trim Hair Curves`.
//!
//! A groomed style's strands are not all the length their guides were: a
//! haircut layers them, and loose ones are shorter. Each strand keeps the
//! first share of its arc length -- `length`, less up to `random` of it at
//! random, never more than `max_length` metres -- resampled over its same
//! number of points.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rng;
use crate::strands::{Strands, arc_lengths, resample_into};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trim {
    /// Share of each strand's length kept.
    pub length: f32,
    /// Up to this share of that is taken off at random.
    pub random: f32,
    /// No strand longer than this, metres (0: no limit).
    pub max_length: f32,
    pub seed: i32,
}

impl Default for Trim {
    fn default() -> Self {
        Self {
            length: 1.0,
            random: 0.2,
            max_length: 0.0,
            seed: 0,
        }
    }
}

impl Trim {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 77_221);
        let points = strands.points;
        let ids = &strands.id;
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(lengths, cut, scratch), (s, strand)| {
                    arc_lengths(strand, lengths);
                    let total = *lengths.last().unwrap_or(&0.0);
                    if total <= 0.0 {
                        return;
                    }
                    let mut keep = total
                        * (self.length * (1.0 - self.random * rng::unit((ids[s]).word(), seed)))
                            .clamp(0.02, 1.0);
                    if self.max_length > 0.0 {
                        keep = keep.min(self.max_length);
                    }
                    if keep >= total * 0.9999 {
                        return;
                    }
                    // The points up to `keep`, the last one cut to length.
                    cut.clear();
                    for j in 0..points {
                        if lengths[j] >= keep {
                            let previous = lengths[j - 1];
                            let t = (keep - previous) / (lengths[j] - previous).max(1e-12);
                            cut.push(strand[j - 1].lerp(strand[j], t));
                            break;
                        }
                        cut.push(strand[j]);
                    }
                    if cut.len() < 2 {
                        return;
                    }
                    resample_into(cut, scratch, strand);
                },
            );
    }
}
