//! Signed displacement along a property frontage, distinct from its scene translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::city_layout::packing) struct FrontageDisplacement(f64);
impl FrontageDisplacement {
    pub(in crate::city_layout::packing) fn from_metres(metres: f64) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub(in crate::city_layout::packing) fn metres(self) -> f64 {
        self.0
    }
}
