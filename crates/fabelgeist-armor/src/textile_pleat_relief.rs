//! Fine repeated cloth folds drawn as physical heights on a material chart.
use crate::{
    ClothPleats, DesignError, GenerateError, Millimeters, Permille, TextileRepeatCount, TextileSpan,
};
use serde::{Deserialize, Serialize};

const MILLIMETRES_PER_METRE: f64 = 1000.;
/// Share of the chart, from its start, that measures the width at that end.
const CHART_START_BAND: f32 = 0.1;

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
    /// Profile override for regular folds; shirring owns its own stitch profile.
    pub zone: Option<crate::TextilePleatZone>,
}

/// Envelope in the normalized cloth chart, independent of its mesh density.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TextilePleatLayout {
    /// Constant longitudinal folds, with no gathering band or free-edge fade.
    Uniform,
    /// Counted-thread shirring inside a bounded chart interval; regular
    /// longitudinal gathers continue outside it without adding mesh vertices.
    Shirred(TextileShirring),
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
        /// Width of the cloth at the waist as a share of its mean width. The
        /// cloth narrows to the waist, so the gathers there are narrower and
        /// steeper than the same depth is lower down.
        waist_width: Permille,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileShirring {
    pub band: TextileSpan,
    pub rows: TextileRepeatCount,
    pub pattern: ShirringPattern,
    pub retention: Permille,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ShirringPattern {
    Diamonds,
    Chevron,
}

impl TextileChartSize {
    /// Native fitted mesh boundary: positions in metres, material UV normalized.
    pub fn from_surface(
        positions: &[[f32; 3]],
        coordinates: &[[f32; 2]],
        indices: &[u32],
    ) -> Result<Self, GenerateError> {
        let dimensions = Self::measure(positions, coordinates, indices, |_| true)?;
        Ok(Self {
            across: Millimeters(dimensions[0].round() as u16),
            along: Millimeters(dimensions[1].round() as u16),
        })
    }

    /// Size on the same surface of one region of its chart, from the
    /// triangles whose middles lie in the region. `None` when no triangle
    /// does, or the region is too small to measure.
    pub fn of_region(
        positions: &[[f32; 3]],
        coordinates: &[[f32; 2]],
        indices: &[u32],
        region: &crate::TextileRegion,
    ) -> Option<Self> {
        let within = |corners: &[[f64; 2]; 3]| {
            let middle = [0, 1]
                .map(|axis| (corners.iter().map(|corner| corner[axis]).sum::<f64>() / 3.) as f32);
            region.chart(middle).is_some()
        };
        let per_unit = Self::measure(positions, coordinates, indices, within).ok()?;
        let span = region.span();
        let size = [
            per_unit[0] * f64::from(span[0]),
            per_unit[1] * f64::from(span[1]),
        ];
        size.iter().all(|&side| side >= 1.).then(|| Self {
            across: Millimeters(size[0].round() as u16),
            along: Millimeters(size[1].round() as u16),
        })
    }

    /// Width of the same surface at the start of its chart, as a share of its
    /// mean width and at most the whole of it.
    pub fn start_width(
        positions: &[[f32; 3]],
        coordinates: &[[f32; 2]],
        indices: &[u32],
    ) -> Result<Permille, GenerateError> {
        let whole = Self::measure(positions, coordinates, indices, |_| true)?;
        let start = Self::measure(positions, coordinates, indices, |corners| {
            corners
                .iter()
                .all(|corner| corner[1] <= f64::from(CHART_START_BAND))
        })?;
        let share = (start[0] / whole[0]).min(1.) * f64::from(Permille::ONE.0);
        Ok(Permille(share.round().max(1.) as u16))
    }

    /// Mean millimetres per unit of each chart axis over the triangles that
    /// `counts` admits by their chart corners.
    fn measure(
        positions: &[[f32; 3]],
        coordinates: &[[f32; 2]],
        indices: &[u32],
        counts: impl Fn(&[[f64; 2]; 3]) -> bool,
    ) -> Result<[f64; 2], GenerateError> {
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
            if !counts(&uv) {
                continue;
            }
            let a: [f64; 3] = std::array::from_fn(|axis| p[1][axis] - p[0][axis]);
            let b: [f64; 3] = std::array::from_fn(|axis| p[2][axis] - p[0][axis]);
            let u = [uv[1][0] - uv[0][0], uv[1][1] - uv[0][1]];
            let v = [uv[2][0] - uv[0][0], uv[2][1] - uv[0][1]];
            let determinant = u[0] * v[1] - u[1] * v[0];
            if determinant == 0. {
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
        if weight <= 0. {
            return Err(GenerateError::InvalidSurface);
        }
        let dimensions = dimensions.map(|value| value / weight * MILLIMETRES_PER_METRE);
        if dimensions
            .iter()
            .any(|&value| !value.is_finite() || value < 1. || value > f64::from(u16::MAX))
        {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(dimensions)
    }
}

impl TextilePleatRelief {
    pub fn validate(&self) -> Result<(), GenerateError> {
        if let Some(zone) = &self.zone {
            zone.validate()?;
            if matches!(self.layout, TextilePleatLayout::Shirred(_)) {
                return Err(DesignError::ParametricParameters.into());
            }
        }
        if self.chart.across.0 == 0
            || self.chart.along.0 == 0
            || !(8..=192).contains(&self.pleats.count.0)
            || !(1..=30).contains(&self.pleats.depth.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let TextilePleatLayout::Shirred(TextileShirring {
            band,
            rows,
            retention,
            ..
        }) = self.layout
            && (band.start.0 >= band.end.0
                || band.end.0 > 1000
                || !(1..=32).contains(&rows.0)
                || retention.0 > 1000)
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
            waist_width,
        } = self.layout
            && (front_panel.0 > 500
                || waist_width.0 == 0
                || waist_width.0 > 1000
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

    /// Metres across the chart at one point along it. A waist layout widens
    /// evenly from the waist and keeps the chart's mean width.
    pub(crate) fn across_metres(&self, along: f32) -> f32 {
        let share = match self.layout {
            TextilePleatLayout::Waist { waist_width, .. } => {
                waist_width.unit() + 2. * (1. - waist_width.unit()) * along
            }
            _ => 1.,
        };
        self.chart.across.metres() * share
    }

    /// Height in metres. V runs from free edge to sewn base; U repeats evenly.
    pub(crate) fn height(&self, uv: [f32; 2]) -> f32 {
        let (across, envelope, rotation) = match self.layout {
            TextilePleatLayout::Uniform => (uv[0], 1., 0.),
            TextilePleatLayout::Shirred(shirring) => return shirring.height(&self.pleats, uv),
            TextilePleatLayout::FreeEdge => (uv[0], smooth_release(1. - uv[1]), 0.),
            TextilePleatLayout::Waist {
                front_panel,
                front_depth,
                rotation,
                falloff,
                retention,
                release,
                ..
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
        let mut profile = self.pleats.profile.height(phase);
        if let Some(zone) = &self.zone {
            profile += (zone.profile.height(phase) - profile) * zone.blend(uv);
        }
        self.pleats.depth.metres() * profile * envelope
    }
}

impl TextileShirring {
    fn height(&self, pleats: &ClothPleats, uv: [f32; 2]) -> f32 {
        let across = uv[0] * f32::from(pleats.count.0);
        let plain = (1. - (std::f32::consts::TAU * across).cos()) * 0.5;
        if uv[1] < self.band.start.unit() || uv[1] > self.band.end.unit() {
            return pleats.depth.metres() * self.retention.unit() * plain;
        }
        let along = (uv[1] - self.band.start.unit())
            / (self.band.end.unit() - self.band.start.unit())
            * f32::from(self.rows.0);
        let shift = match self.pattern {
            ShirringPattern::Diamonds => (2. * (along * 0.5).fract() - 1.).abs() * 0.5,
            ShirringPattern::Chevron => along * 0.5,
        };
        let ridge = (1. - (std::f32::consts::TAU * (across + shift)).cos()) * 0.5;
        let stitch = (std::f32::consts::TAU * along).cos() * 0.5 + 0.5;
        // Blend the band envelope over one stitch row so band boundaries do
        // not introduce discontinuous heights or a false groove in the cloth.
        let edge = smooth_release(along.min(f32::from(self.rows.0) - along));
        pleats.depth.metres()
            * (self.retention.unit() * plain * (1. - edge) + ridge * stitch * edge)
    }
}

fn smooth_release(value: f32) -> f32 {
    let value = value.clamp(0., 1.);
    value * value * (3. - 2. * value)
}
