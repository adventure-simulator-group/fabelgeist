//! Bounded native-detail strategic terrain packs and deterministic routing.
//!
//! The pack keeps GLO-30 pixels in compressed 256×256 chunks on disk. Runtime
//! consumers decompress only a bounded LRU and never put the continental grid
//! in SpacetimeDB or in one allocation.
//! Each entry asserts the full or true-edge chunk dimensions implied by its
//! native tile and ordinals. Encoded cells are row-major, with x varying
//! fastest; cropped packages may omit whole nonintersecting chunks.

use admission::{ChunkGeometry, validate_manifest};
use adventuresim_world_schema::{BASIS_POINTS_PER_WHOLE, TerrainFeature};
use digest::{hex_sha, hex_sha_reader, valid_digest};
use flate2::read::DeflateDecoder;
#[cfg(test)]
use routing::compact_spans;
pub use routing::{
    RoutePlan, RoutePoint, RoutingGrid, TerrainSpan, astar, astar_with_profile,
    astar_with_profile_and_weather,
};
use routing::{astar_with_profile_and_deadline, passable, simplify_route, snap};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom},
    ops::Range,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

mod admission;
mod digest;
pub mod road_pack;
mod routing;
mod terrain_feature;

pub const SCHEMA: u32 = 8;
pub const CHUNK_SIDE: u16 = 256;
pub const MAX_ENTRIES: usize = 20_000;
pub const MAX_PACK_BYTES: usize = 2 * 1024 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u64 = 32 * 1024 * 1024;
const CULTIVATION_GRID_CRS: &str = "EPSG:3035";
const CELL_BYTES: usize = 5;
pub const MAX_DECODED_CHUNK_BYTES: usize = 256 * 256 * CELL_BYTES;
pub const CACHE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_ROUTE_NODES: usize = 750_000;
pub const MAX_ROUTE_POINTS: usize = 512;
pub const MAX_TERRAIN_SPANS: usize = 256;

