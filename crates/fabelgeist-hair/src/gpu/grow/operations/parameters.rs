//! Native uniform packing for strand operations.

use super::*;
use crate::ops::{Clump, Curl, Frizz, Noise, Orient, Shrinkwrap, Smooth, Trim, Wave};

impl Orient {
    pub(super) fn gpu_parameters(&self) -> PassParameters {
        let field = &self.field;
        let w = |i: usize| field.whorls.get(i).copied();
        let w0 = w(0);
        let w1 = w(1);
        let point =
            |w: Option<crate::follicles::Whorl>| w.map_or(Vec3::default(), |w| w.point);
        params! {
            "seed" => rng::seed(self.seed, 0x0F0C),
            "growth" => Growth::ALL.iter().position(|g| *g == field.growth).unwrap_or(0) as u32,
            "memory" => self.memory,
            "direction_spread" => self.direction_spread,
            "angle_spread" => self.angle_spread,
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
        }
    }
}

impl Trim {
    pub(super) fn gpu_parameters(&self) -> PassParameters {
        params! {
            "seed" => rng::seed(self.seed, 77_221),
            "pad0" => 0u32,
            "keep_share" => self.length,
            "random" => self.random,
            "max_length" => self.max_length,
            "pad1" => 0u32,
        }
    }
}

impl Clump {
    pub(super) fn gpu_parameters(&self, seed: u32) -> PassParameters {
        params! {
            "seed" => seed,
            "preserve" => u32::from(self.preserve_length),
            "factor" => self.factor,
            "shape" => self.shape,
            "twist" => self.twist,
            "twist_random" => self.twist_random,
            "twist_factor" => self.twist_factor,
            "tip_spread" => self.tip_spread,
            "clump_offset" => self.clump_offset,
            "distance_falloff" => self.distance_falloff,
            "distance_threshold" => self.distance_threshold,
            "min_length" => self.min_length,
            "pad0" => 0u32,
            "pad1" => 0u32,
        }
    }
}

impl Curl {
    pub(super) fn gpu_parameters(&self, seed: u32) -> PassParameters {
        params! {
            "seed" => seed,
            "pad0" => 0u32,
            "factor" => self.factor,
            "curl_start" => self.curl_start,
            "radius" => self.radius,
            "factor_start" => self.factor_start,
            "factor_end" => self.factor_end,
            "turns_per_metre" => self.frequency * 3.0,
            "random_offset" => self.random_offset,
            "pad1" => 0u32,
        }
    }
}

impl Wave {
    pub(super) fn gpu_parameters(&self, seed: u32) -> PassParameters {
        params! {
            "seed" => seed,
            "pad0" => 0u32,
            "amplitude" => self.amplitude,
            "wavelength" => self.wavelength,
            "start" => self.start,
            "random_phase" => self.random_phase,
        }
    }
}

impl Noise {
    pub(super) fn gpu_parameters(&self, seed: u32) -> PassParameters {
        let origin = rng::signed_vec3(self.seed as u32, seed) * 100.0;
        params! {
            "seed" => seed,
            "cumulative" => u32::from(self.cumulative),
            "scale_total" => self.factor * self.distance,
            "shape" => self.shape,
            "scale" => self.scale,
            "scale_along" => self.scale_along_curve,
            "offset_per_curve" => self.offset_per_curve,
            "preserve" => u32::from(self.preserve_length),
            "origin_x" => origin.x,
            "origin_y" => origin.y,
            "origin_z" => origin.z,
            "pad0" => 0u32,
            "pad1" => 0u32,
            "pad2" => 0u32,
        }
    }
}

impl Frizz {
    pub(super) fn gpu_parameters(&self) -> PassParameters {
        params! {
            "seed" => rng::seed(self.seed, 64_785),
            "cumulative" => u32::from(self.cumulative),
            "scale_total" => self.factor * self.distance,
            "shape" => self.shape,
            "share" => self.share,
            "calm" => self.calm,
            "frequency" => self.frequency,
            "preserve" => u32::from(self.preserve_length),
            "pad0" => 0u32,
            "pad1" => 0u32,
        }
    }
}

impl Smooth {
    pub(super) fn gpu_parameters(&self) -> PassParameters {
        params! {
            "iterations" => self.iterations as u32,
            "preserve" => u32::from(self.preserve_length),
            "factor" => self.factor,
            "pad0" => 0u32,
            "pad1" => 0u32,
            "pad2" => 0u32,
        }
    }
}

impl Shrinkwrap {
    pub(super) fn gpu_parameters(&self) -> PassParameters {
        params! {
            "smoothing" => self.smoothing as u32,
            "lock_roots" => u32::from(self.lock_roots),
            "factor" => self.factor,
            "offset" => self.offset,
            "above_surface" => self.above_surface,
            "preserve" => u32::from(self.preserve_length),
        }
    }
}

