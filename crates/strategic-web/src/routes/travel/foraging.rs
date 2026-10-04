//! Immutable current-vicinity terrain sampling and wet/coastal classification.

use super::{TerrainPlanner, forage_error::TerrainForageError};

impl TerrainPlanner {
    /// Bounded immutable vicinity sample used by personal foraging. The center
    /// cell is authoritative; eight nearby samples only identify coast access.
    pub(in crate::routes) fn forage_environment(
        &self,
        latitude: f64,
        longitude: f64,
    ) -> std::result::Result<(adventuresim_terrain::Cell, bool, bool), TerrainForageError> {
        let center = self
            .pack
            .cell(latitude, longitude)
            .map_err(TerrainForageError::Sample)?
            .ok_or(TerrainForageError::OutsidePackage)?;
        let water_samples = [-0.01, 0.0, 0.01]
            .into_iter()
            .flat_map(|dy| [-0.015, 0.0, 0.015].into_iter().map(move |dx| (dx, dy)))
            .filter(|(dx, dy)| *dx != 0.0 || *dy != 0.0)
            .filter(|(dx, dy)| {
                self.pack
                    .cell(latitude + dy, longitude + dx)
                    .ok()
                    .flatten()
                    .is_some_and(|cell| cell.surface == adventuresim_terrain::Surface::Water)
            })
            .count();
        let coastal = water_samples >= 4;
        let river_or_wet = river_or_wet_ground(center, water_samples);
        Ok((center, river_or_wet, coastal))
    }
}

fn river_or_wet_ground(center: adventuresim_terrain::Cell, water_samples: usize) -> bool {
    center.surface == adventuresim_terrain::Surface::Wetland
        || center.wetland_fraction_percent > 0
        || (1..4).contains(&water_samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn road_over_authoritative_wetland_counts_as_wet_ground() {
        let road = adventuresim_terrain::Cell {
            surface: adventuresim_terrain::Surface::Road,
            wetland_fraction_percent: 100,
            ..Default::default()
        };
        assert!(river_or_wet_ground(road, 0));
        assert!(!river_or_wet_ground(
            adventuresim_terrain::Cell {
                surface: adventuresim_terrain::Surface::Road,
                ..Default::default()
            },
            0
        ));
    }
}
