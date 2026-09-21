//! Coordinate conversions used while placing standalone tactical sites.

pub(super) fn standalone_case_site_northward_offset(
    distance_m: u64,
    coordinates_are_geographic: bool,
) -> f64 {
    let coordinate_unit_m = if coordinates_are_geographic {
        super::METERS_PER_GEOGRAPHIC_LATITUDE_DEGREE
    } else {
        super::METERS_PER_UNBOUNDED_COORDINATE_UNIT
    };
    distance_m as f64 / coordinate_unit_m
}
