//! Longitudinal blade plans use the shared transverse section and closed loft.
use super::*;
pub(super) fn blade(p: &LoftedBladeParameters, detail: Detail) -> Result<Solid, String> {
    blade_sections::blade(
        BladeProfile::from(p),
        blade_sections::BladeSampling::Loft(detail.samples(p.samples.0 as usize, 4)),
        detail,
    )
}
