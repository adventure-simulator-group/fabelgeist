//! The profile node changes strand attributes in place.
use super::*;
use crate::ops::Profile;
impl GpuGroomer {
    pub(super) fn record_profile(
        &self,
        batch: &mut KernelBatch,
        set: &StrandSet,
        r: &Profile,
    ) -> Result<()> {
        let count = set.layout.native_strands();
        batch.dispatch_folded(
            &self.kernels.profile,
            &params! {
                "strands" => set.attributes.clone(),
                "count" => count,
                "seed" => rng::seed(r.seed, 51_823),
                "relative" => u32::from(r.relative_to.is_some()),
                "pad0" => 0u32,
                "radius" => r.radius,
                "random" => r.random,
                "grown" => r.relative_to.unwrap_or(0.0),
                "pad1" => 0u32,
            },
            set.layout.strand_invocations(),
        )?;
        Ok(())
    }
}

impl GpuGroomer {
    pub(super) fn record_guide_pack(
        &self,
        batch: &mut KernelBatch,
        guides: &super::super::guides::GpuGuides,
        guide_points: &Buffer,
        guide_info: &Buffer,
        parting: &Parting,
    ) -> Result<()> {
        let mut pack = params! {
            "positions" => guides.positions.clone(),
            "strands" => guides.attributes.clone(),
            "guide_points" => guide_points.clone(),
            "guide_info" => guide_info.clone(),
            "count" => guides.layout.native_strands(),
            "points" => guides.layout.native_points(),
            "pad0" => 0u32,
        };
        for (key, value) in parting_parameters(parting) {
            pack.insert(key, value);
        }
        batch.dispatch_folded(&self.kernels.guide_pack, &pack, guides.layout.strand_invocations())?;
        Ok(())
    }
}
