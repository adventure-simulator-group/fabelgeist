//! Stable route refusals retain synchronized weather provenance.

use adventuresim_world_schema::calendar::StrategicMinute;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouteAdmissionError {
    Gateway(crate::strategic::GatewayAdmissionError),
    TerrainPackageMismatch,
    InvalidDigest,
    InvalidWeather,
    AggregateBounds,
    InvalidCoordinate,
    EndpointMismatch,
    DiscontinuousPath,
    DistanceOverflow,
    DistanceMismatch,
    ExcessSpeed,
    InvalidSkillMetadata,
    DiscontinuousSpans,
    MinutesOverflow,
    MinutesMismatch,
    ReturnRouteRequired,
    StaleWeather {
        departure: StrategicMinute,
        expected_interval: StrategicMinute,
        snapshot_interval: StrategicMinute,
    },
}
impl From<crate::strategic::GatewayAdmissionError> for RouteAdmissionError {
    fn from(source: crate::strategic::GatewayAdmissionError) -> Self {
        Self::Gateway(source)
    }
}
impl std::fmt::Display for RouteAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gateway(source) => source.fmt(f),
            Self::TerrainPackageMismatch => {
                f.write_str("Terrain route does not match the gateway terrain package")
            }
            Self::InvalidDigest => f.write_str("Terrain route has an invalid package digest"),
            Self::InvalidWeather => {
                f.write_str("Terrain route has an invalid weather departure snapshot")
            }
            Self::AggregateBounds => {
                f.write_str("Terrain route exceeds its collection or aggregate bounds")
            }
            Self::InvalidCoordinate => f.write_str("Terrain route contains an invalid coordinate"),
            Self::EndpointMismatch => {
                f.write_str("Terrain route endpoints do not match the current journey")
            }
            Self::DiscontinuousPath => {
                f.write_str("Terrain route points are not a bounded continuous path")
            }
            Self::DistanceOverflow => f.write_str("Terrain route distance overflow"),
            Self::DistanceMismatch => {
                f.write_str("Terrain route distance does not match its geometry")
            }
            Self::ExcessSpeed => {
                f.write_str("Terrain route duration is faster than the maximum travel speed")
            }
            Self::InvalidSkillMetadata => {
                f.write_str("Terrain route span has invalid bounded skill metadata")
            }
            Self::DiscontinuousSpans => f.write_str("Terrain route spans are discontinuous"),
            Self::MinutesOverflow => f.write_str("Terrain route minutes overflow"),
            Self::MinutesMismatch => {
                f.write_str("Terrain route spans do not match aggregate minutes")
            }
            Self::ReturnRouteRequired => {
                f.write_str("Quest travel requires an independently planned return route")
            }
            Self::StaleWeather { .. } => {
                f.write_str("Terrain route weather snapshot is stale after clock synchronization")
            }
        }
    }
}
impl std::error::Error for RouteAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Gateway(source) => Some(source),
            _ => None,
        }
    }
}
