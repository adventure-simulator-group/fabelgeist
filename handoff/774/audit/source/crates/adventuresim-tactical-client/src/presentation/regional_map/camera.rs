//! Retained geographic camera pose. Native view X is east, Y is elevation and
//! Z is south; the source sampler retains its canonical east/north convention.
use super::protocol::{CanvasRect, MapPointerDisplacement, MapProtocolError};
use adventuresim_building_generator::spatial_geometry::{PositiveLength, Radians};
use adventuresim_tactical_core::{
    regional_map::{MAX_MAP_SPAN_METRES, MIN_MAP_SPAN_METRES, MapRoute, MapSpan, MapZoomRatio},
    regional_terrain::{REGIONAL_TERRAIN_SIDE, RegionalTerrainRequest, RegionalTerrainScale},
};
use adventuresim_world_schema::coordinates::{
    LatitudeMicrodegrees, LongitudeMicrodegrees, Wgs84CoordinateMicrodegrees,
    terrain_projection::NativeTerrainCoordinate,
};
use adventuresim_world_schema::source_package::SourcePackageDigest;
use bevy::{math::DVec2, prelude::*};

pub(super) const MAP_PITCH_RADIANS: f32 = std::f32::consts::PI / 3.0;
const WINDOW_COVERAGE_MARGIN: f64 = 2.0;
const WINDOW_CENTER_STEP_CELLS: usize = (REGIONAL_TERRAIN_SIDE - 1) / 4;
const ROUTE_FRAMING_MARGIN: f64 = 1.25;
pub(super) type Result<T> = std::result::Result<T, MapProtocolError>;

pub(super) struct MapPose {
    pub source: SourcePackageDigest,
    pub home: Wgs84CoordinateMicrodegrees,
    pub origin: NativeTerrainCoordinate,
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
            origin: home.to_e7().into(),
            span,
            yaw: Radians::ZERO,
            rect: Some(rect),
            home_span: span,
        }
    }

    pub fn reset(&mut self) {
        self.origin = self.home.to_e7().into();
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

    pub fn pan(
        &mut self,
        delta: MapPointerDisplacement,
        frame: Wgs84CoordinateMicrodegrees,
    ) -> Result<()> {
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
        let frame = NativeTerrainCoordinate::from(frame.to_e7());
        let current = frame.offset_to(self.origin);
        let point = frame.at_offset(current.east_metres + east, current.north_metres + north);
        self.origin = bounded_origin(point)?;
        Ok(())
    }

    pub fn geographic_origin(&self) -> Result<Wgs84CoordinateMicrodegrees> {
        geographic_origin(self.origin)
    }

    pub fn frame_route(&mut self, route: &MapRoute) -> Result<()> {
        let Some(rect) = self.rect else {
            return Ok(());
        };
        let frame = NativeTerrainCoordinate::from(self.home.to_e7());
        let yaw = f64::from(self.yaw.radians());
        let right = DVec2::new(yaw.cos(), yaw.sin());
        let forward = DVec2::new(-yaw.sin(), yaw.cos());
        let mut minimum = DVec2::splat(f64::INFINITY);
        let mut maximum = DVec2::splat(f64::NEG_INFINITY);
        // Native fitting port: route points become rotated east/north metres.
        for point in route.points() {
            let offset = frame.offset_to(point.to_e7().into());
            let native = DVec2::new(offset.east_metres, offset.north_metres);
            let camera = DVec2::new(native.dot(right), native.dot(forward));
            minimum = minimum.min(camera);
            maximum = maximum.max(camera);
        }
        let centre = (minimum + maximum) * 0.5;
        let native = right * centre.x + forward * centre.y;
        let extent = maximum - minimum;
        let aspect = f64::from(rect.full_width) / f64::from(rect.full_height);
        let span = (extent.x / aspect).max(extent.y * f64::from(MAP_PITCH_RADIANS.sin()))
            * ROUTE_FRAMING_MARGIN;
        let span = MapSpan::try_from(PositiveLength::from_metres(span.clamp(
            f64::from(MIN_MAP_SPAN_METRES),
            f64::from(MAX_MAP_SPAN_METRES),
        ) as f32)?)?;
        self.origin = bounded_origin(frame.at_offset(native.x, native.y))?;
        self.span = span;
        Ok(())
    }

    pub fn requested_window(&self) -> Result<Option<RegionalTerrainRequest>> {
        let Some(rect) = self.rect else {
            return Ok(None);
        };
        let aspect = f64::from(rect.full_width.max(1)) / f64::from(rect.full_height.max(1));
        let width = f64::from(self.span.metres()) * aspect;
        let height = f64::from(self.span.metres()) / f64::from(MAP_PITCH_RADIANS.sin());
        let yaw = f64::from(self.yaw.radians());
        let east_extent = width * yaw.cos().abs() + height * yaw.sin().abs();
        let north_extent = width * yaw.sin().abs() + height * yaw.cos().abs();
        let needed = east_extent.max(north_extent) * WINDOW_COVERAGE_MARGIN;
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
        let offset = home.offset_to(self.origin);
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
    let point = bounded_origin(point)?;
    Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
        point.longitude_degrees,
        point.latitude_degrees,
    )
    .ok_or(MapProtocolError::Origin)
}

fn bounded_origin(point: NativeTerrainCoordinate) -> Result<NativeTerrainCoordinate> {
    if !point.longitude_degrees.is_finite() || !point.latitude_degrees.is_finite() {
        return Err(MapProtocolError::Origin);
    }
    Ok(NativeTerrainCoordinate {
        longitude_degrees: point.longitude_degrees.clamp(
            LongitudeMicrodegrees::MIN.degrees(),
            LongitudeMicrodegrees::MAX.degrees(),
        ),
        latitude_degrees: point.latitude_degrees.clamp(
            LatitudeMicrodegrees::MIN.degrees(),
            LatitudeMicrodegrees::MAX.degrees(),
        ),
    })
}
