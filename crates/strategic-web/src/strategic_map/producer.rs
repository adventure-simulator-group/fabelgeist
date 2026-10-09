//! Admit generated strategic records to the observer's geographic presentation.
use super::markers::MapLink;
use crate::{
    routes::travel::CaseSiteKnowledgePresentation,
    spacetimedb::{BackendCaseSitePin, SettlementView},
};
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength};
use adventuresim_core::strategic_place::{PlaceIdentityError, StrategicPlaceId};
use adventuresim_tactical_core::{
    regional_map::{
        MapMarker, MapMarkerEmphasis, MapMarkerRank, MapOverlay, MapOverlayError, MapRoute,
        MapRouteKind, MapScaleError, MapSpan,
    },
    scene_input::{SceneValidationError, SourcePackageDigest},
};
use adventuresim_terrain::{RoutePlan, TerrainPack};
use adventuresim_world_schema::coordinates::{Wgs84CoordinateE7, Wgs84CoordinateMicrodegrees};
use serde::Serialize;
use std::collections::BTreeSet;

const INITIAL_MAP_SPAN_METRES: f32 = 60_000.0;
type Result<T> = std::result::Result<T, MapBuildError>;

/// Generated database records and the selected HTTP query enter at this port.
pub(super) struct MapLocations<'a> {
    pub settlements: &'a [SettlementView],
    pub case_sites: &'a [BackendCaseSitePin],
    pub current: &'a SettlementView,
    pub connected: &'a BTreeSet<&'a str>,
    pub selected: Option<&'a str>,
    pub route: Option<&'a RoutePlan>,
}

#[derive(Serialize)]
pub(super) struct MapInput {
    pub origin: Wgs84CoordinateMicrodegrees,
    pub span: MapSpan,
    pub selected: Option<StrategicPlaceId>,
    pub overlay: MapOverlay,
}

