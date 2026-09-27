//! Coordinate conversion at the web transport boundary.
use adventuresim_world_schema::coordinates::Wgs84CoordinateE7;

pub(super) fn wgs84_latitude_longitude_degrees(
    latitude_e7: i32,
    longitude_e7: i32,
) -> Result<(f64, f64), &'static str> {
    Wgs84CoordinateE7::new(latitude_e7, longitude_e7)
        .map(Wgs84CoordinateE7::latitude_longitude_degrees)
        .ok_or("persisted coordinate is outside WGS84 bounds")
}

pub(super) fn wgs84_e7(latitude: f64, longitude: f64) -> Result<(i32, i32), &'static str> {
    let coordinate = Wgs84CoordinateE7::from_longitude_latitude_degrees(longitude, latitude)
        .ok_or("route coordinate is outside WGS84 bounds")?;
    Ok((coordinate.latitude().get(), coordinate.longitude().get()))
}
