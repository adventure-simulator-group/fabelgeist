//! One focused city, published only after its exact GPU candidate is installed.
use super::{MapState, surface::SurfaceCoverage};
use crate::presentation::{
    buildings::{self, CityAssemblyFailure, CityAssemblyPublished, CityFrame},
    generation::{PreparationTicket, PreparedCityProduct},
    ownership::PresentationOwner,
    terrain::TacticalTerrainMaterial,
    vista::{CityGroundMaterial, focused::FocusedGround},
};
use adventuresim_tactical_core::{
    regional_terrain::REGIONAL_TERRAIN_SIDE, scene_coordinates::ScenePlanPoint,
};
use adventuresim_world_schema::coordinates::{
    Wgs84CoordinateMicrodegrees, terrain_projection::NativeTerrainCoordinate,
};
use bevy::prelude::*;
use serde::Serialize;
use std::sync::Arc;

mod installation;
pub(super) use installation::install;

const SETTLED_CITY_FRAMES: usize = 4;

#[derive(Default)]
pub(super) struct CityResidency {
    pending: Option<InstalledCity>,
    installed: Option<InstalledCity>,
    pub status: Option<InstallationStatus>,
    pub visible: bool,
    settled_frames: usize,
}

struct InstalledCity {
    root: Entity,
    product: Arc<PreparedCityProduct>,
    ground: FocusedGround,
    frame: CityFrame,
}

#[derive(Clone, Copy)]
pub(super) struct CitySurface<'a> {
    pub root: Entity,
    pub frame: CityFrame,
    pub support: &'a crate::presentation::vista::streets::GroundSupport,
}

#[derive(Clone, Copy, Serialize)]
pub(super) struct InstallationStatus {
    pub preparation: PreparationTicket,
    pub phase: InstallationPhase,
    pub error: Option<InstallationFailure>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum InstallationPhase {
    Preparing,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum InstallationFailure {
    Input,
    Focus,
    Preparation,
    Ground,
    Buildings,
    Assembly,
}

impl InstallationStatus {
    fn failed(preparation: PreparationTicket, error: InstallationFailure) -> Self {
        Self {
            preparation,
            phase: InstallationPhase::Failed,
            error: Some(error),
        }
    }
}

impl MapState {
    pub(super) fn city_surface(&self) -> Option<CitySurface<'_>> {
        if !self.city.visible {
            return None;
        }
        let city = self.city.installed.as_ref()?;
        Some(CitySurface {
            root: city.root,
            frame: city.frame,
            support: &city.ground.support,
        })
    }

    pub(super) fn ground_position(&self, origin: NativeTerrainCoordinate) -> Option<Vec3> {
        if self.city.visible {
            return self.city_position(origin);
        }
        let terrain = self.terrain()?;
        let offset =
            NativeTerrainCoordinate::from(terrain.request().origin.to_e7()).offset_to(origin);
        super::geographic_surface::position_at_offset(terrain, offset)
    }

    /// Geographic pin/camera adapter: query the actual city triangles in their
    /// canonical east/north frame, then apply the same matrix as the renderer.
    pub(super) fn city_position(&self, origin: NativeTerrainCoordinate) -> Option<Vec3> {
        if !self.city.visible {
            return None;
        }
        let city = self.city.installed.as_ref()?;
        let offset =
            NativeTerrainCoordinate::from(city.product.document.origin()).offset_to(origin);
        let local = ScenePlanPoint::try_from(Vec2::new(
            offset.east_metres as f32,
            offset.north_metres as f32,
        ))
        .ok()?;
        city.ground
            .support
            .position(local)
            .map(|position| city.frame.world_from_city().transform_point3(position))
    }
}

impl InstalledCity {
    fn covers(&self, state: &MapState) -> bool {
        let Some(pose) = state.pose.as_ref() else {
            return false;
        };
        if pose.rect.is_none()
            || &pose.source != self.product.document.source()
            || state.requested_city() != Some(self.product.document.place())
        {
            return false;
        }
        let Some(surface) = state.presented.as_ref() else {
            return false;
        };
        let offset = NativeTerrainCoordinate::from(self.product.document.origin())
            .offset_to(surface.request.origin.to_e7().into());
        let half = (REGIONAL_TERRAIN_SIDE - 1) as f64
            * 0.5
            * f64::from(surface.request.scale.spacing_metres());
        let frame = NativeTerrainCoordinate::from(self.product.document.origin())
            .frame_from(surface.request.origin.to_e7().into());
        // Require the entire regional window to fit. The first implementation
        // exchanges complete surfaces, avoiding seams between coarse/fine edges.
        offset.east_metres.abs() + half * frame.east_scale < f64::from(self.ground.half_extent.x)
            && offset.north_metres.abs() + half < f64::from(self.ground.half_extent.y)
    }

