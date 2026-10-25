//! Full canonical road geometry, independent of raster presentation.
//!
//! The binary package preserves source longitude/latitude f64 values exactly.
//! Runtime admission checks their canonical geometry digest against the terrain
//! road mask before converting presentation positions to checked E7 values.
use crate::{TerrainPack, TerrainPurpose, hex_sha, valid_digest};
use adventuresim_world_schema::{
    coordinates::{Wgs84BoundsE7, Wgs84CoordinateE7, coordinates_in_bounds},
    regional_connection::RegionalConnectionKind,
    source_package::{SourcePackageDigest, SourcePackageDigestError},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

pub const ROAD_PACK_SCHEMA: u32 = 1;
pub const ROAD_MANIFEST_NAME: &str = "regional-roads-v1.json";
pub const ROAD_PACK_NAME: &str = "regional-roads-v1.pack";
pub const MAX_ROAD_SOURCE_POINTS: usize = 1_000_000;
pub const MAX_ROAD_SOURCE_LINES: usize = 50_000;
const MAX_ROAD_MANIFEST_BYTES: u64 = 4_096;
const POINT_BYTES: usize = 2 * std::mem::size_of::<f64>();
const LINE_LENGTH_BYTES: usize = std::mem::size_of::<u32>();
const LINE_HEADER_BYTES: usize = 1 + LINE_LENGTH_BYTES;
const MAX_ROAD_PACK_BYTES: u64 =
    (MAX_ROAD_SOURCE_POINTS * POINT_BYTES + MAX_ROAD_SOURCE_LINES * LINE_HEADER_BYTES) as u64;
pub type Result<T> = std::result::Result<T, RoadPackError>;

/// File metadata is an explicit native deployment adapter. Counts and hash
/// strings are admitted together with the bounded binary payload by `load`.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoadManifest {
    pub schema: u32,
    pub source: SourcePackageDigest,
    pub road_geometry_sha256: String,
    pub content_sha256: String,
    pub roads: usize,
    pub points: usize,
}

pub struct RoadPack {
    source: SourcePackageDigest,
    lines: Vec<RoadLine>,
}

pub struct RoadLine {
    kind: RegionalConnectionKind,
    points: Vec<Wgs84CoordinateE7>,
    bounds: Wgs84BoundsE7,
}

/// Offline numerical adapter: full-precision WGS84 longitude/latitude degrees.
pub struct NativeRoadLine {
    pub kind: RegionalConnectionKind,
    pub points: Vec<[f64; 2]>,
}

struct GeometryDigestWriter(Sha256);

#[derive(Debug, thiserror::Error)]
pub enum RoadPackError {
    #[error("road package I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("road package metadata: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Source(#[from] SourcePackageDigestError),
    #[error("road package exceeds its byte, line or point bound")]
    Capacity,
    #[error("road package requires the matching final terrain source")]
    SourceMismatch,
    #[error("road payload does not match its content digest")]
    ContentDigest,
    #[error("road geometry does not match the terrain road mask")]
    GeometryDigest,
    #[error("road geometry or classification is invalid, truncated or outside source bounds")]
    Geometry,
}

