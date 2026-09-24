//! Integer controls for body-mounted weapon holders.
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct Millimeters(pub u32);

impl Millimeters {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub fn meters(self) -> f32 {
        self.0 as f32 / 1_000.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Permille(pub u16);
impl Permille {
    pub fn unit(self) -> f32 {
        self.0 as f32 / 1000.0
    }
}
