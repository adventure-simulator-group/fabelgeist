//! One immutable terrain window and its canonical geographic connections.
use crate::regional_terrain::RegionalTerrain;
use adventuresim_world_schema::{
    coordinates::Wgs84CoordinateE7, regional_connection::RegionalConnectionKind,
};
use serde::{Deserialize, Serialize};

pub const MAX_REGIONAL_CONNECTIONS: usize = 8_192;
pub const MAX_REGIONAL_CONNECTION_POINTS: usize = 65_536;
pub type Result<T> = std::result::Result<T, RegionalEnvironmentError>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RegionalEnvironmentWire", deny_unknown_fields)]
pub struct RegionalEnvironment {
    terrain: RegionalTerrain,
    connections: Vec<RegionalConnection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RegionalConnectionWire", deny_unknown_fields)]
pub struct RegionalConnection {
    kind: RegionalConnectionKind,
    points: Vec<Wgs84CoordinateE7>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalEnvironmentWire {
    terrain: RegionalTerrain,
    connections: Vec<RegionalConnection>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalConnectionWire {
    kind: RegionalConnectionKind,
    points: Vec<Wgs84CoordinateE7>,
}

#[derive(Debug, thiserror::Error)]
pub enum RegionalEnvironmentError {
    #[error("regional connections exceed their bounded line or point capacity")]
    Capacity,
    #[error("regional connection requires at least two distinct positions")]
    Geometry,
}

impl RegionalEnvironment {
    pub fn new(terrain: RegionalTerrain, connections: Vec<RegionalConnection>) -> Result<Self> {
        let points = connections.iter().try_fold(0_usize, |total, line| {
            total
                .checked_add(line.points.len())
                .ok_or(RegionalEnvironmentError::Capacity)
        })?;
        if connections.len() > MAX_REGIONAL_CONNECTIONS || points > MAX_REGIONAL_CONNECTION_POINTS {
            return Err(RegionalEnvironmentError::Capacity);
        }
        Ok(Self {
            terrain,
            connections,
        })
    }

    pub fn terrain(&self) -> &RegionalTerrain {
        &self.terrain
    }
    pub fn connections(&self) -> &[RegionalConnection] {
        &self.connections
    }
}

impl RegionalConnection {
    pub fn new(kind: RegionalConnectionKind, mut points: Vec<Wgs84CoordinateE7>) -> Result<Self> {
        if points.len() > MAX_REGIONAL_CONNECTION_POINTS {
            return Err(RegionalEnvironmentError::Capacity);
        }
        points.dedup();
        if points.len() < 2 {
            return Err(RegionalEnvironmentError::Geometry);
        }
        Ok(Self { kind, points })
    }

    pub fn kind(&self) -> RegionalConnectionKind {
        self.kind
    }
    pub fn points(&self) -> &[Wgs84CoordinateE7] {
        &self.points
    }
}

impl TryFrom<RegionalEnvironmentWire> for RegionalEnvironment {
    type Error = RegionalEnvironmentError;
    fn try_from(wire: RegionalEnvironmentWire) -> Result<Self> {
        Self::new(wire.terrain, wire.connections)
    }
}

impl TryFrom<RegionalConnectionWire> for RegionalConnection {
    type Error = RegionalEnvironmentError;
    fn try_from(wire: RegionalConnectionWire) -> Result<Self> {
        Self::new(wire.kind, wire.points)
    }
}