pub(super) struct MapPresentation {
    pub input: MapInput,
    pub links: Vec<MapLink>,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum MapBuildError {
    #[error("the current place has no geographic position in the terrain source")]
    Origin,
    #[error("the selected route contains an invalid geographic coordinate")]
    RouteCoordinate,
    #[error(transparent)]
    Identity(#[from] PlaceIdentityError),
    #[error(transparent)]
    Source(#[from] SceneValidationError),
    #[error(transparent)]
    Overlay(#[from] MapOverlayError),
    #[error(transparent)]
    Scale(#[from] MapScaleError),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl MapPresentation {
    pub fn capture(pack: &TerrainPack, locations: MapLocations<'_>) -> Result<Self> {
        Self::from_native_source(
            SourcePackageDigest::from_hex(pack.digest())?,
            pack.bounds(),
            locations,
        )
    }

    /// Native imported-source port: bounds are TerrainPack's west/south/east/
    /// north degrees. Only coordinate coverage arithmetic consumes this array.
    pub(super) fn from_native_source(
        source: SourcePackageDigest,
        bounds: [f64; 4],
        locations: MapLocations<'_>,
    ) -> Result<Self> {
        let origin = settlement_position(locations.current, bounds).ok_or(MapBuildError::Origin)?;
        let mut links = Vec::new();
        for settlement in locations.settlements {
            let Some(position) = settlement_position(settlement, bounds) else {
                continue;
            };
            let marker = MapMarker {
                place: StrategicPlaceId::settlement(&settlement.id)?,
                origin: position,
                rank: MapLink::settlement_rank(&settlement.category),
                emphasis: locations.settlement_emphasis(settlement),
            };
            links.push(MapLink::settlement(marker, settlement));
        }
        for site in locations.case_sites {
            let Some(knowledge) = CaseSiteKnowledgePresentation::from_stage(site.knowledge_stage)
            else {
                continue;
            };
            let Some(position) = case_position(site, bounds) else {
                continue;
            };
            let marker = MapMarker {
                place: StrategicPlaceId::case_site(&site.case_site_id.value)?,
                origin: position,
                rank: MapMarkerRank::CaseSite,
                emphasis: locations.case_emphasis(site),
            };
            links.push(MapLink::case_site(marker, site, knowledge));
        }
        links.sort_by_key(|link| std::cmp::Reverse(link.priority()));
        let selected = links.iter().find(|link| {
            matches!(
                link.marker.emphasis,
                MapMarkerEmphasis::Selected | MapMarkerEmphasis::CurrentSelected
            )
        });
        let selected_place = selected.map(|link| link.marker.place.clone());
        let route = match selected {
            Some(link) if link.marker.origin != origin => Some(match locations.route {
                Some(route) => MapRoute::new(
                    MapRouteKind::Computed,
                    route
                        .points
                        .iter()
                        .map(|point| {
                            Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
                                point.longitude,
                                point.latitude,
                            )
                            .ok_or(MapBuildError::RouteCoordinate)
                        })
                        .collect::<Result<Vec<_>>>()?,
                )?,
                None => MapRoute::new(MapRouteKind::Estimate, vec![origin, link.marker.origin])?,
            }),
            _ => None,
        };
        let markers = links.iter().map(|link| link.marker.clone()).collect();
        Ok(Self {
            input: MapInput {
                origin,
                span: MapSpan::try_from(PositiveLength::from_metres(INITIAL_MAP_SPAN_METRES)?)?,
                selected: selected_place,
                overlay: MapOverlay::new(source, markers, route)?,
            },
            links,
        })
    }
}

impl MapLocations<'_> {
    fn settlement_emphasis(&self, settlement: &SettlementView) -> MapMarkerEmphasis {
        let selected = self.selected == Some(settlement.id.as_str());
        if settlement.id == self.current.id {
            return if selected {
                MapMarkerEmphasis::CurrentSelected
            } else {
                MapMarkerEmphasis::Current
            };
        }
        if selected {
            return MapMarkerEmphasis::Selected;
        }
        if self.connected.contains(settlement.id.as_str()) {
            return MapMarkerEmphasis::Connected;
        }
        MapMarkerEmphasis::Ordinary
    }

    fn case_emphasis(&self, site: &BackendCaseSitePin) -> MapMarkerEmphasis {
        if self.selected == Some(site.case_site_id.value.as_str()) {
            return MapMarkerEmphasis::Selected;
        }
        if site.tracked {
            return MapMarkerEmphasis::Connected;
        }
        MapMarkerEmphasis::Ordinary
    }
}

pub(crate) fn has_geographic_source(settlement: &SettlementView) -> bool {
    settlement.source_node_id.is_some()
        && Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
            settlement.longitude,
            settlement.latitude,
        )
        .is_some()
}

/// Native source-coverage adapter; returned positions are checked microdegrees.
fn settlement_position(
    settlement: &SettlementView,
    bounds: [f64; 4],
) -> Option<Wgs84CoordinateMicrodegrees> {
    if !has_geographic_source(settlement)
        || !adventuresim_world_schema::coordinates_in_bounds(
            settlement.longitude,
            settlement.latitude,
            bounds,
        )
    {
        return None;
    }
    Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
        settlement.longitude,
        settlement.latitude,
    )
}

/// The generated record's geographic flag and E7 leaves are admitted once here.
fn case_position(
    site: &BackendCaseSitePin,
    bounds: [f64; 4],
) -> Option<Wgs84CoordinateMicrodegrees> {
    if !site.coordinates_are_geographic {
        return None;
    }
    let point = Wgs84CoordinateE7::new(site.latitude_e_7, site.longitude_e_7)?;
    let (longitude, latitude) = point.longitude_latitude_degrees();
    if !adventuresim_world_schema::coordinates_in_bounds(longitude, latitude, bounds) {
        return None;
    }
    Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(longitude, latitude)
}