#[cfg(feature = "builder")]
pub mod builder;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("terrain I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("terrain manifest: {0}")]
    Json(#[from] serde_json::Error),
    #[error("terrain validation: {0}")]
    Validation(String),
    #[error("no bounded route was found")]
    NoRoute,
    #[error("terrain route planning deadline elapsed")]
    Deadline,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Road,
    #[default]
    Open,
    SparseWoods,
    DeepWoods,
    Water,
    Wetland,
}

impl Surface {
    // The route heuristic uses this upper bound for every surface speed.
    const MAX_SPEED_METRES_PER_HOUR: u32 = 5_000;

    pub fn speed_metres_per_hour(self) -> u32 {
        let speed = match self {
            Self::Road => Self::MAX_SPEED_METRES_PER_HOUR,
            Self::Open => 1_250,
            Self::SparseWoods => 1_000,
            Self::DeepWoods => 750,
            Self::Water => 0,
            Self::Wetland => 500,
        };
        assert!(speed <= Self::MAX_SPEED_METRES_PER_HOUR);
        speed
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    pub elevation_m: i16,
    pub surface: Surface,
    /// Roads over water are valid bridges/ferries/fords.
    pub crossing: bool,
    /// Whether the canonical EPSG:3035 1 km square is cultivated.
    /// This uses a spare bit in the existing flags byte.
    pub cultivated: bool,
    /// Copernicus TCD canopy cover, retained independently of routing classes.
    pub canopy_percent: u8,
    /// Area share whose native neighbours form slopes steeper than 15 degrees.
    /// Native pack bits decode to 0 or 100; coarser cells retain the average.
    pub hilly_fraction_percent: u8,
    /// Area share covered by wetlands. Native pack bits decode to 0 or 100;
    /// coarser routing cells retain the average.
    pub wetland_fraction_percent: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainWeights {
    pub plains: u16,
    pub forest: u16,
    pub hills: u16,
    pub wetlands: u16,
    pub urban: u16,
}

impl TerrainWeights {
    pub const TOTAL: u16 = 1_000;
    /// Dry ground below ten percent imported wetlands absorbs rainfall without
    /// becoming mechanically wetland.
    pub const WETLAND_SUSCEPTIBILITY_THRESHOLD: u16 = 100;
    pub fn from_cover(
        canopy_percent: u8,
        hilly_fraction_percent: u8,
        wetland_fraction_percent: u8,
    ) -> Self {
        let wetlands = u16::from(wetland_fraction_percent.min(100)) * 10;
        let remainder = Self::TOTAL - wetlands;
        let forest = (u32::from(remainder) * u32::from(canopy_percent.min(100)) / 100) as u16;
        let hills = (u32::from(remainder - forest) * u32::from(hilly_fraction_percent.min(100))
            / 100) as u16;
        Self {
            plains: remainder - forest - hills,
            forest,
            hills,
            wetlands,
            urban: 0,
        }
    }
    pub const fn is_normalized(self) -> bool {
        self.plains <= Self::TOTAL
            && self.forest <= Self::TOTAL
            && self.hills <= Self::TOTAL
            && self.wetlands <= Self::TOTAL
            && self.urban <= Self::TOTAL
            && self.plains + self.forest + self.hills + self.wetlands + self.urban == Self::TOTAL
    }
    pub fn dot(self, profile: TerrainSkillProfile) -> u16 {
        debug_assert!(self.is_normalized());
        ((u32::from(self.plains) * u32::from(profile.plains)
            + u32::from(self.forest) * u32::from(profile.forest)
            + u32::from(self.hills) * u32::from(profile.hills)
            + u32::from(self.wetlands) * u32::from(profile.wetlands)
            + u32::from(self.urban) * u32::from(profile.urban))
            / u32::from(Self::TOTAL)) as u16
    }

    /// Derive temporary route weights and independent mud severity from an
    /// authoritative departure-time ground-moisture snapshot.
    ///
    /// The imported five-member mixture is never changed in storage. Wetland
    /// gain displaces the four underlying weights proportionally and the
    /// final remainder is assigned deterministically, preserving normalization.
    pub fn with_ground_moisture(self, moisture_bps: u16) -> WeatheredTerrain {
        debug_assert!(self.is_normalized());
        let moisture = u32::from(moisture_bps.min(BASIS_POINTS_PER_WHOLE));
        let susceptibility = if self.wetlands >= Self::WETLAND_SUSCEPTIBILITY_THRESHOLD {
            u32::from(Self::TOTAL - self.wetlands)
        } else {
            0
        };
        let bonus = (susceptibility * moisture / 20_000) as u16;
        let target_wetlands = self.wetlands.saturating_add(bonus).min(Self::TOTAL);
        let old_underlying = Self::TOTAL - self.wetlands;
        let new_underlying = Self::TOTAL - target_wetlands;
        let scale = |weight: u16| -> u16 {
            if old_underlying == 0 {
                0
            } else {
                (u32::from(weight) * u32::from(new_underlying) / u32::from(old_underlying)) as u16
            }
        };
        let forest = scale(self.forest);
        let hills = scale(self.hills);
        let urban = scale(self.urban);
        let plains = new_underlying - forest - hills - urban;
        let weights = Self {
            plains,
            forest,
            hills,
            wetlands: target_wetlands,
            urban,
        };
        // Independent severity lets already-100% wetlands continue to worsen.
        let saturation_bps = if self.wetlands < Self::WETLAND_SUSCEPTIBILITY_THRESHOLD {
            0
        } else {
            (moisture * (5_000 + u32::from(self.wetlands) * 5) / u32::from(BASIS_POINTS_PER_WHOLE))
                .min(u32::from(BASIS_POINTS_PER_WHOLE)) as u16
        };
        WeatheredTerrain {
            weights,
            saturation_bps,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatheredTerrain {
    pub weights: TerrainWeights,
    /// Mud/waterlogging speed severity, independent of normalized biome mix.
    pub saturation_bps: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainSkillProfile {
    pub plains: u16,
    pub forest: u16,
    pub hills: u16,
    pub wetlands: u16,
    pub urban: u16,
}

/// Departure-time overlays used consistently by route search and attestation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingWeather {
    pub ground_moisture_bps: u16,
    pub snow_cover_bps: u16,
    pub snow_check_millirank: u16,
}

impl RoutingWeather {
    pub const fn is_valid(self) -> bool {
        self.ground_moisture_bps <= BASIS_POINTS_PER_WHOLE
            && self.snow_cover_bps <= BASIS_POINTS_PER_WHOLE
            && self.snow_check_millirank <= TerrainSkillProfile::MAX
    }
}

impl TerrainSkillProfile {
    pub const MAX: u16 = 5_000;
    pub const fn is_valid(self) -> bool {
        self.plains <= Self::MAX
            && self.forest <= Self::MAX
            && self.hills <= Self::MAX
            && self.wetlands <= Self::MAX
            && self.urban <= Self::MAX
    }
}

impl Cell {
    pub fn terrain_weights(self) -> TerrainWeights {
        TerrainWeights::from_cover(
            self.canopy_percent,
            self.hilly_fraction_percent,
            self.wetland_fraction_percent,
        )
    }
    fn underlying_surface(self) -> Surface {
        if self.wetland_fraction_percent >= 50 {
            return Surface::Wetland;
        }
        match self.canopy_percent {
            45..=u8::MAX => Surface::DeepWoods,
            10..=44 => Surface::SparseWoods,
            _ => Surface::Open,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub purpose: TerrainPurpose,
    pub bounds: [f64; 4],
    pub source_resolution_m: u16,
    pub content_sha256: String,
    pub road_geometry_sha256: String,
    pub wetland_source_sha256: String,
    pub wetland_cells: u64,
    pub cultivation_grid_crs: String,
    pub cultivation_grid_resolution_m: u16,
    pub cultivation_rules_version: u16,
    pub cultivation_source_sha256: String,
    pub cultivated_square_count: u64,
    pub cultivated_native_cells: u64,
    /// Bounded vector observations used to derive local terrain features.
    pub terrain_features: Vec<TerrainFeature>,
    pub entries: Vec<Entry>,
    pub package_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerrainPurpose {
    DocumentedBase,
    Final,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub south: i16,
    pub west: i16,
    pub tile_width: u16,
    pub tile_height: u16,
    pub chunk_x: u16,
    pub chunk_y: u16,
    /// Producer assertion for the encoded chunk's native-pixel width.
    pub width: u16,
    /// Producer assertion for the encoded chunk's native-pixel height.
    pub height: u16,
    pub offset: u64,
    pub length: u32,
    pub decoded_sha256: String,
}

/// Bounded decoded chunks owned by the immutable package. Cache clocks select
/// eviction order; they are not world time or an independent terrain version.
struct Cache {
    clock: u64,
    bytes: usize,
    chunks: HashMap<usize, (u64, Arc<[Cell]>)>,
}

struct IndexedChunk {
    range: Range<usize>,
    geometry: ChunkGeometry,
}

pub struct TerrainPack {
    manifest: Manifest,
    file: Mutex<File>,
    chunks: Vec<IndexedChunk>,
    index: HashMap<(i16, i16, u16, u16), usize>,
    tiles: HashMap<(i16, i16), (u16, u16)>,
    cache: Mutex<Cache>,
}

impl TerrainPack {
    pub fn load(manifest_path: &Path, pack_path: &Path) -> Result<Self> {
        let manifest_length = fs::metadata(manifest_path)?.len();
        if manifest_length == 0 || manifest_length > MAX_MANIFEST_BYTES {
            return Err(Error::Validation(
                "terrain manifest exceeds its byte bound".into(),
            ));
        }
        let json = fs::read(manifest_path)?;
        let manifest: Manifest = serde_json::from_slice(&json)?;
        validate_manifest(&manifest)?;
        let mut unsigned = manifest.clone();
        unsigned.package_sha256 = "0".repeat(64);
        if hex_sha(&serde_json::to_vec(&unsigned)?) != manifest.package_sha256 {
            return Err(Error::Validation("manifest digest mismatch".into()));
        }
        let pack_length = fs::metadata(pack_path)?.len();
        if pack_length == 0 || pack_length > MAX_PACK_BYTES as u64 {
            return Err(Error::Validation("pack exceeds 2 GiB bound".into()));
        }
        let mut file = File::open(pack_path)?;
        if hex_sha_reader(BufReader::new(&mut file))? != manifest.content_sha256 {
            return Err(Error::Validation("pack digest mismatch".into()));
        }
        file.seek(SeekFrom::Start(0))?;
        let mut ranges = Vec::with_capacity(manifest.entries.len());
        let mut previous_end = 0;
        for entry in &manifest.entries {
            let start = usize::try_from(entry.offset)
                .map_err(|_| Error::Validation("chunk offset overflow".into()))?;
            let end = start
                .checked_add(entry.length as usize)
                .ok_or_else(|| Error::Validation("chunk range overflow".into()))?;
            if start < previous_end || end as u64 > pack_length {
                return Err(Error::Validation(
                    "chunk ranges overlap or exceed pack".into(),
                ));
            }
            previous_end = end;
            ranges.push(start..end);
        }
        let mut index = HashMap::with_capacity(manifest.entries.len());
        let mut tiles = HashMap::new();
        for (position, entry) in manifest.entries.iter().enumerate() {
            if index
                .insert(
                    (entry.south, entry.west, entry.chunk_x, entry.chunk_y),
                    position,
                )
                .is_some()
            {
                return Err(Error::Validation("duplicate terrain chunk".into()));
            }
            if tiles
                .insert(
                    (entry.south, entry.west),
                    (entry.tile_width, entry.tile_height),
                )
                .is_some_and(|dimensions| dimensions != (entry.tile_width, entry.tile_height))
            {
                return Err(Error::Validation(
                    "inconsistent source tile dimensions".into(),
                ));
            }
        }
        let chunks = ranges
            .into_iter()
            .zip(&manifest.entries)
            .map(|(range, entry)| {
                Ok(IndexedChunk {
                    range,
                    geometry: ChunkGeometry::admit(entry)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            manifest,
            file: Mutex::new(file),
            chunks,
            index,
            tiles,
            cache: Mutex::new(Cache {
                clock: 0,
                bytes: 0,
                chunks: HashMap::new(),
            }),
        })
    }

    pub const fn purpose(&self) -> TerrainPurpose {
        self.manifest.purpose
    }
    pub fn package_sha256(&self) -> &str {
        &self.manifest.package_sha256
    }
    pub fn road_geometry_sha256(&self) -> &str {
        &self.manifest.road_geometry_sha256
    }
    pub fn wetland_source_sha256(&self) -> &str {
        &self.manifest.wetland_source_sha256
    }
    pub const fn source_resolution_m(&self) -> u16 {
        self.manifest.source_resolution_m
    }
    pub const fn wetland_cells(&self) -> u64 {
        self.manifest.wetland_cells
    }
    pub fn cultivation_source_sha256(&self) -> &str {
        &self.manifest.cultivation_source_sha256
    }
    pub fn digest(&self) -> &str {
        &self.manifest.package_sha256
    }
    pub fn bounds(&self) -> [f64; 4] {
        self.manifest.bounds
    }

    pub fn cell(&self, latitude: f64, longitude: f64) -> Result<Option<Cell>> {
        if !latitude.is_finite() || !longitude.is_finite() {
            return Ok(None);
        }
        let [west_bound, south_bound, east_bound, north_bound] = self.bounds();
        if longitude < west_bound
            || longitude > east_bound
            || latitude < south_bound
            || latitude > north_bound
        {
            return Ok(None);
        }
        let south = latitude.floor() as i16;
        let west = longitude.floor() as i16;
        let Some(&(tile_width, tile_height)) = self.tiles.get(&(south, west)) else {
            return Ok(None);
        };
        let x = ((longitude - f64::from(west)) * f64::from(tile_width))
            .floor()
            .clamp(0.0, f64::from(tile_width - 1)) as u16;
        let y = ((f64::from(south + 1) - latitude) * f64::from(tile_height))
            .floor()
            .clamp(0.0, f64::from(tile_height - 1)) as u16;
        let chunk_x = x / CHUNK_SIDE;
        let chunk_y = y / CHUNK_SIDE;
        let &index = self
            .index
            .get(&(south, west, chunk_x, chunk_y))
            .ok_or_else(|| Error::Validation("manifest has a hole in a source tile".into()))?;
        let cells = self.chunk(index)?;
        let local_x = x - chunk_x * CHUNK_SIDE;
        let local_y = y - chunk_y * CHUNK_SIDE;
        Ok(self.chunks[index]
            .geometry
            .cell_index(local_x, local_y)
            .and_then(|position| cells.get(position))
            .copied())
    }

    fn chunk(&self, index: usize) -> Result<Arc<[Cell]>> {
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|_| Error::Validation("terrain cache poisoned".into()))?;
            cache.clock = cache
                .clock
                .checked_add(1)
                .ok_or_else(|| Error::Validation("cache clock overflow".into()))?;
            let clock = cache.clock;
            if let Some((used, cells)) = cache.chunks.get_mut(&index) {
                *used = clock;
                return Ok(cells.clone());
            }
        }
        let entry = &self.manifest.entries[index];
        let expected = self.chunks[index].geometry.decoded_bytes();
        if expected > MAX_DECODED_CHUNK_BYTES {
            return Err(Error::Validation("decoded chunk exceeds bound".into()));
        }
        let range = self.chunks[index].range.clone();
        let mut compressed = vec![0_u8; range.len()];
        {
            let mut file = self
                .file
                .lock()
                .map_err(|_| Error::Validation("terrain pack file lock poisoned".into()))?;
            file.seek(SeekFrom::Start(range.start as u64))?;
            file.read_exact(&mut compressed)?;
        }
        let mut decoded = Vec::with_capacity(expected);
        DeflateDecoder::new(compressed.as_slice())
            .take((MAX_DECODED_CHUNK_BYTES + 1) as u64)
            .read_to_end(&mut decoded)?;
        if decoded.len() != expected || hex_sha(&decoded) != entry.decoded_sha256 {
            return Err(Error::Validation("chunk is corrupt or truncated".into()));
        }
        if decoded
            .as_chunks::<CELL_BYTES>()
            .0
            .iter()
            .any(|bytes| bytes[2] > 5 || bytes[3] & !0b1111 != 0)
        {
            return Err(Error::Validation(
                "chunk contains an unknown surface or flag discriminant".into(),
            ));
        }
        let cells: Arc<[Cell]> = decoded
            .as_chunks::<CELL_BYTES>()
            .0
            .iter()
            .map(|bytes| Cell {
                elevation_m: i16::from_le_bytes([bytes[0], bytes[1]]),
                surface: match bytes[2] {
                    0 => Surface::Road,
                    1 => Surface::Open,
                    2 => Surface::SparseWoods,
                    3 => Surface::DeepWoods,
                    4 => Surface::Water,
                    5 => Surface::Wetland,
                    _ => unreachable!("surface discriminants were validated"),
                },
                crossing: bytes[3] & 1 != 0,
                hilly_fraction_percent: if bytes[3] & 2 != 0 { 100 } else { 0 },
                cultivated: bytes[3] & 4 != 0,
                wetland_fraction_percent: if bytes[3] & 8 != 0 { 100 } else { 0 },
                canopy_percent: bytes[4],
            })
            .collect::<Vec<_>>()
            .into();
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| Error::Validation("terrain cache poisoned".into()))?;
        cache.clock = cache
            .clock
            .checked_add(1)
            .ok_or_else(|| Error::Validation("cache clock overflow".into()))?;
        let clock = cache.clock;
        if let Some((used, existing)) = cache.chunks.get_mut(&index) {
            *used = clock;
            return Ok(existing.clone());
        }
        while cache.bytes + decoded.len() > CACHE_BYTES {
            let victim = cache
                .chunks
                .iter()
                .min_by_key(|(key, (used, _))| (*used, **key))
                .map(|(key, _)| *key)
                .ok_or_else(|| Error::Validation("cache cannot admit chunk".into()))?;
            let removed = cache.chunks.remove(&victim).expect("cache victim exists");
            cache.bytes -= removed.1.len() * CELL_BYTES;
        }
        cache.bytes += decoded.len();
        cache.chunks.insert(index, (clock, cells.clone()));
        Ok(cells)
    }

    /// Plan fastest travel through a bounded geographic window. The window
    /// starts at native 30 m spacing and deterministically coarsens only when
    /// the hard node cap would otherwise be exceeded.
    pub fn plan(&self, start: (f64, f64), goal: (f64, f64)) -> Result<RoutePlan> {
        self.plan_with_profile(start, goal, TerrainSkillProfile::default())
    }

    pub fn plan_with_profile(
        &self,
        start: (f64, f64),
        goal: (f64, f64),
        profile: TerrainSkillProfile,
    ) -> Result<RoutePlan> {
        self.plan_with_deadline(start, goal, profile, RoutingWeather::default(), None)
    }

    /// Plan with a cooperative deadline. This is intended for request-serving
    /// callers so a timed-out blocking task stops consuming its worker slot.
    pub fn plan_until(
        &self,
        start: (f64, f64),
        goal: (f64, f64),
        deadline: Instant,
    ) -> Result<RoutePlan> {
        self.plan_until_with_profile(start, goal, TerrainSkillProfile::default(), deadline)
    }

    pub fn plan_until_with_profile(
        &self,
        start: (f64, f64),
        goal: (f64, f64),
        profile: TerrainSkillProfile,
        deadline: Instant,
    ) -> Result<RoutePlan> {
        self.plan_with_deadline(
            start,
            goal,
            profile,
            RoutingWeather::default(),
            Some(deadline),
        )
    }

    pub fn plan_until_with_profile_and_weather(
        &self,
        start: (f64, f64),
        goal: (f64, f64),
        profile: TerrainSkillProfile,
        weather: RoutingWeather,
        deadline: Instant,
    ) -> Result<RoutePlan> {
        self.plan_with_deadline(start, goal, profile, weather, Some(deadline))
    }

    fn plan_with_deadline(
        &self,
        start: (f64, f64),
        goal: (f64, f64),
        profile: TerrainSkillProfile,
        weather: RoutingWeather,
        deadline: Option<Instant>,
    ) -> Result<RoutePlan> {
        if !profile.is_valid() {
            return Err(Error::Validation(
                "terrain skill profile exceeds rank five".into(),
            ));
        }
        if !weather.is_valid() {
            return Err(Error::Validation(
                "routing weather exceeds bounded values".into(),
            ));
        }
        if ![start.0, start.1, goal.0, goal.1]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(Error::NoRoute);
        }
        let [west, south, east, north] = self.bounds();
        let inside = |(latitude, longitude): (f64, f64)| {
            longitude >= west && longitude <= east && latitude >= south && latitude <= north
        };
        if !inside(start) || !inside(goal) {
            return Err(Error::NoRoute);
        }
        for window in expanding_windows(start, goal, self.bounds()) {
            check_deadline(deadline)?;
            match self.plan_window(start, goal, window, profile, weather, deadline) {
                Ok(plan) => return Ok(plan),
                Err(Error::NoRoute) => continue,
                Err(error) => return Err(error),
            }
        }
        Err(Error::NoRoute)
    }

    fn plan_window(
        &self,
        start: (f64, f64),
        goal: (f64, f64),
        [west, south, east, north]: [f64; 4],
        profile: TerrainSkillProfile,
        weather: RoutingWeather,
        deadline: Option<Instant>,
    ) -> Result<RoutePlan> {
        if west >= east || south >= north {
            return Err(Error::NoRoute);
        }
        let latitude = (south + north) * 0.5;
        let mut step_lat = 30.0 / 111_320.0;
        let mut step_lon = 30.0 / (111_320.0 * latitude.to_radians().cos().abs().max(0.2));
        loop {
            let width = ((east - west) / step_lon).ceil() as usize + 1;
            let height = ((north - south) / step_lat).ceil() as usize + 1;
            if width
                .checked_mul(height)
                .is_some_and(|count| count <= MAX_ROUTE_NODES)
            {
                break;
            }
            step_lat *= 1.25;
            step_lon *= 1.25;
        }
        let width = ((east - west) / step_lon).ceil() as u32 + 1;
        let height = ((north - south) / step_lat).ceil() as u32 + 1;
        let mut cells = Vec::with_capacity(width as usize * height as usize);
        for y in 0..height {
            if y % 16 == 0 {
                check_deadline(deadline)?;
            }
            let lat = north - f64::from(y) * step_lat;
            for x in 0..width {
                let lon = west + f64::from(x) * step_lon;
                cells.push(self.sample_coarsened(lat, lon, step_lat, step_lon)?);
            }
        }
        let grid = WindowGrid {
            width,
            height,
            cells,
            metres: (step_lat * 111_320.0).round().max(1.0) as u32,
        };
        let coordinate = |value: (f64, f64)| {
            (
                ((value.1 - west) / step_lon)
                    .round()
                    .clamp(0.0, f64::from(width - 1)) as u32,
                ((north - value.0) / step_lat)
                    .round()
                    .clamp(0.0, f64::from(height - 1)) as u32,
            )
        };
        let start_node = snap(&grid, coordinate(start), 8).ok_or(Error::NoRoute)?;
        let goal_node = snap(&grid, coordinate(goal), 8).ok_or(Error::NoRoute)?;
        let mut plan = astar_with_profile_and_deadline(
            &grid, start_node, goal_node, profile, weather, deadline,
        )?;
        for point in &mut plan.points {
            let x = point.longitude;
            let y = point.latitude;
            point.longitude = west + x * step_lon;
            point.latitude = north - y * step_lat;
        }
        simplify_route(&mut plan.points, MAX_ROUTE_POINTS);
        Ok(plan)
    }

    fn sample_coarsened(
        &self,
        latitude: f64,
        longitude: f64,
        step_lat: f64,
        step_lon: f64,
    ) -> Result<Cell> {
        let native_lat = 30.0 / 111_320.0;
        let samples = (step_lat / native_lat).ceil().clamp(1.0, 9.0) as usize;
        let mut cells = Vec::with_capacity(samples * samples);
        for row in 0..samples {
            for column in 0..samples {
                let x = (column as f64 + 0.5) / samples as f64 - 0.5;
                let y = (row as f64 + 0.5) / samples as f64 - 0.5;
                cells.push(
                    self.cell(latitude - y * step_lat, longitude + x * step_lon)?
                        .unwrap_or(Cell {
                            elevation_m: 0,
                            surface: Surface::Water,
                            crossing: false,
                            cultivated: false,
                            canopy_percent: 0,
                            hilly_fraction_percent: 0,
                            wetland_fraction_percent: 0,
                        }),
                );
            }
        }
        Ok(aggregate_cells(&cells))
    }
}

fn check_deadline(deadline: Option<Instant>) -> Result<()> {
    if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        Err(Error::Deadline)
    } else {
        Ok(())
    }
}

fn expanding_windows(start: (f64, f64), goal: (f64, f64), bounds: [f64; 4]) -> Vec<[f64; 4]> {
    let [pack_west, pack_south, pack_east, pack_north] = bounds;
    let span = (start.1 - goal.1).abs().max((start.0 - goal.0).abs());
    let mut windows = Vec::new();
    for factor in [0.25, 0.5, 1.0, 2.0] {
        let padding = (span * factor).max(0.01);
        let window = [
            (start.1.min(goal.1) - padding).max(pack_west),
            (start.0.min(goal.0) - padding).max(pack_south),
            (start.1.max(goal.1) + padding).min(pack_east),
            (start.0.max(goal.0) + padding).min(pack_north),
        ];
        if windows.last() != Some(&window) {
            windows.push(window);
        }
    }
    if windows.last() != Some(&bounds) {
        windows.push(bounds);
    }
    windows
}

fn aggregate_cells(cells: &[Cell]) -> Cell {
    if let Some(road) = cells.iter().find(|cell| cell.surface == Surface::Road) {
        return Cell {
            elevation_m: road.elevation_m,
            surface: Surface::Road,
            crossing: cells.iter().any(|cell| cell.crossing),
            cultivated: cells.iter().any(|cell| cell.cultivated),
            canopy_percent: cells
                .iter()
                .map(|cell| u64::from(cell.canopy_percent))
                .sum::<u64>()
                .checked_div(cells.len() as u64)
                .unwrap_or_default() as u8,
            hilly_fraction_percent: (cells
                .iter()
                .map(|cell| u64::from(cell.hilly_fraction_percent))
                .sum::<u64>()
                / cells.len() as u64) as u8,
            wetland_fraction_percent: (cells
                .iter()
                .map(|cell| u64::from(cell.wetland_fraction_percent))
                .sum::<u64>()
                / cells.len() as u64) as u8,
        };
    }
    let passable = cells
        .iter()
        .filter(|cell| passable(**cell))
        .collect::<Vec<_>>();
    if passable.is_empty() {
        return Cell {
            elevation_m: 0,
            surface: Surface::Water,
            crossing: false,
            cultivated: false,
            canopy_percent: 0,
            hilly_fraction_percent: 0,
            wetland_fraction_percent: 0,
        };
    }
    let mut counts = [0_usize; 4];
    for cell in &passable {
        match cell.surface {
            Surface::Open => counts[0] += 1,
            Surface::SparseWoods => counts[1] += 1,
            Surface::DeepWoods => counts[2] += 1,
            Surface::Wetland => counts[3] += 1,
            _ => {}
        }
    }
    let surface = [
        Surface::Open,
        Surface::SparseWoods,
        Surface::DeepWoods,
        Surface::Wetland,
    ]
    .into_iter()
    .enumerate()
    .max_by_key(|(index, _)| (counts[*index], *index))
    .map(|(_, surface)| surface)
    .unwrap_or(Surface::Open);
    Cell {
        elevation_m: (passable
            .iter()
            .map(|cell| i64::from(cell.elevation_m))
            .sum::<i64>()
            / passable.len() as i64) as i16,
        surface,
        crossing: passable.iter().any(|cell| cell.crossing),
        cultivated: passable.iter().any(|cell| cell.cultivated),
        canopy_percent: (passable
            .iter()
            .map(|cell| u64::from(cell.canopy_percent))
            .sum::<u64>()
            / passable.len() as u64) as u8,
        hilly_fraction_percent: (passable
            .iter()
            .map(|cell| u64::from(cell.hilly_fraction_percent))
            .sum::<u64>()
            / passable.len() as u64) as u8,
        wetland_fraction_percent: (passable
            .iter()
            .map(|cell| u64::from(cell.wetland_fraction_percent))
            .sum::<u64>()
            / passable.len() as u64) as u8,
    }
}

struct WindowGrid {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    metres: u32,
}
impl RoutingGrid for WindowGrid {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn cell(&self, x: u32, y: u32) -> Option<Cell> {
        self.cells.get((y * self.width + x) as usize).copied()
    }
    fn metres_per_cell(&self) -> u32 {
        self.metres
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    struct Grid {
        w: u32,
        h: u32,
        cells: Vec<Cell>,
    }
    impl RoutingGrid for Grid {
        fn width(&self) -> u32 {
            self.w
        }
        fn height(&self) -> u32 {
            self.h
        }
        fn cell(&self, x: u32, y: u32) -> Option<Cell> {
            self.cells.get((y * self.w + x) as usize).copied()
        }
        fn metres_per_cell(&self) -> u32 {
            100
        }
    }
    fn grid(w: u32, h: u32) -> Grid {
        Grid {
            w,
            h,
            cells: vec![Cell::default(); (w * h) as usize],
        }
    }
    #[test]
    fn road_detour_beats_direct_open() {
        let mut g = grid(7, 3);
        for x in 0..7 {
            g.cells[(2 * 7 + x) as usize].surface = Surface::Road;
        }
        let p = astar(&g, (0, 1), (6, 1)).unwrap();
        assert!(p.spans.iter().any(|s| s.surface == Surface::Road));
    }

    #[test]
    fn wet_weather_changes_search_cost_and_persisted_weathered_check() {
        let mut g = grid(5, 3);
        for x in 1..4 {
            g.cells[(5 + x) as usize].wetland_fraction_percent = 100;
        }
        let profile = TerrainSkillProfile {
            wetlands: 4_000,
            plains: 500,
            ..Default::default()
        };
        let dry =
            astar_with_profile_and_weather(&g, (0, 1), (4, 1), profile, RoutingWeather::default())
                .unwrap();
        let wet = astar_with_profile_and_weather(
            &g,
            (0, 1),
            (4, 1),
            profile,
            RoutingWeather {
                ground_moisture_bps: 10_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert_ne!(
            dry.distance_m, wet.distance_m,
            "weather must participate in A* search"
        );
        for span in &wet.spans {
            assert_eq!(span.check_millirank, span.terrain.dot(profile));
            assert!(span.terrain.is_normalized());
        }
    }

    #[test]
    fn snow_skill_changes_covered_cost_and_zero_cover_has_parity() {
        let g = grid(3, 1);
        let profile = TerrainSkillProfile {
            plains: 1_000,
            ..Default::default()
        };
        let unskilled = astar_with_profile_and_weather(
            &g,
            (0, 0),
            (2, 0),
            profile,
            RoutingWeather {
                snow_cover_bps: 10_000,
                snow_check_millirank: 0,
                ..Default::default()
            },
        )
        .unwrap();
        let skilled = astar_with_profile_and_weather(
            &g,
            (0, 0),
            (2, 0),
            profile,
            RoutingWeather {
                snow_cover_bps: 10_000,
                snow_check_millirank: 5_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(skilled.minutes < unskilled.minutes);
        assert_eq!(unskilled.spans[0].check_millirank, 0);
        assert_eq!(skilled.spans[0].check_millirank, 5_000);

        let clear_unskilled =
            astar_with_profile_and_weather(&g, (0, 0), (2, 0), profile, RoutingWeather::default())
                .unwrap();
        let clear_skilled = astar_with_profile_and_weather(
            &g,
            (0, 0),
            (2, 0),
            profile,
            RoutingWeather {
                snow_check_millirank: 5_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(clear_unskilled, clear_skilled);
    }
    #[test]
    fn woods_and_uphill_are_slower_directionally() {
        let mut g = grid(3, 1);
        g.cells[1].surface = Surface::SparseWoods;
        g.cells[2].surface = Surface::DeepWoods;
        g.cells[2].elevation_m = 100;
        let up = astar(&g, (0, 0), (2, 0)).unwrap();
        let down = astar(&g, (2, 0), (0, 0)).unwrap();
        assert!(up.minutes > down.minutes);
        assert_eq!(up.spans.len(), 2);
    }
    #[test]
    fn water_needs_crossing() {
        let mut g = grid(3, 1);
        g.cells[1].surface = Surface::Water;
        assert!(astar(&g, (0, 0), (2, 0)).is_err());
        g.cells[1].crossing = true;
        assert!(astar(&g, (0, 0), (2, 0)).is_ok());
    }
    #[test]
    fn ties_are_deterministic() {
        let g = grid(4, 4);
        assert_eq!(
            astar(&g, (0, 0), (3, 2)).unwrap(),
            astar(&g, (0, 0), (3, 2)).unwrap()
        );
    }

    #[test]
    fn surface_speeds_are_applied_without_per_cell_minute_rounding() {
        let expected = [
            (Surface::Road, 2),
            (Surface::Open, 5),
            (Surface::SparseWoods, 6),
            (Surface::DeepWoods, 8),
        ];
        for (surface, minutes) in expected {
            let mut g = grid(2, 1);
            g.cells[0].surface = surface;
            assert_eq!(astar(&g, (0, 0), (1, 0)).unwrap().minutes, minutes);
        }
    }

    #[test]
    fn first_span_describes_the_surface_being_departed() {
        let mut g = grid(3, 1);
        g.cells[0].surface = Surface::Road;
        let route = astar(&g, (0, 0), (2, 0)).unwrap();
        assert_eq!(route.spans[0].surface, Surface::Road);
        assert_eq!(route.spans[0].start_minute, 0);
    }

    #[test]
    fn coarsening_preserves_a_road_cell() {
        let mut cells = vec![Cell::default(); 25];
        cells[12] = Cell {
            surface: Surface::Road,
            elevation_m: 17,
            crossing: true,
            ..Cell::default()
        };
        assert_eq!(
            aggregate_cells(&cells),
            Cell {
                surface: Surface::Road,
                elevation_m: 17,
                crossing: true,
                ..Cell::default()
            }
        );
    }

    #[test]
    fn terrain_weights_are_normalized_monotonic_and_keep_mixed_hills() {
        let open_hills = TerrainWeights::from_cover(0, 100, 0);
        let mixed = TerrainWeights::from_cover(40, 50, 0);
        let dense = TerrainWeights::from_cover(80, 50, 0);
        assert!(open_hills.is_normalized() && mixed.is_normalized() && dense.is_normalized());
        assert_eq!(
            open_hills,
            TerrainWeights {
                plains: 0,
                forest: 0,
                hills: 1_000,
                wetlands: 0,
                urban: 0
            }
        );
        assert_eq!(
            mixed,
            TerrainWeights {
                plains: 300,
                forest: 400,
                hills: 300,
                wetlands: 0,
                urban: 0
            }
        );
        assert!(dense.forest > mixed.forest && dense.hills < mixed.hills);
        assert_eq!(
            TerrainWeights::from_cover(40, 50, 25),
            TerrainWeights {
                plains: 225,
                forest: 300,
                hills: 225,
                wetlands: 250,
                urban: 0,
            }
        );
    }

    #[test]
    fn rainfall_respects_threshold_normalization_and_max_wetland_severity() {
        let dry = TerrainWeights::from_cover(20, 20, 9).with_ground_moisture(10_000);
        assert_eq!(dry.weights.wetlands, 90);
        assert_eq!(dry.saturation_bps, 0);
        assert!(dry.weights.is_normalized());

        let susceptible = TerrainWeights::from_cover(20, 20, 10).with_ground_moisture(10_000);
        assert!(susceptible.weights.wetlands > 100);
        assert!(susceptible.weights.is_normalized());

        let saturated = TerrainWeights::from_cover(0, 0, 100).with_ground_moisture(10_000);
        assert_eq!(saturated.weights.wetlands, 1_000);
        assert_eq!(saturated.saturation_bps, 10_000);
    }

    #[test]
    fn coarse_hill_fraction_is_area_average_not_any() {
        let cells = [
            Cell {
                hilly_fraction_percent: 100,
                ..Cell::default()
            },
            Cell::default(),
            Cell::default(),
            Cell::default(),
        ];
        assert_eq!(aggregate_cells(&cells).hilly_fraction_percent, 25);
    }

    #[test]
    fn roads_train_underlying_terrain_at_exact_inverse_speed_discount() {
        for (canopy, expected) in [(0, 250), (20, 200), (60, 150)] {
            let mut g = grid(2, 1);
            g.cells[0].surface = Surface::Road;
            g.cells[0].canopy_percent = canopy;
            let span = astar(&g, (0, 0), (1, 0)).unwrap().spans[0].clone();
            assert_eq!(span.training_multiplier_permille, expected);
            assert_eq!(span.terrain, TerrainWeights::from_cover(canopy, 0, 0));
        }
    }

    #[test]
    fn road_over_wetland_uses_and_trains_wetlands() {
        let mut g = grid(2, 1);
        g.cells[0].surface = Surface::Road;
        g.cells[0].wetland_fraction_percent = 100;
        let profile = TerrainSkillProfile {
            wetlands: 5_000,
            ..Default::default()
        };
        let plan = astar_with_profile(&g, (0, 0), (1, 0), profile).unwrap();
        let span = &plan.spans[0];
        assert_eq!(span.terrain.wetlands, 1_000);
        assert_eq!(span.check_millirank, 5_000);
        assert_eq!(span.training_multiplier_permille, 100);
    }

    #[test]
    fn matching_skill_shortens_route_and_is_stored_on_span() {
        let mut g = grid(2, 1);
        g.cells[0].canopy_percent = 100;
        let novice = astar(&g, (0, 0), (1, 0)).unwrap();
        let expert = astar_with_profile(
            &g,
            (0, 0),
            (1, 0),
            TerrainSkillProfile {
                forest: 5_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(expert.minutes < novice.minutes);
        assert_eq!(expert.spans[0].check_millirank, 5_000);
    }

    #[test]
    fn skill_profile_can_change_the_fastest_path() {
        let mut g = grid(7, 3);
        for x in 0..7 {
            g.cells[(7 + x) as usize].surface = Surface::DeepWoods;
            g.cells[(7 + x) as usize].canopy_percent = 100;
            g.cells[(14 + x) as usize].surface = Surface::Water;
        }
        let novice = astar(&g, (0, 1), (6, 1)).unwrap();
        let expert = astar_with_profile(
            &g,
            (0, 1),
            (6, 1),
            TerrainSkillProfile {
                forest: 5_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert_ne!(novice.points, expert.points);
        assert!(expert.spans.iter().all(|span| span.terrain.forest == 1_000));
    }

    #[test]
    fn expanding_search_finishes_with_full_pack_bounds() {
        let bounds = [5.0, 50.0, 15.0, 55.0];
        let windows = expanding_windows((53.5, 9.8), (53.7, 10.2), bounds);
        assert!(windows.len() > 1);
        assert_eq!(windows.last(), Some(&bounds));
        assert_ne!(windows.first(), windows.last());
    }

    #[test]
    fn terrain_spans_are_deterministically_bounded() {
        let mut g = grid(700, 1);
        for (index, cell) in g.cells.iter_mut().enumerate() {
            cell.surface = if index % 2 == 0 {
                Surface::Road
            } else {
                Surface::DeepWoods
            };
        }
        let first = astar(&g, (0, 0), (699, 0)).unwrap();
        let second = astar(&g, (0, 0), (699, 0)).unwrap();
        assert_eq!(first, second);
        assert!(first.spans.len() <= MAX_TERRAIN_SPANS);
        assert_eq!(
            first
                .spans
                .iter()
                .map(|span| span.duration_minutes)
                .sum::<u64>(),
            first.minutes
        );
        assert!(first
            .spans
            .windows(2)
            .all(|pair| pair[0].start_minute + pair[0].duration_minutes
                == pair[1].start_minute));
    }

    #[test]
    fn span_compaction_conserves_weighted_skill_exposure_with_road_discounts() {
        let terrains = [
            TerrainWeights {
                plains: 1_000,
                forest: 0,
                hills: 0,
                wetlands: 0,
                urban: 0,
            },
            TerrainWeights {
                plains: 0,
                forest: 1_000,
                hills: 0,
                wetlands: 0,
                urban: 0,
            },
            TerrainWeights {
                plains: 0,
                forest: 0,
                hills: 1_000,
                wetlands: 0,
                urban: 0,
            },
            TerrainWeights {
                plains: 400,
                forest: 200,
                hills: 200,
                wetlands: 200,
                urban: 0,
            },
        ];
        let mut spans = (0..300_u64)
            .map(|minute| TerrainSpan {
                surface: if minute % 3 == 0 {
                    Surface::Road
                } else {
                    Surface::Open
                },
                terrain: terrains[minute as usize % terrains.len()],
                training_multiplier_permille: if minute % 3 == 0 {
                    [150, 200, 250][minute as usize / 3 % 3]
                } else {
                    1_000
                },
                check_millirank: (minute % 6) as u16 * 1_000,
                start_minute: minute,
                duration_minutes: 1,
            })
            .collect::<Vec<_>>();
        let exposure = |values: &[TerrainSpan]| {
            let mut result = [0.0_f64; 5];
            for span in values {
                for (total, weight) in result.iter_mut().zip([
                    span.terrain.plains,
                    span.terrain.forest,
                    span.terrain.hills,
                    span.terrain.wetlands,
                    span.terrain.urban,
                ]) {
                    *total += span.duration_minutes as f64
                        * f64::from(span.training_multiplier_permille)
                        / 1_000.0
                        * f64::from(weight)
                        / 1_000.0;
                }
            }
            result
        };
        let before = exposure(&spans);
        compact_spans(&mut spans, 300, 8);
        let after = exposure(&spans);
        assert!(spans.len() <= 8);
        assert!(spans.iter().all(|span| span.terrain.is_normalized()));
        for (before, after) in before.into_iter().zip(after) {
            assert!(
                (before - after).abs() <= before.max(1.0) * 0.005,
                "{before} != {after}"
            );
        }
    }

    #[test]
    fn expired_deadline_stops_search_cooperatively() {
        let g = grid(100, 100);
        assert!(matches!(
            astar_with_profile_and_deadline(
                &g,
                (0, 0),
                (99, 99),
                TerrainSkillProfile::default(),
                RoutingWeather::default(),
                Some(Instant::now())
            ),
            Err(Error::Deadline)
        ));
    }

    #[test]
    fn long_native_routes_are_compacted_after_search() {
        let g = grid(10_000, 1);
        let route = astar(&g, (0, 0), (9_999, 0)).unwrap();
        assert_eq!(route.distance_m, 999_900);
        assert_eq!(route.points.len(), 2);
        assert_eq!(
            route
                .spans
                .iter()
                .map(|span| span.duration_minutes)
                .sum::<u64>(),
            route.minutes
        );
    }

    #[test]
    fn endpoint_snapping_and_blocked_diagonal_are_deterministic() {
        let mut g = grid(5, 5);
        g.cells[(2 * 5 + 2) as usize].surface = Surface::Water;
        assert_eq!(snap(&g, (2, 2), 1), Some((1, 1)));

        let mut corner = grid(2, 2);
        corner.cells[1].surface = Surface::Water;
        corner.cells[2].surface = Surface::Water;
        assert!(astar(&corner, (0, 0), (1, 1)).is_err());
    }

    #[test]
    fn oversized_manifest_is_rejected_before_reading_or_parsing() {
        let root =
            std::env::temp_dir().join(format!("adventuresim-terrain-bound-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let manifest = root.join("terrain.json");
        let pack = root.join("terrain.pack");
        File::create(&manifest)
            .unwrap()
            .set_len(MAX_MANIFEST_BYTES + 1)
            .unwrap();
        File::create(&pack).unwrap().write_all(b"x").unwrap();
        assert!(matches!(
            TerrainPack::load(&manifest, &pack),
            Err(Error::Validation(message)) if message.contains("manifest exceeds")
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn self_consistent_pack_with_reserved_surface_is_rejected_on_decode() {
        use flate2::{Compression, write::DeflateEncoder};
        let root = std::env::temp_dir().join(format!(
            "adventuresim-terrain-surface-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let mut decoded = [0, 0, 1, 0, 0].repeat(usize::from(CHUNK_SIDE) * usize::from(CHUNK_SIDE));
        decoded[2] = 6;
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&decoded).unwrap();
        let compressed = encoder.finish().unwrap();
        let mut manifest = Manifest {
            schema: SCHEMA,
            purpose: TerrainPurpose::Final,
            bounds: [10.0, 50.0, 11.0, 51.0],
            source_resolution_m: 30,
            content_sha256: hex_sha(&compressed),
            road_geometry_sha256: hex_sha(b"roads"),
            wetland_source_sha256: hex_sha(b"wetlands"),
            wetland_cells: 0,
            cultivation_grid_crs: "EPSG:3035".into(),
            cultivation_grid_resolution_m: 1_000,
            cultivation_rules_version: 1,
            cultivation_source_sha256: hex_sha(b"cultivation"),
            cultivated_square_count: 0,
            cultivated_native_cells: 0,
            terrain_features: Vec::new(),
            entries: vec![Entry {
                south: 50,
                west: 10,
                tile_width: 1_800,
                tile_height: 3_600,
                chunk_x: 0,
                chunk_y: 0,
                width: CHUNK_SIDE,
                height: CHUNK_SIDE,
                offset: 0,
                length: compressed.len() as u32,
                decoded_sha256: hex_sha(&decoded),
            }],
            package_sha256: "0".repeat(64),
        };
        manifest.package_sha256 = hex_sha(&serde_json::to_vec(&manifest).unwrap());
        let manifest_path = root.join("terrain.json");
        let pack_path = root.join("terrain.pack");
        std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        std::fs::write(&pack_path, compressed).unwrap();
        let pack = TerrainPack::load(&manifest_path, &pack_path).unwrap();
        assert!(
            matches!(pack.cell(50.9999,10.0001),Err(Error::Validation(message)) if message.contains("unknown surface"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
