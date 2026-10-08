//! Shared native terrain-cell and geographic projection adapters.
//! Sampling kernels retain continuous degrees and east/north metres; geographic
//! origin admission and serialized payloads use the canonical checked types.
use adventuresim_tactical_core::scene_input::{EnvironmentalSample, TacticalSurface};
use adventuresim_terrain::{Cell, Surface};
use adventuresim_world_schema::BASIS_POINTS_PER_WHOLE;

const PERCENT_PER_WHOLE: u16 = 100;
const BASIS_POINTS_PER_PERCENT: u16 = BASIS_POINTS_PER_WHOLE / PERCENT_PER_WHOLE;
pub(crate) const METRES_PER_LATITUDE_DEGREE: f64 = 111_320.0;
pub(crate) const MIN_LONGITUDE_SCALE: f64 = 0.01;

/// Native TerrainPack projection port: continuous WGS84 degrees and scene
/// east/north metres. Quantizing these intermediate values would move samples.
pub(crate) struct GeographicSampleCoordinate {
    pub(crate) latitude_degrees: f64,
    pub(crate) longitude_degrees: f64,
}

pub(crate) fn environment_sample(cell: Cell) -> EnvironmentalSample {
    EnvironmentalSample {
        canopy_bps: u16::from(cell.canopy_percent) * BASIS_POINTS_PER_PERCENT,
        wetland_bps: u16::from(cell.wetland_fraction_percent) * BASIS_POINTS_PER_PERCENT,
        cultivation_bps: if cell.cultivated {
            BASIS_POINTS_PER_WHOLE
        } else {
            0
        },
        water_bps: if cell.surface == Surface::Water && !cell.crossing {
            BASIS_POINTS_PER_WHOLE
        } else {
            0
        },
        hilly_bps: u16::from(cell.hilly_fraction_percent) * BASIS_POINTS_PER_PERCENT,
        crossing_bps: if cell.crossing {
            BASIS_POINTS_PER_WHOLE
        } else {
            0
        },
        surface: match cell.surface {
            Surface::Road => TacticalSurface::Road,
            Surface::Open => TacticalSurface::Open,
            Surface::SparseWoods => TacticalSurface::SparseWoods,
            Surface::DeepWoods => TacticalSurface::DeepWoods,
            Surface::Water => TacticalSurface::Water,
            Surface::Wetland => TacticalSurface::Wetland,
        },
    }
}

pub(crate) fn offset_coordinate(
    latitude: f64,
    longitude: f64,
    east: f64,
    north: f64,
) -> GeographicSampleCoordinate {
    let latitude_delta = north / METRES_PER_LATITUDE_DEGREE;
    let longitude_scale = latitude.to_radians().cos().abs().max(MIN_LONGITUDE_SCALE);
    let longitude_delta = east / (METRES_PER_LATITUDE_DEGREE * longitude_scale);
    GeographicSampleCoordinate {
        latitude_degrees: latitude + latitude_delta,
        longitude_degrees: longitude + longitude_delta,
    }
}
