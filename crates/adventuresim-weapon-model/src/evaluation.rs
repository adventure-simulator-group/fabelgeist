//! One authenticated construction evaluation reused within an operation.
use crate::*;
mod memo;
use memo::PhysicalMemo;
static WEAPONS: PhysicalMemo<WeaponDesign> = PhysicalMemo::new();
static HOLDERS: PhysicalMemo<WeaponHolderDesign> = PhysicalMemo::new();

/// Validated recipe and physical values computed from its material solids.
/// This owns the design so callers cannot invalidate the evaluation by editing it.
pub struct EvaluatedWeapon {
    design: WeaponDesign,
    derived: DerivedProperties,
}
impl EvaluatedWeapon {
    pub fn new(design: WeaponDesign) -> Result<Self, CodecError> {
        let derived = WEAPONS
            .evaluate(&design, || derive_properties(&design))
            .map_err(CodecError::InvalidDesign)?;
        Ok(Self { design, derived })
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::new(crate::codec::parse_weapon(bytes)?)
    }
    pub fn design(&self) -> &WeaponDesign {
        &self.design
    }
    pub fn derived(&self) -> DerivedProperties {
        self.derived
    }
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        crate::codec::serialize_weapon(&self.design)
    }
}

/// Validated fitted holder and the mass of its hollow material construction.
pub struct EvaluatedHolder {
    design: WeaponHolderDesign,
    derived: DerivedProperties,
}
impl EvaluatedHolder {
    pub fn new(design: WeaponHolderDesign) -> Result<Self, CodecError> {
        let derived = HOLDERS
            .evaluate(&design, || derive_holder_properties(&design))
            .map_err(CodecError::InvalidDesign)?;
        Ok(Self { design, derived })
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::new(crate::codec::parse_holder(bytes)?)
    }
    pub fn design(&self) -> &WeaponHolderDesign {
        &self.design
    }
    pub fn derived(&self) -> DerivedProperties {
        self.derived
    }
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        crate::codec::serialize_holder(&self.design)
    }
}
