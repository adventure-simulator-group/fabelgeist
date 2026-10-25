#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// 4-component RGBA color float vector.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(0.95, 0.2, 0.2, 1.0);
    pub const GREEN: Self = Self::new(0.2, 0.85, 0.3, 1.0);
    pub const BLUE: Self = Self::new(0.2, 0.5, 0.95, 1.0);
    pub const YELLOW: Self = Self::new(0.95, 0.85, 0.1, 1.0);
    pub const ORANGE: Self = Self::new(0.95, 0.55, 0.1, 1.0);
    pub const CYAN: Self = Self::new(0.1, 0.85, 0.9, 1.0);
    pub const MAGENTA: Self = Self::new(0.85, 0.2, 0.85, 1.0);
    pub const GRAY: Self = Self::new(0.5, 0.5, 0.5, 1.0);
    pub const DARK_GRAY: Self = Self::new(0.2, 0.2, 0.2, 1.0);
    pub const STEEL: Self = Self::new(0.65, 0.7, 0.78, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_array(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constructors_and_lerp() {
        let red = Color::RED;
        let blue = Color::BLUE;
        let mid = red.lerp(blue, 0.5);
        assert!((mid.r - 0.575).abs() < 1e-4);
        assert_eq!(red.to_array(), [0.95, 0.2, 0.2, 1.0]);
        assert_eq!(Color::from_array([1.0, 1.0, 1.0, 1.0]), Color::WHITE);
    }
}
