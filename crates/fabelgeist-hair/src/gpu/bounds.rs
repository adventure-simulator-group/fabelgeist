//! Native measurement of a groom's point bounds.

use super::error::Result;
use super::grow::params;
use super::{GpuGroom, GpuGroomer};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_math::Vec3;

impl GpuGroomer {
    /// The box every point of `groom` is in, measured on the GPU.
    pub fn measure_bounds(&mut self, groom: &GpuGroom) -> Result<[Vec3; 2]> {
        groom.ensure_device(&self.context)?;
        if groom.layout.native_samples() == 0 {
            return Ok([Vec3::default(); 2]);
        }
        let context = self.context.clone();
        let words = self
            .scratch
            .box_words
            .fit(&context, 32_u64.into(), "hair box".into())?;
        let mut batch = KernelBatch::labelled(&context, "hair box");
        batch.clear_buffer(&words);
        let count = groom.layout.native_samples();
        batch.dispatch_folded(
            &self.kernels.bounds,
            &params! {
                "positions" => groom.positions.clone(),
                "box_words" => words.clone(),
                "count" => count,
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            groom.layout.sample_invocations(),
        )?;
        let read: Vec<(Buffer, u32)> = (0..6).map(|k| (words.clone(), k)).collect();
        let values = self.read_words(batch, &read)?;
        // Back from ordered words: a set top bit was a positive float.
        let float = |key: u32| {
            f32::from_bits(if key & 0x8000_0000 != 0 {
                key & 0x7FFF_FFFF
            } else {
                !key
            })
        };
        let v = |k: usize, least: bool| {
            if least {
                float(!values[k])
            } else {
                float(values[k])
            }
        };
        Ok([
            Vec3::new(v(0, true), v(1, true), v(2, true)),
            Vec3::new(v(3, false), v(4, false), v(5, false)),
        ])
    }
}
