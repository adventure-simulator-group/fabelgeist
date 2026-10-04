//! Custody admission retains identity causes and the failed stored authority.

use adventuresim_core::{
    identity::CharacterId,
    physical_object::{CustodyIdentityError, InventoryLocation, OperationalCustody},
    strategic_place::{PlaceIdentityError, StrategicFixtureId},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CustodyPlacePurpose {
    PersistedPlace,
    PersistedFixture,
    Fireplace,
    Repair,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackingFailure {
    CharacterUnavailable,
    PersonalRowMissing,
    PersonalRowMismatch,
    PartyUnavailable,
    PartyRowMissing,
    PartyRowMismatch,
    RepairRowMissing,
    RepairOrderMissing,
    RepairRowMismatch,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ObjectCustodyError {
    Identity(CustodyIdentityError),
    PlaceIdentity {
        purpose: CustodyPlacePurpose,
        source: PlaceIdentityError,
    },
    CharacterHasNoParty(CharacterId),
    PartyUnavailable(super::PersistedOperationalCustody),
    NotCarried(InventoryLocation),
    ActorDestinationMismatch {
        expected: CharacterId,
        actual: OperationalCustody,
    },
    NotCarriedDestination(OperationalCustody),
    Backing {
        location: InventoryLocation,
        failure: BackingFailure,
    },
    NotFireplace(StrategicFixtureId),
    BackingMultiplicity,
    DepthOverflow,
    DepthExceeded,
    ContainmentCycle,
    ParentMissing,
    ConflictingCarriedCustody,
    OutsideActorCustody {
        actor: CharacterId,
        actual: OperationalCustody,
    },
    FixturePlaceMismatch {
        expected: StrategicFixtureId,
        actual: StrategicFixtureId,
    },
    FixtureMismatch {
        expected: StrategicFixtureId,
        actual: StrategicFixtureId,
    },
    NotFixture(OperationalCustody),
}

impl From<CustodyIdentityError> for ObjectCustodyError {
    fn from(source: CustodyIdentityError) -> Self {
        Self::Identity(source)
    }
}

impl std::fmt::Display for ObjectCustodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identity(source) => source.fmt(f),
            Self::PlaceIdentity { purpose, source } => {
                let context = match purpose {
                    CustodyPlacePurpose::PersistedPlace => {
                        "Custody place identity is not canonical"
                    }
                    CustodyPlacePurpose::PersistedFixture => {
                        "Custody fixture identity is not canonical"
                    }
                    CustodyPlacePurpose::Fireplace => {
                        "Inventory object fireplace fixture is not canonical"
                    }
                    CustodyPlacePurpose::Repair => "Inventory object repair place is not canonical",
                };
                write!(f, "{context}: {source}")
            }
            Self::CharacterHasNoParty(actor) => {
                write!(f, "Character has no party inventory ({actor})")
            }
            Self::PartyUnavailable(custody) => {
                write!(f, "Party inventory custody is unavailable ({custody:?})")
            }
            Self::NotCarried(location) => {
                write!(f, "Custody is not a carried inventory ({location:?})")
            }
            Self::ActorDestinationMismatch { expected, actual } => write!(
                f,
                "Personal custody conflicts with the acting character {expected} ({actual:?})"
            ),
            Self::NotCarriedDestination(custody) => write!(
                f,
                "Custody is not a carried inventory destination ({custody:?})"
            ),
            Self::Backing { location, failure } => {
                let context = match failure {
                    BackingFailure::CharacterUnavailable => {
                        "Inventory object character custody is unavailable"
                    }
                    BackingFailure::PersonalRowMissing => {
                        "Inventory object personal row is missing"
                    }
                    BackingFailure::PersonalRowMismatch => {
                        "Inventory object conflicts with its personal row custody"
                    }
                    BackingFailure::PartyUnavailable => {
                        "Inventory object party custody is unavailable"
                    }
                    BackingFailure::PartyRowMissing => "Inventory object party row is missing",
                    BackingFailure::PartyRowMismatch => {
                        "Inventory object conflicts with its party row custody"
                    }
                    BackingFailure::RepairRowMissing => "Inventory object repair row is missing",
                    BackingFailure::RepairOrderMissing => {
                        "Inventory object repair order is missing"
                    }
                    BackingFailure::RepairRowMismatch => {
                        "Inventory object conflicts with its repair escrow custody"
                    }
                };
                write!(f, "{context} ({location:?})")
            }
            Self::NotFireplace(fixture) => write!(
                f,
                "Inventory object fireplace custody names another fixture kind ({fixture})"
            ),
            Self::BackingMultiplicity => {
                f.write_str("Inventory row must have exactly one physical object identity")
            }
            Self::DepthOverflow => f.write_str("Inventory containment custody depth overflow"),
            Self::DepthExceeded => {
                f.write_str("Inventory containment custody exceeds the maximum depth")
            }
            Self::ContainmentCycle => f.write_str("Inventory containment custody contains a cycle"),
            Self::ParentMissing => f.write_str("Inventory containment parent object is missing"),
            Self::ConflictingCarriedCustody => {
                f.write_str("Contained objects have conflicting carried custody")
            }
            Self::OutsideActorCustody { actor, actual } => write!(
                f,
                "Physical object is outside the actor's exact carried custody (actor {actor}, custody {actual:?})"
            ),
            Self::FixturePlaceMismatch { expected, actual } => write!(
                f,
                "Object fixture custody conflicts with the expected place: expected {expected}, found {actual}"
            ),
            Self::FixtureMismatch { expected, actual } => write!(
                f,
                "Object custody names another fixture at this place: expected {expected}, found {actual}"
            ),
            Self::NotFixture(custody) => {
                write!(f, "Object is not in exact fixture custody ({custody:?})")
            }
        }
    }
}

impl std::error::Error for ObjectCustodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Identity(source) => Some(source),
            Self::PlaceIdentity { source, .. } => Some(source),
            _ => None,
        }
    }
}
