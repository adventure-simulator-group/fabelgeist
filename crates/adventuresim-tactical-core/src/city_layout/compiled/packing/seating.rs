//! Actual floor bearings fit owned land before a property enters block packing.
use super::*;

impl CompiledCityLayout {
    pub(super) fn seat_single_bearings(&mut self) -> CityCompileResult<()> {
        for property in &self.single_properties {
            let building = self
                .buildings
                .iter_mut()
                .find(|b| b.id == property.building_id)
                .ok_or(CityCompileError::Packing {
                    property: property.id,
                    issue: CityPackingIssue::MissingMember {
                        building: property.building_id,
                    },
                })?;
            let recipe = self.support_recipes.for_program(&building.program)?;
            let footprint = recipe
                .collision
                .ground_floor_footprint()
                .map_err(|issue| CityCompileError::Packing {
                    property: property.id,
                    issue: CityPackingIssue::InvalidBearing {
                        building: building.id,
                        issue,
                    },
                })?
                .ok_or(CityCompileError::Packing {
                    property: property.id,
                    issue: CityPackingIssue::MissingBearing {
                        building: building.id,
                    },
                })?;
            let origin = recipe.collision.bounds.centre()?.metres();
            let (min, max) = footprint
                .vertices()
                .iter()
                .map(|point| {
                    property.plot.orientation().world_to_local(
                        building.centre_metres.metres()
                            + building
                                .orientation
                                .local_to_world(point.metres() - Vec2::new(origin.x, origin.z))
                            - property.plot.centre_metres(),
                    )
                })
                .fold(
                    (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                    |(min, max), point| (min.min(point), max.max(point)),
                );
            let half = property.plot.dimensions_metres() * 0.5;
            let tolerance = Vec2::splat(CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32);
            let lower = -half - min + tolerance;
            let upper = half - max - tolerance;
            if !lower.cmple(upper).all() {
                return Err(CityCompileError::Packing {
                    property: property.id,
                    issue: CityPackingIssue::BearingOutsidePlot {
                        building: building.id,
                        minimum_local_metres: min,
                        maximum_local_metres: max,
                        plot_half_dimensions_metres: half,
                    },
                });
            }
            // The programme and rotation stay fixed. Reserve the existing
            // side passage and rear court; no footprint is scaled or clipped.
            building.centre_metres = building.centre_metres.translated(
                crate::scene_coordinates::PlanDisplacement::try_from(
                    property
                        .plot
                        .orientation()
                        .local_to_world(Vec2::ZERO.clamp(lower, upper)),
                )?,
            )?;
        }
        self.reconnect_garden_lanes()
    }

    fn reconnect_garden_lanes(&mut self) -> CityCompileResult<()> {
        for garden in &mut self.gardens {
            let building = self
                .buildings
                .iter()
                .find(|b| b.id == garden.front_building_id)
                .ok_or(CityCompileError::Packing {
                    property: garden.owner,
                    issue: CityPackingIssue::MissingMember {
                        building: garden.front_building_id,
                    },
                })?;
            let recipe = self.support_recipes.for_program(&building.program)?;
            let bounds = gardens::envelope(building, &recipe)?;
            let (min, max) = bounds
                .corners()
                .into_iter()
                .map(|point| {
                    garden
                        .plot
                        .orientation()
                        .world_to_local(point - garden.plot.centre_metres())
                        .x
                })
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| {
                    (min.min(x), max.max(x))
                });
            let junction = garden.access[0].end_metres();
            let lane = garden
                .plot
                .orientation()
                .world_to_local(junction - garden.plot.centre_metres())
                .x;
            let clearance = garden.access[0].half_width_metres()
                + CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32;
            let offset = if lane > 0.0 {
                (max + clearance - lane).max(0.0)
            } else {
                (min - clearance - lane).min(0.0)
            };
            let delta = crate::scene_coordinates::PlanDisplacement::try_from(
                garden.plot.orientation().local_to_world(Vec2::X * offset),
            )?;
            garden.access[0] = garden.access[0].translated(delta, delta)?;
            for route in &mut garden.access[1..] {
                if route.start_metres().distance(junction)
                    <= CityAccessSegment::JOIN_TOLERANCE_METRES
                {
                    *route = route.with_endpoints(route.start().translated(delta)?, route.end())?;
                }
                if route.end_metres().distance(junction) <= CityAccessSegment::JOIN_TOLERANCE_METRES
                {
                    *route = route.with_endpoints(route.start(), route.end().translated(delta)?)?;
                }
            }
            garden
                .validate_geometry(&self.streets)
                .map_err(|issue| CityCompileError::Packing {
                    property: garden.owner,
                    issue: CityPackingIssue::Garden { issue },
                })?;
        }
        Ok(())
    }
}
