//! SpacetimeDB representation of a checked calendar year.

use spacetimedb_lib::{
    AlgebraicType, SpacetimeType,
    de::{self, Deserialize},
    sats::typespace::TypespaceBuilder,
};

use super::CalendarYear;

#[derive(Deserialize)]
#[sats(crate = spacetimedb_lib, name = "CalendarYear")]
struct CalendarYearWire {
    year: i32,
}

impl SpacetimeType for CalendarYear {
    fn make_type<S: TypespaceBuilder>(typespace: &mut S) -> AlgebraicType {
        typespace.add(
            core::any::TypeId::of::<Self>(),
            Some("CalendarYear"),
            |typespace| {
                AlgebraicType::product([(
                    Some("year"),
                    <i32 as SpacetimeType>::make_type(typespace),
                )])
            },
        )
    }
}

impl<'de> Deserialize<'de> for CalendarYear {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = CalendarYearWire::deserialize(deserializer)?;
        Self::new(wire.year).ok_or_else(|| de::Error::custom("calendar year must be positive"))
    }
}
