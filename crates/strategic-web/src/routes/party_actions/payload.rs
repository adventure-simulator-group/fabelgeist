//! Complete planned-route protocol payload, including weather and return leg.
use super::super::coordinates::wgs84_e7;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
#[serde(transparent)]
pub(super) struct TravelRoutePayload(serde_json::Value);

impl TravelRoutePayload {
    pub(super) fn from_plan(
        digest: &str,
        plan: &adventuresim_terrain::RoutePlan,
        return_plan: Option<&adventuresim_terrain::RoutePlan>,
        weather: adventuresim_core::weather::WeatherSnapshot,
    ) -> Self {
        let point_json = |point: &adventuresim_terrain::RoutePoint| {
            let (latitude_e7, longitude_e7) = wgs84_e7(point.latitude, point.longitude)
                .expect("terrain planner returned an invalid WGS84 route coordinate");
            json!({"latitude_e7": latitude_e7, "longitude_e7": longitude_e7})
        };
        let leg_json = |plan: &adventuresim_terrain::RoutePlan| {
            json!({
                "distance_m": plan.distance_m,
                "minutes": plan.minutes,
                "points": plan.points.iter().map(point_json).collect::<Vec<_>>(),
                "spans": plan.spans.iter().filter_map(|span| { let kind=match span.surface { adventuresim_terrain::Surface::Road=>adventuresim_stdb_client::JourneyTerrainKind::Road,adventuresim_terrain::Surface::Open=>adventuresim_stdb_client::JourneyTerrainKind::Open,adventuresim_terrain::Surface::SparseWoods=>adventuresim_stdb_client::JourneyTerrainKind::SparseWoods,adventuresim_terrain::Surface::DeepWoods=>adventuresim_stdb_client::JourneyTerrainKind::DeepWoods,adventuresim_terrain::Surface::Wetland=>adventuresim_stdb_client::JourneyTerrainKind::Wetland,adventuresim_terrain::Surface::Water=>return None};let kind=serde_json::to_value(spacetimedb_sats::serde::SerdeWrapper::from_ref(&kind)).expect("generated terrain kind serializes");Some(json!({"kind":kind,"terrain":span.terrain,"training_multiplier_permille":span.training_multiplier_permille,"check_millirank":span.check_millirank,"start_minute":span.start_minute,"duration_minutes":span.duration_minutes})) }).collect::<Vec<_>>()
            })
        };
        let precipitation = match weather.precipitation {
            adventuresim_core::weather::Precipitation::Clear => {
                adventuresim_stdb_client::Precipitation::Clear
            }
            adventuresim_core::weather::Precipitation::Rain => {
                adventuresim_stdb_client::Precipitation::Rain
            }
            adventuresim_core::weather::Precipitation::Snow => {
                adventuresim_stdb_client::Precipitation::Snow
            }
        };
        let precipitation = serde_json::to_value(spacetimedb_sats::serde::SerdeWrapper::from_ref(
            &precipitation,
        ))
        .expect("generated precipitation serializes");
        Self(json!({
            "package_digest": digest,
            "weather_rules_version": weather.rules_version,
            "weather_interval_start": weather.interval_start_minute,
            "temperature_deci_c": weather.temperature_deci_c,
            "wind_speed_bps": weather.wind_speed_bps,
            "precipitation": precipitation,
            "intensity_bps": weather.intensity_bps,
            "ground_moisture_bps": weather.ground_moisture_bps,
            "snow_cover_bps": weather.snow_cover_bps,
            "atmosphere": weather.atmosphere,
            "distance_m": plan.distance_m,
            "minutes": plan.minutes,
            "points": plan.points.iter().map(point_json).collect::<Vec<_>>(),
            "spans": plan.spans.iter().filter_map(|span| { let kind=match span.surface { adventuresim_terrain::Surface::Road=>adventuresim_stdb_client::JourneyTerrainKind::Road,adventuresim_terrain::Surface::Open=>adventuresim_stdb_client::JourneyTerrainKind::Open,adventuresim_terrain::Surface::SparseWoods=>adventuresim_stdb_client::JourneyTerrainKind::SparseWoods,adventuresim_terrain::Surface::DeepWoods=>adventuresim_stdb_client::JourneyTerrainKind::DeepWoods,adventuresim_terrain::Surface::Wetland=>adventuresim_stdb_client::JourneyTerrainKind::Wetland,adventuresim_terrain::Surface::Water=>return None};let kind=serde_json::to_value(spacetimedb_sats::serde::SerdeWrapper::from_ref(&kind)).expect("generated terrain kind serializes");Some(json!({"kind":kind,"terrain":span.terrain,"training_multiplier_permille":span.training_multiplier_permille,"check_millirank":span.check_millirank,"start_minute":span.start_minute,"duration_minutes":span.duration_minutes})) }).collect::<Vec<_>>(),
            "return_route": return_plan.map(leg_json)
        }))
    }
}

#[cfg(test)]
mod terrain_route_payload_tests {
    use super::TravelRoutePayload;
    use crate::spacetimedb::{JourneyTerrainKind, JourneyTerrainSpan};

    #[test]
    fn wetland_span_survives_gateway_payload_boundary() {
        let plan = adventuresim_terrain::RoutePlan {
            points: vec![],
            spans: vec![adventuresim_terrain::TerrainSpan {
                surface: adventuresim_terrain::Surface::Wetland,
                terrain: adventuresim_terrain::TerrainWeights {
                    wetlands: 1_000,
                    ..Default::default()
                },
                training_multiplier_permille: 1_000,
                check_millirank: 2_500,
                start_minute: 0,
                duration_minutes: 60,
            }],
            distance_m: 500,
            minutes: 60,
        };
        let payload = TravelRoutePayload::from_plan(
            &"a".repeat(64),
            &plan,
            None,
            adventuresim_core::weather::weather_at(
                adventuresim_core::weather::WORLD_WEATHER_SEED,
                adventuresim_world_schema::calendar::StrategicMinute::new(0),
                53_000_000,
                10_000_000,
                0,
            ),
        );
        let payload = serde_json::to_value(payload).unwrap();
        let spacetimedb_sats::serde::SerdeWrapper(span) = serde_json::from_value::<
            spacetimedb_sats::serde::SerdeWrapper<JourneyTerrainSpan>,
        >(payload["spans"][0].clone())
        .unwrap();
        assert!(matches!(span.kind, JourneyTerrainKind::Wetland));
        assert_eq!(span.terrain.wetlands, 1_000);
        assert_eq!(
            payload["weather_rules_version"],
            adventuresim_core::weather::WEATHER_RULES_VERSION
        );
        assert!(payload["temperature_deci_c"].as_i64().is_some());
        assert!(payload["wind_speed_bps"].as_u64().is_some());
        assert!(payload["weather_interval_start"].as_u64().is_some());
        assert!(payload["ground_moisture_bps"].as_u64().unwrap() <= 10_000);
        assert!(payload["snow_cover_bps"].as_u64().unwrap() <= 10_000);
    }
}
