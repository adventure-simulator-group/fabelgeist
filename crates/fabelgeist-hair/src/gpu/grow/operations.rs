//! Operation recording over a checked native strand set.

use super::*;

impl GpuGroomer {
    /// Record `ops` over `set`, each reading the current positions and
    /// writing the other buffer.
    pub(in crate::gpu) fn record_ops(
        &mut self,
        batch: &mut KernelBatch,
        ops: &[Op],
        set: &mut StrandSet,
        head_shape: &Buffer,
    ) -> Result<()> {
        let context = self.context.clone();
        let layout = set.layout;
        let count = layout.native_strands();
        let points = layout.native_points();
        let bytes = layout.positions_bytes();
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
                    batch.dispatch_folded(&self.kernels.orient, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.trim, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.clump, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.curl, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.wave, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.noise, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.frizz, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.smooth, &p, layout.strand_invocations())?;
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
                    batch.dispatch_folded(&self.kernels.shrinkwrap, &p, layout.strand_invocations())?;
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
                    let few = count <= 16_384 && points <= crate::gpu::wgsl::groom::SETTLE_FEW_POINTS;
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
                            [count, if settle.relative { 2 } else { 1 }, 1].into(),
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
                                layout.strand_invocations(),
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
                    batch.dispatch_folded(&self.kernels.settle_apply, &p, layout.strand_invocations())?;
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
}
