//! `Shrinkwrap Hair Curves`: out of the body.
//!
//! Every point that has sunk below the surface -- or lies closer to it than
//! `Offset Distance` -- is moved to the surface plus that offset along the
//! outward direction: fully when below, fading to `Above Surface` of the
//! way at the offset distance. The moves are blurred along each strand
//! (`Smoothing Steps`) so that a strand bends around the body rather than
//! kinking at it, and mixed in by `Factor`. `Lock Roots` keeps roots on the
//! scalp. DiffLocks applies it "at various stages in the pipeline"; after
//! the noise that pushes strands into the head, and after the drape.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::collider::push_out;
use crate::ops::{Context, restore_lengths};
use crate::strands::{Strands, blur};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shrinkwrap {
    pub factor: f32,
    /// Metres off the surface.
    pub offset: f32,
    /// How much of the way points at the offset distance still move.
    pub above_surface: f32,
    pub smoothing: usize,
    pub lock_roots: bool,
    pub preserve_length: bool,
}

impl Default for Shrinkwrap {
    fn default() -> Self {
        Self {
            factor: 1.0,
            offset: 0.002,
            above_surface: 0.5,
            smoothing: 2,
            lock_roots: true,
            preserve_length: false,
        }
    }
}

impl Shrinkwrap {
    pub fn apply(&self, strands: &mut Strands, context: &Context) {
        let points = strands.points;
        let collider = context.collider;
        let offset = self.offset.max(1e-10);
        let reference = self.preserve_length.then(|| strands.positions.clone());
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new()),
                |(moves, weights), (s, strand)| {
                    moves.clear();
                    weights.clear();
                    let mut any = false;
                    for (j, &p) in strand.iter().enumerate() {
                        let locked = j == 0 && self.lock_roots;
                        let (distance, out) = collider.query(p);
                        let weight = if locked || distance >= offset {
                            0.0
                        } else if distance < 0.0 {
                            1.0
                        } else {
                            1.0 + (self.above_surface - 1.0) * (distance / offset)
                        };
                        let target = p + out * (offset - distance);
                        moves.push((target - p) * weight);
                        weights.push(if locked { 0.0 } else { 1.0 });
                        any |= weight > 0.0;
                    }
                    if !any {
                        return;
                    }
                    if self.smoothing > 0 {
                        blur(moves, self.smoothing, weights);
                    }
                    for (p, m) in strand.iter_mut().zip(moves.iter()) {
                        *p += *m * self.factor;
                    }
                    if let Some(reference) = &reference {
                        restore_lengths(strand, &reference[s * points..(s + 1) * points], 1.0);
                    }
                    // Blurring spreads a deep point's push over its
                    // neighbours, and restoring lengths can dip a point back
                    // in: whatever is still below the surface goes straight
                    // out, unsmoothed.
                    if self.factor >= 1.0 {
                        for p in strand.iter_mut().skip(1) {
                            *p = push_out(collider, *p, self.offset * 0.5);
                        }
                    }
                },
            );
    }
}
