//! Fine repeated cloth folds drawn as physical heights on a material chart.
use crate::{ClothPleatProfile, ClothPleats, DesignError, GenerateError, Millimeters, Permille};
use serde::{Deserialize, Serialize};

const MILLIMETRES_PER_METRE: f64 = 1000.;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileChartSize {
    pub across: Millimeters,
    pub along: Millimeters,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextilePleatRelief {
    pub pleats: ClothPleats,
    pub chart: TextileChartSize,
    pub layout: TextilePleatLayout,
}

/// Envelope in the normalized cloth chart, independent of its mesh density.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TextilePleatLayout {
    /// V=1 is the sewn base; relief releases toward the free edge at V=0.
    FreeEdge,
    /// V=0 is the waist. The quiet front is centered at U=0.5, matching the
    /// geometric garment chart's rear seam.
    Waist {
        front_panel: Permille,
        front_depth: Permille,
        rotation: Permille,
        falloff: Permille,
        retention: Permille,
        release: Permille,
    },
}

impl TextileChartSize {
    /// Native fitted mesh boundary: positions in metres, material UV normalized.
    pub fn from_surface(
        positions: &[[f32; 3]],
        coordinates: &[[f32; 2]],
        indices: &[u32],
    ) -> Result<Self, GenerateError> {
        if positions.len() != coordinates.len()
            || indices.is_empty()
            || !indices.len().is_multiple_of(3)
            || indices
                .iter()
                .any(|&vertex| vertex as usize >= positions.len())
            || positions
                .iter()
                .flatten()
                .chain(coordinates.iter().flatten())
                .any(|value| !value.is_finite())
        {
            return Err(GenerateError::InvalidSurface);
        }
        let mut dimensions = [0_f64; 2];
        let mut weight = 0_f64;
        for triangle in indices.as_chunks::<3>().0 {
            let p = triangle.map(|i| positions[i as usize].map(f64::from));
            let uv = triangle.map(|i| coordinates[i as usize].map(f64::from));
            let a: [f64; 3] = std::array::from_fn(|axis| p[1][axis] - p[0][axis]);
            let b: [f64; 3] = std::array::from_fn(|axis| p[2][axis] - p[0][axis]);
            let u = [uv[1][0] - uv[0][0], uv[1][1] - uv[0][1]];
            let v = [uv[2][0] - uv[0][0], uv[2][1] - uv[0][1]];
            let determinant = u[0] * v[1] - u[1] * v[0];
            if determinant.abs() <= f64::EPSILON {
                continue;
            }
            let derivatives: [[f64; 3]; 2] = [
                std::array::from_fn(|axis| (a[axis] * v[1] - b[axis] * u[1]) / determinant),
                std::array::from_fn(|axis| (b[axis] * u[0] - a[axis] * v[0]) / determinant),
            ];
            for axis in 0..2 {
                dimensions[axis] +=
                    derivatives[axis].iter().map(|v| v * v).sum::<f64>().sqrt() * determinant.abs();
            }
            weight += determinant.abs();
        }
        let dimensions = dimensions.map(|value| value / weight * MILLIMETRES_PER_METRE);
        if dimensions
            .iter()
            .any(|&value| !value.is_finite() || value < 1. || value > f64::from(u16::MAX))
        {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(Self {
            across: Millimeters(dimensions[0].round() as u16),
            along: Millimeters(dimensions[1].round() as u16),
        })
    }
}

impl TextilePleatRelief {
    pub fn validate(&self) -> Result<(), GenerateError> {
        if self.chart.across.0 == 0
            || self.chart.along.0 == 0
            || !(8..=192).contains(&self.pleats.count.0)
            || !(1..=30).contains(&self.pleats.depth.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let TextilePleatLayout::Waist {
            front_panel,
            front_depth,
            rotation,
            falloff,
            retention,
            release,
        } = self.layout
            && (front_panel.0 > 500
                || front_depth.0 > 1000
                || rotation.0 > 1000
                || falloff.0 == 0
                || falloff.0 > 1000
                || retention.0 > 1000
                || release.0 == 0
                || release.0 > 1000)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    /// Height in metres. V runs from free edge to sewn base; U repeats evenly.
    pub(crate) fn height(&self, uv: [f32; 2]) -> f32 {
        let (across, envelope, rotation) = match self.layout {
            TextilePleatLayout::FreeEdge => (uv[0], smooth_release(1. - uv[1]), 0.),
            TextilePleatLayout::Waist {
                front_panel,
                front_depth,
                rotation,
                falloff,
                retention,
                release,
            } => {
                let around = (uv[0] + 0.5).rem_euclid(1.);
                let half = front_panel.unit() * 0.5;
                if half > 0. && (around <= half || around >= 1. - half) {
                    return 0.;
                }
                let chart = (around - half) / (1. - 2. * half);
                let seam = (1. / f32::from(self.pleats.count.0)).min(0.05);
                let edge = if half > 0. {
                    smooth_release((around - half).min(1. - half - around) / seam)
                } else {
                    1.
                };
                let front = (std::f32::consts::TAU * around).cos().max(0.).powi(2);
                let depth = 1. + (front_depth.unit() - 1.) * front;
                let retained =
                    1. + (retention.unit() - 1.) * smooth_release(uv[1] / falloff.unit());
                (
                    chart,
                    depth * retained * edge * smooth_release(uv[1] / release.unit()),
                    rotation.unit(),
                )
            }
        };
        let phase = (across * f32::from(self.pleats.count.0) + rotation).rem_euclid(1.);
        let profile = match self.pleats.profile {
            ClothPleatProfile::Fluted => (1. - (std::f32::consts::TAU * phase).cos()) * 0.5,
            ClothPleatProfile::RoundedGathers => {
                let mouth = ClothPleatProfile::MOUTH_SHARE.unit();
                let valley = ClothPleatProfile::VALLEY_DEPTH_SHARE.unit();
                let (local, low, high) = if phase < mouth {
                    (phase / mouth, valley, 1. - valley)
                } else {
                    ((phase - mouth) / (1. - mouth), valley, -valley)
                };
                low + high * (1. - (2. * local - 1.).powi(2)).max(0.).sqrt()
            }
        };
        self.pleats.depth.metres() * profile * envelope
    }
}

fn smooth_release(value: f32) -> f32 {
    let value = value.clamp(0., 1.);
    value * value * (3. - 2. * value)
}
