//! Capture observations from the serialized geographic backdrop grids.

use adventuresim_building_generator::spatial_geometry::{GeometryError, SignedLength};
use adventuresim_tactical_core::{
    city_layout::grounding::{GeographicHeightControl, GeographicHeightRange, SupportElevation},
    scene_coordinates::ScenePlanPoint,
    scene_input::{TacticalSceneInput, VistaLevelIndex},
};
use bevy::math::Vec2;

pub(super) type VistaGeometryResult<T> = Result<T, VistaGeometryError>;

/// A nonempty observation; extrema keep their actual sampling locations.
#[derive(Clone, Copy, Debug)]
pub(super) struct VistaCaptureGeometry {
    diameter: SignedLength,
    extrema: GeographicHeightRange,
    relief: SignedLength,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum VistaGeometryError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error("vista elevation is not finite at {point:?}")]
    InvalidElevation { point: ScenePlanPoint },
    #[error("vista level {level} has no columns")]
    MissingColumns { level: VistaLevelIndex },
}

impl VistaCaptureGeometry {
    pub(super) fn from_source(input: &TacticalSceneInput) -> VistaGeometryResult<Option<Self>> {
        let mut diameter = 0.0_f32;
        let mut extrema: Option<GeographicHeightRange> = None;
        for lod in &input.vista.lods {
            if lod.width == 0 {
                return Err(VistaGeometryError::MissingColumns { level: lod.level });
            }
            diameter = diameter.max(f32::from(lod.width.saturating_sub(1)) * lod.spacing_metres);
            let center_x = f32::from(lod.width.saturating_sub(1)) * 0.5;
            let center_z = f32::from(lod.depth.saturating_sub(1)) * 0.5;
            for (index, height) in lod.heights_metres.iter().copied().enumerate() {
                // Complete the original row-major grid-to-scene projection before admission.
                let x = (index % usize::from(lod.width)) as f32;
                let z = (index / usize::from(lod.width)) as f32;
                let point = ScenePlanPoint::try_from(Vec2::new(
                    (x - center_x) * lod.spacing_metres + lod.origin_east_metres as f32,
                    (z - center_z) * lod.spacing_metres + lod.origin_north_metres as f32,
                ))?;
                let elevation = SupportElevation::from_metres(height)
                    .ok_or(VistaGeometryError::InvalidElevation { point })?;
                let control = GeographicHeightControl { point, elevation };
                match &mut extrema {
                    Some(range) => {
                        if elevation.metres() < range.minimum.elevation.metres() {
                            range.minimum = control;
                        }
                        if elevation.metres() > range.maximum.elevation.metres() {
                            range.maximum = control;
                        }
                    }
                    None => {
                        extrema = Some(GeographicHeightRange {
                            minimum: control,
                            maximum: control,
                        })
                    }
                }
            }
        }
        let Some(extrema) = extrema else {
            return Ok(None);
        };
        Ok(Some(Self {
            diameter: SignedLength::from_metres(diameter)?,
            extrema,
            relief: SignedLength::from_metres(
                extrema.maximum.elevation.metres() - extrema.minimum.elevation.metres(),
            )?,
        }))
    }

    pub(super) fn diameter(self) -> SignedLength {
        self.diameter
    }

    pub(super) fn minimum(self) -> GeographicHeightControl {
        self.extrema.minimum
    }

    pub(super) fn maximum(self) -> GeographicHeightControl {
        self.extrema.maximum
    }

    pub(super) fn relief(self) -> SignedLength {
        self.relief
    }
}
