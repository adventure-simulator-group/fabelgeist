//! Spatial marker projection belongs to the camera; HTML retains link semantics.
use super::{MapState, RegionalMapCamera, geographic_surface};
use adventuresim_core::strategic_place::StrategicPlaceId;
use adventuresim_tactical_core::regional_map::{MapMarkerEmphasis, MapMarkerRank};
use bevy::prelude::*;
use serde::Serialize;

const VILLAGE_VIEW_SPAN_METRES: f32 = 100_000.0;
const TOWN_VIEW_SPAN_METRES: f32 = 300_000.0;
const CITY_VIEW_SPAN_METRES: f32 = 1_000_000.0;

/// Absolute logical canvas pixels, from Bevy's actual cropped camera projection.
#[derive(Serialize)]
pub(super) struct ProjectedMarker<'a> {
    place: &'a StrategicPlaceId,
    x: f32,
    y: f32,
}

pub(super) fn project<'a>(
    state: &'a MapState,
    cameras: &Query<(&Camera, &GlobalTransform), With<RegionalMapCamera>>,
) -> Vec<ProjectedMarker<'a>> {
    let (Some(pose), Some(terrain), Some(surface), Some(overlay)) = (
        &state.pose,
        state.terrain(),
        &state.presented,
        &state.overlay,
    ) else {
        return Vec::new();
    };
    if pose.rect.is_none()
        || overlay.source() != &pose.source
        || terrain.source() != &pose.source
        || terrain.request() != surface.request
    {
        return Vec::new();
    }
    let Ok((camera, transform)) = cameras.single() else {
        return Vec::new();
    };
    let Some(viewport) = camera.logical_viewport_rect() else {
        return Vec::new();
    };
    if !camera.is_active {
        return Vec::new();
    }
    overlay
        .markers()
        .iter()
        .filter_map(|marker| {
            let essential = marker.emphasis != MapMarkerEmphasis::Ordinary;
            let visible = essential
                || match marker.rank {
                    MapMarkerRank::Hamlet | MapMarkerRank::Village => {
                        pose.span.metres() <= VILLAGE_VIEW_SPAN_METRES
                    }
                    MapMarkerRank::Town => pose.span.metres() <= TOWN_VIEW_SPAN_METRES,
                    MapMarkerRank::City => pose.span.metres() <= CITY_VIEW_SPAN_METRES,
                    MapMarkerRank::Capital | MapMarkerRank::CaseSite => true,
                };
            if !visible {
                return None;
            }
            let point = geographic_surface::position(terrain, marker.origin)?;
            let screen = camera.world_to_viewport(transform, point).ok()?;
            viewport.contains(screen).then_some(ProjectedMarker {
                place: &marker.place,
                x: screen.x,
                y: screen.y,
            })
        })
        .collect()
}
