//! Scene support queries share the rendered and collided triangle surface.
use super::{SceneTerrain, TerrainGeometry};
use crate::{
    city_layout::grounding::{SupportElevation, SurfaceHit},
    scene_coordinates::ScenePlanPoint,
};
use bevy::math::{Dir3, Vec2, Vec3};

pub type TerrainSupportResult<T> = Result<T, TerrainSupportError>;

/// Nonempty ordered bounds in absolute scene elevation metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneElevationRange {
    minimum: SupportElevation,
    maximum: SupportElevation,
}

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum TerrainSupportError {
    #[error("terrain has no physical support at {point:?}")]
    MissingSupport { point: ScenePlanPoint },
    #[error("terrain has no finite nonempty elevation bounds")]
    MissingElevationRange,
}

impl SceneElevationRange {
    pub fn new(minimum: SupportElevation, maximum: SupportElevation) -> Option<Self> {
        (minimum.metres() <= maximum.metres()).then_some(Self { minimum, maximum })
    }

    pub fn minimum(self) -> SupportElevation {
        self.minimum
    }

    pub fn maximum(self) -> SupportElevation {
        self.maximum
    }

    /// Complete the reduction over native heightfield or compiled mesh samples.
    fn from_native_metres(mut heights: impl Iterator<Item = f32>) -> Option<Self> {
        let first = heights.next()?;
        if !first.is_finite() {
            return None;
        }
        let mut minimum = first;
        let mut maximum = first;
        for height in heights {
            if !height.is_finite() {
                return None;
            }
            minimum = minimum.min(height);
            maximum = maximum.max(height);
        }
        Self::new(
            SupportElevation::from_metres(minimum)?,
            SupportElevation::from_metres(maximum)?,
        )
    }
}

impl SceneTerrain {
    pub fn required_height_at(&self, point: ScenePlanPoint) -> TerrainSupportResult<SupportElevation> {
        self.height_at(point)
            .ok_or(TerrainSupportError::MissingSupport { point })
    }

    pub fn required_elevation_range(&self) -> TerrainSupportResult<SceneElevationRange> {
        self.elevation_range()
            .ok_or(TerrainSupportError::MissingElevationRange)
    }

    pub fn elevation_range(&self) -> Option<SceneElevationRange> {
        match &self.geometry {
            TerrainGeometry::Sampled => {
                SceneElevationRange::from_native_metres(self.heightmap.iter().copied())
            }
            TerrainGeometry::Owned(surface) => {
                SceneElevationRange::from_native_metres(surface.support_heights())
            }
        }
    }

    pub fn height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation> {
        self.surface_at_stride(point, 1).map(|sample| sample.elevation)
    }

    /// Samples the triangle surface used by the coarse render LOD.
    pub fn coarse_height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation> {
        self.surface_at_stride(point, self.coarse_stride.max(1))
            .map(|sample| sample.elevation)
    }

    /// Returns the finite, normalized normal of the rendered/collided triangle.
    pub fn normal_at(&self, point: ScenePlanPoint) -> Option<Dir3> {
        self.surface_at_stride(point, 1).map(|sample| sample.normal)
    }

    /// Samples the same triangle surface used by the rendered mesh and
    /// authoritative collider. Returning the triangle normal alongside the
    /// height keeps terrain IK from fitting a foot to a different, bilinear
    /// surface than the one visible beneath it.
    pub(super) fn surface_at_stride(
        &self,
        point: ScenePlanPoint,
        stride: usize,
    ) -> Option<SurfaceHit> {
        if let TerrainGeometry::Owned(surface) = &self.geometry {
            return surface.highest_surface_at(point);
        }
        if self.scale <= 0.0 || self.grid_width() < 2 || self.grid_depth() < 2 {
            return None;
        }
        // The sampled grid is centered in scene-local east/north metres.
        let pos = point.metres() + Vec2::new(self.width(), self.depth()) * 0.5;
        let grid = pos / self.scale;
        if grid.x < 0.0
            || grid.y < 0.0
            || grid.x > (self.grid_width() - 1) as f32
            || grid.y > (self.grid_depth() - 1) as f32
        {
            return None;
        }
        // The final vertex has no cell to its right/below. Sample its adjacent
        // cell with a coordinate of one so edges remain well-defined.
        let stride = stride.max(1);
        let last_cell_x = ((self.grid_width() - 2) / stride) * stride;
        let last_cell_y = ((self.grid_depth() - 2) / stride) * stride;
        let min_x = ((grid.x / stride as f32).floor() as usize * stride).min(last_cell_x);
        let min_y = ((grid.y / stride as f32).floor() as usize * stride).min(last_cell_y);
        let max_x = (min_x + stride).min(self.grid_width() - 1);
        let max_y = (min_y + stride).min(self.grid_depth() - 1);
        let cell_scale_x = (max_x - min_x) as f32 * self.scale;
        let cell_scale_z = (max_y - min_y) as f32 * self.scale;
        let fraction = Vec2::new(
            (grid.x - min_x as f32) / (max_x - min_x) as f32,
            (grid.y - min_y as f32) / (max_y - min_y) as f32,
        );

        let x0y0 = *self.heightmap.get(min_x + min_y * self.width)?;
        let x1y0 = *self.heightmap.get(max_x + min_y * self.width)?;
        let x0y1 = *self.heightmap.get(min_x + max_y * self.width)?;
        let x1y1 = *self.heightmap.get(max_x + max_y * self.width)?;

        let (height, tangent_x, tangent_z) = if fraction.x + fraction.y <= 1.0 {
            // Matches [x0y0, x0y1, x1y0] in `mesh_components_with_stride` and
            // Avian's default heightfield subdivision.
            (
                x0y0 + (x1y0 - x0y0) * fraction.x + (x0y1 - x0y0) * fraction.y,
                Vec3::new(cell_scale_x, x1y0 - x0y0, 0.0),
                Vec3::new(0.0, x0y1 - x0y0, cell_scale_z),
            )
        } else {
            // Matches [x1y0, x0y1, x1y1].
            (
                x1y1 + (x0y1 - x1y1) * (1.0 - fraction.x) + (x1y0 - x1y1) * (1.0 - fraction.y),
                Vec3::new(cell_scale_x, x1y1 - x0y1, 0.0),
                Vec3::new(0.0, x1y1 - x1y0, cell_scale_z),
            )
        };
        let normal = tangent_z.cross(tangent_x);
        SurfaceHit::from_geometry(height, normal)
    }

}

#[cfg(test)]
mod tests;
