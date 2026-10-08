//! Door approaches stop inside the owned plot or its near half of the street.
use super::*;
mod surface;
use super::super::access::{access_regions, available_run};

pub(super) fn prepare(
    request: &SingleBuildingSupportRequest<'_>,
    door: DoorwaySupportBinding,
    surface: &PropertySupportSurface,
) -> Result<PreparedApron, SupportDiagnostic> {
    let reject = |constraint, point, measured, permitted| {
        surface.rejection(
            constraint,
            SupportBoundary::StreetLanding,
            point,
            measured,
            permitted,
        )
    };
    if !door.outward.vector().is_finite()
        || (door.outward.vector().length_squared() - 1.0).abs()
            > surface.limits.contact_tolerance_metres.metres()
    {
        return Err(reject(
            SupportConstraint::ThresholdBinding,
            door.threshold_metres.metres(),
            1.0,
            0.0,
        ));
    }
    let direction = door.outward.vector();
    let tangent = Vec2::new(direction.y, -direction.x);
    let outside = planar::distance_outside(
        door.threshold_metres.metres().as_dvec2(),
        &surface.clipping_outlines[0],
    ) as f32;
    if outside > surface.limits.contact_tolerance_metres.metres() {
        return Err(reject(
            SupportConstraint::ThresholdBearing,
            door.threshold_metres.metres(),
            outside,
            surface.limits.contact_tolerance_metres.metres(),
        ));
    }
    let edge = door.threshold_metres.metres();
    let width = request.policy.doorway_apron.dimensions_metres().x;
    let offset = tangent * width * 0.5;
    let private = vec![
        request.property.plot.corners().map(Vec2::as_dvec2).to_vec(),
        surface.clipping_outlines[0].clone(),
    ];
    let mut regions =
        access_regions(request.property.plot, request.streets, edge).map_err(|cause| {
            let mut error = surface.rejection(
                SupportConstraint::Reservation,
                SupportBoundary::FrontBearing,
                edge,
                1.0,
                0.0,
            );
            error.construction_failure =
                Some(Box::new(SupportConstructionError::FramedGeometry(cause)));
            error
        })?;
    regions.push(surface.clipping_outlines[0].clone());
    let internal_run = available_run(
        edge,
        direction,
        offset,
        &private,
        f32::INFINITY,
        request.policy.limits.contact_tolerance_metres(),
    );
    let run = available_run(
        edge,
        direction,
        offset,
        &regions,
        internal_run + request.policy.doorway_apron.dimensions_metres().y,
        request.policy.limits.contact_tolerance_metres(),
    );
    // Keep a contact-sized natural strip at the approach boundary. Opposing
    // approaches must not acquire the same street support through f32 rounding.
    let run = (run - request.policy.limits.contact_tolerance_metres()).max(0.0);
    let landing = request
        .policy
        .stairs
        .minimum_floor_landing_run_metres
        .metres();
    if run <= landing {
        let mut error = reject(SupportConstraint::StairClearance, edge, run, landing);
        error.violation = error.violation.with_bound(SupportBound::Minimum);
        return Err(error);
    }
    let outer = edge + direction * run;
    let points = [outer - offset, outer + offset, edge + offset, edge - offset];
    let source = source_pair(request, surface, [points[0], points[1]])?;
    Ok(PreparedApron {
        outer,
        direction,
        offset,
        run,
        landing,
        source,
        edge,
    })
}

#[derive(Clone, Copy)]
pub(super) struct PreparedApron {
    outer: Vec2,
    direction: Vec2,
    offset: Vec2,
    run: f32,
    landing: f32,
    source: [f32; 2],
    edge: Vec2,
}

/// Independent lower and upper floor constraints. A crossed pair represents
/// infeasibility and is diagnosed by floor selection, never silently reordered.
pub(super) struct FloorConstraints {
    pub minimum: SupportElevation,
    pub maximum: SupportElevation,
}