impl RoadPack {
    /// Call from blocking work on first road demand, rather than game startup.
    /// Admission temporarily retains at most the bounded payload and decoded
    /// source geometry. The resident package keeps only checked E7 positions.
    pub fn load(manifest_path: &Path, pack_path: &Path, terrain: &TerrainPack) -> Result<Self> {
        let manifest: RoadManifest =
            serde_json::from_slice(&read_bounded(manifest_path, MAX_ROAD_MANIFEST_BYTES)?)?;
        manifest.validate(terrain)?;
        let bytes = read_bounded(pack_path, MAX_ROAD_PACK_BYTES)?;
        if hex_sha(&bytes) != manifest.content_sha256 {
            return Err(RoadPackError::ContentDigest);
        }
        let geometry = decode(&bytes, &manifest)?;
        if geometry_digest(&geometry)? != manifest.road_geometry_sha256 {
            return Err(RoadPackError::GeometryDigest);
        }
        let lines = geometry
            .into_iter()
            .map(|line| RoadLine::from_native_degrees(line, terrain.bounds()))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            source: manifest.source,
            lines,
        })
    }

    pub fn source(&self) -> &SourcePackageDigest {
        &self.source
    }

    pub fn intersecting(&self, window: Wgs84BoundsE7) -> impl Iterator<Item = &RoadLine> {
        self.lines
            .iter()
            .filter(move |line| line.bounds.intersects(window))
    }

    #[cfg(feature = "builder")]
    /// Offline source adapter: pairs are full-precision WGS84 longitude and
    /// latitude degrees, in the exact order used by the terrain road-mask hash.
    pub fn write(
        manifest_path: &Path,
        pack_path: &Path,
        terrain: &TerrainPack,
        geometry: &[NativeRoadLine],
    ) -> Result<RoadManifest> {
        let points = geometry.iter().try_fold(0_usize, |total, line| {
            total
                .checked_add(line.points.len())
                .ok_or(RoadPackError::Capacity)
        })?;
        if points > MAX_ROAD_SOURCE_POINTS || geometry.len() > MAX_ROAD_SOURCE_LINES {
            return Err(RoadPackError::Capacity);
        }
        let digest = geometry_digest(geometry)?;
        let source = SourcePackageDigest::from_hex(terrain.digest())?;
        if terrain.purpose() != TerrainPurpose::Final || digest != terrain.road_geometry_sha256() {
            return Err(RoadPackError::GeometryDigest);
        }
        let mut bytes =
            Vec::with_capacity(points * POINT_BYTES + geometry.len() * LINE_HEADER_BYTES);
        for line in geometry {
            // Validate the actual payload before any deployment output exists.
            RoadLine::from_native_degrees(
                NativeRoadLine {
                    kind: line.kind,
                    points: line.points.clone(),
                },
                terrain.bounds(),
            )?;
            bytes.push(connection_tag(line.kind));
            bytes.extend_from_slice(&(line.points.len() as u32).to_le_bytes());
            for [longitude, latitude] in &line.points {
                bytes.extend_from_slice(&longitude.to_le_bytes());
                bytes.extend_from_slice(&latitude.to_le_bytes());
            }
        }
        let manifest = RoadManifest {
            schema: ROAD_PACK_SCHEMA,
            source,
            road_geometry_sha256: digest,
            content_sha256: hex_sha(&bytes),
            roads: geometry.len(),
            points,
        };
        let mut metadata = serde_json::to_vec(&manifest)?;
        metadata.push(b'\n');
        for path in [manifest_path, pack_path] {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(pack_path, bytes)?;
        fs::write(manifest_path, metadata)?;
        Ok(manifest)
    }
}

impl RoadManifest {
    fn validate(&self, terrain: &TerrainPack) -> Result<()> {
        if self.schema != ROAD_PACK_SCHEMA
            || terrain.purpose() != TerrainPurpose::Final
            || self.source.as_str() != terrain.digest()
            || self.road_geometry_sha256 != terrain.road_geometry_sha256()
        {
            return Err(RoadPackError::SourceMismatch);
        }
        if !valid_digest(&self.content_sha256) {
            return Err(RoadPackError::ContentDigest);
        }
        if self.points > MAX_ROAD_SOURCE_POINTS || self.roads > MAX_ROAD_SOURCE_LINES {
            return Err(RoadPackError::Capacity);
        }
        Ok(())
    }
}

impl RoadLine {
    pub fn kind(&self) -> RegionalConnectionKind {
        self.kind
    }

    pub fn points(&self) -> &[Wgs84CoordinateE7] {
        &self.points
    }

