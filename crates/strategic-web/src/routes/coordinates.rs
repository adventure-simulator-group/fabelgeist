//! Coordinate conversion at the web transport boundary.
use adventuresim_world_schema::coordinates::Wgs84CoordinateE7;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum CoordinateAdmissionError {
    #[error("persisted coordinate is outside WGS84 bounds")]
    Persisted,
    #[error("route coordinate is outside WGS84 bounds")]
    Route,
}

pub(super) fn wgs84_latitude_longitude_degrees(
    latitude_e7: i32,
    longitude_e7: i32,
) -> std::result::Result<(f64, f64), CoordinateAdmissionError> {
    Wgs84CoordinateE7::new(latitude_e7, longitude_e7)
        .map(Wgs84CoordinateE7::latitude_longitude_degrees)
        .ok_or(CoordinateAdmissionError::Persisted)
}

pub(super) fn wgs84_e7(
    latitude: f64,
    longitude: f64,
) -> std::result::Result<(i32, i32), CoordinateAdmissionError> {
    let coordinate = Wgs84CoordinateE7::from_longitude_latitude_degrees(longitude, latitude)
        .ok_or(CoordinateAdmissionError::Route)?;
    Ok((coordinate.latitude().get(), coordinate.longitude().get()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinate_admission_classifies_the_source_boundary() {
        assert_eq!(
            wgs84_latitude_longitude_degrees(900_000_001, 0),
            Err(CoordinateAdmissionError::Persisted)
        );
        assert_eq!(
            wgs84_latitude_longitude_degrees(0, 1_800_000_001),
            Err(CoordinateAdmissionError::Persisted)
        );
        assert_eq!(wgs84_e7(90.1, 0.0), Err(CoordinateAdmissionError::Route));
        assert_eq!(
            wgs84_e7(0.0, f64::NAN),
            Err(CoordinateAdmissionError::Route)
        );
        assert_eq!(
            wgs84_latitude_longitude_degrees(-900_000_000, 1_800_000_000),
            Ok((-90.0, 180.0))
        );
        assert_eq!(
            CoordinateAdmissionError::Persisted.to_string(),
            "persisted coordinate is outside WGS84 bounds"
        );
        assert_eq!(
            CoordinateAdmissionError::Route.to_string(),
            "route coordinate is outside WGS84 bounds"
        );
    }
}
