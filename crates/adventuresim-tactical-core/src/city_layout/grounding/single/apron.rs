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
    if !door.outward.is_finite()
        || (door.outward.length_squared() - 1.0).abs() > surface.limits.contact_tolerance_metres
    {
        return Err(reject(
            SupportConstraint::ThresholdBinding,
            door.threshold_metres,
            1.0,
            0.0,
        ));
    }
    let direction = door.outward;
    let tangent = Vec2::new(direction.y, -direction.x);
    let outside = planar::distance_outside(
        door.threshold_metres.as_dvec2(),
        &surface.clipping_outlines[0],
    ) as f32;
    if outside > surface.limits.contact_tolerance_metres {
        return Err(reject(
            SupportConstraint::ThresholdBearing,
            door.threshold_metres,
            outside,
            surface.limits.contact_tolerance_metres,
        ));
    }
    let edge = door.threshold_metres;
    let width = request.policy.doorway_apron.dimensions_metres().x;
    let offset = tangent * width * 0.5;
    let private = vec![
        request.property.plot.corners().map(Vec2::as_dvec2).to_vec(),
        surface.clipping_outlines[0].clone(),
    ];
    let mut regions = access_regions(request.property.plot, request.streets, edge);
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
    let landing = request.policy.stairs.minimum_floor_landing_run_metres;
    if run <= landing {
        return Err(reject(
            SupportConstraint::StairClearance,
            edge,
            landing,
            run,
        ));
    }
    let outer = edge + direction * run;
    let points = [outer - offset, outer + offset, edge + offset, edge - offset];
    let mut source = [0.0; 2];
    for i in 0..2 {
        source[i] = request
            .geographic
            .elevation_at(points[i])
            .ok_or_else(|| reject(SupportConstraint::SourceSample, points[i], 1.0, 0.0))?
            .metres();
    }
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
            * (policy.limits.maximum_grade.powi(2) - across.powi(2))
                .max(0.0)
                .sqrt();
        let steps = (available / policy.stairs.minimum_going_metres).floor()
            * policy.stairs.maximum_riser_metres;
        let rise = if across <= policy.limits.maximum_grade {
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
        let region = CityPlotBounds {
            centre_metres: (outer + edge) * 0.5,
            dimensions_metres: Vec2::new(width, run),
            orientation: BuildingOrientation::from_frontage_tangent(tangent)
                .expect("finite nonzero bound doorway/street axis"),
        };
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
            limits: SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
            stairs: CourtStairLimits::new(0.19, 0.25, 1.0, 1.05, 0.5).unwrap(),
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
