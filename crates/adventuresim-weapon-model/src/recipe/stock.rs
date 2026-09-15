//! Stock identities currently traded by gameplay forges.
use super::Material;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForgeStock {
    Steel,
    Leather,
    Brass,
    Wood,
}
impl ForgeStock {
    pub const fn item_id(self) -> &'static str {
        match self {
            Self::Steel => "steel_stock",
            Self::Leather => "leather_stock",
            Self::Brass => "brass_stock",
            Self::Wood => "wood_stock",
        }
    }
    pub const fn unit_mass_kg(self) -> f32 {
        match self {
            Self::Steel => 5.0,
            Self::Leather | Self::Brass => 2.0,
            Self::Wood => 4.0,
        }
    }
}
impl Material {
    /// Materials without a traded stock remain valid authoring materials.
    pub const fn forge_stock(self) -> Option<ForgeStock> {
        match self {
            Self::Steel | Self::DarkSteel => Some(ForgeStock::Steel),
            Self::Leather | Self::DarkLeather => Some(ForgeStock::Leather),
            Self::Brass => Some(ForgeStock::Brass),
            Self::Wood => Some(ForgeStock::Wood),
            _ => None,
        }
    }
}
