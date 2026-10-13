//! Street-scale preparation follows only observer-admitted settlement markers.
use super::MapState;
use adventuresim_core::strategic_place::StrategicPlaceId;
use adventuresim_tactical_core::regional_map::MapMarker;
use adventuresim_world_schema::coordinates::terrain_projection::NativeTerrainCoordinate;

const CITY_DETAIL_VIEW_SPAN_METRES: f32 = 2_000.0;
const CITY_FOCUS_RADIUS_METRES: f64 = 2_000.0;

impl MapState {
    pub(super) fn requested_city(&self) -> Option<&StrategicPlaceId> {
        let pose = self.pose.as_ref()?;
        if pose.rect.is_none() || pose.span.metres() > CITY_DETAIL_VIEW_SPAN_METRES {
            return None;
        }
        let overlay = self
            .overlay
            .as_ref()
            .filter(|overlay| overlay.source() == &pose.source)?;
        overlay
            .markers()
            .iter()
            .filter(|marker| matches!(marker.place, StrategicPlaceId::Settlement { .. }))
            .filter(|marker| {
                distance_squared_metres(pose.origin, marker) <= CITY_FOCUS_RADIUS_METRES.powi(2)
            })
            .min_by(|first, second| {
                distance_squared_metres(pose.origin, first)
                    .total_cmp(&distance_squared_metres(pose.origin, second))
            })
            .map(|marker| &marker.place)
    }
}

/// Native projection kernel: components are east/north metres in the camera's
/// continuous geographic frame. No coordinate rounding chooses the settlement.
fn distance_squared_metres(origin: NativeTerrainCoordinate, marker: &MapMarker) -> f64 {
    let offset = origin.offset_to(marker.origin.to_e7().into());
    offset.east_metres * offset.east_metres + offset.north_metres * offset.north_metres
}
