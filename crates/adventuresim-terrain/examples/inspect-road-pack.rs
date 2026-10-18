//! Read-only road source admission and selection diagnostics.
use adventuresim_terrain::{TerrainPack, road_pack::RoadPack};
use adventuresim_world_schema::coordinates::Wgs84BoundsE7;
use std::{ffi::OsString, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    let [terrain_manifest, terrain_bytes, road_manifest, road_bytes]: [OsString; 4] = arguments
        .try_into()
        .map_err(|_| "usage: inspect-road-pack TERRAIN_JSON TERRAIN_PACK ROAD_JSON ROAD_PACK")?;
    let terrain = TerrainPack::load(
        &PathBuf::from(terrain_manifest),
        &PathBuf::from(terrain_bytes),
    )?;
    let started = Instant::now();
    let roads = RoadPack::load(
        &PathBuf::from(road_manifest),
        &PathBuf::from(road_bytes),
        &terrain,
    )?;
    let admission = started.elapsed();
    // Native TerrainPack bounds are west/south/east/north degrees. Admit them
    // once before selecting the complete bounded source for diagnostic counts.
    let window = Wgs84BoundsE7::from_longitude_latitude_degrees(terrain.bounds())
        .ok_or("invalid terrain bounds")?;
    let lines = roads.intersecting(window).collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "source": roads.source(),
            "road_lines": lines.len(),
            "points": lines.iter().map(|line| line.points().len()).sum::<usize>(),
            "admission_milliseconds": admission.as_millis(),
        }))?
    );
    Ok(())
}
