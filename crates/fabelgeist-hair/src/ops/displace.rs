//! Displacements along each strand's own frame: `Frizz Hair Curves`, `Hair
//! Curves Noise` and `Smooth Hair Curves`.
//!
//! Frizz and noise both build an offset per point in the strand's frame
//! (tangent, binormal, normal), scaled by `Factor * Distance`. Frizz draws
//! it at random per point; noise samples 3D Perlin noise at the root's
//! position (times `Scale`), moved along one axis by the arc length (times
//! `Scale along Curve`) and a random amount per strand (`Offset per Curve`)
//! -- so neighbouring strands wave together. With `Cumulative Offset` the
//! offsets are summed from the root out (each point's own counted half), a
//! walk rather than a jitter; either way the root's own offset is taken
//! away, and the result is faded in along the strand by
//! `shape_range(factor, Shape)`.

use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::noise;
use crate::ops::{parameters, restore_lengths, shape_range};
use crate::rng;
use crate::strands::{Strands, blur, normals, tangents};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frizz {
    pub cumulative: bool,
    pub factor: f32,
    /// Metres.
    pub distance: f32,
    pub shape: f32,
    pub preserve_length: bool,
    pub seed: i32,
    /// Share of strands that frizz fully; the rest frizz `calm` times as
    /// much. Real hair has a few flyaways, not a halo (1 frizzes all, as
    /// Blender's node does).
    #[serde(default = "one")]
    pub share: f32,
    #[serde(default = "one")]
    pub calm: f32,
    /// Turns of the frizz a metre along the strand: smooth wandering in
    /// place of Blender's new direction at every point (0).
    #[serde(default)]
    pub frequency: f32,
}

fn one() -> f32 {
    1.0
}

impl Default for Frizz {
    fn default() -> Self {
        Self {
            cumulative: true,
            factor: 1.0,
            distance: 0.01,
            shape: 0.5,
            preserve_length: false,
            seed: 0,
            share: 1.0,
            calm: 1.0,
            frequency: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Noise {
    pub cumulative: bool,
    pub factor: f32,
    /// Metres.
    pub distance: f32,
    pub shape: f32,
    /// Noise cells a metre across the scalp (times the node's fixed 5).
    pub scale: f32,
    /// Noise cells a metre along a strand (times the node's fixed 5).
    pub scale_along_curve: f32,
    pub offset_per_curve: f32,
    pub preserve_length: bool,
    pub seed: i32,
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            cumulative: true,
            factor: 1.0,
            distance: 0.01,
            shape: 0.5,
            scale: 1.0,
            scale_along_curve: 1.0,
            offset_per_curve: 0.0,
            preserve_length: false,
            seed: 0,
        }
    }
}

/// Add frame-space offsets to a strand, walked from the root if
/// `cumulative`, faded in by `shape`.
fn displace(
    strand: &mut [Vec3],
    offsets: &mut [Vec3],
    factors: &[f32],
    cumulative: bool,
    shape: f32,
) {
    let root = offsets[0];
    if cumulative {
        let mut trailing = Vec3::default();
        for o in offsets.iter_mut() {
            let v = *o;
            let leading = trailing + v;
            *o = (leading + trailing - root) * 0.5;
            trailing = leading;
        }
    } else {
        for o in offsets.iter_mut() {
            *o -= root;
        }
    }
    for (j, p) in strand.iter_mut().enumerate() {
        *p += offsets[j] * shape_range(factors[j], 0.0, 1.0, shape, 2.0);
    }
}

impl Frizz {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 64_785);
        let scale = self.factor * self.distance;
        let points = strands.points;
        let reference = self.preserve_length.then(|| strands.positions.clone());
        let ids = &strands.id;
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(lengths, factors, t, n, offsets), (s, strand)| {
                    parameters(strand, lengths, factors);
                    tangents(strand, t);
                    normals(t, n);
                    offsets.clear();
                    let wild = rng::unit(ids[s], seed ^ 0xF1A7) < self.share;
                    let scale = if wild { scale } else { scale * self.calm };
                    if scale == 0.0 {
                        return;
                    }
                    let origin = rng::signed_vec3(ids[s], seed ^ 0x0F75) * 50.0;
                    for j in 0..points {
                        let r = if self.frequency > 0.0 {
                            // A smooth wander: noise along the strand, the
                            // step between neighbours small.
                            let q = origin + Vec3::new(0.0, 0.0, lengths[j] * self.frequency);
                            let c = noise::noise_color(q, 1.0) - Vec3::splat(0.5);
                            c * (2.0 * scale * lengths[1].max(1e-6) * self.frequency)
                        } else {
                            rng::signed_vec3(rng::hash(ids[s], j as u32), seed) * scale
                        };
                        let b = t[j].cross(n[j]);
                        offsets.push(t[j] * r.x + b * r.y + n[j] * r.z);
                    }
                    displace(strand, offsets, factors, self.cumulative, self.shape);
                    if let Some(reference) = &reference {
                        restore_lengths(strand, &reference[s * points..(s + 1) * points], 1.0);
                    }
                },
            );
    }
}

