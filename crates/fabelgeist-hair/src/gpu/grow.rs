//! [`GpuGroomer::grow`]: a groom's stack on the GPU, from its guides on.

use crate::gpu::error::{HairGpuError, Result};
use crate::parameters::StrandCount;
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::ScanElementCount;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_math::Vec3;

use super::guides::Guides;
use super::{GpuGroom, GpuGroomer, GpuScalp, StrandGpu, head_shape, storage, upload};
use crate::follicles::Growth;
use crate::groom::Groom;
use crate::head::Head;
use crate::look::Look;
use crate::ops::{DensityMask, Interpolate, Op, Parting, STANDARD_GRAVITY};
use crate::rng;

mod input;
mod profile;

/// Parameters by name, as the compute kernels bind them.
macro_rules! params {
    ($($key:literal => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut p = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        $(p.insert($key, $value);)*
        p
    }};
}
pub(super) use params;

/// The two position buffers a stack ping-pongs between: the strands' own,
/// and a scratch one.
pub(super) struct PingPong {
    pub(super) own: Buffer,
    pub(super) scratch: Buffer,
    /// The current positions are in `scratch`.
    pub(super) flipped: bool,
}

impl PingPong {
    fn src(&self) -> Buffer {
        if self.flipped {
            self.scratch.clone()
        } else {
            self.own.clone()
        }
    }

    fn dst(&self) -> Buffer {
        if self.flipped {
            self.own.clone()
        } else {
            self.scratch.clone()
        }
    }

    fn swap(&mut self) {
        self.flipped = !self.flipped;
    }
}

/// Strands a stack runs over -- the guides before interpolation, the
/// groom after -- and what its operations share.
pub(super) struct StrandSet {
    pub(super) positions: PingPong,
    pub(super) attributes: Buffer,
    /// Each strand's root, for the guide maps.
    pub(super) roots: Buffer,
    pub(super) count: u32,
    pub(super) points: u32,
    /// A box the roots are inside.
    pub(super) root_bounds: [Vec3; 2],
    /// The guide map, and whether one has been made.
    pub(super) guide_of: Buffer,
    pub(super) mapped: bool,
}

impl StrandSet {
    fn bytes(&self) -> u64 {
        self.count as u64 * self.points as u64 * 16
    }

    /// The positions back in the strands' own buffer.
    pub(super) fn settle(&mut self, batch: &mut KernelBatch) -> Result<()> {
        if self.positions.flipped {
            batch.copy_buffer(
                &self.positions.scratch,
                &self.positions.own,
                self.bytes().into(),
            )?;
            self.positions.flipped = false;
        }
        Ok(())
    }
}

impl GpuGroomer {
    /// Grow `look` on `head` (its hairline's, [`Look::head`]), rooted on
    /// `scalp` (that head's, uploaded): the guides and the whole groom, on
    /// the GPU.
    pub fn grow_look(&mut self, look: &Look, head: &Head, scalp: &GpuScalp) -> Result<GpuGroom> {
        self.grow(
            &look.groom(head),
            Guides::Hairdo {
                hairdo: &look.hairdo,
                variation: look.variation.ratio(),
                seed: look.seed.word() as u64,
            },
            head,
            scalp,
        )
    }

