//! Climate space (Fig. 2, Sec. 6.4): biomes, and how well a species is
//! adapted to a place.

use crate::species::PlantType;
use crate::units::{Celsius, CelsiusPerMetre, Millimetres};
use fabelgeist_math::units::{Metres, Ratio};

/// The climate of a scene.
///
/// Precipitation is constant over the scene; temperature falls linearly with
/// elevation, `T(h) = T(0) + eta * h` with `eta < 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Climate {
    /// Mean annual temperature at elevation zero, degrees Celsius.
    pub temperature: Celsius,
    /// Mean annual precipitation, mm.
    pub precipitation: Millimetres,
    /// `eta`: degrees Celsius per metre of elevation.
    pub lapse_rate: CelsiusPerMetre,
}

impl Default for Climate {
    fn default() -> Self {
        Self {
            temperature: Celsius::from_degrees(15.0),
            precipitation: Millimetres::from_millimetres(900.0),
            lapse_rate: CelsiusPerMetre::from_degrees_per_metre(-0.0065),
        }
    }
}

impl Climate {
    /// Temperature and rainfall cannot be interchanged.
    ///
    /// ```compile_fail
    /// use fabelgeist_botany::{Climate, units::{Celsius, Millimetres}};
    /// let temperature = Celsius::from_degrees(12.0);
    /// let rainfall = Millimetres::from_millimetres(900.0);
    /// let _ = Climate::new(rainfall, temperature);
    /// ```
    pub fn new(temperature: Celsius, precipitation: Millimetres) -> Self {
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
        Self::new(Celsius::from_degrees(t), Millimetres::from_millimetres(p))
    }

    pub fn temperature_at(&self, elevation: Metres) -> Celsius {
        Celsius::from_degrees(
            self.temperature.degrees() + self.lapse_rate.degrees_per_metre() * elevation.metres(),
        )
    }

    pub fn biome_at(&self, elevation: Metres) -> Biome {
        Biome::classify(self.temperature_at(elevation), self.precipitation)
    }

    /// Eq. 11: how well `species` is adapted at `elevation`, in `(0, 1]`.
    ///
    /// The product of the two Gaussian adaptation functions, normalised by
    /// their value at the optimum -- so one at the species' optimal habitat,
    /// falling off with the distance to it in climate space. Used to scale
    /// `v_root_max` and the seeding frequency.
    pub fn adaptation(&self, species: &PlantType, elevation: Metres) -> Ratio {
        let t = self.temperature_at(elevation);
        let dt = (t.degrees() - species.temperature_optimum.degrees())
            / species.temperature_tolerance.degrees().max(1e-3);
        let dp = (self.precipitation.millimetres() - species.precipitation_optimum.millimetres())
            / species.precipitation_tolerance.millimetres().max(1e-3);
        Ratio::from_ratio((-0.5 * (dt * dt + dp * dp)).exp())
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
    pub fn classify(temperature: Celsius, precipitation_mm: Millimetres) -> Biome {
        let temperature = temperature.degrees();
        let p = precipitation_mm.millimetres();
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
            assert_eq!(climate.biome_at(Metres::from_metres(0.0)), biome);
        }
    }

    #[test]
    fn adaptation_peaks_at_the_optimum_and_falls_with_elevation() {
        let species = PlantType {
            temperature_optimum: Celsius::from_degrees(15.0),
            precipitation_optimum: Millimetres::from_millimetres(900.0),
            ..PlantType::default()
        };
        let climate = Climate::new(
            Celsius::from_degrees(15.0),
            Millimetres::from_millimetres(900.0),
        );
        assert!(
            (climate
                .adaptation(&species, Metres::from_metres(0.0))
                .ratio()
                - 1.0)
                .abs()
                < 1e-6
        );
        let high = climate
            .adaptation(&species, Metres::from_metres(1500.0))
            .ratio();
        assert!(high < 0.5, "{high}");
    }
}
