//! Scene materialization must admit only chunks placed in their native tiles.

use adventuresim_tactical_core::{
    regional_terrain::{
        REGIONAL_TERRAIN_SIDE, REGIONAL_TERRAIN_VERTICES, RegionalTerrain, RegionalTerrainRequest,
        RegionalTerrainScale,
    },
    scene_input::{SceneSource, TacticalSceneInput},
};
use adventuresim_tactical_server_dispatcher::regional_terrain::{
    self, RegionalTerrainCaptureError,
};
use adventuresim_terrain::{
    CHUNK_SIDE, Entry, Manifest, SCHEMA, TerrainPack, TerrainPurpose, builder,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use flate2::{Compression, write::DeflateEncoder};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
use tiff::encoder::{TiffEncoder, colortype::Gray32Float};

#[derive(Clone, Copy, Debug)]
enum ChunkFault {
    None,
    EqualAreaEdge,
    WidthOnly,
    HeightOnly,
    ShortInterior,
    OutsideX,
    OutsideY,
    MaximumX,
    MaximumY,
}

#[derive(Clone, Copy, Debug)]
enum PayloadFault {
    None,
    Short,
    WrongDigest,
    ReservedSurface,
}

#[derive(Clone, Copy)]
enum NeighborCoverage {
    Missing,
    Present,
}

#[test]
fn scene_materialization_rejects_misplaced_or_misshaped_chunks() {
    let root = temporary_directory();
    let (manifest_path, pack_path, digest) = write_fixture(&root, ChunkFault::None);
    let valid_pack_bytes = fs::read(&pack_path).unwrap();
    let valid_output = root.join("valid-scene");
    let result = materialize(&manifest_path, &pack_path, &valid_output);
    assert!(
        result.status.success(),
        "valid terrain must create a scene: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let scenes: Vec<_> = fs::read_dir(&valid_output)
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(scenes.len(), 1);
    let scene = TacticalSceneInput::load(&scenes[0].path()).unwrap();
    assert_eq!(scene.absolute_elevation_metres.get(), 222);
    let SceneSource::ImportedPackage(source) = scene.source else {
        panic!("the scene must retain its imported terrain identity");
    };
    assert_eq!(source.as_str(), digest);

    for fault in [
        ChunkFault::EqualAreaEdge,
        ChunkFault::WidthOnly,
        ChunkFault::HeightOnly,
        ChunkFault::ShortInterior,
        ChunkFault::OutsideX,
        ChunkFault::OutsideY,
        ChunkFault::MaximumX,
        ChunkFault::MaximumY,
    ] {
        let (manifest_path, pack_path, _) = write_fixture(&root, fault);
        if matches!(fault, ChunkFault::EqualAreaEdge) {
            assert_eq!(fs::read(&pack_path).unwrap(), valid_pack_bytes);
        }
        let output = root.join(format!("invalid-{fault:?}"));
        let result = materialize(&manifest_path, &pack_path, &output);
        assert!(
            !result.status.success(),
            "{fault:?} must prevent scene materialization"
        );
        assert!(
            !output.exists() || fs::read_dir(&output).unwrap().next().is_none(),
            "{fault:?} must not produce a scene document"
        );
    }
    for tile_width in [2_400, 3_600] {
        let (manifest, pack, _) = write_fixture_for_width(&root, ChunkFault::None, tile_width);
        let output = root.join(format!("valid-{tile_width}"));
        let result = materialize(&manifest, &pack, &output);
        assert!(
            result.status.success(),
            "valid {tile_width}-wide terrain must create a scene: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(fs::read_dir(output).unwrap().next().is_some());

        let (manifest, pack, _) = write_fixture_for_width(&root, ChunkFault::OutsideX, tile_width);
        let output = root.join(format!("outside-x-{tile_width}"));
        let result = materialize(&manifest, &pack, &output);
        assert!(
            !result.status.success(),
            "the first invalid x ordinal for width {tile_width} must prevent a scene"
        );
        assert!(!output.exists() || fs::read_dir(output).unwrap().next().is_none());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn regional_capture_preserves_nonuniform_interior_cells() {
    let root = temporary_directory();
    let (manifest, pack, _) = write_fixture(&root, ChunkFault::None);
    let terrain = TerrainPack::load(&manifest, &pack).unwrap();
    let origin =
        Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(10.5003, 50.4996).unwrap();
    let capture = regional_terrain::capture(
        &terrain,
        RegionalTerrainRequest {
            origin,
            scale: RegionalTerrainScale::Neighborhood,
        },
    )
    .unwrap();
    let center = REGIONAL_TERRAIN_VERTICES / 2;
    assert_eq!(capture.vertices()[center].unwrap().elevation.get(), 444);
    assert_eq!(capture.vertices()[center + 1].unwrap().elevation.get(), 333);
    assert_eq!(
        capture.vertices()[center + REGIONAL_TERRAIN_SIDE]
            .unwrap()
            .elevation
            .get(),
        222
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn regional_capture_preserves_nonuniform_native_edge_cells() {
    let root = temporary_directory();
    for (tile_width, edge_width) in [(1_800, 8), (2_400, 96), (3_600, 16)] {
        let directory = root.join(format!("tile-{tile_width}"));
        let (manifest, pack, _) = write_edge_fixture(
            &directory,
            tile_width,
            edge_width,
            PayloadFault::None,
            edge_bounds(),
            NeighborCoverage::Missing,
        );
        let terrain = TerrainPack::load(&manifest, &pack).unwrap();
        assert_known_regional_cells(&terrain);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn regional_capture_rejects_corrupt_payloads_and_missing_selected_chunks() {
    let root = temporary_directory();
    for fault in [
        PayloadFault::Short,
        PayloadFault::WrongDigest,
        PayloadFault::ReservedSurface,
    ] {
        let directory = root.join(format!("payload-{fault:?}"));
        let (manifest, pack, _) = write_edge_fixture(
            &directory,
            1_800,
            8,
            fault,
            edge_bounds(),
            NeighborCoverage::Missing,
        );
        let terrain = TerrainPack::load(&manifest, &pack).unwrap();
        assert!(
            matches!(
                capture_region(&terrain),
                Err(RegionalTerrainCaptureError::Terrain(
                    adventuresim_terrain::Error::Validation(_)
                ))
            ),
            "{fault:?} must not yield a regional presentation"
        );
    }
    let missing = root.join("missing-chunk");
    let (manifest, pack, _) = write_edge_fixture(
        &missing,
        1_800,
        8,
        PayloadFault::None,
        [10.98, 50.00001, 10.99999, 50.0043],
        NeighborCoverage::Missing,
    );
    let terrain = TerrainPack::load(&manifest, &pack).unwrap();
    assert!(matches!(
        capture_region(&terrain),
        Err(RegionalTerrainCaptureError::Terrain(
            adventuresim_terrain::Error::Validation(_)
        ))
    ));
    let complete = root.join("complete-chunks");
    let (manifest, pack, _) = write_edge_fixture(
        &complete,
        1_800,
        8,
        PayloadFault::None,
        [10.98, 50.00001, 10.99999, 50.0043],
        NeighborCoverage::Present,
    );
    let terrain = TerrainPack::load(&manifest, &pack).unwrap();
    assert_known_regional_cells(&terrain);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn regional_capture_preserves_pixels_from_the_cropped_raster_builder() {
    let root = temporary_directory();
    let elevation = root.join("elevation");
    let forest = root.join("forest");
    fs::create_dir_all(&elevation).unwrap();
    fs::create_dir_all(&forest).unwrap();
    let mut pixels = vec![100.0_f32; 1_800 * 3_600];
    pixels[3_600 * 1_800 - 1] = 333.0;
    pixels[3_599 * 1_800 - 1] = 222.0;
    let raster = elevation.join("Copernicus_DSM_COG_10_N50_00_E010_00_DEM.tif");
    let mut encoder = TiffEncoder::new(fs::File::create(raster).unwrap()).unwrap();
    encoder
        .write_image::<Gray32Float>(1_800, 3_600, &pixels)
        .unwrap();
    drop(encoder);
    let manifest_path = root.join("built-terrain.json");
    let pack_path = root.join("built-terrain.pack");
    let features = builder::Features {
        wetland_source_sha256: sha(b"wetlands"),
        cultivation_source_sha256: sha(b"cultivation"),
        cultivation_rules_version: 1,
        ..Default::default()
    };
    let manifest = builder::build(
        &elevation,
        &forest,
        edge_bounds(),
        &manifest_path,
        &pack_path,
        &features,
        TerrainPurpose::Final,
    )
    .unwrap();
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(
        (manifest.entries[0].width, manifest.entries[0].height),
        (8, 16)
    );
    let terrain = TerrainPack::load(&manifest_path, &pack_path).unwrap();
    assert_known_regional_cells(&terrain);
    fs::remove_dir_all(root).unwrap();
}

fn assert_known_regional_cells(terrain: &TerrainPack) {
    let capture = capture_region(terrain).unwrap();
    let center = REGIONAL_TERRAIN_VERTICES / 2;
    assert_eq!(capture.vertices()[center].unwrap().elevation.get(), 333);
    assert_eq!(
        capture.vertices()[center + REGIONAL_TERRAIN_SIDE]
            .unwrap()
            .elevation
            .get(),
        222
    );
    assert!(capture.vertices()[0].is_none());
    assert_eq!(capture.source().as_str(), terrain.digest());
}

fn capture_region(terrain: &TerrainPack) -> regional_terrain::Result<RegionalTerrain> {
    let origin =
        Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(10.9999, 50.0001).unwrap();
    regional_terrain::capture(
        terrain,
        RegionalTerrainRequest {
            origin,
            scale: RegionalTerrainScale::Neighborhood,
        },
    )
}

fn edge_bounds() -> [f64; 4] {
    [10.998, 50.00001, 10.99999, 50.0043]
}

fn write_edge_fixture(
    directory: &Path,
    tile_width: u16,
    edge_width: usize,
    fault: PayloadFault,
    bounds: [f64; 4],
    neighbor_coverage: NeighborCoverage,
) -> (PathBuf, PathBuf, String) {
    fs::create_dir_all(directory).unwrap();
    let mut decoded = [100, 0, 1, 0, 0].repeat(edge_width * 16);
    let last = (edge_width * 16 - 1) * 5;
    let above = (edge_width * 15 - 1) * 5;
    decoded[last..last + 2].copy_from_slice(&333_i16.to_le_bytes());
    decoded[above..above + 2].copy_from_slice(&222_i16.to_le_bytes());
    match fault {
        PayloadFault::Short => decoded.truncate(decoded.len() - 5),
        PayloadFault::ReservedSurface => decoded[2] = 6,
        PayloadFault::None | PayloadFault::WrongDigest => {}
    }
    let mut compressed = compress(&decoded);
    let digest = if matches!(fault, PayloadFault::WrongDigest) {
        sha(b"different decoded cells")
    } else {
        sha(&decoded)
    };
    let mut entries = vec![Entry {
        south: 50,
        west: 10,
        tile_width,
        tile_height: 3_600,
        chunk_x: (tile_width - 1) / CHUNK_SIDE,
        chunk_y: 14,
        width: edge_width as u16,
        height: 16,
        offset: 0,
        length: compressed.len() as u32,
        decoded_sha256: digest,
    }];
    if matches!(neighbor_coverage, NeighborCoverage::Present) {
        let neighbor = [100, 0, 1, 0, 0].repeat(usize::from(CHUNK_SIDE) * 16);
        let neighbor_compressed = compress(&neighbor);
        entries.push(Entry {
            south: 50,
            west: 10,
            tile_width,
            tile_height: 3_600,
            chunk_x: (tile_width - 1) / CHUNK_SIDE - 1,
            chunk_y: 14,
            width: CHUNK_SIDE,
            height: 16,
            offset: compressed.len() as u64,
            length: neighbor_compressed.len() as u32,
            decoded_sha256: sha(&neighbor),
        });
        compressed.extend_from_slice(&neighbor_compressed);
    }
    write_package(directory, entries, &compressed, bounds)
}

fn materialize(manifest: &Path, pack: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_materialize-real-world-scene"))
        .args(["--latitude", "50.5", "--longitude", "10.5"])
        .arg("--terrain-manifest")
        .arg(manifest)
        .arg("--terrain-pack")
        .arg(pack)
        .arg("--output-dir")
        .arg(output)
        .args(["--scene-key", "geometry-regression"])
        .output()
        .expect("run scene materializer")
}

fn write_fixture(root: &Path, fault: ChunkFault) -> (PathBuf, PathBuf, String) {
    write_fixture_for_width(root, fault, 1_800)
}

fn write_fixture_for_width(
    root: &Path,
    fault: ChunkFault,
    tile_width: u16,
) -> (PathBuf, PathBuf, String) {
    let directory = root.join(format!("package-{tile_width}-{fault:?}"));
    fs::create_dir_all(&directory).unwrap();
    let mut entries = Vec::new();
    let mut pack = Vec::new();
    let tile_height = 3_600_u16;
    for y in 0..tile_height.div_ceil(CHUNK_SIDE) {
        for x in 0..tile_width.div_ceil(CHUNK_SIDE) {
            let mut width = CHUNK_SIDE.min(tile_width - x * CHUNK_SIDE);
            let mut height = CHUNK_SIDE.min(tile_height - y * CHUNK_SIDE);
            let (mut declared_x, mut declared_y) = (x, y);
            if (x, y) == (tile_width.div_ceil(CHUNK_SIDE) - 1, 14) {
                match fault {
                    ChunkFault::EqualAreaEdge => (width, height) = (16, 8),
                    ChunkFault::WidthOnly => width = 7,
                    ChunkFault::HeightOnly => height = 15,
                    ChunkFault::OutsideX => declared_x = x + 1,
                    ChunkFault::OutsideY => declared_y = 15,
                    ChunkFault::MaximumX => declared_x = u16::MAX,
                    ChunkFault::MaximumY => declared_y = u16::MAX,
                    ChunkFault::None | ChunkFault::ShortInterior => {}
                }
            }
            if (x, y) == (0, 0) && matches!(fault, ChunkFault::ShortInterior) {
                (width, height) = (1, 2);
            }
            // The edge's equal-area case has nonuniform cells, though the
            // scene check exercises admission rather than their row order.
            let mut decoded = [65, 1, 1, 0, 0].repeat(usize::from(width) * usize::from(height));
            if (x, y) == (tile_width.div_ceil(CHUNK_SIDE) - 1, 14) {
                decoded[0..2].copy_from_slice(&222_i16.to_le_bytes());
                decoded[5..7].copy_from_slice(&333_i16.to_le_bytes());
            }
            if (x, y) == (3, 7) {
                for (local_x, local_y, elevation) in
                    [(132, 8, 222_i16), (132, 9, 444_i16), (133, 9, 333_i16)]
                {
                    let offset = (local_y * usize::from(width) + local_x) * 5;
                    decoded[offset..offset + 2].copy_from_slice(&elevation.to_le_bytes());
                }
            }
            let compressed = compress(&decoded);
            let offset = pack.len() as u64;
            pack.extend_from_slice(&compressed);
            entries.push(Entry {
                south: 50,
                west: 10,
                tile_width,
                tile_height,
                chunk_x: declared_x,
                chunk_y: declared_y,
                width,
                height,
                offset,
                length: compressed.len() as u32,
                decoded_sha256: sha(&decoded),
            });
        }
    }
    write_package(&directory, entries, &pack, [10.0, 50.0, 11.0, 51.0])
}

fn write_package(
    directory: &Path,
    entries: Vec<Entry>,
    pack: &[u8],
    bounds: [f64; 4],
) -> (PathBuf, PathBuf, String) {
    let mut manifest = Manifest {
        schema: SCHEMA,
        purpose: TerrainPurpose::Final,
        bounds,
        source_resolution_m: 30,
        content_sha256: sha(pack),
        road_geometry_sha256: sha(b"roads"),
        wetland_source_sha256: sha(b"wetlands"),
        wetland_cells: 0,
        cultivation_grid_crs: "EPSG:3035".into(),
        cultivation_grid_resolution_m: 1_000,
        cultivation_rules_version: 1,
        cultivation_source_sha256: sha(b"cultivation"),
        cultivated_square_count: 0,
        cultivated_native_cells: 0,
        terrain_features: Vec::new(),
        entries,
        package_sha256: "0".repeat(64),
    };
    manifest.package_sha256 = sha(&serde_json::to_vec(&manifest).unwrap());
    let digest = manifest.package_sha256.clone();
    let manifest_path = directory.join("terrain.json");
    let pack_path = directory.join("terrain.pack");
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    fs::write(&pack_path, pack).unwrap();
    (manifest_path, pack_path, digest)
}

fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn temporary_directory() -> PathBuf {
    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    loop {
        let nonce = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "terrain-geometry-scene-{}-{nonce}",
            std::process::id()
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create test directory: {error}"),
        }
    }
}
