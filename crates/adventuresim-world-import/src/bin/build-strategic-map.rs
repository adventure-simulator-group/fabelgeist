use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use adventuresim_world_import::hyde_crop_cells;
use adventuresim_world_schema::calendar::WORLD_START_YEAR;
use adventuresim_world_schema::coordinates::Wgs84BoundsE7;
use adventuresim_world_schema::regional_connection::RegionalConnectionKind;
use adventuresim_world_schema::{CompiledWorld, PLAYABLE_BOUNDS, TravelEdgeProvenance};
use clap::{Parser, ValueEnum};
use road_history::active;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use simplification::{WATER_RING_TOLERANCE_DEGREES, simplify};

#[path = "build-strategic-map/distance_index.rs"]
mod distance_index;
#[path = "build-strategic-map/road_history.rs"]
mod road_history;
#[path = "build-strategic-map/simplification.rs"]
mod simplification;
#[path = "build-strategic-map/terrain_features.rs"]
mod terrain_features;

const RECORD_URL: &str = "https://zenodo.org/api/records/16611998";
const BOUNDS: [f64; 4] = PLAYABLE_BOUNDS;
const MAX_SOURCE_FILES: usize = 64;
const DATA_LICENSE_FILENAME: &str = "STRATEGIC_MAP_DATA_LICENSE.md";
const DATA_LICENSE: &str = include_str!("../../../../MAP_DATA_LICENSE.md");
const COMPILER_STACK_BYTES: usize = 64 * 1024 * 1024;

#[derive(Parser)]
#[command(about = "Build canonical regional roads and native terrain from initialized world data")]
struct Args {
    #[arg(long, value_enum, default_value = "final")]
    purpose: BuildPurpose,
    #[arg(long, default_value = "viabundus")]
    viabundus_dir: PathBuf,
    #[arg(long, default_value = "target/world-data-sources/raw/elevation")]
    elevation_dir: PathBuf,
    #[arg(long, default_value = "target/world-data-sources/raw/forest-cover")]
    forest_cover_dir: PathBuf,
    #[arg(long, default_value = "target/world-data-sources/raw/jung-pnv")]
    potential_vegetation_dir: PathBuf,
    #[arg(long, default_value = "target/world-data-sources/raw/hyde35-land-use")]
    hyde_dir: PathBuf,
    #[arg(long, default_value = "target/world-1544.json")]
    compiled_world: PathBuf,
    #[arg(long, default_value = "target/strategic-map/regional-roads-v1.json")]
    output: PathBuf,
    #[arg(long, default_value = "target/strategic-map/regional-roads-v1.pack")]
    roads_pack_output: PathBuf,
    #[arg(long, default_value = "target/strategic-map/terrain-routing-v3.json")]
    terrain_output: PathBuf,
    #[arg(long, default_value = "target/strategic-map/terrain-routing-v3.pack")]
    terrain_pack_output: PathBuf,
    #[arg(
        long,
        default_value = "target/strategic-map/terrain-routing-base-v3.json"
    )]
    base_terrain_output: PathBuf,
    #[arg(
        long,
        default_value = "target/strategic-map/terrain-routing-base-v3.pack"
    )]
    base_terrain_pack_output: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceManifest {
    record_url: String,
    version: String,
    files: Vec<SourceFile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    name: String,
    sha256: String,
    url: String,
    size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Point([f64; 2]);

#[derive(Clone, Debug, PartialEq)]
struct Package {
    bounds: Wgs84BoundsE7,
    /// Full active Viabundus and inferred geometry shared by routing and
    /// presentation. Coordinates remain native longitude/latitude degrees
    /// until the source compiler or checked road package admits them.
    routing_roads: Vec<SourceRoad>,
    water: Vec<WaterPolygon>,
}

#[derive(Clone, Debug, PartialEq)]
struct SourceRoad {
    kind: RegionalConnectionKind,
    points: Vec<Point>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct WaterPolygon {
    rings: Vec<Vec<Point>>,
}

struct CultivatedLand {
    polygons: Vec<Vec<Vec<[f64; 2]>>>,
    source_sha256: String,
}

#[derive(Clone, Copy, Eq, PartialEq, ValueEnum)]
enum BuildPurpose {
    DocumentedBase,
    Final,
    Roads,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let render = std::thread::Builder::new()
        .name("regional-map-builder".into())
        .stack_size(COMPILER_STACK_BYTES)
        .spawn(move || run(args).map_err(|error| error.to_string()))?;
    match render.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(message)) => Err(std::io::Error::other(message).into()),
        Err(_) => Err(std::io::Error::other("regional map compiler panicked").into()),
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.purpose == BuildPurpose::Roads {
        return compile_roads(&args);
    }
    let mut package = build(
        &args.viabundus_dir,
        Wgs84BoundsE7::from_longitude_latitude_degrees(BOUNDS).ok_or("invalid playable bounds")?,
    )?;
    let wetland =
        adventuresim_world_import::wetland_spatial_data(&args.potential_vegetation_dir, BOUNDS)?;
    let base_features = terrain_features::build(
        &package,
        wetland.polygons.clone(),
        wetland.source_sha256.clone(),
    );
    if args.purpose == BuildPurpose::DocumentedBase {
        let terrain = adventuresim_terrain::builder::build(
            &args.elevation_dir,
            &args.forest_cover_dir,
            BOUNDS,
            &args.base_terrain_output,
            &args.base_terrain_pack_output,
            &base_features,
            adventuresim_terrain::TerrainPurpose::DocumentedBase,
        )?;
        write_data_license(&[&args.base_terrain_output, &args.base_terrain_pack_output])?;
        println!(
            "Wrote documented-road base terrain {} (digest {}, {} wetland pixels)",
            args.base_terrain_output.display(),
            terrain.package_sha256,
            terrain.wetland_cells
        );
        return Ok(());
    }
    let base = adventuresim_terrain::TerrainPack::load(
        &args.base_terrain_output,
        &args.base_terrain_pack_output,
    )?;
    let current_base_road_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&base_features.roads)?)
    );
    if !base_contract_matches(
        base.purpose(),
        base.bounds(),
        base.source_resolution_m(),
        base.road_geometry_sha256(),
        base.wetland_source_sha256(),
        &current_base_road_digest,
        &base_features.wetland_source_sha256,
    ) {
        return Err(
            "base terrain does not match current documented roads, wetlands, bounds, or resolution"
                .into(),
        );
    }
    let world: CompiledWorld = serde_json::from_slice(&fs::read(&args.compiled_world)?)?;
    adventuresim_world_import::validate_world(&world)?;
    if world.report.base_terrain_package_sha256 != base.package_sha256() {
        return Err("compiled world was inferred against a different base terrain digest".into());
    }
    append_inferred_roads(&mut package, &world);
    package
        .routing_roads
        .sort_by(|a, b| point_order(&a.points, &b.points));
    let cultivated = cultivated_land(&args.hyde_dir, &base, &package, &world)?;
    let terrain_features = terrain_features::finalize(
        &package,
        wetland.polygons,
        wetland.source_sha256,
        cultivated,
        &world,
    );
    let terrain = adventuresim_terrain::builder::build(
        &args.elevation_dir,
        &args.forest_cover_dir,
        BOUNDS,
        &args.terrain_output,
        &args.terrain_pack_output,
        &terrain_features,
        adventuresim_terrain::TerrainPurpose::Final,
    )?;
    let expected_road_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&terrain_features.roads)?)
    );
    if terrain.road_geometry_sha256 != expected_road_digest {
        return Err(
            "final terrain road mask identity differs from rendered routing geometry".into(),
        );
    }
    let native_terrain =
        adventuresim_terrain::TerrainPack::load(&args.terrain_output, &args.terrain_pack_output)?;
    let roads = adventuresim_terrain::road_pack::RoadPack::write(
        &args.output,
        &args.roads_pack_output,
        &native_terrain,
        &package.native_road_lines(),
    )?;
    write_data_license(&[
        &args.output,
        &args.roads_pack_output,
        &args.terrain_output,
        &args.terrain_pack_output,
    ])?;
    println!(
        "Wrote {} canonical roads with {} source points to {} and {}",
        roads.roads,
        roads.points,
        args.output.display(),
        args.roads_pack_output.display()
    );
    println!(
        "Wrote {} native 30 m terrain chunks to {} and {} (digest {})",
        terrain.entries.len(),
        args.terrain_output.display(),
        args.terrain_pack_output.display(),
        terrain.package_sha256
    );
    Ok(())
}

