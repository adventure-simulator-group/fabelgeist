//! `Clump Hair Curves_V2`: strands gathering into locks.
//!
//! Every strand is pulled towards its clump guide ([`GuideMap`]). At each
//! point, at its own factor along its length, the guide is sampled at the
//! same factor; the strand's root offset from the guide's root, taken in the
//! guide's root frame, is carried along in the guide's frame there, so that
//! with no pull a strand runs parallel to its guide. The pull mixes the
//! point towards that parallel position shrunk by the falloff `f =
//! shape_range(factor, Shape)` -- towards the guide itself where `f` is
//! one -- plus the tip spread (a random offset per strand in the guide's tip
//! frame) and the clump offset (one per clump), both grown by `f`. The mix
//! is `f * Factor`, faded by the root's distance from the guide's
//! (`Distance Threshold`, `Distance Falloff`).
//!
//! Twist then turns each point about the guide's tangent through the
//! guide's point, by an angle growing with arc length (radians per metre),
//! randomised per clump by `Twist Random Clump`. `Preserve Length` restores
//! the segment lengths afterwards.

use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::noise;
use crate::ops::{GuideMap, parameters, restore_lengths, shape_range, smoothstep};
use crate::rng;
use crate::strands::{Strands, rotate};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clump {
    /// Clump guides are at least this far apart at the root.
    pub guide_distance: f32,
    /// Chance a strand is a candidate guide.
    pub guide_mask: f32,
    /// Reuse the clumps an earlier operation chose (`Existing Guide Map`).
    pub reuse_guide_map: bool,
    pub factor: f32,
    /// Falloff along the strand: 0 clumps all along, 0.5 grows linearly to
    /// the tip, towards 1 only the tips gather.
    pub shape: f32,
    /// Radians per metre of arc length.
    pub twist: f32,
    /// How much of the twist is random per clump.
    pub twist_random: f32,
    pub twist_factor: f32,
    /// How far tips stray from their clump, metres.
    pub tip_spread: f32,
    /// How far each clump's tips stray from its guide, metres.
    pub clump_offset: f32,
    /// Roots within the threshold of their guide's clump fully; the pull
    /// fades out over the falloff beyond it.
    pub distance_falloff: f32,
    pub distance_threshold: f32,
    pub preserve_length: bool,
    pub seed: i32,
    /// Strands shorter than this barely clump, fading in to twice it:
    /// clumped stubble is a honeycomb, not hair.
    #[serde(default)]
    pub min_length: f32,
}

impl Default for Clump {
    fn default() -> Self {
        Self {
            guide_distance: 0.01,
            guide_mask: 1.0,
            reuse_guide_map: false,
            factor: 1.0,
            shape: 0.5,
            twist: 0.0,
            twist_random: 0.0,
            twist_factor: 1.0,
            tip_spread: 0.0,
            clump_offset: 0.0,
            distance_falloff: 0.0,
            distance_threshold: 0.0,
            preserve_length: false,
            seed: 0,
            min_length: 0.0,
        }
    }
}

