//! Map preparation and native dispatch for lock-following operations.

use super::*;

impl GpuGroomer {
    pub(super) fn record_clump(
        &mut self,
        batch: &mut KernelBatch,
        set: &mut StrandSet,
        bindings: &OperationBindings,
        clump: &Clump,
    ) -> Result<()> {
        let seed = rng::seed(clump.seed, 148_762);
        if !(clump.reuse_guide_map && set.mapped) {
            self.record_set_map(batch, set, clump.guide_distance, clump.guide_mask, seed)?;
        }
        let mut parameters = clump.gpu_parameters(seed);
        parameters.insert("guide_of", set.guide_of.clone());
        bindings.record(batch, &self.kernels.clump, parameters)
    }

    pub(super) fn record_curl(
        &mut self,
        batch: &mut KernelBatch,
        set: &mut StrandSet,
        bindings: &OperationBindings,
        curl: &Curl,
    ) -> Result<()> {
        let seed = rng::seed(curl.seed, 91_487);
        if !(curl.reuse_guide_map && set.mapped) {
            self.record_set_map(batch, set, curl.guide_distance, curl.guide_mask, seed)?;
        }
        let mut parameters = curl.gpu_parameters(seed);
        parameters.insert("guide_of", set.guide_of.clone());
        bindings.record(batch, &self.kernels.curl, parameters)
    }

    pub(super) fn record_wave(
        &mut self,
        batch: &mut KernelBatch,
        set: &mut StrandSet,
        bindings: &OperationBindings,
        wave: &Wave,
    ) -> Result<()> {
        let seed = rng::seed(wave.seed, 0x3A7E);
        if !(wave.reuse_guide_map && set.mapped) {
            self.record_set_map(batch, set, wave.guide_distance, 1.0, seed)?;
        }
        let mut parameters = wave.gpu_parameters(seed);
        parameters.insert("guide_of", set.guide_of.clone());
        bindings.record(batch, &self.kernels.wave, parameters)
    }

}