impl Noise {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 368_741);
        let scale = self.factor * self.distance;
        let points = strands.points;
        let reference = self.preserve_length.then(|| strands.positions.clone());
        let ids = &strands.id;
        let origin = rng::signed_vec3(self.seed as u32, seed) * 100.0;
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(lengths, factors, t, n, offsets), (s, strand)| {
                    parameters(strand, lengths, factors);
                    tangents(strand, t);
                    normals(t, n);
                    let base = strand[0] * self.scale + origin;
                    let along = rng::signed(ids[s], seed) * self.offset_per_curve;
                    offsets.clear();
                    for j in 0..points {
                        let z = lengths[j] * self.scale_along_curve + along;
                        let c = noise::noise_color(base + Vec3::new(0.0, 0.0, z), 5.0);
                        let r = (c - Vec3::splat(0.5)) * (2.0 * scale);
                        let b = t[j].cross(n[j]);
                        offsets.push(t[j] * r.x + b * r.y + n[j] * r.z);
                    }
                    displace(strand, offsets, factors, self.cumulative, self.shape);
                    if let Some(reference) = &reference {
                        restore_lengths(strand, &reference[s * points..(s + 1) * points], 1.0);
                    }
                },
            );
    }
}

/// `Smooth Hair Curves`: average each point with its neighbours, root held.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Smooth {
    pub iterations: usize,
    pub factor: f32,
    pub preserve_length: bool,
}

impl Default for Smooth {
    fn default() -> Self {
        Self {
            iterations: 5,
            factor: 1.0,
            preserve_length: true,
        }
    }
}

impl Smooth {
    pub fn apply(&self, strands: &mut Strands) {
        let points = strands.points;
        let weight = vec![self.factor; points];
        let reference = self.preserve_length.then(|| strands.positions.clone());
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each(|(s, strand)| {
                // The tip may move too; only the root is pinned.
                let tip = strand.len() - 1;
                let extended_tip = strand[tip] * 2.0 - strand[tip - 1];
                let mut values: Vec<Vec3> = strand.to_vec();
                values.push(extended_tip);
                blur(&mut values, self.iterations, &weight);
                strand.copy_from_slice(&values[..points]);
                if let Some(reference) = &reference {
                    restore_lengths(strand, &reference[s * points..(s + 1) * points], 1.0);
                }
            });
    }
}

/// Waves: hair that is neither straight nor curled, bending side to side in
/// the plane it lies in -- across the strand and along the head, so a lock
/// of wavy hair ripples rather than spirals. Not one of DiffLocks' node
/// groups (its curls are all helices); waves are what most hair is.
///
/// Each point moves across its strand by `amplitude` times a sine of its
/// arc length over `wavelength`, eased in from `start` of the way along.
/// The phase is shared by a lock (the guide map, as the curl uses) and
/// jittered per strand, and segment lengths are restored, so a wavy strand
/// is as long as a straight one and hangs shorter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wave {
    /// Metres either side.
    pub amplitude: f32,
    /// Metres from crest to crest.
    pub wavelength: f32,
    /// Share of the strand before the waves start.
    pub start: f32,
    /// Locks that wave together are this far apart at the root.
    pub guide_distance: f32,
    pub reuse_guide_map: bool,
    /// Turns a strand's phase may stray from its lock's.
    pub random_phase: f32,
    pub seed: i32,
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            amplitude: 0.01,
            wavelength: 0.07,
            start: 0.15,
            guide_distance: 0.02,
            reuse_guide_map: true,
            random_phase: 0.08,
            seed: 0,
        }
    }
}

impl Wave {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 0x3A7E);
        let map = crate::ops::GuideMap::get(
            strands,
            self.reuse_guide_map,
            self.guide_distance,
            1.0,
            seed,
        );
        let points = strands.points;
        let reference = strands.positions.clone();
        let ids = &strands.id;
        let normals_at_root = &strands.root_normal;
        strands
            .positions
            .par_chunks_mut(points)
            .enumerate()
            .for_each_init(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(lengths, factors, t), (s, strand)| {
                    parameters(strand, lengths, factors);
                    tangents(strand, t);
                    let lock = ids[map.guide[s] as usize];
                    let phase = (rng::unit(lock, seed)
                        + rng::signed(ids[s], seed ^ 0x51) * self.random_phase)
                        * std::f32::consts::TAU;
                    let out = normals_at_root[s];
                    for j in 1..points {
                        let across = crate::strands::safe_normalize(
                            t[j].cross(out),
                            crate::strands::orthogonal(t[j]),
                        );
                        let ease =
                            crate::ops::smoothstep(self.start * 0.5, self.start + 0.1, factors[j]);
                        let wave = (lengths[j] / self.wavelength.max(1e-3) * std::f32::consts::TAU
                            + phase)
                            .sin();
                        strand[j] += across * (self.amplitude * ease * wave);
                    }
                    restore_lengths(strand, &reference[s * points..(s + 1) * points], 1.0);
                },
            );
    }
}