    fn reanchor(&mut self, world: &mut World, origin: Wgs84CoordinateMicrodegrees) -> Result {
        let frame = CityFrame::from_geographic_city(&self.product.document, origin);
        if frame == self.frame {
            return Ok(());
        }
        frame.install(world, PresentationOwner::RegionalMap)?;
        if let Some(mut transform) = world.get_mut::<Transform>(self.root) {
            *transform = Transform::from_matrix(frame.world_from_city());
        }
        let mut terrain_materials = world.resource_mut::<Assets<TacticalTerrainMaterial>>();
        for handle in &self.ground.terrain_materials {
            if let Some(mut material) = terrain_materials.get_mut(handle) {
                material.extension.set_geographic_frame(frame);
            }
        }
        let mut materials = world.resource_mut::<Assets<CityGroundMaterial>>();
        for handle in &self.ground.paving_materials {
            if let Some(mut material) = materials.get_mut(handle) {
                material.extension.set_geographic_frame(frame);
            }
        }
        self.frame = frame;
        Ok(())
    }
}

pub(super) fn sync(world: &mut World) {
    world.resource_scope(|world, mut state: Mut<MapState>| {
        publish_candidate(world, &mut state.city);
        let mut draw_city = state
            .city
            .installed
            .as_ref()
            .is_some_and(|city| city.covers(&state));
        let origin = state
            .presented
            .as_ref()
            .map(|surface| surface.request.origin);
        let replaced = state
            .city
            .pending
            .as_ref()
            .is_some_and(|pending| world.get::<CityAssemblyPublished>(pending.root).is_some());
        if let (Some(city), Some(origin)) = (state.city.installed.as_mut(), origin) {
            // A staged replacement owns its frame once assembly publishes.
            // Do not reanchor old ground against the new candidate's buffers.
            if replaced {
                draw_city = false;
            } else if let Err(error) = city.reanchor(world, origin) {
                warn!(%error, "Could not reanchor map city");
                draw_city = false;
            }
            if let Some(mut visibility) = world.get_mut::<Visibility>(city.root) {
                *visibility = if draw_city {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
        buildings::set_city_visible(world, PresentationOwner::RegionalMap, draw_city);
        // Keep the already-settled regional ground underneath while the first
        // fine-ground pipelines specialize. Height queries exchange only after
        // the candidate has rendered enough settled frames to replace it.
        let visible = draw_city && state.city.settled_frames >= SETTLED_CITY_FRAMES;
        if let Some(surface) = state.presented.as_ref()
            && let SurfaceCoverage::Drawn { entity, .. } = surface.coverage
            && let Some(mut visibility) = world.get_mut::<Visibility>(entity)
        {
            *visibility = if visible {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
        if state.city.visible != visible {
            state.city.visible = visible;
            state.settled_frames = 0;
        }
        if draw_city
            && state.city.pending.is_none()
            && state.city.installed.is_some()
            && crate::strategic_scene::status::waiting_pipelines() == 0
        {
            state.city.settled_frames = state
                .city
                .settled_frames
                .saturating_add(1)
                .min(SETTLED_CITY_FRAMES);
            if state.city.settled_frames == SETTLED_CITY_FRAMES
                && let Some(status) = &mut state.city.status
                && matches!(status.phase, InstallationPhase::Preparing)
            {
                status.phase = InstallationPhase::Ready;
            }
        } else {
            state.city.settled_frames = 0;
        }
    });
}

fn publish_candidate(world: &mut World, residency: &mut CityResidency) {
    let Some(pending) = residency.pending.as_ref() else {
        return;
    };
    if let Some(error) = world.get::<CityAssemblyFailure>(pending.root) {
        warn!(error = %error.0, "Could not assemble focused map city");
        let root = pending.root;
        residency.pending = None;
        if let Some(status) = &mut residency.status {
            *status = InstallationStatus::failed(status.preparation, InstallationFailure::Assembly);
        }
        if let Ok(entity) = world.get_entity_mut(root) {
            entity.despawn();
        }
        return;
    }
    if world.get::<CityAssemblyPublished>(pending.root).is_none()
        || !buildings::city_gpu_ready(PresentationOwner::RegionalMap)
    {
        return;
    }
    if let Some(pending) = residency.pending.take()
        && let Some(old) = residency.installed.replace(pending)
        && let Ok(entity) = world.get_entity_mut(old.root)
    {
        entity.despawn();
    }
    residency.settled_frames = 0;
}
