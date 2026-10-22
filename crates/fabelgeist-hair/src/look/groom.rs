//! Expansion of semantic hair parameters into the native numerical node graph.
//!
//! Unit getters here are the handoff to the CPU/WGSL numerical operations.
//! Keep operation order, seed words and floating-point expression order stable.

use super::Look;
use crate::ops::{
    Clump, Curl, DensityMask, Frizz, Interpolate, Noise, Op, Orient, Profile, Settle, Shrinkwrap,
    Trim, Wave,
};
use crate::{Groom, Head};

impl Look {
    /// Expand into the original numerical operation graph, in evaluation order.
    pub fn groom(&self, head: &Head) -> Groom {
        let mut ops = Vec::new();
        self.guide_operations(head, &mut ops);
        self.coarse_texture(&mut ops);
        self.fine_texture(&mut ops);
        Groom {
            points: self.points.count(),
            ops,
        }
    }

    fn guide_operations(&self, head: &Head, ops: &mut Vec<Op>) {
        let seed = self.seed.word();
        let hairdo = &self.hairdo;
        let follicles = &self.hairdo.follicles;
        if self.drape.enabled() {
            ops.push(Op::Settle(Settle {
                gravity: self.gravity.metres_per_second_squared(),
                wind: self.wind.components(),
                ..Default::default()
            }));
            ops.push(Op::Shrinkwrap(Shrinkwrap {
                offset: 0.003,
                smoothing: 4,
                preserve_length: true,
                ..Default::default()
            }));
        }
        let [crown, temples, top] = follicles.balding;
        ops.push(Op::Interpolate(Interpolate {
            count: self.strands.count(),
            neighbors: self.neighbors.count(),
            parting: hairdo.parting(),
            // Where neighbouring guides are combed apart, a strand follows
            // one of them: the hair parts, it does not fill the gap.
            divergence: 0.5,
            // Hair that stands out follows its own root's normal all the
            // way; hair that lies on the head only leaves the scalp so.
            follow_length: if hairdo.stands() { 1.0 } else { 0.05 },
            density: if crown + temples + top > 0.0 {
                DensityMask::Balding {
                    crown,
                    temples,
                    top,
                }
            } else {
                DensityMask::Uniform
            },
            patchiness: follicles.patchiness,
            edge: follicles.hairline.softness,
            radius: follicles.radius,
            seed,
            ..Default::default()
        }));
        ops.push(Op::Orient(Orient {
            field: follicles.field(head),
            memory: follicles.memory,
            direction_spread: follicles.direction_spread,
            angle_spread: follicles.angle_spread,
            seed: seed + 9,
        }));
        if self.length_variation.ratio() > 0.0 {
            ops.push(Op::Trim(Trim {
                length: 1.0,
                random: self.length_variation.ratio(),
                max_length: 0.0,
                seed: seed + 1,
            }));
        }
    }

    fn coarse_texture(&self, ops: &mut Vec<Op>) {
        let seed = self.seed.word();
        if self.clump.ratio() > 0.0 {
            // Dreadlocks are clumped all along, into ropes.
            let rope = self.locs.enabled();
            ops.push(Op::Clump(Clump {
                guide_distance: self.clump_size.metres(),
                factor: if rope {
                    0.93 * self.clump.ratio()
                } else {
                    self.clump.ratio() * 0.8
                },
                // A rope gathers within a few centimetres of the scalp.
                shape: if rope { 0.05 } else { 0.3 },
                twist: self.twist.radians_per_metre(),
                twist_random: 0.5,
                tip_spread: self.clump_size.metres() * 0.15,
                clump_offset: self.clump_size.metres() * 0.2,
                preserve_length: true,
                seed: seed + 2,
                min_length: 0.015,
                ..Default::default()
            }));
        }
        if self.curl.ratio() > 0.0 {
            ops.push(Op::Curl(Curl {
                guide_distance: self.curl_size.metres(),
                reuse_guide_map: false,
                factor: self.curl.ratio(),
                curl_start: self.curl_start.ratio(),
                radius: self.curl_radius.metres(),
                factor_start: 0.7,
                factor_end: 1.0,
                frequency: self.curl_turns.turns_per_metre() / 3.0,
                random_offset: 0.15,
                seed: seed + 3,
                ..Default::default()
            }));
        }
        if self.wave.metres() > 0.0 {
            ops.push(Op::Wave(Wave {
                amplitude: self.wave.metres(),
                wavelength: self.wavelength.metres(),
                seed: seed + 8,
                ..Default::default()
            }));
        }
    }

    fn fine_texture(&self, ops: &mut Vec<Op>) {
        let seed = self.seed.word();
        let follicles = &self.hairdo.follicles;
        if self.clump.ratio() > 0.0 && !self.locs.enabled() {
            ops.push(Op::Clump(Clump {
                guide_distance: (self.clump_size.metres() * 0.3).max(0.003),
                factor: self.clump.ratio() * 0.6,
                shape: 0.75,
                tip_spread: 0.001,
                preserve_length: true,
                seed: seed + 4,
                min_length: 0.015,
                ..Default::default()
            }));
        }
        if self.noise.metres() > 0.0 {
            ops.push(Op::Noise(Noise {
                // Offsets, not a walk: Blender's cumulative noise sums a
                // point's offset per point, so its reach grows with the
                // point count. Here `distance` is how far hair strays.
                cumulative: false,
                distance: self.noise.metres(),
                scale: 6.0,
                scale_along_curve: self.noise_scale.per_metre() / 5.0,
                offset_per_curve: 0.02,
                shape: 0.6,
                preserve_length: true,
                seed: seed + 5,
                ..Default::default()
            }));
        }
        if self.frizz.metres() > 0.0 {
            ops.push(Op::Frizz(Frizz {
                distance: self.frizz.metres(),
                shape: 0.5,
                cumulative: true,
                seed: seed + 6,
                share: self.flyaways.ratio(),
                // Every strand wanders a little on its own, so that no two
                // lie quite parallel: the highlight breaks into strands.
                calm: 0.12,
                frequency: 15.0,
                ..Default::default()
            }));
        }
        ops.push(Op::Shrinkwrap(Shrinkwrap {
            offset: 0.0015,
            smoothing: 3,
            preserve_length: false,
            ..Default::default()
        }));
        ops.push(Op::Profile(Profile {
            radius: follicles.radius,
            random: follicles.radius_spread,
            seed: seed + 7,
            relative_to: Some(follicles.radius),
        }));
    }
}
