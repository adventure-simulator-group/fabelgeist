//! Shared axial samples keep overlapping courses on the same carrier facets.
use crate::{DesignError, GARMENT_AXIAL_SEGMENTS, GenerateError};

use crate::garment_armor::GARMENT_LAME_OVERLAP;

/// A course's span in the whole fauld and local row fractions within it.
/// Sampling depends only on the course count, preserving morph connectivity.
pub struct FauldLameChart {
    pub span: [f32; 2],
    pub rows: Vec<f32>,
}

impl FauldLameChart {
    pub fn new(lame: usize, count: usize) -> Result<Self, GenerateError> {
        if count == 0 || lame >= count {
            return Err(DesignError::ParametricParameters.into());
        }
        let span = |course: usize| {
            [
                (1.0 - (course + 1) as f32 / count as f32
                    - if course + 1 < count {
                        GARMENT_LAME_OVERLAP / count as f32
                    } else {
                        0.0
                    })
                .max(0.0),
                1.0 - course as f32 / count as f32,
            ]
        };
        let [bottom, top] = span(lame);
        let mut rows = Vec::new();
        for course in lame.saturating_sub(1)..=(lame + 1).min(count - 1) {
            let [lo, hi] = span(course);
            for row in 0..=GARMENT_AXIAL_SEGMENTS {
                let axial = lo + (hi - lo) * row as f32 / GARMENT_AXIAL_SEGMENTS as f32;
                if axial >= bottom && axial <= top {
                    rows.push((axial - bottom) / (top - bottom));
                }
            }
        }
        rows.sort_unstable_by(f32::total_cmp);
        rows.dedup();
        Ok(Self {
            span: [bottom, top],
            rows,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_courses_share_every_axial_facet_boundary() {
        for count in 2..=8 {
            for upper in 0..count - 1 {
                let a = FauldLameChart::new(upper, count).unwrap();
                let b = FauldLameChart::new(upper + 1, count).unwrap();
                let overlap = |chart: &FauldLameChart| {
                    chart
                        .rows
                        .iter()
                        .map(|row| chart.span[0] + row * (chart.span[1] - chart.span[0]))
                        .filter(|axial| *axial >= a.span[0] && *axial <= b.span[1])
                        .collect::<Vec<_>>()
                };
                let a = overlap(&a);
                let b = overlap(&b);
                assert_eq!(a.len(), b.len());
                assert!(a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-6));
            }
        }
    }
}