impl PreparedApron {
    pub fn floor_interval(&self, policy: SinglePropertyGradingPolicy) -> FloorConstraints {
        let available = self.run - self.landing;
        let width = self.offset.length() * 2.0;
        let across = (self.source[0] - self.source[1]).abs() / width;
        let ramp = available
            * (policy.limits.maximum_grade.ratio().powi(2) - across.powi(2))
                .max(0.0)
                .sqrt();
        let steps = (available / policy.stairs.minimum_going_metres.metres()).floor()
            * policy.stairs.maximum_riser_metres.metres();
        let rise = if across <= policy.limits.maximum_grade.ratio() {
            ramp.max(steps)
        } else {
            0.0
        };
        FloorConstraints {
            minimum: SupportElevation(
                self.source.into_iter().fold(f32::NEG_INFINITY, f32::max) - rise,
            ),
            maximum: SupportElevation(self.source.into_iter().fold(f32::INFINITY, f32::min) + rise),
        }
    }

    pub fn append_to(
        self,
        request: &SingleBuildingSupportRequest<'_>,
        floor: SupportElevation,
        surface: &mut PropertySupportSurface,
    ) -> Result<(), SupportDiagnostic> {
        let Self {
            outer,
            direction,
            offset,
            run,
            landing,
            source,
            edge,
        } = self;
        let mesh = surface::ApronSurfaceGeometry {
            outer,
            direction,
            offset,
            run,
            landing,
            source,
            floor,
        }
        .compile(
            surface,
            surface::ApronSurfaceBounds {
                stairs: request.policy.stairs,
                limits: surface.limits,
            },
        )?;
        let tangent = Vec2::new(direction.y, -direction.x);
        let width = offset.length() * 2.0;
        let region = (|| -> adventuresim_building_generator::spatial_geometry::GeometryResult<CityPlotBounds> { CityPlotBounds::new(crate::scene_coordinates::ScenePlanPoint::try_from((outer + edge) * 0.5)?,adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(Vec2::new(width, run))?,BuildingOrientation::from_frontage_tangent(tangent)
                .ok_or(adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection)?) })().map_err(|cause| { let mut error = surface.rejection(SupportConstraint::Reservation, SupportBoundary::FrontBearing, outer, 1.0, 0.0); error.construction_failure = Some(Box::new(SupportConstructionError::FramedGeometry(cause))); error })?;
        surface
            .mesh
            .append_outside_floor(&mesh, &surface.clipping_outlines[0])?;
        surface.regions.push(region);
        surface
            .clipping_outlines
            .push(region.corners().map(Vec2::as_dvec2).to_vec());
        Ok(())
    }
}

/// Native clipping vertices enter the scene point frame before geographic queries.
fn source_pair(
    request: &SingleBuildingSupportRequest<'_>,
    surface: &PropertySupportSurface,
    points: [Vec2; 2],
) -> Result<[f32; 2], SupportDiagnostic> {
    let mut source = [0.0; 2];
    for (index, point) in points.into_iter().enumerate() {
        let rejection = || {
            surface.rejection(
                SupportConstraint::SourceSample,
                SupportBoundary::StreetLanding,
                point,
                1.0,
                0.0,
            )
        };
        let admitted =
            crate::scene_coordinates::ScenePlanPoint::try_from(point).map_err(|cause| {
                let mut error = rejection();
                error.construction_failure =
                    Some(Box::new(SupportConstructionError::FramedGeometry(cause)));
                error
            })?;
        source[index] = request
            .geographic
            .elevation_at(admitted)
            .ok_or_else(rejection)?
            .metres();
    }
    Ok(source)
}

#[cfg(test)]
mod constraint_tests {
    use super::*;
    #[test]
    fn impossible_crossfall_preserves_crossed_floor_constraints_for_rejection() {
        let apron = PreparedApron {
            outer: Vec2::Y * 4.0,
            direction: Vec2::Y,
            offset: Vec2::X,
            run: 4.0,
            landing: 0.5,
            source: [6.0, -6.0],
            edge: Vec2::ZERO,
        };
        let policy = SinglePropertyGradingPolicy {
            limits: SupportLimits::new(
                crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.001,
                )
                .unwrap(),
            ),
            stairs: CourtStairLimits::new(
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.19,
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.25,
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.0)
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    1.05,
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
                    .unwrap(),
            ),
            embedment: FoundationEmbedment::from_metres(0.2).unwrap(),
            doorway_apron: crate::city_layout::StreetApronDimensions::from_metres(Vec2::new(
                1.0, 4.0,
            ))
            .unwrap(),
        };
        let bounds = apron.floor_interval(policy);
        assert!(bounds.minimum.metres() > bounds.maximum.metres());
    }
}
