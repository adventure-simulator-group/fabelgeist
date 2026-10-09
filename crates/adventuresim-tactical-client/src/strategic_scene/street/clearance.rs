//! Reserve the approach before procedurally placing distant tree stands.
use super::*;

impl Street {
    pub(in crate::strategic_scene) fn clear_foreground_canopy(&self, lods: &mut [VistaLod]) {
        // The street is outside the original city. Its approach must remain
        // empty for every scene digest, including crowns reaching in from cells
        // beside the camera's sight line. Keep the city behind it forested.
        let minimum = Vec2::new(
            -self.width * 0.5 - CITY_CLEARANCE_METRES,
            self.front.z - CITY_CLEARANCE_METRES,
        );
        let maximum = Vec2::new(
            self.width * 0.5 + CITY_CLEARANCE_METRES,
            self.camera().translation.z + CITY_CLEARANCE_METRES,
        );
        for lod in lods {
            let width = usize::from(lod.width);
            let center = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1)) * 0.5;
            for (index, environment) in lod.environment.iter_mut().enumerate() {
                let cell_min = (Vec2::new((index % width) as f32, (index / width) as f32) - center)
                    * lod.spacing_metres;
                let cell_max = cell_min + Vec2::splat(lod.spacing_metres);
                if cell_min.cmple(maximum).all() && cell_max.cmpge(minimum).all() {
                    environment.canopy_bps = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn street_approach_clears_intersecting_stands_but_keeps_background_trees() {
        let street = Street {
            bays: Vec::new(),
            width: 100.0,
            height: 30.0,
            front: Vec3::new(0.0, 0.0, 200.0),
        };
        let mut lods = [VistaLod {
            level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0),
            width: 21,
            depth: 21,
            spacing_metres: 100.0,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![0.0; 21 * 21],
            environment: vec![
                EnvironmentalSample {
                    canopy_bps: 10_000,
                    ..default()
                };
                21 * 21
            ],
        }];
        street.clear_foreground_canopy(&mut lods);
        let canopy = |x: usize, z: usize| lods[0].environment[z * 21 + x].canopy_bps;
        assert_eq!(canopy(10, 12), 0); // Street front.
        assert_eq!(canopy(10, 13), 0); // Camera approach.
        assert_eq!(canopy(8, 12), 0); // A stand extends in from the side.
        assert_eq!(canopy(10, 5), 10_000); // City backdrop.
        assert_eq!(canopy(16, 12), 10_000); // Unrelated landscape.
        assert_eq!(canopy(10, 18), 10_000); // Beyond the camera reservation.
    }
}
