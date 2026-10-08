//! Ordered physical membership rejects duplicates without canonicalizing input.
use crate::scene_input::SceneBuildingId;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, bevy::reflect::Reflect)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct PropertyMembers(Vec<SceneBuildingId>);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, thiserror::Error)]
pub enum PropertyMemberIssue {
    #[error("property has no physical members")]
    Empty,
    #[error("property member {index} has zero physical identity")]
    Zero { index: usize },
    #[error("building {building} is repeated at member {index}, first at {first}")]
    Duplicate {
        building: SceneBuildingId,
        first: usize,
        index: usize,
    },
}
impl PropertyMembers {
    pub fn new(ids: Vec<SceneBuildingId>) -> Result<Self, PropertyMemberIssue> {
        Self::validate(&ids)?;
        Ok(Self(ids))
    }
    pub fn ids(&self) -> &[SceneBuildingId] {
        &self.0
    }
    pub(super) fn validate(ids: &[SceneBuildingId]) -> Result<(), PropertyMemberIssue> {
        if ids.is_empty() {
            return Err(PropertyMemberIssue::Empty);
        }
        for (index, building) in ids.iter().enumerate() {
            if building.0 == 0 {
                return Err(PropertyMemberIssue::Zero { index });
            }
            if let Some(first) = ids[..index].iter().position(|id| id == building) {
                return Err(PropertyMemberIssue::Duplicate {
                    building: *building,
                    first,
                    index,
                });
            }
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for PropertyMembers {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(Vec::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
