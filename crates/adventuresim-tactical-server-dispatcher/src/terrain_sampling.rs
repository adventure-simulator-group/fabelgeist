//! Native terrain-cell classification shared by scene and regional capture.
use adventuresim_tactical_core::scene_input::{EnvironmentalSample, TacticalSurface};
use adventuresim_terrain::{Cell, Surface};
use adventuresim_world_schema::BASIS_POINTS_PER_WHOLE;

const PERCENT_PER_WHOLE: u16 = 100;
const BASIS_POINTS_PER_PERCENT: u16 = BASIS_POINTS_PER_WHOLE / PERCENT_PER_WHOLE;

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
