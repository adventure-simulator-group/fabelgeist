//! Affine handoff from city east/up/north to geographic east/up/south.
use super::{CityGpuScenes, PresentationOwner};
use adventuresim_tactical_core::regional_city::RegionalCityInput;
use adventuresim_world_schema::coordinates::{
    Wgs84CoordinateMicrodegrees, terrain_projection::NativeTerrainCoordinate,
};
use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, storage::ShaderBuffer},
};

/// The east scale follows the shared sampler projection. The sign corrects
/// winding and tangent handedness; other vector lanes are alignment padding.
#[derive(Clone, Copy, Debug, PartialEq, ShaderType)]
pub(in crate::presentation) struct CityFrame {
    pub(super) world_from_city: Mat4,
    normal_from_city: Mat4,
    handedness: Vec4,
}

#[derive(Debug, thiserror::Error)]
pub(in crate::presentation) enum CityFrameError {
    #[error("installed city frame buffer is unavailable")]
    Buffer,
}

impl Default for CityFrame {
    fn default() -> Self {
        Self {
            world_from_city: Mat4::IDENTITY,
            normal_from_city: Mat4::IDENTITY,
            handedness: Vec4::X,
        }
    }
}

impl CityFrame {
    /// Conservative radius scale for projected sphere level selection.
    pub(super) fn radius_scale(self) -> f32 {
        self.world_from_city.x_axis.x.max(1.0)
    }

    /// Native mesh adapter; columns carry east/up/north city metres into the
    /// current east/up/south map frame. This includes absolute source elevation.
    pub(in crate::presentation) fn world_from_city(self) -> Mat4 {
        self.world_from_city
    }

    pub(in crate::presentation) fn from_geographic_city(
        city: &RegionalCityInput,
        window_origin: Wgs84CoordinateMicrodegrees,
    ) -> Self {
        let projection =
            NativeTerrainCoordinate::from(window_origin.to_e7()).frame_from(city.origin().into());
        let offset = projection.origin;
        // Native GPU port: metres relative to the current terrain window,
        // including the city's absolute source elevation. Reflection changes
        // north-positive local Z into the map's south-positive world Z.
        let translation = Vec3::new(
            offset.east_metres as f32,
            f32::from(city.input().absolute_elevation_metres.get()),
            -offset.north_metres as f32,
        );
        let world_from_city = Mat4::from_scale_rotation_translation(
            Vec3::new(projection.east_scale as f32, 1.0, -1.0),
            Quat::IDENTITY,
            translation,
        );
        Self {
            world_from_city,
            normal_from_city: world_from_city.inverse().transpose(),
            handedness: -Vec4::X,
        }
    }

    /// Moving the terrain window updates this small buffer; building placement
    /// and geometry buffers remain resident in their canonical city frame.
    pub(in crate::presentation) fn install(
        self,
        world: &mut World,
        owner: PresentationOwner,
    ) -> Result<(), CityFrameError> {
        world.init_resource::<CityGpuScenes>();
        let scene = world.resource::<CityGpuScenes>().owners.get(owner);
        if scene.frame == self {
            return Ok(());
        }
        if scene.count != 0 {
            let handle = scene.frame_buffer.clone();
            let mut buffers = world.resource_mut::<Assets<ShaderBuffer>>();
            buffers
                .get_mut(&handle)
                .ok_or(CityFrameError::Buffer)?
                .set_data(self);
        }
        world
            .resource_mut::<CityGpuScenes>()
            .owners
            .get_mut(owner)
            .frame = self;
        Ok(())
    }
}
