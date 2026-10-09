//! Read-only settlement scene documents for focused geographic presentation.
//! This admits static source data, not strategic discovery or tactical state.
use crate::scene_input::{SceneInputError, SceneSource, TacticalSceneInput};
use adventuresim_core::strategic_place::StrategicPlaceId;
use adventuresim_world_schema::{
    coordinates::Wgs84CoordinateE7, source_package::SourcePackageDigest,
};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, RegionalCityError>;

/// The exact capture centre remains in E7 units, before microdegree rounding.
/// JavaScript carries the complete document as text so scene seeds stay exact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RegionalCityWire", deny_unknown_fields)]
pub struct RegionalCityInput {
    source: SourcePackageDigest,
    place: StrategicPlaceId,
    origin: Wgs84CoordinateE7,
    input: Box<TacticalSceneInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalCityWire {
    source: SourcePackageDigest,
    place: StrategicPlaceId,
    origin: Wgs84CoordinateE7,
    input: Box<TacticalSceneInput>,
}

#[derive(Debug, thiserror::Error)]
pub enum RegionalCityError {
    #[error("A focused city must identify a settlement")]
    Place,
    #[error("City input belongs to another terrain source")]
    Source,
    #[error("City input belongs to another geographic origin")]
    Origin,
    #[error("City property catalog belongs to another settlement")]
    Settlement,
    #[error(transparent)]
    Scene(#[from] Box<SceneInputError>),
}

impl RegionalCityInput {
    pub fn from_scene(
        source: SourcePackageDigest,
        place: StrategicPlaceId,
        origin: Wgs84CoordinateE7,
        input: TacticalSceneInput,
    ) -> Result<Self> {
        RegionalCityWire {
            source,
            place,
            origin,
            input: Box::new(input),
        }
        .try_into()
    }

    pub fn source(&self) -> &SourcePackageDigest {
        &self.source
    }

    pub fn place(&self) -> &StrategicPlaceId {
        &self.place
    }

    pub fn origin(&self) -> Wgs84CoordinateE7 {
        self.origin
    }

    pub fn input(&self) -> &TacticalSceneInput {
        &self.input
    }
}

impl TryFrom<RegionalCityWire> for RegionalCityInput {
    type Error = RegionalCityError;

    fn try_from(wire: RegionalCityWire) -> Result<Self> {
        let StrategicPlaceId::Settlement { settlement_id } = &wire.place else {
            return Err(RegionalCityError::Place);
        };
        wire.input.validate()?;
        if wire.input.source != SceneSource::ImportedPackage(wire.source.clone()) {
            return Err(RegionalCityError::Source);
        }
        if wire.input.latitude_microdegrees != wire.origin.latitude().to_microdegrees()
            || wire.input.longitude_microdegrees != wire.origin.longitude().to_microdegrees()
        {
            return Err(RegionalCityError::Origin);
        }
        if wire
            .input
            .properties
            .as_ref()
            .is_none_or(|catalog| catalog.settlement_id != settlement_id.as_str())
        {
            return Err(RegionalCityError::Settlement);
        }
        Ok(Self {
            source: wire.source,
            place: wire.place,
            origin: wire.origin,
            input: wire.input,
        })
    }
}

impl From<SceneInputError> for RegionalCityError {
    fn from(error: SceneInputError) -> Self {
        Self::Scene(Box::new(error))
    }
}
