//! Read-only real-source capture timings and bounded response diagnostics.
use adventuresim_tactical_core::{
    regional_environment::RegionalEnvironment,
    regional_terrain::{RegionalTerrainRequest, RegionalTerrainScale},
};
use adventuresim_tactical_server_dispatcher::regional_terrain;
use adventuresim_terrain::{TerrainPack, road_pack::RoadPack};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use std::{ffi::OsString, path::PathBuf, time::Instant};

#[path = "../src/routes/map_environment/connection_window.rs"]
mod connection_window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [terrain_manifest, terrain_bytes, road_manifest, road_bytes, latitude, longitude, output]: [OsString; 7] =
        std::env::args_os().skip(1).collect::<Vec<_>>().try_into().map_err(|_| "usage: inspect-map-environment TERRAIN_JSON TERRAIN_PACK ROAD_JSON ROAD_PACK LATITUDE_MICRODEGREES LONGITUDE_MICRODEGREES OUTPUT_DIR")?;
    let terrain = TerrainPack::load(
        &PathBuf::from(terrain_manifest),
        &PathBuf::from(terrain_bytes),
    )?;
    let roads = RoadPack::load(
        &PathBuf::from(road_manifest),
        &PathBuf::from(road_bytes),
        &terrain,
    )?;
    let origin = Wgs84CoordinateMicrodegrees::new(
        latitude.to_str().ok_or("invalid latitude")?.parse()?,
        longitude.to_str().ok_or("invalid longitude")?.parse()?,
    )
    .ok_or("origin outside WGS84")?;
    let output = PathBuf::from(output);
    std::fs::create_dir_all(&output)?;
    for scale in [
        RegionalTerrainScale::Neighborhood,
        RegionalTerrainScale::District,
        RegionalTerrainScale::Region,
        RegionalTerrainScale::Country,
        RegionalTerrainScale::Continent,
    ] {
        let request = RegionalTerrainRequest { origin, scale };
        let started = Instant::now();
        let connections = connection_window::capture(&roads, request)?;
        let connection_ms = started.elapsed().as_millis();
        let environment =
            RegionalEnvironment::new(regional_terrain::capture(&terrain, request)?, connections)?;
        let capture_ms = started.elapsed().as_millis();
        let json = serde_json::to_vec(&environment)?;
        let scale_name = serde_json::to_value(scale)?
            .as_str()
            .ok_or("invalid scale")?
            .to_owned();
        std::fs::write(output.join(format!("{scale_name}.json")), &json)?;
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "scale": scale, "connections": environment.connections().len(),
                "points": environment.connections().iter().map(|line| line.points().len()).sum::<usize>(),
                "connection_ms": connection_ms, "capture_ms": capture_ms, "response_bytes": json.len(),
            }))?
        );
    }
    Ok(())
}