fn compile_roads(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let world: CompiledWorld = serde_json::from_slice(&fs::read(&args.compiled_world)?)?;
    adventuresim_world_import::validate_world(&world)?;
    let terrain =
        adventuresim_terrain::TerrainPack::load(&args.terrain_output, &args.terrain_pack_output)?;
    let bounds = Wgs84BoundsE7::from_longitude_latitude_degrees(terrain.bounds())
        .ok_or("invalid terrain source bounds")?;
    let mut package = build(&args.viabundus_dir, bounds)?;
    append_inferred_roads(&mut package, &world);
    package
        .routing_roads
        .sort_by(|left, right| point_order(&left.points, &right.points));
    let geometry = package.native_road_lines();
    let roads = adventuresim_terrain::road_pack::RoadPack::write(
        &args.output,
        &args.roads_pack_output,
        &terrain,
        &geometry,
    )?;
    write_data_license(&[&args.output, &args.roads_pack_output])?;
    println!(
        "Wrote {} canonical roads with {} source points",
        roads.roads, roads.points
    );
    Ok(())
}

fn base_contract_matches(
    purpose: adventuresim_terrain::TerrainPurpose,
    bounds: [f64; 4],
    resolution: u16,
    road_digest: &str,
    wetland_digest: &str,
    current_road_digest: &str,
    current_wetland_digest: &str,
) -> bool {
    purpose == adventuresim_terrain::TerrainPurpose::DocumentedBase
        && bounds == BOUNDS
        && resolution == 30
        && road_digest == current_road_digest
        && wetland_digest == current_wetland_digest
}

