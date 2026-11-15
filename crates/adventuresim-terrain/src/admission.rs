//! Admission of manifest metadata and native chunk geometry.

use crate::{
    CELL_BYTES, CHUNK_SIDE, CULTIVATION_GRID_CRS, Entry, Error, MAX_ENTRIES, Manifest, Result,
    SCHEMA, digest::valid_digest, terrain_feature,
};

/// A full or true-edge chunk whose wire dimensions match its native placement.
#[derive(Clone, Copy)]
pub(super) struct ChunkGeometry {
    width: u16,
    height: u16,
}

impl ChunkGeometry {
    pub(super) fn admit(entry: &Entry) -> Result<Self> {
        let width = axis_extent(entry.tile_width, entry.chunk_x)?;
        let height = axis_extent(entry.tile_height, entry.chunk_y)?;
        if (entry.width, entry.height) != (width, height) {
            return Err(Error::Validation(
                "chunk dimensions do not match native tile placement".into(),
            ));
        }
        Ok(Self { width, height })
    }

    pub(super) fn decoded_bytes(self) -> usize {
        usize::from(self.width) * usize::from(self.height) * CELL_BYTES
    }

    pub(super) fn cell_index(self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(usize::from(y) * usize::from(self.width) + usize::from(x))
    }
}

pub(super) fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.schema != SCHEMA
        || manifest.source_resolution_m != 30
        || manifest.entries.is_empty()
        || manifest.entries.len() > MAX_ENTRIES
    {
        return Err(Error::Validation(
            "unsupported or unbounded manifest".into(),
        ));
    }
    let [west, south, east, north] = manifest.bounds;
    if ![west, south, east, north].into_iter().all(f64::is_finite) || west >= east || south >= north
    {
        return Err(Error::Validation("invalid geographic bounds".into()));
    }
    if !valid_digest(&manifest.content_sha256) || !valid_digest(&manifest.package_sha256) {
        return Err(Error::Validation("invalid digest".into()));
    }
    if !valid_digest(&manifest.road_geometry_sha256)
        || !valid_digest(&manifest.wetland_source_sha256)
        || !valid_digest(&manifest.cultivation_source_sha256)
        || manifest.wetland_cells > 100_000
        || manifest.cultivation_grid_crs != CULTIVATION_GRID_CRS
        || manifest.cultivation_grid_resolution_m != 1_000
        || manifest.cultivation_rules_version == 0
        || manifest.cultivated_square_count > 5_000_000
        || manifest.cultivated_native_cells > 1_000_000_000
    {
        return Err(Error::Validation("invalid terrain feature identity".into()));
    }
    terrain_feature::validate(&manifest.terrain_features, [west, south, east, north])?;
    for entry in &manifest.entries {
        if ![1_800, 2_400, 3_600].contains(&entry.tile_width)
            || entry.tile_height != 3_600
            || entry.width == 0
            || entry.height == 0
            || entry.width > CHUNK_SIDE
            || entry.height > CHUNK_SIDE
            || !valid_digest(&entry.decoded_sha256)
        {
            return Err(Error::Validation(
                "invalid native GLO-30 chunk metadata".into(),
            ));
        }
    }
    Ok(())
}

fn axis_extent(tile_extent: u16, ordinal: u16) -> Result<u16> {
    let start = u32::from(ordinal) * u32::from(CHUNK_SIDE);
    let remaining = u32::from(tile_extent)
        .checked_sub(start)
        .filter(|remaining| *remaining != 0)
        .ok_or_else(|| Error::Validation("chunk ordinal is outside its native tile".into()))?;
    Ok(remaining.min(u32::from(CHUNK_SIDE)) as u16)
}
