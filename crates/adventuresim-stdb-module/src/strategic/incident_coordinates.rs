fn incident_settlement_coordinates(settlement: &Settlement) -> Result<(i32, i32), String> {
    Ok(if settlement.source_node_id.is_some() {
        let coordinate = adventuresim_world_schema::coordinates::Wgs84CoordinateE7::from_longitude_latitude_degrees(
            settlement.coord_x,
            settlement.coord_y,
        )
        .ok_or("Incident settlement has invalid WGS84 coordinates")?;
        (coordinate.longitude().get(), coordinate.latitude().get())
    } else {
        let longitude =
            adventuresim_world_schema::coordinates::UnboundedCoordinateE7::from_coordinate_units(
                settlement.coord_x,
            )
            .ok_or("Incident settlement longitude cannot be represented as E7")?;
        let latitude =
            adventuresim_world_schema::coordinates::UnboundedCoordinateE7::from_coordinate_units(
                settlement.coord_y,
            )
            .ok_or("Incident settlement latitude cannot be represented as E7")?;
        (longitude.raw(), latitude.raw())
    })
}
