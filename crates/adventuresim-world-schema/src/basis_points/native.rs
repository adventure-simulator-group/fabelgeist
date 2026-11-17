//! SpacetimeDB representations of the checked basis-point values.

use spacetimedb_lib::{
    AlgebraicType, SpacetimeType,
    de::{self, Deserialize},
    sats::typespace::TypespaceBuilder,
};

use super::{SignedUnitBasisPoints, UnitBasisPoints};

#[derive(Deserialize)]
#[sats(crate = spacetimedb_lib, name = "UnitBasisPoints")]
struct UnitBasisPointsWire {
    basis_points: u16,
}

#[derive(Deserialize)]
#[sats(crate = spacetimedb_lib, name = "SignedUnitBasisPoints")]
struct SignedUnitBasisPointsWire {
    basis_points: i16,
}

impl SpacetimeType for UnitBasisPoints {
    fn make_type<S: TypespaceBuilder>(typespace: &mut S) -> AlgebraicType {
        typespace.add(
            core::any::TypeId::of::<Self>(),
            Some("UnitBasisPoints"),
            |typespace| {
                AlgebraicType::product([(
                    Some("basis_points"),
                    <u16 as SpacetimeType>::make_type(typespace),
                )])
            },
        )
    }
}

impl SpacetimeType for SignedUnitBasisPoints {
    fn make_type<S: TypespaceBuilder>(typespace: &mut S) -> AlgebraicType {
        typespace.add(
            core::any::TypeId::of::<Self>(),
            Some("SignedUnitBasisPoints"),
            |typespace| {
                AlgebraicType::product([(
                    Some("basis_points"),
                    <i16 as SpacetimeType>::make_type(typespace),
                )])
            },
        )
    }
}

impl<'de> Deserialize<'de> for UnitBasisPoints {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = UnitBasisPointsWire::deserialize(deserializer)?;
        Self::new(wire.basis_points)
            .ok_or_else(|| de::Error::custom("unit basis points are out of range"))
    }
}

impl<'de> Deserialize<'de> for SignedUnitBasisPoints {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = SignedUnitBasisPointsWire::deserialize(deserializer)?;
        Self::new(wire.basis_points)
            .ok_or_else(|| de::Error::custom("signed unit basis points are out of range"))
    }
}