impl Clump {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 148_762);
        let map = GuideMap::get(
            strands,
            self.reuse_guide_map,
            self.guide_distance,
            self.guide_mask,
            seed,
        );
        let samplers = map.samplers(strands);
        let reference = strands.positions.clone();
        let points = strands.points;
        let ids = &strands.id;
        let normals = &strands.root_normal;

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
                    let root = reference[s * points];
                    let guide_root = guide.at_factor(0.0);
                    let offset = root - guide_root.position;
                    // The root's offset lies in the scalp; the guide leaves
                    // the scalp nearly along it. Across the flow stays
                    // across; along the flow becomes depth -- so a patch of
                    // roots becomes a round lock rather than a flat one.
                    let scalp = normals[g];
                    let t0 = guide_root.tangent;
                    let across = crate::strands::safe_normalize(t0.cross(scalp), guide_root.normal);
                    let along = crate::strands::safe_normalize(t0 - scalp * t0.dot(scalp), t0);
                    let depth =
                        crate::strands::safe_normalize(across.cross(t0), guide_root.binormal());
                    let placed = across * offset.dot(across) + depth * offset.dot(along);
                    let x = guide_root.normal.dot(placed);
                    let y = guide_root.binormal().dot(placed);

                    // What the tips gather to, in the guide's tip frame.
                    let tip = guide.at_factor(1.0);
                    let mut gather = Vec3::default();
                    if self.tip_spread != 0.0 {
                        let r = rng::signed_vec3(ids[s], seed) * self.tip_spread;
                        gather += tip.normal * r.x + tip.binormal() * r.y + tip.tangent * r.z;
                    }
                    if self.clump_offset != 0.0 {
                        let r = rng::signed_vec3(ids[g], seed ^ 0x0FF5) * self.clump_offset;
                        gather += tip.normal * r.x + tip.binormal() * r.y;
                    }

                    let d = offset.length();
                    let weight = if self.distance_threshold != 0.0 && self.distance_falloff == 0.0 {
                        if d <= self.distance_threshold {
                            1.0
                        } else {
                            0.0
                        }
                    } else {
                        1.0 - smoothstep(
                            self.distance_threshold,
                            self.distance_threshold + self.distance_falloff,
                            d,
                        )
                    };
                    let clump_random = noise::white(Vec3::new(g as f32, 0.0, 0.0));

                    parameters(strand, lengths, factors);
                    let weight = if self.min_length > 0.0 {
                        let length = lengths.last().copied().unwrap_or(0.0);
                        weight * smoothstep(self.min_length, 2.0 * self.min_length, length)
                    } else {
                        weight
                    };
                    // The root stays in its follicle.
                    for (j, p) in strand.iter_mut().enumerate().skip(1) {
                        let t = factors[j];
                        let f = shape_range(t, 0.0, 1.0, self.shape, 2.0);
                        let at = guide.at_factor(t);
                        let parallel = at.normal * x + at.binormal() * y;
                        let target = at.position + parallel * (1.0 - f) + gather * f;
                        let mix = f * self.factor * weight;
                        *p = p.lerp(target, mix);

                        let base = lengths[j] * self.twist;
                        let angle = self.twist_factor
                            * (base + (base * clump_random - base) * self.twist_random);
                        if angle != 0.0 {
                            *p = at.position + rotate(*p - at.position, at.tangent, angle);
                        }
                    }
                    if self.preserve_length {
                        restore_lengths(strand, &reference[s * points..(s + 1) * points], 1.0);
                    }
                },
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strands::Attachment;

    /// Parallel straight strands on a line of roots.
    fn comb(n: usize) -> Strands {
        let mut strands = Strands::new(16);
        for i in 0..n {
            let x = i as f32 * 0.002;
            let positions: Vec<Vec3> = (0..16)
                .map(|j| Vec3::new(x, -(j as f32) * 0.01, 0.0))
                .collect();
            strands.push(
                &positions,
                Attachment {
                    normal: Vec3::up(),
                    id: i as u32,
                    group: -1,
                    ..Default::default()
                },
            );
        }
        strands
    }

    #[test]
    fn tips_gather_to_their_guide_and_roots_stay() {
        let mut strands = comb(20);
        let before = strands.clone();
        Clump {
            guide_distance: 0.05,
            ..Default::default()
        }
        .apply(&mut strands);
        let map = strands.guide_map.clone().unwrap();
        assert_eq!(map.guides.len(), 1);
        let guide_tip = strands.strand(map.guides[0] as usize)[15];
        for s in 0..strands.len() {
            assert!((strands.root(s) - before.root(s)).length() < 1e-6);
            assert!((strands.strand(s)[15] - guide_tip).length() < 1e-4);
        }
    }
}
