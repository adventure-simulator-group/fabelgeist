//! Persisted fireplace admission distinguishes fixture and custody failures.

use crate::object_custody::ObjectCustodyError;
use adventuresim_core::strategic_place::PlaceIdentityError;

#[derive(Debug)]
pub(crate) enum FireplaceCustodyError {
    Fixture(PlaceIdentityError),
    Custody(Box<ObjectCustodyError>),
    NonFireplaceFixture,
    StationKeyMismatch,
    AmbiguousReturnCustody,
    MissingStationObject,
    StationObjectMismatch,
    MissingDishStation,
    DishStationMismatch,
    NotJourneyCamp,
    OccupiedCamp,
}

impl From<ObjectCustodyError> for FireplaceCustodyError {
    fn from(source: ObjectCustodyError) -> Self {
        Self::Custody(Box::new(source))
    }
}

impl std::fmt::Display for FireplaceCustodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixture(_) => {
                f.write_str("Persisted fireplace custody has an invalid canonical fixture")
            }
            Self::Custody(source) => source.fmt(f),
            Self::NonFireplaceFixture => {
                f.write_str("Persisted fireplace custody names a non-fireplace fixture")
            }
            Self::StationKeyMismatch => {
                f.write_str("Persisted fireplace station conflicts with its canonical fixture")
            }
            Self::AmbiguousReturnCustody => {
                f.write_str("Persisted fireplace station has ambiguous return custody")
            }
            Self::MissingStationObject => {
                f.write_str("Persisted fireplace station object is missing")
            }
            Self::StationObjectMismatch => {
                f.write_str("Persisted fireplace station conflicts with its object identity")
            }
            Self::MissingDishStation => {
                f.write_str("Persisted fireplace dish has no station authority")
            }
            Self::DishStationMismatch => {
                f.write_str("Persisted fireplace dish conflicts with its station authority")
            }
            Self::NotJourneyCamp => f.write_str("Camp custody gate requires an exact journey camp"),
            Self::OccupiedCamp => f.write_str(
                "Retrieve every dish and remove every cooking instrument before breaking camp",
            ),
        }
    }
}

impl std::error::Error for FireplaceCustodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fixture(source) => Some(source),
            Self::Custody(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}
