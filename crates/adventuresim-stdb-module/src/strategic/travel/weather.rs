//! Route weather must still describe the synchronized departure interval.

use super::route_error::RouteAdmissionError;
use adventuresim_world_schema::calendar::StrategicMinute;

pub(crate) fn require_departure_weather_interval(
    snapshot_interval: StrategicMinute,
    departure: StrategicMinute,
) -> Result<(), RouteAdmissionError> {
    let expected_interval = departure
        .floor_to_interval_minutes(adventuresim_core::weather::WEATHER_INTERVAL_MINUTES)
        .expect("weather interval is positive");
    if snapshot_interval != expected_interval {
        return Err(RouteAdmissionError::StaleWeather {
            departure,
            expected_interval,
            snapshot_interval,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn interval_change_retains_departure_and_both_weather_frontiers() {
        require_departure_weather_interval(StrategicMinute::ZERO, StrategicMinute::new(359))
            .unwrap();
        let error =
            require_departure_weather_interval(StrategicMinute::ZERO, StrategicMinute::new(360))
                .unwrap_err();
        assert_eq!(
            error,
            RouteAdmissionError::StaleWeather {
                departure: StrategicMinute::new(360),
                expected_interval: StrategicMinute::new(360),
                snapshot_interval: StrategicMinute::ZERO,
            }
        );
        assert_eq!(
            error.to_string(),
            "Terrain route weather snapshot is stale after clock synchronization"
        );
        assert!(error.source().is_none());
        require_departure_weather_interval(StrategicMinute::new(360), StrategicMinute::new(360))
            .unwrap();
    }

    #[test]
    fn camp_redirect_uses_the_same_weather_boundary() {
        require_departure_weather_interval(StrategicMinute::new(360), StrategicMinute::new(719))
            .unwrap();
        assert!(
            matches!(require_departure_weather_interval(StrategicMinute::new(360), StrategicMinute::new(720)),
            Err(RouteAdmissionError::StaleWeather { departure, .. }) if departure == StrategicMinute::new(720))
        );
        require_departure_weather_interval(StrategicMinute::new(720), StrategicMinute::new(720))
            .unwrap();
    }
}
