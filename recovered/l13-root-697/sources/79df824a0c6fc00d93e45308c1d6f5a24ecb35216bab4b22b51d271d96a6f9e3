//! Admission of the guide interpolation boundary in a groom stack.
use super::*;
pub(super) struct Interpolation<'a> {
    pub(super) at: usize,
    pub(super) interpolate: &'a Interpolate,
}
impl<'a> Interpolation<'a> {
    pub(super) fn find(groom: &'a Groom) -> Result<Self> {
        groom
            .ops
            .iter()
            .enumerate()
            .find_map(|(at, op)| {
                if let Op::Interpolate(interpolate) = op {
                    Some(Self { at, interpolate })
                } else {
                    None
                }
            })
            .ok_or(HairGpuError::MissingInterpolation)
    }
}
