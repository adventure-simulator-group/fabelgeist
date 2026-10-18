//! `Curl Hair Curves`: ringlets.
//!
//! Strands curl in locks: each follows its clump guide ([`GuideMap`]),
//! sampled at the same arc length, around a circle in the guide's frame
//! there. The angle grows with the strand's own arc length, `Frequency`
//! times three turns a metre, from a phase random per clump plus a little
//! random per strand (`Random Offset` turns at most). The radius runs from
//! `Factor Start` to `Factor End` of `Radius` along the strand, and the curl
//! eases in over the first `Curl Start` of it (smootherstep, then square
//! root). A strand moves to guide point plus circle, by `Factor` -- so a
//! clump curls together, a guide around its own path and its strands along
//! with it.
//!
//! Where it departs: the node group can subdivide strands first
//! (`Subdivision`) for points enough to draw the curl; strands here keep
//! their point count, which a groom sets for the tightest curl it wants.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ops::{GuideMap, parameters, smootherstep};
use crate::rng;
use crate::strands::Strands;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Curl {
    pub guide_distance: f32,
    pub guide_mask: f32,
    pub reuse_guide_map: bool,
    pub factor: f32,
    /// Share of the strand over which the curl eases in.
    pub curl_start: f32,
    /// Metres.
    pub radius: f32,
    pub factor_start: f32,
    pub factor_end: f32,
    /// Three turns a metre per unit.
    pub frequency: f32,
    /// Turns a strand's phase may differ from its clump's.
    pub random_offset: f32,
    pub seed: i32,
}

impl Default for Curl {
    fn default() -> Self {
        Self {
            guide_distance: 0.1,
            guide_mask: 1.0,
            reuse_guide_map: true,
            factor: 1.0,
            curl_start: 0.1,
            radius: 0.1,
            factor_start: 1.0,
            factor_end: 1.0,
            frequency: 1.0,
            random_offset: 0.25,
            seed: 0,
        }
    }
}

impl Curl {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 91_487);
        let map = GuideMap::get(
            strands,
            self.reuse_guide_map,
            self.guide_distance,
            self.guide_mask,
            seed,
        );
        let samplers = map.samplers(strands);
        let points = strands.points;
        let ids = &strands.id;
        let turns_per_metre = self.frequency * 3.0;

        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new()),
                |(lengths, factors), (s, strand)| {
                    let g = map.guide[s] as usize;
                    let Some(guide) = samplers[g].as_ref() else {
                        return;
                    };
                    parameters(strand, lengths, factors);
                    let phase0 = rng::unit(ids[g], seed) * 3.9
                        + rng::unit(ids[s], seed ^ 0xC0FF) * self.random_offset;
                    // The root stays attached whatever the ease.
                    for (j, p) in strand.iter_mut().enumerate().skip(1) {
                        let t = factors[j];
                        let phase = (lengths[j] * turns_per_metre + phase0) * std::f32::consts::TAU;
                        let radius = self.radius
                            * (self.factor_start + (self.factor_end - self.factor_start) * t);
                        let (sin, cos) = phase.sin_cos();
                        let at = guide.at_length(lengths[j]);
                        let circle = at.normal * (sin * radius) + at.binormal() * (cos * radius);
                        let ease = smootherstep(-1e-5, self.curl_start, t).sqrt();
                        *p += (at.position - *p + circle) * (ease * self.factor);
                    }
                },
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strands::Attachment;
    use fabelgeist_math::Vec3;

    #[test]
    fn a_lone_strand_becomes_a_helix_of_the_radius() {
        let mut strands = Strands::new(200);
        let positions: Vec<Vec3> = (0..200)
            .map(|j| Vec3::new(0.0, -(j as f32) * 0.002, 0.0))
            .collect();
        strands.push(
            &positions,
            Attachment {
                normal: Vec3::up(),
                ..Default::default()
            },
        );
        Curl {
            radius: 0.01,
            frequency: 10.0,
            curl_start: 0.0,
            random_offset: 0.0,
            ..Default::default()
        }
        .apply(&mut strands);
        // Away from the root, every point is a radius from the old axis.
        for p in &strands.strand(0)[20..] {
            let r = (p.x * p.x + p.z * p.z).sqrt();
            assert!((r - 0.01).abs() < 1e-3, "{r}");
        }
    }
}