fn append_inferred_roads(package: &mut Package, world: &CompiledWorld) {
    let geometries = world
        .edges
        .iter()
        .filter(|edge| edge.provenance == TravelEdgeProvenance::InferredWalkingLink)
        .map(|edge| edge.geometry.as_slice())
        .collect::<Vec<_>>();
    append_inferred_geometry(package, &geometries);
}

fn append_inferred_geometry(
    package: &mut Package,
    geometries: &[&[adventuresim_world_schema::TravelGeometryPoint]],
) {
    for geometry in geometries {
        let points = geometry
            .iter()
            .map(|point| Point([point.longitude(), point.latitude()]))
            .collect::<Vec<_>>();
        package.routing_roads.push(SourceRoad {
            kind: RegionalConnectionKind::InferredWalkingLink,
            points,
        });
    }
}

fn cultivated_land(
    hyde_dir: &Path,
    terrain: &adventuresim_terrain::TerrainPack,
    package: &Package,
    world: &CompiledWorld,
) -> Result<CultivatedLand, Box<dyn std::error::Error>> {
    use adventuresim_world_import::{
        cultivation::{
            CultivationCandidate, CultivationCell, HydeCropQuota, MetricSegment,
            SegmentDistanceIndex, allocate, square_is_usable,
        },
        spatial::{ProjectedCoordinate, SpatialProjection},
    };
    let projection = SpatialProjection::new()?;
    let projected_corners = [
        projection.project(BOUNDS[1], BOUNDS[0])?,
        projection.project(BOUNDS[1], BOUNDS[2])?,
        projection.project(BOUNDS[3], BOUNDS[0])?,
        projection.project(BOUNDS[3], BOUNDS[2])?,
    ];
    let min_column = projected_corners
        .iter()
        .map(|point| point.easting_millimeters().div_euclid(1_000_000))
        .min()
        .ok_or("projected map has no corners")?
        - 1;
    let max_column = projected_corners
        .iter()
        .map(|point| point.easting_millimeters().div_euclid(1_000_000))
        .max()
        .ok_or("projected map has no corners")?
        + 1;
    let min_row = projected_corners
        .iter()
        .map(|point| point.northing_millimeters().div_euclid(1_000_000))
        .min()
        .ok_or("projected map has no corners")?
        - 1;
    let max_row = projected_corners
        .iter()
        .map(|point| point.northing_millimeters().div_euclid(1_000_000))
        .max()
        .ok_or("projected map has no corners")?
        + 1;
    let metric_point = |point: ProjectedCoordinate| {
        [
            point.easting_millimeters().div_euclid(1_000),
            point.northing_millimeters().div_euclid(1_000),
        ]
    };
    let settlement_segments = world
        .settlements
        .iter()
        .map(|settlement| projection.project(settlement.latitude, settlement.longitude))
        .map(|point| {
            point.map(|point| {
                let point = metric_point(point);
                MetricSegment {
                    from: point,
                    to: point,
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let settlement_index = SegmentDistanceIndex::new(settlement_segments)?;
    let road_lines = package
        .routing_roads
        .iter()
        .map(|line| line.points.as_slice())
        .collect::<Vec<_>>();
    let road_index = distance_index::build(&projection, &road_lines)?;
    let water_lines = package
        .water
        .iter()
        .flat_map(|polygon| polygon.rings.iter().map(Vec::as_slice))
        .collect::<Vec<_>>();
    let water_index = distance_index::build(&projection, &water_lines)?;
    let mut candidates = Vec::new();
    for row in min_row..=max_row {
        for column in min_column..=max_column {
            let center = ProjectedCoordinate::from_meters(
                column as f64 * 1_000.0 + 500.0,
                row as f64 * 1_000.0 + 500.0,
            )?;
            let (latitude, longitude) = projection.unproject(center)?;
            if longitude < BOUNDS[0]
                || longitude >= BOUNDS[2]
                || latitude < BOUNDS[1]
                || latitude >= BOUNDS[3]
            {
                continue;
            }
            let native = terrain.cell(latitude, longitude)?;
            let mut non_water_samples = 0u16;
            for sample_row in 0..4 {
                for sample_column in 0..4 {
                    let sample = ProjectedCoordinate::from_meters(
                        column as f64 * 1_000.0 + (f64::from(sample_column) + 0.5) * 250.0,
                        row as f64 * 1_000.0 + (f64::from(sample_row) + 0.5) * 250.0,
                    )?;
                    let (sample_latitude, sample_longitude) = projection.unproject(sample)?;
                    if terrain
                        .cell(sample_latitude, sample_longitude)?
                        .is_some_and(|cell| {
                            !matches!(cell.surface, adventuresim_terrain::Surface::Water)
                        })
                    {
                        non_water_samples += 1;
                    }
                }
            }
            let usable_land = square_is_usable(non_water_samples, 16);
            let elevation_samples = [
                (latitude, longitude),
                (latitude + 0.0045, longitude),
                (latitude - 0.0045, longitude),
                (latitude, longitude + 0.0075),
                (latitude, longitude - 0.0075),
            ]
            .into_iter()
            .filter_map(|(latitude, longitude)| terrain.cell(latitude, longitude).ok().flatten())
            .map(|cell| cell.elevation_m)
            .collect::<Vec<_>>();
            let relief = elevation_samples
                .iter()
                .max()
                .zip(elevation_samples.iter().min())
                .map_or(0, |(high, low)| high.saturating_sub(*low).max(0) as u16);
            let hyde_row = ((90.0 - latitude) * 12.0).floor().clamp(0.0, 2_159.0) as i16;
            let hyde_column = ((longitude + 180.0) * 12.0).floor().clamp(0.0, 4_319.0) as i16;
            candidates.push(CultivationCandidate {
                cell: CultivationCell { column, row },
                hyde_cell: (hyde_row, hyde_column),
                usable_land,
                settlement_distance_m: settlement_index
                    .nearest_distance_m(metric_point(center), 100_000),
                road_distance_m: road_index.nearest_distance_m(metric_point(center), 10_000),
                water_distance_m: water_index.nearest_distance_m(metric_point(center), 10_000),
                slope_permille: native.map_or(0, |cell| {
                    if cell.hilly_fraction_percent >= 50 {
                        268
                    } else {
                        0
                    }
                }),
                relief_m: relief,
                canopy_percent: native.map_or(0, |cell| cell.canopy_percent),
            });
        }
    }
    let (hyde, source_sha256) = hyde_crop_cells(hyde_dir, WORLD_START_YEAR, BOUNDS)?;
    let quotas = hyde
        .into_iter()
        .map(|cell| {
            let longitude_fraction = ((cell.bounds[2].min(BOUNDS[2])
                - cell.bounds[0].max(BOUNDS[0]))
                / (cell.bounds[2] - cell.bounds[0]))
                .clamp(0.0, 1.0);
            let latitude_fraction = ((cell.bounds[3].min(BOUNDS[3])
                - cell.bounds[1].max(BOUNDS[1]))
                / (cell.bounds[3] - cell.bounds[1]))
                .clamp(0.0, 1.0);
            HydeCropQuota {
                cell: (cell.row, cell.column),
                crop_km2: cell.crop_km2 * longitude_fraction * latitude_fraction,
                boundary_clipped: longitude_fraction < 1.0 || latitude_fraction < 1.0,
            }
        })
        .collect::<Vec<_>>();
    let allocation = allocate(&candidates, &quotas)?;
    if allocation.residual_km2.abs() >= 0.500_001 {
        return Err("cultivation quota rounding residual exceeded 0.5 km2".into());
    }
    if allocation.capacity_limited_km2 > 0 {
        eprintln!(
            "Cultivation capacity omitted {} km2 that cannot be represented by usable canonical squares under the bounded grid rules",
            allocation.capacity_limited_km2
        );
    }
    let polygons = allocation
        .cells
        .into_iter()
        .map(|cell| {
            let ring = [
                (cell.column as f64 * 1_000.0, cell.row as f64 * 1_000.0),
                (
                    (cell.column + 1) as f64 * 1_000.0,
                    cell.row as f64 * 1_000.0,
                ),
                (
                    (cell.column + 1) as f64 * 1_000.0,
                    (cell.row + 1) as f64 * 1_000.0,
                ),
                (
                    cell.column as f64 * 1_000.0,
                    (cell.row + 1) as f64 * 1_000.0,
                ),
                (cell.column as f64 * 1_000.0, cell.row as f64 * 1_000.0),
            ]
            .into_iter()
            .map(|(easting, northing)| {
                let (latitude, longitude) =
                    projection.unproject(ProjectedCoordinate::from_meters(easting, northing)?)?;
                Ok([longitude, latitude])
            })
            .collect::<Result<Vec<_>, adventuresim_world_import::Error>>()?;
            Ok(vec![ring])
        })
        .collect::<Result<Vec<_>, adventuresim_world_import::Error>>()?;
    Ok(CultivatedLand {
        polygons,
        source_sha256,
    })
}

fn write_data_license(outputs: &[&Path]) -> std::io::Result<()> {
    let directories = outputs
        .iter()
        .map(|output| {
            output
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();
    for directory in directories {
        fs::create_dir_all(&directory)?;
        fs::write(directory.join(DATA_LICENSE_FILENAME), DATA_LICENSE)?;
    }
    Ok(())
}

fn build(root: &Path, bounds: Wgs84BoundsE7) -> Result<Package, Box<dyn std::error::Error>> {
    let manifest: SourceManifest =
        serde_json::from_slice(&fs::read(root.join(".viabundus-source.json"))?)?;
    if manifest.version != "2" || manifest.record_url != RECORD_URL {
        return Err("strategic map requires Viabundus v2".into());
    }
    if manifest.files.is_empty() || manifest.files.len() > MAX_SOURCE_FILES {
        return Err("Viabundus sidecar file inventory is outside its bound".into());
    }
    let mut names = BTreeSet::new();
    for entry in &manifest.files {
        let safe = !entry.name.is_empty()
            && entry.name.len() <= 128
            && entry
                .name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
            && entry.name.ends_with(".csv");
        let hash_ok = entry.sha256.len() == 64
            && entry
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        let expected_url = format!("{RECORD_URL}/files/{}/content", entry.name);
        if !safe || !hash_ok || entry.url != expected_url || !names.insert(entry.name.as_str()) {
            return Err(format!("invalid Viabundus sidecar entry {}", entry.name).into());
        }
    }
    let required = ["edges.csv", "water-1500.csv"];
    for name in required {
        let entry = manifest
            .files
            .iter()
            .find(|entry| entry.name == name)
            .ok_or("source manifest is incomplete")?;
        let bytes = fs::read(root.join(name))?;
        if entry.size != bytes.len() as u64 {
            return Err(format!("{name} does not match its initialized size").into());
        }
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if actual != entry.sha256 {
            return Err(format!("{name} does not match its initialized SHA-256").into());
        }
    }

    let mut routing_roads = Vec::new();
    let mut reader = csv::Reader::from_path(root.join("edges.csv"))?;
    for row in reader.deserialize::<BTreeMap<String, String>>() {
        let row = row?;
        if !active(&row, WORLD_START_YEAR) {
            continue;
        }
        let kind = row
            .get("type")
            .ok_or("regional connection has no classification")?
            .parse::<RegionalConnectionKind>()?;
        let Some(wkt) = row.get("wkt") else { continue };
        for points in clip_polyline(&coordinates(wkt), bounds.longitude_latitude_degrees()) {
            if points.len() >= 2 {
                routing_roads.push(SourceRoad { kind, points });
            }
        }
    }
    routing_roads.sort_by(|a, b| point_order(&a.points, &b.points));

    let mut water = Vec::new();
    let mut reader = csv::Reader::from_path(root.join("water-1500.csv"))?;
    for row in reader.records() {
        let row = row?;
        let Some(wkt) = row.get(0) else { continue };
        for polygon in wkt_polygons(wkt) {
            let rings: Vec<_> = polygon
                .into_iter()
                .map(|ring| {
                    simplify(
                        &clip_polygon(&ring, bounds.longitude_latitude_degrees()),
                        WATER_RING_TOLERANCE_DEGREES,
                    )
                })
                .filter(|ring| ring.len() >= 4)
                .collect();
            if !rings.is_empty() {
                water.push(WaterPolygon { rings });
            }
        }
    }
    water.sort_by(|a, b| point_order(&a.rings[0], &b.rings[0]));

    let package = Package {
        bounds,
        routing_roads,
        water,
    };
    validate_geometry(&package)?;
    Ok(package)
}

fn coordinates(wkt: &str) -> Vec<Point> {
    wkt.split(['(', ')', ','])
        .filter_map(|part| {
            let mut values = part
                .split_whitespace()
                .filter_map(|v| v.parse::<f64>().ok());
            Some(Point([values.next()?, values.next()?]))
        })
        .collect()
}

fn wkt_polygons(wkt: &str) -> Vec<Vec<Vec<Point>>> {
    let polygon_depth = if wkt.trim_start().starts_with("MULTIPOLYGON") {
        2
    } else if wkt.trim_start().starts_with("POLYGON") {
        1
    } else {
        return Vec::new();
    };
    let ring_depth = polygon_depth + 1;
    let mut depth = 0_usize;
    let mut polygons = Vec::new();
    let mut polygon = Vec::new();
    let mut ring = String::new();
    for character in wkt.chars() {
        match character {
            '(' => {
                depth += 1;
                if depth == polygon_depth {
                    polygon.clear();
                } else if depth == ring_depth {
                    ring.clear();
                }
            }
            ')' => {
                if depth == ring_depth {
                    let points = coordinates(&ring);
                    if points.len() >= 4 {
                        polygon.push(points);
                    }
                } else if depth == polygon_depth && !polygon.is_empty() {
                    polygons.push(std::mem::take(&mut polygon));
                }
                depth = depth.saturating_sub(1);
            }
            _ if depth == ring_depth => ring.push(character),
            _ => {}
        }
    }
    polygons
}

fn clip_polyline(points: &[Point], bounds: [f64; 4]) -> Vec<Vec<Point>> {
    let mut output = Vec::new();
    let mut current = Vec::new();
    for pair in points.windows(2) {
        if let Some((start, end)) = clip_segment(&pair[0], &pair[1], bounds) {
            if current.last() != Some(&start) {
                if current.len() >= 2 {
                    output.push(std::mem::take(&mut current));
                }
                current.push(start);
            }
            current.push(end);
        } else if current.len() >= 2 {
            output.push(std::mem::take(&mut current));
        }
    }
    if current.len() >= 2 {
        output.push(current);
    }
    output
}

fn clip_segment(
    start: &Point,
    end: &Point,
    [west, south, east, north]: [f64; 4],
) -> Option<(Point, Point)> {
    let [x0, y0] = start.0;
    let [x1, y1] = end.0;
    if ![x0, y0, x1, y1].into_iter().all(f64::is_finite) {
        return None;
    }
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut low: f64 = 0.0;
    let mut high: f64 = 1.0;
    for (p, q) in [
        (-dx, x0 - west),
        (dx, east - x0),
        (-dy, y0 - south),
        (dy, north - y0),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                low = low.max(t);
            } else {
                high = high.min(t);
            }
        }
        if low > high {
            return None;
        }
    }
    Some((
        Point([x0 + low * dx, y0 + low * dy]),
        Point([x0 + high * dx, y0 + high * dy]),
    ))
}

fn clip_polygon(points: &[Point], bounds: [f64; 4]) -> Vec<Point> {
    let mut output = points.to_vec();
    for edge in 0..4 {
        let input = std::mem::take(&mut output);
        if input.is_empty() {
            break;
        }
        let mut previous = input.last().expect("nonempty").clone();
        for current in input {
            let previous_inside = polygon_inside(&previous, edge, bounds);
            let current_inside = polygon_inside(&current, edge, bounds);
            if current_inside {
                if !previous_inside {
                    output.push(polygon_intersection(&previous, &current, edge, bounds));
                }
                output.push(current.clone());
            } else if previous_inside {
                output.push(polygon_intersection(&previous, &current, edge, bounds));
            }
            previous = current;
        }
    }
    if output.len() >= 3 && output.first() != output.last() {
        output.push(output[0].clone());
    }
    output
}

fn polygon_inside(point: &Point, edge: usize, [west, south, east, north]: [f64; 4]) -> bool {
    point.0.into_iter().all(f64::is_finite)
        && match edge {
            0 => point.0[0] >= west,
            1 => point.0[0] <= east,
            2 => point.0[1] >= south,
            _ => point.0[1] <= north,
        }
}

fn polygon_intersection(start: &Point, end: &Point, edge: usize, bounds: [f64; 4]) -> Point {
    let [x0, y0] = start.0;
    let [x1, y1] = end.0;
    if edge < 2 {
        let x = if edge == 0 { bounds[0] } else { bounds[2] };
        let t = if x1 == x0 { 0.0 } else { (x - x0) / (x1 - x0) };
        Point([x, y0 + t * (y1 - y0)])
    } else {
        let y = if edge == 2 { bounds[1] } else { bounds[3] };
        let t = if y1 == y0 { 0.0 } else { (y - y0) / (y1 - y0) };
        Point([x0 + t * (x1 - x0), y])
    }
}

fn validate_geometry(package: &Package) -> Result<(), Box<dyn std::error::Error>> {
    let bounds = package.bounds.longitude_latitude_degrees();
    let valid = |point: &Point| {
        point.0[0].is_finite()
            && point.0[1].is_finite()
            && point.0[0] >= bounds[0]
            && point.0[0] <= bounds[2]
            && point.0[1] >= bounds[1]
            && point.0[1] <= bounds[3]
    };
    if package
        .routing_roads
        .iter()
        .any(|line| line.points.len() < 2 || line.points.iter().any(|point| !valid(point)))
        || package.water.iter().any(|polygon| {
            polygon.rings.is_empty()
                || polygon
                    .rings
                    .iter()
                    .any(|ring| ring.len() < 4 || ring.iter().any(|point| !valid(point)))
        })
    {
        return Err("strategic map geometry is non-finite or outside package bounds".into());
    }
    Ok(())
}

fn point_order(left: &[Point], right: &[Point]) -> std::cmp::Ordering {
    left.first()
        .and_then(|p| {
            right.first().map(|q| {
                p.0[0]
                    .total_cmp(&q.0[0])
                    .then_with(|| p.0[1].total_cmp(&q.0[1]))
            })
        })
        .unwrap_or_else(|| left.len().cmp(&right.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn cli_defaults_to_the_initialized_hyde_directory() {
        let args = Args::try_parse_from(["build-strategic-map"]).unwrap();

        assert_eq!(
            args.hyde_dir,
            PathBuf::from("target/world-data-sources/raw/hyde35-land-use")
        );
    }

    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "adventuresim-map-package-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let edges = b"id,section,type,certainty,zoomlevel,fromyear,toyear,descriptionid,length,fromnode,tonode,wkt,slopemultiplier\n1,A,land,1,2,1500,,x,100,1,2,\"LINESTRING(9 51,10 51.2,11 51.3)\",1\n2,B,river,1,6,1500,,x,100,2,3,\"LINESTRING(10 51.3,10.123456 51.345678,10.5 51.5)\",1\n";
        let water = b"WKT\n\"MULTIPOLYGON (((9 51,10 51,10 52,9 51)))\"\n";
        fs::write(root.join("edges.csv"), edges).unwrap();
        fs::write(root.join("water-1500.csv"), water).unwrap();
        let manifest = serde_json::json!({"record_url":RECORD_URL,"version":"2","files":[
            {"name":"edges.csv","sha256":format!("{:x}", Sha256::digest(edges)),"url":format!("{RECORD_URL}/files/edges.csv/content"),"size":edges.len()},
            {"name":"water-1500.csv","sha256":format!("{:x}", Sha256::digest(water)),"url":format!("{RECORD_URL}/files/water-1500.csv/content"),"size":water.len()}
        ]});
        fs::write(
            root.join(".viabundus-source.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        root
    }

    #[test]
    fn year_filter_uses_half_open_intervals() {
        let row = BTreeMap::from([
            ("fromyear".into(), "1544".into()),
            ("toyear".into(), "1545".into()),
        ]);
        assert!(active(&row, WORLD_START_YEAR));
        assert!(!active(
            &row,
            adventuresim_world_schema::calendar::CalendarYear::new(1545).unwrap()
        ));
    }

    #[test]
    fn wkt_water_parser_preserves_polygon_ring_groups() {
        let polygons = wkt_polygons(
            "MULTIPOLYGON (((0 0,10 0,10 10,0 0),(2 2,3 2,3 3,2 2)),((20 20,21 20,21 21,20 20)))",
        );
        assert_eq!(polygons.len(), 2);
        assert_eq!(polygons[0].len(), 2);
        assert_eq!(polygons[1].len(), 1);
    }

    #[test]
    fn fixture_build_is_deterministic_and_rejects_changed_source_bytes() {
        let root = fixture();
        let first = build(
            &root,
            Wgs84BoundsE7::from_longitude_latitude_degrees(BOUNDS).unwrap(),
        )
        .unwrap();
        let second = build(
            &root,
            Wgs84BoundsE7::from_longitude_latitude_degrees(BOUNDS).unwrap(),
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.routing_roads.len(), 2);
        assert_eq!(
            first.routing_roads[1].points[1],
            Point([10.123456, 51.345678])
        );
        assert_eq!(first.water.len(), 1);
        assert_eq!(first.water[0].rings.len(), 1);

        let features = terrain_features::build(&first, Vec::new(), "0".repeat(64));
        let terrain = terrain_fixture(&root, &features.roads);
        let manifest_path = root.join(adventuresim_terrain::road_pack::ROAD_MANIFEST_NAME);
        let pack_path = root.join(adventuresim_terrain::road_pack::ROAD_PACK_NAME);
        let manifest = adventuresim_terrain::road_pack::RoadPack::write(
            &manifest_path,
            &pack_path,
            &terrain,
            &first.native_road_lines(),
        )
        .unwrap();
        let first_bytes = fs::read(&pack_path).unwrap();
        adventuresim_terrain::road_pack::RoadPack::write(
            &manifest_path,
            &pack_path,
            &terrain,
            &first.native_road_lines(),
        )
        .unwrap();
        assert_eq!(first_bytes, fs::read(&pack_path).unwrap());
        assert_eq!(manifest.roads, 2);
        assert_eq!(manifest.points, 6);
        let roads =
            adventuresim_terrain::road_pack::RoadPack::load(&manifest_path, &pack_path, &terrain)
                .unwrap();
        let window = Wgs84BoundsE7::new(
            adventuresim_world_schema::coordinates::Wgs84CoordinateE7::from_longitude_latitude_degrees(10.1, 51.34).unwrap(),
            adventuresim_world_schema::coordinates::Wgs84CoordinateE7::from_longitude_latitude_degrees(10.2, 51.35).unwrap(),
        ).unwrap();
        let selected = roads.intersecting(window).collect::<Vec<_>>();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].kind(), RegionalConnectionKind::River);
        assert_eq!(selected[0].points()[1].longitude().degrees(), 10.123456);
        assert_eq!(selected[0].points()[1].latitude().degrees(), 51.345678);
        let mut tampered = first_bytes;
        tampered[4] ^= 1;
        fs::write(&pack_path, tampered).unwrap();
        assert!(matches!(
            adventuresim_terrain::road_pack::RoadPack::load(&manifest_path, &pack_path, &terrain,),
            Err(adventuresim_terrain::road_pack::RoadPackError::ContentDigest)
        ));

        fs::write(root.join("edges.csv"), b"changed").unwrap();
        assert!(
            build(
                &root,
                Wgs84BoundsE7::from_longitude_latitude_degrees(BOUNDS).unwrap()
            )
            .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn terrain_fixture(root: &Path, roads: &[Vec<[f64; 2]>]) -> adventuresim_terrain::TerrainPack {
        use adventuresim_terrain::{Entry, Manifest, SCHEMA, TerrainPurpose};
        let bytes = [0_u8];
        let mut manifest = Manifest {
            schema: SCHEMA,
            purpose: TerrainPurpose::Final,
            bounds: BOUNDS,
            source_resolution_m: 30,
            content_sha256: format!("{:x}", Sha256::digest(bytes)),
            road_geometry_sha256: format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(roads).unwrap())
            ),
            wetland_source_sha256: "0".repeat(64),
            wetland_cells: 0,
            cultivation_grid_crs: "EPSG:3035".into(),
            cultivation_grid_resolution_m: 1_000,
            cultivation_rules_version: 1,
            cultivation_source_sha256: "0".repeat(64),
            cultivated_square_count: 0,
            cultivated_native_cells: 0,
            terrain_features: Vec::new(),
            entries: vec![Entry {
                south: 51,
                west: 9,
                tile_width: 3_600,
                tile_height: 3_600,
                chunk_x: 0,
                chunk_y: 0,
                width: 1,
                height: 1,
                offset: 0,
                length: 1,
                decoded_sha256: "0".repeat(64),
            }],
            package_sha256: "0".repeat(64),
        };
        manifest.package_sha256 = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&manifest).unwrap())
        );
        let manifest_path = root.join("fixture-terrain.json");
        let pack_path = root.join("fixture-terrain.pack");
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        fs::write(&pack_path, bytes).unwrap();
        adventuresim_terrain::TerrainPack::load(&manifest_path, &pack_path).unwrap()
    }

    #[test]
    fn generated_bundle_directories_receive_the_canonical_data_license() {
        let root = fixture();
        let map_dir = root.join("map");
        let terrain_dir = root.join("terrain");
        write_data_license(&[
            &map_dir.join("regional-roads-v1.json"),
            &map_dir.join("regional-roads-v1.pack"),
            &terrain_dir.join("terrain-routing-v1.json"),
            &terrain_dir.join("terrain-routing-v1.pack"),
        ])
        .unwrap();
        assert_eq!(
            fs::read_to_string(map_dir.join(DATA_LICENSE_FILENAME)).unwrap(),
            DATA_LICENSE
        );
        assert_eq!(
            fs::read_to_string(terrain_dir.join(DATA_LICENSE_FILENAME)).unwrap(),
            DATA_LICENSE
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inferred_geometry_is_identical_in_road_package_and_routing_inputs() {
        let root = fixture();
        let mut package = build(
            &root,
            Wgs84BoundsE7::from_longitude_latitude_degrees(BOUNDS).unwrap(),
        )
        .unwrap();
        let geometry = [
            adventuresim_world_schema::TravelGeometryPoint::new(9.2, 51.2).unwrap(),
            adventuresim_world_schema::TravelGeometryPoint::new(9.3, 51.25).unwrap(),
        ];
        append_inferred_geometry(&mut package, &[&geometry]);
        let routing = package.routing_roads.last().unwrap();
        assert_eq!(routing.kind, RegionalConnectionKind::InferredWalkingLink);
        let features = terrain_features::build(&package, Vec::new(), "0".repeat(64));
        assert_eq!(
            features.roads.last().unwrap(),
            &routing
                .points
                .iter()
                .map(|point| point.0)
                .collect::<Vec<_>>()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn current_base_inputs_must_match_loaded_contract() {
        let digest = "a".repeat(64);
        let wet = "b".repeat(64);
        assert!(base_contract_matches(
            adventuresim_terrain::TerrainPurpose::DocumentedBase,
            BOUNDS,
            30,
            &digest,
            &wet,
            &digest,
            &wet
        ));
        assert!(!base_contract_matches(
            adventuresim_terrain::TerrainPurpose::Final,
            BOUNDS,
            30,
            &digest,
            &wet,
            &digest,
            &wet
        ));
        assert!(!base_contract_matches(
            adventuresim_terrain::TerrainPurpose::DocumentedBase,
            BOUNDS,
            30,
            &digest,
            &wet,
            &"c".repeat(64),
            &wet
        ));
        assert!(!base_contract_matches(
            adventuresim_terrain::TerrainPurpose::DocumentedBase,
            BOUNDS,
            30,
            &digest,
            &wet,
            &digest,
            &"d".repeat(64)
        ));
    }

    #[test]
    fn sidecar_rejects_unknown_fields_duplicates_and_fabricated_urls() {
        for mutation in 0..3 {
            let root = fixture();
            let path = root.join(".viabundus-source.json");
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            match mutation {
                0 => value["fabricated"] = serde_json::json!(true),
                1 => {
                    let duplicate = value["files"][0].clone();
                    value["files"].as_array_mut().unwrap().push(duplicate);
                }
                _ => {
                    value["files"][0]["url"] =
                        serde_json::json!("https://example.invalid/edges.csv")
                }
            }
            fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            assert!(
                build(
                    &root,
                    Wgs84BoundsE7::from_longitude_latitude_degrees(BOUNDS).unwrap()
                )
                .is_err()
            );
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn clipping_drops_outside_geometry_and_bounds_crossings() {
        assert!(clip_polyline(&[Point([-20.0, 40.0]), Point([-15.0, 41.0])], BOUNDS).is_empty());
        let crossing = clip_polyline(&[Point([0.0, 51.5]), Point([20.0, 51.5])], BOUNDS);
        assert_eq!(
            crossing,
            vec![vec![Point([BOUNDS[0], 51.5]), Point([BOUNDS[2], 51.5])]]
        );
        let polygon = clip_polygon(
            &[
                Point([8.0, 51.0]),
                Point([10.0, 51.0]),
                Point([10.0, 53.0]),
                Point([8.0, 51.0]),
            ],
            BOUNDS,
        );
        assert!(
            polygon
                .iter()
                .all(|point| point.0[0] >= BOUNDS[0] && point.0[0] <= BOUNDS[2])
        );
    }
}
