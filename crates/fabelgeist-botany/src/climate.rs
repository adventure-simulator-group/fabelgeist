//! Climate space (Fig. 2, Sec. 6.4): biomes, and how well a species is
//! adapted to a place.

use crate::species::PlantType;

/// The climate of a scene.
///
/// Precipitation is constant over the scene; temperature falls linearly with
/// elevation, `T(h) = T(0) + eta * h` with `eta < 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    /// Mean annual temperature at elevation zero, degrees Celsius.
    pub temperature: f32,
    /// Mean annual precipitation, mm.
    pub precipitation: f32,
    /// `eta`: degrees Celsius per metre of elevation.
    pub lapse_rate: f32,
}

impl Default for Climate {
    fn default() -> Self {
        Self {
            temperature: 15.0,
            precipitation: 900.0,
            lapse_rate: -0.0065,
        }
    }
}

impl Climate {
    pub fn new(temperature: f32, precipitation: f32) -> Self {
        Self {
            temperature,
            precipitation,
            ..Self::default()
        }
    }

    /// The climate typical of `biome`: the middle of its region of the
    /// Whittaker diagram.
    pub fn of(biome: Biome) -> Self {
        let (t, p) = match biome {
            Biome::Tundra => (-8.0, 250.0),
            Biome::BorealForest => (1.0, 700.0),
            Biome::TemperateGrassland => (10.0, 350.0),
            Biome::Shrubland => (16.0, 650.0),
            Biome::TemperateSeasonalForest => (12.0, 1400.0),
            Biome::TemperateRainforest => (13.0, 2800.0),
            Biome::Desert => (25.0, 200.0),
            Biome::Savanna => (26.0, 1000.0),
            Biome::TropicalSeasonalForest => (26.0, 1900.0),
            Biome::TropicalRainforest => (27.0, 3500.0),
        };
        Self::new(t, p)
    }

    pub fn temperature_at(&self, elevation: f32) -> f32 {
        self.temperature + self.lapse_rate * elevation
    }

    pub fn biome_at(&self, elevation: f32) -> Biome {
        Biome::classify(self.temperature_at(elevation), self.precipitation)
    }

    /// Eq. 11: how well `species` is adapted at `elevation`, in `(0, 1]`.
    ///
    /// The product of the two Gaussian adaptation functions, normalised by
    /// their value at the optimum -- so one at the species' optimal habitat,
    /// falling off with the distance to it in climate space. Used to scale
    /// `v_root_max` and the seeding frequency.
    pub fn adaptation(&self, species: &PlantType, elevation: f32) -> f32 {
        let t = self.temperature_at(elevation);
        let dt = (t - species.temperature_optimum) / species.temperature_tolerance.max(1e-3);
        let dp = (self.precipitation - species.precipitation_optimum)
            / species.precipitation_tolerance.max(1e-3);
        (-0.5 * (dt * dt + dp * dp)).exp()
    }
}

/// The biomes of the temperature-precipitation diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    Tundra,
    BorealForest,
    TemperateGrassland,
    Shrubland,
    TemperateSeasonalForest,
    TemperateRainforest,
    Desert,
    Savanna,
    TropicalSeasonalForest,
    TropicalRainforest,
}

impl Biome {
    pub const ALL: [Biome; 10] = [
        Biome::Tundra,
        Biome::BorealForest,
        Biome::TemperateGrassland,
        Biome::Shrubland,
        Biome::TemperateSeasonalForest,
        Biome::TemperateRainforest,
        Biome::Desert,
        Biome::Savanna,
        Biome::TropicalSeasonalForest,
        Biome::TropicalRainforest,
    ];

    /// Whittaker's diagram, as axis-aligned bands: temperature picks the
    /// climatic zone, precipitation the biome within it.
    pub fn classify(temperature: f32, precipitation_mm: f32) -> Biome {
        let p = precipitation_mm;
        if temperature < -4.0 {
            Biome::Tundra
        } else if temperature < 5.0 {
            if p < 300.0 {
                Biome::Tundra
            } else {
                Biome::BorealForest
            }
        } else if temperature < 20.0 {
            match p {
                p if p < 250.0 => Biome::Desert,
                p if p < 500.0 => Biome::TemperateGrassland,
                p if p < 1000.0 => Biome::Shrubland,
                p if p < 2250.0 => Biome::TemperateSeasonalForest,
                _ => Biome::TemperateRainforest,
            }
        } else {
            match p {
                p if p < 500.0 => Biome::Desert,
                p if p < 1400.0 => Biome::Savanna,
                p if p < 2500.0 => Biome::TropicalSeasonalForest,
                _ => Biome::TropicalRainforest,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_biome_climate_classifies_as_itself() {
        for biome in Biome::ALL {
            let climate = Climate::of(biome);
            assert_eq!(climate.biome_at(0.0), biome);
        }
    }

    #[test]
    fn adaptation_peaks_at_the_optimum_and_falls_with_elevation() {
        let species = PlantType {
            temperature_optimum: 15.0,
            precipitation_optimum: 900.0,
            ..PlantType::default()
        };
        let climate = Climate::new(15.0, 900.0);
        assert!((climate.adaptation(&species, 0.0) - 1.0).abs() < 1e-6);
        let high = climate.adaptation(&species, 1500.0);
        assert!(high < 0.5, "{high}");
    }
}
