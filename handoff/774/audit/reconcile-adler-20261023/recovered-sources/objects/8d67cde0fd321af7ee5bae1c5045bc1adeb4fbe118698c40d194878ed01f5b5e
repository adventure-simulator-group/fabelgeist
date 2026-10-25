//! Investigation methods and action environments at storage boundaries.
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Canonical methods for capability storage, generated bindings, and execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationActionKind {
    InspectSite,
    SearchArea,
    FollowTracks,
    ReacquireTracks,
    LocateContact,
    Watch,
    Patrol,
    LayAmbush,
    ApproachLead,
}

impl InvestigationActionKind {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::InspectSite => "inspect_site",
            Self::SearchArea => "search_area",
            Self::FollowTracks => "follow_tracks",
            Self::ReacquireTracks => "reacquire_tracks",
            Self::LocateContact => "locate_contact",
            Self::Watch => "watch",
            Self::Patrol => "patrol",
            Self::LayAmbush => "lay_ambush",
            Self::ApproachLead => "approach_lead",
        }
    }
}

/// The supplied key is outside the canonical InvestigationActionKind vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseInvestigationActionKindError;

impl fmt::Display for ParseInvestigationActionKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Unknown investigation action method")
    }
}

impl std::error::Error for ParseInvestigationActionKindError {}

impl FromStr for InvestigationActionKind {
    type Err = ParseInvestigationActionKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseInvestigationActionKindError)
    }
}

/// Action environment/difficulty, distinct from terrain-pack surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    Road,
    Settlement,
    Plains,
    Forest,
    Hills,
    Marsh,
    Ruins,
    Underground,
}

impl Terrain {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Road => "road",
            Self::Settlement => "settlement",
            Self::Plains => "plains",
            Self::Forest => "forest",
            Self::Hills => "hills",
            Self::Marsh => "marsh",
            Self::Ruins => "ruins",
            Self::Underground => "underground",
        }
    }
}

/// The supplied key is outside the canonical Terrain vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseTerrainError;

impl fmt::Display for ParseTerrainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Unknown investigation terrain")
    }
}

impl std::error::Error for ParseTerrainError {}

impl FromStr for Terrain {
    type Err = ParseTerrainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseTerrainError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investigation_action::{InvestigationTargetKind, tracking_route_edge_is_coherent};

    #[test]
    fn capability_methods_round_trip_through_storage_and_serde() {
        for method in [
            InvestigationActionKind::InspectSite,
            InvestigationActionKind::SearchArea,
            InvestigationActionKind::FollowTracks,
            InvestigationActionKind::ReacquireTracks,
            InvestigationActionKind::LocateContact,
            InvestigationActionKind::Watch,
            InvestigationActionKind::Patrol,
            InvestigationActionKind::LayAmbush,
            InvestigationActionKind::ApproachLead,
        ] {
            assert_eq!(method.stable_id().parse(), Ok(method));
            let json = serde_json::to_string(&method).unwrap();
            assert_eq!(json, format!("\"{}\"", method.stable_id()));
            assert_eq!(
                serde_json::from_str::<InvestigationActionKind>(&json).unwrap(),
                method
            );
        }
    }

    #[test]
    fn action_environments_round_trip_through_storage_and_serde() {
        for terrain in [
            Terrain::Road,
            Terrain::Settlement,
            Terrain::Plains,
            Terrain::Forest,
            Terrain::Hills,
            Terrain::Marsh,
            Terrain::Ruins,
            Terrain::Underground,
        ] {
            assert_eq!(terrain.stable_id().parse(), Ok(terrain));
            let json = serde_json::to_string(&terrain).unwrap();
            assert_eq!(json, format!("\"{}\"", terrain.stable_id()));
            assert_eq!(serde_json::from_str::<Terrain>(&json).unwrap(), terrain);
        }
    }

    #[test]
    fn malformed_storage_keys_are_rejected_before_tracking_admission() {
        for key in [
            "",
            "unknown",
            "FollowTracks",
            "follow-tracks",
            " follow_tracks",
        ] {
            assert_eq!(
                key.parse::<InvestigationActionKind>(),
                Err(ParseInvestigationActionKindError)
            );
        }
        for key in ["", "unknown", "Road", "Wetlands", "road "] {
            assert_eq!(key.parse::<Terrain>(), Err(ParseTerrainError));
        }
        let follow = "follow_tracks".parse().unwrap();
        let reacquire = "reacquire_tracks".parse().unwrap();
        assert!(tracking_route_edge_is_coherent(
            follow,
            InvestigationTargetKind::Site,
            reacquire,
            InvestigationTargetKind::Tracks
        ));
        assert!(!tracking_route_edge_is_coherent(
            follow,
            InvestigationTargetKind::Site,
            "watch".parse().unwrap(),
            InvestigationTargetKind::Contact
        ));
    }
}
