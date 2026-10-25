//! Material palette and mass densities shared by authoring and gameplay.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Material {
    #[default]
    Steel,
    DarkSteel,
    Wood,
    Leather,
    DarkLeather,
    Brass,
    Horn,
    Cord,
    Feather,
    Sinew,
    DarkFeather,
    Lead,
    Cherry,
    Walnut,
    RedBeech,
    Gold,
    Silver,
    Latten,
    Bone,
    Staghorn,
    MotherOfPearl,
    Pyrite,
    MatchCord,
}

impl Material {
    pub fn is_metal(self) -> bool {
        matches!(
            self,
            Self::Steel
                | Self::DarkSteel
                | Self::Brass
                | Self::Latten
                | Self::Lead
                | Self::Gold
                | Self::Silver
        )
    }
    pub fn density(self) -> f64 {
        match self {
            Self::Steel | Self::DarkSteel => 7850.0,
            Self::Wood | Self::Walnut | Self::RedBeech => 720.0,
            Self::Leather | Self::DarkLeather => 920.0,
            Self::Brass | Self::Latten => 8500.0,
            Self::Horn | Self::Sinew | Self::Staghorn => 1250.0,
            Self::Cord => 1200.0,
            Self::Feather | Self::DarkFeather => 350.0,
            Self::Lead => 11340.0,
            Self::Cherry => 620.0,
            Self::Gold => 19300.0,
            // RSC room-temperature elemental density: 10.5 g/cm^3.
            Self::Silver => 10500.0,
            Self::Bone => 1850.0,
            Self::MotherOfPearl => 2700.0,
            Self::Pyrite => 5000.0,
            Self::MatchCord => 900.0,
        }
    }

    pub fn color(self) -> [f64; 3] {
        match self {
            Self::Steel => [0.58, 0.62, 0.64],
            Self::DarkSteel => [0.29, 0.31, 0.31],
            Self::Wood => [0.34, 0.19, 0.085],
            Self::Leather => [0.24, 0.11, 0.055],
            Self::DarkLeather => [0.055, 0.045, 0.038],
            Self::Brass => [0.58, 0.43, 0.18],
            Self::Horn => [0.17, 0.13, 0.09],
            Self::Cord => [0.12, 0.095, 0.065],
            Self::Feather => [0.72, 0.67, 0.53],
            Self::Sinew => [0.62, 0.48, 0.30],
            Self::DarkFeather => [0.20, 0.17, 0.14],
            Self::Lead => [0.25, 0.27, 0.29],
            Self::Cherry => [0.38, 0.16, 0.08],
            Self::Walnut => [0.24, 0.13, 0.06],
            Self::RedBeech => [0.45, 0.25, 0.13],
            Self::Gold => [0.82, 0.61, 0.16],
            Self::Silver => [0.78, 0.80, 0.82],
            Self::Latten => [0.64, 0.49, 0.18],
            Self::Bone => [0.79, 0.75, 0.61],
            Self::Staghorn => [0.61, 0.53, 0.39],
            Self::MotherOfPearl => [0.78, 0.82, 0.78],
            Self::Pyrite => [0.66, 0.54, 0.17],
            Self::MatchCord => [0.18, 0.13, 0.08],
        }
    }
}
