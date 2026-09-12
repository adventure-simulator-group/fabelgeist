//! Authored relief fields, independent of the direction used to form wall gauge.
use crate::{GenerateError, PartFrame, ShellExtrusion};

#[derive(Clone, Debug)]
pub enum SurfaceRelief {
    /// Nonnegative heights following the shell's carrier extrusion direction.
    ShellHeights(Vec<f32>),
    /// Displacements in the plate's local chart frame, retained through refitting.
    ChartOffsets(Vec<[f32; 3]>),
}

impl SurfaceRelief {
    pub(crate) fn validate(&self, count: usize) -> Result<(), GenerateError> {
        let valid = match self {
            Self::ShellHeights(heights) => {
                heights.len() == count && heights.iter().all(|h| h.is_finite() && *h >= 0.0)
            }
            Self::ChartOffsets(offsets) => {
                offsets.len() == count && offsets.iter().flatten().all(|v| v.is_finite())
            }
        };
        if valid {
            Ok(())
        } else {
            Err(GenerateError::InvalidSurface)
        }
    }

    pub(crate) fn offset(
        &self,
        index: usize,
        extrusion: ShellExtrusion,
        point: [f32; 3],
        normal: [f32; 3],
    ) -> Result<[f32; 3], GenerateError> {
        match self {
            Self::ShellHeights(heights) => {
                Ok(extrusion.offset(point, normal)?.map(|v| v * heights[index]))
            }
            Self::ChartOffsets(offsets) => Ok(offsets[index]),
        }
    }

    pub(crate) fn transform(&mut self, frame: &PartFrame) {
        if let Self::ChartOffsets(offsets) = self {
            for offset in offsets {
                *offset = std::array::from_fn(|axis| {
                    (0..3).map(|i| frame.axes[i][axis] * offset[i]).sum()
                });
            }
        }
    }
}
