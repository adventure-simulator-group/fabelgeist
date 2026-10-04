//! Longitudinal blade plans use the shared transverse section and closed loft.
use super::*;
use crate::ConstructionError;
pub(super) fn blade(p: &LoftedBladeParameters, detail: Detail) -> Result<Solid, ConstructionError> {
    blade_sections::blade(
        BladeProfile::from(p),
        blade_sections::BladeSampling::Loft(detail.samples(p.samples.0 as usize, 4)),
        detail,
    )
}