    fn from_native_degrees(line: NativeRoadLine, bounds: [f64; 4]) -> Result<Self> {
        if line.points.len() < 2 {
            return Err(RoadPackError::Geometry);
        }
        let points = line
            .points
            .into_iter()
            .map(|[longitude, latitude]| {
                if !coordinates_in_bounds(longitude, latitude, bounds) {
                    return Err(RoadPackError::Geometry);
                }
                Wgs84CoordinateE7::from_longitude_latitude_degrees(longitude, latitude)
                    .ok_or(RoadPackError::Geometry)
            })
            .collect::<Result<Vec<_>>>()?;
        let west = points
            .iter()
            .map(|point| point.longitude())
            .min()
            .ok_or(RoadPackError::Geometry)?;
        let east = points
            .iter()
            .map(|point| point.longitude())
            .max()
            .ok_or(RoadPackError::Geometry)?;
        let south = points
            .iter()
            .map(|point| point.latitude())
            .min()
            .ok_or(RoadPackError::Geometry)?;
        let north = points
            .iter()
            .map(|point| point.latitude())
            .max()
            .ok_or(RoadPackError::Geometry)?;
        Ok(Self {
            kind: line.kind,
            points,
            bounds: Wgs84BoundsE7::new(
                Wgs84CoordinateE7::from_components(south, west),
                Wgs84CoordinateE7::from_components(north, east),
            )
            .ok_or(RoadPackError::Geometry)?,
        })
    }
}

impl Write for GeometryDigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err(RoadPackError::Capacity);
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(RoadPackError::Capacity);
    }
    Ok(bytes)
}

fn geometry_digest(geometry: &[NativeRoadLine]) -> Result<String> {
    let mut writer = GeometryDigestWriter(Sha256::new());
    let points = geometry.iter().map(|line| &line.points).collect::<Vec<_>>();
    serde_json::to_writer(&mut writer, &points)?;
    Ok(format!("{:x}", writer.0.finalize()))
}

fn decode(bytes: &[u8], manifest: &RoadManifest) -> Result<Vec<NativeRoadLine>> {
    let mut cursor = bytes;
    let mut geometry = Vec::with_capacity(manifest.roads);
    let mut total = 0_usize;
    for _ in 0..manifest.roads {
        let [tag] = take_array(&mut cursor)?;
        let kind = connection_kind(tag)?;
        let count = u32::from_le_bytes(take_array(&mut cursor)?) as usize;
        total = total.checked_add(count).ok_or(RoadPackError::Capacity)?;
        if count < 2 || total > manifest.points || total > MAX_ROAD_SOURCE_POINTS {
            return Err(RoadPackError::Geometry);
        }
        let mut line = Vec::with_capacity(count);
        for _ in 0..count {
            let longitude = f64::from_le_bytes(take_array(&mut cursor)?);
            let latitude = f64::from_le_bytes(take_array(&mut cursor)?);
            line.push([longitude, latitude]);
        }
        geometry.push(NativeRoadLine { kind, points: line });
    }
    if !cursor.is_empty() || total != manifest.points {
        return Err(RoadPackError::Geometry);
    }
    Ok(geometry)
}

#[cfg(feature = "builder")]
fn connection_tag(kind: RegionalConnectionKind) -> u8 {
    match kind {
        RegionalConnectionKind::Land => 0,
        RegionalConnectionKind::River => 1,
        RegionalConnectionKind::Coast => 2,
        RegionalConnectionKind::Canal => 3,
        RegionalConnectionKind::Ferry => 4,
        RegionalConnectionKind::Winter => 5,
        RegionalConnectionKind::InferredWalkingLink => 6,
    }
}

fn connection_kind(tag: u8) -> Result<RegionalConnectionKind> {
    match tag {
        0 => Ok(RegionalConnectionKind::Land),
        1 => Ok(RegionalConnectionKind::River),
        2 => Ok(RegionalConnectionKind::Coast),
        3 => Ok(RegionalConnectionKind::Canal),
        4 => Ok(RegionalConnectionKind::Ferry),
        5 => Ok(RegionalConnectionKind::Winter),
        6 => Ok(RegionalConnectionKind::InferredWalkingLink),
        _ => Err(RoadPackError::Geometry),
    }
}

fn take_array<const N: usize>(cursor: &mut &[u8]) -> Result<[u8; N]> {
    let Some((bytes, rest)) = cursor.split_at_checked(N) else {
        return Err(RoadPackError::Geometry);
    };
    let array = bytes.try_into().map_err(|_| RoadPackError::Geometry)?;
    *cursor = rest;
    Ok(array)
}
