use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::AsRefStr,
    strum::EnumString,
)]
pub enum CubeFace {
    #[default]
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeFace {
    pub const ALL: [CubeFace; 6] = [
        CubeFace::PositiveX,
        CubeFace::NegativeX,
        CubeFace::PositiveY,
        CubeFace::NegativeY,
        CubeFace::PositiveZ,
        CubeFace::NegativeZ,
    ];

    pub const fn index(self) -> u32 {
        match self {
            CubeFace::PositiveX => 0,
            CubeFace::NegativeX => 1,
            CubeFace::PositiveY => 2,
            CubeFace::NegativeY => 3,
            CubeFace::PositiveZ => 4,
            CubeFace::NegativeZ => 5,
        }
    }

    pub const fn from_index(index: u32) -> Option<Self> {
        match index {
            0 => Some(CubeFace::PositiveX),
            1 => Some(CubeFace::NegativeX),
            2 => Some(CubeFace::PositiveY),
            3 => Some(CubeFace::NegativeY),
            4 => Some(CubeFace::PositiveZ),
            5 => Some(CubeFace::NegativeZ),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            CubeFace::PositiveX => "Right (+X)",
            CubeFace::NegativeX => "Left (-X)",
            CubeFace::PositiveY => "Top (+Y)",
            CubeFace::NegativeY => "Bottom (-Y)",
            CubeFace::PositiveZ => "Front (+Z)",
            CubeFace::NegativeZ => "Back (-Z)",
        }
    }
}
