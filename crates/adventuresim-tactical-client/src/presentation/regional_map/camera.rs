//! Retained geographic camera pose. Native view X is east, Y is elevation and
//! Z is south; the source sampler retains its canonical east/north convention.
use super::protocol::{
    CanvasRect, MapPointerDisplacement, MapProtocolError, MapSpan, MapZoomRatio,
};
use adventuresim_building_generator::spatial_geometry::Radians;
use adventuresim_tactical_core::{
    regional_terrain::{REGIONAL_TERRAIN_SIDE, RegionalTerrainRequest, RegionalTerrainScale},
    scene_input::SourcePackageDigest,
};
use adventuresim_world_schema::coordinates::{
    LatitudeMicrodegrees, LongitudeMicrodegrees, Wgs84CoordinateMicrodegrees,
    terrain_projection::NativeTerrainCoordinate,
};
use bevy::prelude::*;

pub(super) const MAP_PITCH_RADIANS: f32 = std::f32::consts::PI / 3.0;
const WINDOW_COVERAGE_MARGIN: f64 = 2.0;
const WINDOW_CENTER_STEP_CELLS: usize = (REGIONAL_TERRAIN_SIDE - 1) / 4;
pub(super) type Result<T> = std::result::Result<T, MapProtocolError>;

pub(super) struct MapPose {
    pub source: SourcePackageDigest,
    pub home: Wgs84CoordinateMicrodegrees,
    pub origin: Wgs84CoordinateMicrodegrees,
    pub span: MapSpan,
    pub yaw: Radians,
    pub rect: Option<CanvasRect>,
    home_span: MapSpan,
}

impl MapPose {
    pub fn new(
        source: SourcePackageDigest,
        home: Wgs84CoordinateMicrodegrees,
        span: MapSpan,
        rect: CanvasRect,
    ) -> Self {
        Self {
            source,
            home,
            origin: home,
            span,
            yaw: Radians::ZERO,
            rect: Some(rect),
            home_span: span,
        }
    }

    pub fn reset(&mut self) {
        self.origin = self.home;
        self.span = self.home_span;
        self.yaw = Radians::ZERO;
    }

    pub fn zoom(&mut self, ratio: MapZoomRatio) -> Result<()> {
        self.span = self.span.zoomed(ratio)?;
        Ok(())
    }

    pub fn rotate(&mut self, angle: Radians) -> Result<()> {
        let radians = (f64::from(self.yaw.radians()) + f64::from(angle.radians()))
            .rem_euclid(std::f64::consts::TAU) as f32;
        self.yaw = Radians::new(radians)?;
        Ok(())
    }

    pub fn pan(&mut self, delta: MapPointerDisplacement) -> Result<()> {
        let Some(rect) = self.rect else {
            return Ok(());
        };
        // Physical canvas pixels enter the orthographic/geographic arithmetic
        // port once. f64 retains finite products for all admitted input floats.
        let pixels = delta.pixels();
        let metres_per_pixel = f64::from(self.span.metres()) / f64::from(rect.full_height.max(1));
        let horizontal = -f64::from(pixels.x) * metres_per_pixel;
        let vertical = f64::from(pixels.y) * metres_per_pixel / f64::from(MAP_PITCH_RADIANS.sin());
        let yaw = f64::from(self.yaw.radians());
        let east = horizontal * yaw.cos() - vertical * yaw.sin();
        let north = horizontal * yaw.sin() + vertical * yaw.cos();
        let point = NativeTerrainCoordinate::from(self.origin.to_e7()).at_offset(east, north);
        self.origin = geographic_origin(point)?;
        Ok(())
    }

    pub fn requested_window(&self) -> Result<Option<RegionalTerrainRequest>> {
        let Some(rect) = self.rect else {
            return Ok(None);
        };
        let aspect = f64::from(rect.full_width.max(1)) / f64::from(rect.full_height.max(1));
        let needed = f64::from(self.span.metres()) * aspect.max(1.0) * WINDOW_COVERAGE_MARGIN;
        let scale = [
            RegionalTerrainScale::Neighborhood,
            RegionalTerrainScale::District,
            RegionalTerrainScale::Region,
            RegionalTerrainScale::Country,
            RegionalTerrainScale::Continent,
        ]
        .into_iter()
        .find(|scale| {
            f64::from(scale.spacing_metres()) * (REGIONAL_TERRAIN_SIDE - 1) as f64 >= needed
        })
        .unwrap_or(RegionalTerrainScale::Continent);
        let home = NativeTerrainCoordinate::from(self.home.to_e7());
        let offset = home.offset_to(self.origin.to_e7());
        let step = f64::from(scale.spacing_metres()) * WINDOW_CENTER_STEP_CELLS as f64;
        let point = home.at_offset(
            (offset.east_metres / step).round() * step,
            (offset.north_metres / step).round() * step,
        );
        Ok(Some(RegionalTerrainRequest {
            origin: geographic_origin(point)?,
            scale,
        }))
    }
}

fn geographic_origin(point: NativeTerrainCoordinate) -> Result<Wgs84CoordinateMicrodegrees> {
    Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
        point.longitude_degrees.clamp(
            LongitudeMicrodegrees::MIN.degrees(),
            LongitudeMicrodegrees::MAX.degrees(),
        ),
        point.latitude_degrees.clamp(
            LatitudeMicrodegrees::MIN.degrees(),
            LatitudeMicrodegrees::MAX.degrees(),
        ),
    )
    .ok_or(MapProtocolError::Origin)
}
