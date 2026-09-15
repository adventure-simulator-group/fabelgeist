//! Painted tonal modeling is separate from illumination and metal reflectance.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintRole {
    Field,
    Charge,
    Accent,
    Shadow,
    Highlight,
}
impl PaintRole {
    pub fn is_charge(self) -> bool {
        matches!(self, Self::Charge | Self::Shadow | Self::Highlight)
    }
    pub fn tone(self) -> PaintTone {
        match self {
            Self::Shadow => PaintTone::Shadow,
            Self::Highlight => PaintTone::Highlight,
            _ => PaintTone::Base,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintTone {
    Base,
    Shadow,
    Highlight,
}
impl PaintTone {
    pub const ALL: [Self; 3] = [Self::Base, Self::Shadow, Self::Highlight];
    pub fn index(self) -> usize {
        self as usize
    }
}
