//! Chevron course cuts in the measured, unposed wearer frame.
use super::shape_wgsl::DESIGN_WORDS;
use crate::GenerateError;

pub(super) struct CourseFrame {
    lateral: [f64; 3],
    front: [f64; 3],
    origin: f64,
}

impl CourseFrame {
    pub(super) fn from_words(words: &[f32]) -> Result<Self, GenerateError> {
        let wearer = words
            .get(DESIGN_WORDS as usize..)
            .filter(|w| w.len() >= 15)
            .ok_or(GenerateError::InvalidSurface)?;
        Ok(Self {
            lateral: std::array::from_fn(|k| f64::from(wearer[k])),
            front: std::array::from_fn(|k| f64::from(wearer[k + 3])),
            origin: f64::from(wearer[13]),
        })
    }
    pub(super) fn course_level(&self, point: [f32; 3], slope: f32) -> f64 {
        f64::from(point[1]) - f64::from(slope) * self.lateral_coordinate(point).abs()
    }
    pub(super) fn lateral_coordinate(&self, point: [f32; 3]) -> f64 {
        self.lateral
            .iter()
            .zip(point)
            .map(|(a, p)| a * f64::from(p))
            .sum::<f64>()
            - self.origin
    }
    pub(super) fn local_point(&self, point: [f32; 3]) -> [f64; 3] {
        [
            self.lateral_coordinate(point),
            f64::from(point[1]),
            self.front
                .iter()
                .zip(point)
                .map(|(a, p)| a * f64::from(p))
                .sum(),
        ]
    }
}