    /// Grow `groom` from `guides` on `head` (its hairline's), rooted on
    /// `scalp`. Everything runs on the GPU: the guides (grown there, or
    /// brought), the operations before interpolation on them, interpolation
    /// and everything after. The roots' and guides' counts are read back
    /// once, to size what follows; nothing else waits.
    pub fn grow(
        &mut self,
        groom: &Groom,
        guides: Guides,
        head: &Head,
        scalp: &GpuScalp,
    ) -> Result<GpuGroom> {
        let points = groom.points.max(2) as u32;
        let input::Interpolation { at, interpolate } = input::Interpolation::find(groom)?;
        let context = self.context.clone();
        let head_shape = upload(&context, &head_shape(head), "hair head shape".into())?;

        // Roots for the guides and for the strands, then how many of each.
        let mut batch = KernelBatch::labelled(&context, "hair roots");
        let guide_roots = match &guides {
            Guides::Hairdo { hairdo, seed, .. } => {
                Some(self.record_guide_roots(&mut batch, hairdo, *seed, scalp)?)
            }
            Guides::Host(_) => None,
        };
        let (n, candidates, normals, keep) = self.record_roots(&mut batch, interpolate, scalp)?;
        let mut words = vec![(keep.clone(), n)];
        if let Some(roots) = &guide_roots {
            words.push((roots.counts.clone(), 1));
        }
        let counts = self.read_words(batch, &words)?;
        let count = counts[0];
        if count == 0 {
            return Err(HairGpuError::EmptyRoots);
        }
        if count >= 1 << 20 {
            return Err(HairGpuError::StrandLimit {
                requested: StrandCount::from_count(count as usize),
            });
        }

        let mut batch = KernelBatch::labelled(&context, "hair groom");
        let guides = match guides {
            Guides::Host(strands) => self.upload_guides(strands, points)?,
            Guides::Hairdo {
                hairdo,
                variation,
                seed,
            } => {
                let roots = guide_roots.as_ref().expect("placed above");
                self.record_hairdo_guides(
                    &mut batch,
                    hairdo,
                    variation,
                    seed,
                    head,
                    roots,
                    counts[1],
                    points,
                    &head_shape,
                )?
            }
        };
        if guides.count == 0 {
            return Err(HairGpuError::EmptyGuides);
        }

        // The operations before interpolation, on the guides.
        let guide_bytes = guides.count as u64 * points as u64 * 16;
        let mut guide_set = StrandSet {
            positions: PingPong {
                own: guides.positions.clone(),
                scratch: self.scratch.guide_positions.fit(
                    &context,
                    (guide_bytes).into(),
                    "hair guide positions scratch".into(),
                )?,
                flipped: false,
            },
            attributes: guides.attributes.clone(),
            roots: guides.roots.clone(),
            count: guides.count,
            points,
            root_bounds: scalp.bounds,
            guide_of: self.scratch.guide_map.fit(
                &context,
                (guides.count as u64 * 4).into(),
                "hair guide guide map".into(),
            )?,
            mapped: false,
        };
        self.record_ops(&mut batch, &groom.ops[..at], &mut guide_set, &head_shape)?;
        guide_set.settle(&mut batch)?;

        // The guides as interpolation reads them.
        let guide_points = self.scratch.guide_points.fit(
            &context,
            (guide_bytes).into(),
            "hair guide points".into(),
        )?;
        let guide_info = self.scratch.guide_info.fit(
            &context,
            (guides.count as u64 * 16).into(),
            "hair guide info".into(),
        )?;
        self.record_guide_pack(
            &mut batch,
            &guides,
            &guide_points,
            &guide_info,
            points,
            &interpolate.parting,
        )?;

        // The strands' roots, compacted.
        let roots = storage(&context, (count as u64 * 16).into(), "hair roots".into())?;
        let attributes = storage(
            &context,
            (count as u64 * std::mem::size_of::<StrandGpu>() as u64).into(),
            "hair strands".into(),
        )?;
        let mut compact = params! {
            "cand_position" => candidates.clone(),
            "cand_normal" => normals.clone(),
            "keep" => keep.clone(),
            "roots" => roots.clone(),
            "strands" => attributes.clone(),
            "count" => interpolate.count as u32,
            "id_seed" => rng::seed(interpolate.seed, 0x51D),
            "radius" => interpolate.radius,
        };
        for (key, value) in parting_parameters(&interpolate.parting) {
            compact.insert(key, value);
        }
        batch.dispatch_folded(
            &self.kernels.roots_compact,
            &compact,
            interpolate.count as u32,
        )?;

        // Interpolation.
        let neighbors = self.scratch.neighbors.fit(
            &context,
            (count as u64 * 32).into(),
            "hair neighbours".into(),
        )?;
        let weights = self.scratch.weights.fit(
            &context,
            (count as u64 * 32).into(),
            "hair weights".into(),
        )?;
        batch.dispatch_folded(
            &self.kernels.interp_select,
            &params! {
                "roots" => roots.clone(),
                "strands" => attributes.clone(),
                "guide_points" => guide_points.clone(),
                "guide_info" => guide_info.clone(),
                "neighbors" => neighbors.clone(),
                "weights" => weights.clone(),
                "count" => count,
                "guides" => guides.count,
                "points" => points,
                "wanted" => interpolate.neighbors.clamp(1, 8) as u32,
                "divergence" => interpolate.divergence,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            count,
        )?;
        let bytes = count as u64 * points as u64 * 16;
        let mut strands = StrandSet {
            positions: PingPong {
                own: storage(&context, (bytes).into(), "hair positions".into())?,
                scratch: self.scratch.positions.fit(
                    &context,
                    (bytes).into(),
                    "hair positions scratch".into(),
                )?,
                flipped: false,
            },
            attributes: attributes.clone(),
            roots: roots.clone(),
            count,
            points,
            root_bounds: scalp.bounds,
            guide_of: self.scratch.guide_of.fit(
                &context,
                (count as u64 * 4).into(),
                "hair guide map".into(),
            )?,
            mapped: false,
        };
        batch.dispatch_folded(
            &self.kernels.interp_points,
            &params! {
                "roots" => roots.clone(),
                "strands" => attributes.clone(),
                "guide_points" => guide_points,
                "guide_info" => guide_info,
                "neighbors" => neighbors,
                "weights" => weights,
                "dst" => strands.positions.own.clone(),
                "count" => count,
                "points" => points,
                "follow" => u32::from(interpolate.follow_surface_normal),
                "pad0" => 0u32,
                "follow_length" => interpolate.follow_length,
                "pad1" => 0u32,
                "pad2" => 0u32,
                "pad3" => 0u32,
            },
            count * points,
        )?;

        // Everything after.
        self.record_ops(&mut batch, &groom.ops[at + 1..], &mut strands, &head_shape)?;
        strands.settle(&mut batch)?;
        batch.submit();

        Ok(GpuGroom {
            strands: count,
            points,
            positions: strands.positions.own,
            attributes,
            roots,
            bounds: None,
            root_bounds: scalp.bounds,
            head: Some(head.clone()),
            guides: Some(guides),
        })
    }

    /// Record `ops` over `set`, each reading the current positions and
    /// writing the other buffer.
    pub(super) fn record_ops(
        &mut self,
        batch: &mut KernelBatch,
        ops: &[Op],
        set: &mut StrandSet,
        head_shape: &Buffer,
    ) -> Result<()> {
        let context = self.context.clone();
        let (count, points, bytes) = (set.count, set.points, set.bytes());
        for op in ops {
            let (src, dst) = (set.positions.src(), set.positions.dst());
            let attributes = set.attributes.clone();
            let strand_params = |mut p: PassParameters| {
                p.insert("src", src.clone());
                p.insert("dst", dst.clone());
                p.insert("strands", attributes.clone());
                p.insert("count", count);
                p.insert("points", points);
                p
            };
            match op {
                Op::Orient(o) => {
                    if o.memory <= 0.0 {
                        continue;
                    }
                    let field = &o.field;
                    let w = |i: usize| field.whorls.get(i).copied();
                    let w0 = w(0);
                    let w1 = w(1);
                    let point =
                        |w: Option<crate::follicles::Whorl>| w.map_or(Vec3::default(), |w| w.point);
                    let p = strand_params(params! {
                        "seed" => rng::seed(o.seed, 0x0F0C),
                        "growth" => Growth::ALL.iter().position(|g| *g == field.growth).unwrap_or(0) as u32,
                        "memory" => o.memory,
                        "direction_spread" => o.direction_spread,
                        "angle_spread" => o.angle_spread,
                        "whorls" => field.whorls.len() as u32,
                        "center_x" => field.center.x,
                        "center_y" => field.center.y,
                        "center_z" => field.center.z,
                        "pad0" => 0u32,
                        "w0_x" => point(w0).x,
                        "w0_y" => point(w0).y,
                        "w0_z" => point(w0).z,
                        "w0_spiral" => w0.map_or(0.0, |w| w.spiral),
                        "w0_hand" => w0.map_or(1.0, |w| w.hand),
                        "w0_reach" => w0.map_or(0.07, |w| w.reach),
                        "w1_hand" => w1.map_or(1.0, |w| w.hand),
                        "w1_reach" => w1.map_or(0.07, |w| w.reach),
                        "w1_x" => point(w1).x,
                        "w1_y" => point(w1).y,
                        "w1_z" => point(w1).z,
                        "w1_spiral" => w1.map_or(0.0, |w| w.spiral),
                    });
                    batch.dispatch_folded(&self.kernels.orient, &p, count)?;
                }
                Op::Trim(t) => {
                    let p = strand_params(params! {
                        "seed" => rng::seed(t.seed, 77_221),
                        "pad0" => 0u32,
                        "keep_share" => t.length,
                        "random" => t.random,
                        "max_length" => t.max_length,
                        "pad1" => 0u32,
                    });
                    batch.dispatch_folded(&self.kernels.trim, &p, count)?;
                }
                Op::Clump(c) => {
                    let seed = rng::seed(c.seed, 148_762);
                    if !(c.reuse_guide_map && set.mapped) {
                        self.record_set_map(batch, set, c.guide_distance, c.guide_mask, seed)?;
                    }
                    let mut p = strand_params(params! {
                        "seed" => seed,
                        "preserve" => u32::from(c.preserve_length),
                        "factor" => c.factor,
                        "shape" => c.shape,
                        "twist" => c.twist,
                        "twist_random" => c.twist_random,
                        "twist_factor" => c.twist_factor,
                        "tip_spread" => c.tip_spread,
                        "clump_offset" => c.clump_offset,
                        "distance_falloff" => c.distance_falloff,
                        "distance_threshold" => c.distance_threshold,
                        "min_length" => c.min_length,
                        "pad0" => 0u32,
                        "pad1" => 0u32,
                    });
                    p.insert("guide_of", set.guide_of.clone());
                    batch.dispatch_folded(&self.kernels.clump, &p, count)?;
                }
                Op::Curl(c) => {
                    let seed = rng::seed(c.seed, 91_487);
                    if !(c.reuse_guide_map && set.mapped) {
                        self.record_set_map(batch, set, c.guide_distance, c.guide_mask, seed)?;
                    }
                    let mut p = strand_params(params! {
                        "seed" => seed,
                        "pad0" => 0u32,
                        "factor" => c.factor,
                        "curl_start" => c.curl_start,
                        "radius" => c.radius,
                        "factor_start" => c.factor_start,
                        "factor_end" => c.factor_end,
                        "turns_per_metre" => c.frequency * 3.0,
                        "random_offset" => c.random_offset,
                        "pad1" => 0u32,
                    });
                    p.insert("guide_of", set.guide_of.clone());
                    batch.dispatch_folded(&self.kernels.curl, &p, count)?;
                }
                Op::Wave(w) => {
                    let seed = rng::seed(w.seed, 0x3A7E);
                    if !(w.reuse_guide_map && set.mapped) {
                        self.record_set_map(batch, set, w.guide_distance, 1.0, seed)?;
                    }
                    let mut p = strand_params(params! {
                        "seed" => seed,
                        "pad0" => 0u32,
                        "amplitude" => w.amplitude,
                        "wavelength" => w.wavelength,
                        "start" => w.start,
                        "random_phase" => w.random_phase,
                    });
                    p.insert("guide_of", set.guide_of.clone());
                    batch.dispatch_folded(&self.kernels.wave, &p, count)?;
                }
                Op::Noise(n) => {
                    let seed = rng::seed(n.seed, 368_741);
                    let origin = rng::signed_vec3(n.seed as u32, seed) * 100.0;
                    let p = strand_params(params! {
                        "seed" => seed,
                        "cumulative" => u32::from(n.cumulative),
                        "scale_total" => n.factor * n.distance,
                        "shape" => n.shape,
                        "scale" => n.scale,
                        "scale_along" => n.scale_along_curve,
                        "offset_per_curve" => n.offset_per_curve,
                        "preserve" => u32::from(n.preserve_length),
                        "origin_x" => origin.x,
                        "origin_y" => origin.y,
                        "origin_z" => origin.z,
                        "pad0" => 0u32,
                        "pad1" => 0u32,
                        "pad2" => 0u32,
                    });
                    batch.dispatch_folded(&self.kernels.noise, &p, count)?;
                }
                Op::Frizz(f) => {
                    let p = strand_params(params! {
                        "seed" => rng::seed(f.seed, 64_785),
                        "cumulative" => u32::from(f.cumulative),
                        "scale_total" => f.factor * f.distance,
                        "shape" => f.shape,
                        "share" => f.share,
                        "calm" => f.calm,
                        "frequency" => f.frequency,
                        "preserve" => u32::from(f.preserve_length),
                        "pad0" => 0u32,
                        "pad1" => 0u32,
                    });
                    batch.dispatch_folded(&self.kernels.frizz, &p, count)?;
                }
                Op::Smooth(m) => {
                    let p = strand_params(params! {
                        "iterations" => m.iterations as u32,
                        "preserve" => u32::from(m.preserve_length),
                        "factor" => m.factor,
                        "pad0" => 0u32,
                        "pad1" => 0u32,
                        "pad2" => 0u32,
                    });
                    batch.dispatch_folded(&self.kernels.smooth, &p, count)?;
                }
                Op::Shrinkwrap(w) => {
                    let moves = self.scratch.moves.fit(
                        &context,
                        (bytes).into(),
                        "hair shrinkwrap moves".into(),
                    )?;
                    let mut p = strand_params(params! {
                        "smoothing" => w.smoothing as u32,
                        "lock_roots" => u32::from(w.lock_roots),
                        "factor" => w.factor,
                        "offset" => w.offset,
                        "above_surface" => w.above_surface,
                        "preserve" => u32::from(w.preserve_length),
                    });
                    p.insert("head_shape", head_shape.clone());
                    p.insert("moves", moves);
                    batch.dispatch_folded(&self.kernels.shrinkwrap, &p, count)?;
                }
                Op::Settle(settle) => {
                    let load = Vec3::new(0.0, -settle.gravity, 0.0) + Vec3::from_array(settle.wind);
                    let reference = Vec3::new(0.0, -STANDARD_GRAVITY, 0.0);
                    if settle.relative && (load - reference).length() < 1e-4 {
                        // Standard conditions: nothing changes.
                        continue;
                    }
                    let compliance = 1.0 / (STANDARD_GRAVITY * settle.stiffness.max(1e-3).powi(3));
                    let moments = self.scratch.settle_moments.fit(
                        &context,
                        (bytes).into(),
                        "hair drape moments".into(),
                    )?;
                    let dirs = self.scratch.settle_dirs.fit(
                        &context,
                        (bytes).into(),
                        "hair drape directions".into(),
                    )?;
                    let under = self.scratch.settle_under.fit(
                        &context,
                        (bytes).into(),
                        "hair drape".into(),
                    )?;
                    let standard = self.scratch.settle_standard.fit(
                        &context,
                        (bytes).into(),
                        "hair drape at rest".into(),
                    )?;
                    let few = count <= 16_384 && points <= super::wgsl::groom::SETTLE_FEW_POINTS;
                    if few {
                        // A workgroup a strand and a solve, both at once.
                        batch.dispatch(
                            &self.kernels.settle_solve_few,
                            &params! {
                                "src" => src.clone(),
                                "under" => under.clone(),
                                "standard" => standard.clone(),
                                "head_shape" => head_shape.clone(),
                                "count" => count,
                                "points" => points,
                                "iterations" => settle.iterations as u32,
                                "pad0" => 0u32,
                                "load_x" => load.x,
                                "load_y" => load.y,
                                "load_z" => load.z,
                                "compliance" => compliance,
                                "reference_x" => reference.x,
                                "reference_y" => reference.y,
                                "reference_z" => reference.z,
                                "collision_offset" => settle.collision_offset,
                            },
                            [count, if settle.relative { 2 } else { 1 }, 1],
                        )?;
                    }
                    let mut solve = |load: Vec3, solved: &Buffer| {
                        batch
                            .dispatch_folded(
                                &self.kernels.settle_solve,
                                &params! {
                                    "src" => src.clone(),
                                    "solved" => solved.clone(),
                                    "moments" => moments.clone(),
                                    "dirs" => dirs.clone(),
                                    "head_shape" => head_shape.clone(),
                                    "count" => count,
                                    "points" => points,
                                    "iterations" => settle.iterations as u32,
                                    "pad0" => 0u32,
                                    "load_x" => load.x,
                                    "load_y" => load.y,
                                    "load_z" => load.z,
                                    "compliance" => compliance,
                                    "collision_offset" => settle.collision_offset,
                                    "pad1" => 0u32,
                                    "pad2" => 0u32,
                                    "pad3" => 0u32,
                                },
                                count,
                            )
                            .map(|_| ())
                    };
                    if !few {
                        solve(load, &under)?;
                        if settle.relative {
                            solve(reference, &standard)?;
                        }
                    }
                    let mut p = strand_params(params! {
                        "relative" => u32::from(settle.relative),
                        "pad0" => 0u32,
                        "collision_offset" => settle.collision_offset,
                        "pad1" => 0u32,
                        "pad2" => 0u32,
                        "pad3" => 0u32,
                    });
                    p.insert("under", under);
                    p.insert("standard", standard);
                    p.insert("head_shape", head_shape.clone());
                    batch.dispatch_folded(&self.kernels.settle_apply, &p, count)?;
                }
                Op::Profile(r) => {
                    self.record_profile(batch, set, r)?;
                    // In place: the positions stay where they are.
                    continue;
                }
                Op::Interpolate(_) => return Err(HairGpuError::RepeatedInterpolation),
            }
            set.positions.swap();
        }
        Ok(())
    }

    /// The candidate roots over the scalp and which are kept, scanned into
    /// strand slots: how many is in `keep`'s last word.
    fn record_roots(
        &mut self,
        batch: &mut KernelBatch,
        interpolate: &Interpolate,
        scalp: &GpuScalp,
    ) -> Result<(u32, Buffer, Buffer, Buffer)> {
        let context = self.context.clone();
        let n = interpolate.count.max(1) as u32;
        // How far in from the hairline each scalp vertex is.
        let extra = self.scratch.scalp_extra.fit(
            &context,
            (scalp.vertices as u64 * 16).into(),
            "hair scalp uv and edge".into(),
        )?;
        batch.dispatch_folded(
            &self.kernels.edge_distances,
            &params! {
                "scalp_points" => scalp.points.clone(),
                "scalp_uvs" => scalp.uvs.clone(),
                "boundary" => scalp.boundary_fine.clone(),
                "extra" => extra.clone(),
                "count" => scalp.vertices,
                "boundary_count" => scalp.boundary_fine_count,
                "edge" => interpolate.edge,
                "pad0" => 0u32,
            },
            scalp.vertices,
        )?;

        let (balding, [crown, temples, top]) = match &interpolate.density {
            DensityMask::Balding {
                crown,
                temples,
                top,
            } => (1u32, [*crown, *temples, *top]),
            _ => (0, [0.0; 3]),
        };
        let seed = rng::seed(interpolate.seed, 0x1A7E);
        let mask_seed = rng::seed(interpolate.seed, 0xD35E);
        let offset = rng::signed_vec3(mask_seed, 0x9A7C) * 50.0;

        let candidates = self.scratch.candidates.fit(
            &context,
            (n as u64 * 32).into(),
            "hair root candidates".into(),
        )?;
        let candidate_normals = self.scratch.normals.fit(
            &context,
            (n as u64 * 16).into(),
            "hair root candidate normals".into(),
        )?;
        let keep = self.scratch.keep.fit(
            &context,
            ((n as u64 + 1) * 4).into(),
            "hair roots kept".into(),
        )?;
        batch.clear_buffer(&keep);
        batch.dispatch_folded(
            &self.kernels.roots,
            &params! {
                "tri_cdf" => scalp.tri_cdf.clone(),
                "tri_index" => scalp.tri_index.clone(),
                "scalp_points" => scalp.points.clone(),
                "scalp_normals" => scalp.normals.clone(),
                "scalp_extra" => extra,
                "cand_position" => candidates.clone(),
                "cand_normal" => candidate_normals.clone(),
                "keep" => keep.clone(),
                "count" => n,
                "triangles" => scalp.triangles,
                "seed" => seed,
                "balding" => balding,
                "total_area" => scalp.area.max(1e-9),
                "crown" => crown,
                "temples" => temples,
                "top" => top,
                "patchiness" => interpolate.patchiness,
                "edge" => interpolate.edge,
                "offset_x" => offset.x,
                "offset_y" => offset.y,
                "offset_z" => offset.z,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            n,
        )?;
        self.ensure_scan(n + 1)?;
        self.scan.record(
            batch,
            &keep,
            &self.scan_scratch,
            ScanElementCount::from_elements(n + 1),
        )?;
        Ok((n, candidates, candidate_normals, keep))
    }

    /// A guide map of `set` into its `guide_of`.
    fn record_set_map(
        &mut self,
        batch: &mut KernelBatch,
        set: &mut StrandSet,
        distance: f32,
        mask: f32,
        seed: u32,
    ) -> Result<()> {
        self.record_map(
            batch,
            &set.roots,
            &set.attributes,
            &set.guide_of,
            set.count,
            set.root_bounds,
            distance,
            mask,
            seed,
        )?;
        set.mapped = true;
        Ok(())
    }

    /// A guide map into `out`: on a grid of `distance` over the roots' box,
    /// each cell's lowest-hashed candidate of each parting group leads it,
    /// and every strand takes the nearest leader around it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_map(
        &mut self,
        batch: &mut KernelBatch,
        roots: &Buffer,
        attributes: &Buffer,
        out: &Buffer,
        count: u32,
        bounds: [Vec3; 2],
        distance: f32,
        mask: f32,
        seed: u32,
    ) -> Result<()> {
        let mut cell = distance.max(0.002);
        let pad = Vec3::splat(0.01);
        let (lo, hi) = (bounds[0] - pad, bounds[1] + pad);
        let extent = hi - lo;
        let dims =
            |cell: f32| [extent.x, extent.y, extent.z].map(|e| ((e / cell).ceil() as u32).max(1));
        while dims(cell).iter().map(|&d| d as u64).product::<u64>() * 5 > 8 << 20 {
            cell *= 1.25;
        }
        let [nx, ny, nz] = dims(cell);
        let leaders = self.scratch.leaders.fit(
            &self.context,
            (nx as u64 * ny as u64 * nz as u64 * 5 * 4).into(),
            "hair guide leaders".into(),
        )?;
        batch.clear_buffer(&leaders);
        let grid = |mut p: PassParameters| {
            p.insert("roots", roots.clone());
            p.insert("strands", attributes.clone());
            p.insert("leaders", leaders.clone());
            p.insert("count", count);
            p.insert("nx", nx);
            p.insert("ny", ny);
            p.insert("nz", nz);
            p.insert("lo_x", lo.x);
            p.insert("lo_y", lo.y);
            p.insert("lo_z", lo.z);
            p.insert("inverse_cell", 1.0 / cell);
            p.insert("mask", mask);
            p.insert("seed", seed);
            p.insert("pad0", 0u32);
            p.insert("pad1", 0u32);
            p
        };
        batch.dispatch_folded(
            &self.kernels.map_candidates,
            &grid(PassParameters::new()),
            count,
        )?;
        batch.dispatch_folded(
            &self.kernels.map_assign,
            &grid(params! { "guide_of" => out.clone() }),
            count,
        )?;
        Ok(())
    }
}

/// A parting as the kernels' parameters.
pub(super) fn parting_parameters(
    parting: &Parting,
) -> Vec<(
    &'static str,
    fabelgeist_gpu::data::gpu::parameters::PassParameter,
)> {
    let (kind, lanes, plane, until, front, half_width) = match parting {
        Parting::None => (0u32, 0u32, [0.0f32; 4], 0.0f32, 0.0f32, 0.0f32),
        Parting::Plane {
            normal,
            offset,
            until,
        } => (
            1,
            0,
            [normal[0], normal[1], normal[2], *offset],
            *until,
            0.0,
            0.0,
        ),
        Parting::Lanes { count } => (2, *count, [0.0; 4], 0.0, 0.0, 0.0),
        Parting::Fringe { front, half_width } => (3, 0, [0.0; 4], 0.0, *front, *half_width),
    };
    vec![
        ("parting", kind.into()),
        ("lanes", lanes.into()),
        ("plane_x", plane[0].into()),
        ("plane_y", plane[1].into()),
        ("plane_z", plane[2].into()),
        ("plane_offset", plane[3].into()),
        ("until", until.into()),
        ("fringe_front", front.into()),
        ("fringe_half_width", half_width.into()),
    ]
}
