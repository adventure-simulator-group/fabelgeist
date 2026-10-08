//! Plan clearance against measured plant hulls and resolved building envelopes.
use super::*;
use bevy::math::Vec2;

const MAX_PROPERTY_EXTENT_METRES: f32 = 100.0;
const MAX_GARDEN_ELEMENTS: usize = 16;
const MAX_ACCESS_HALF_WIDTH_METRES: f32 = 2.0;
const MAX_STREET_APPROACH_METRES: f32 = 6.0;
const MAX_PLANT_SCALE: f32 = 2.0;

/// Exact working rectangles and wind-expanded planting hulls, not a plot box.
pub(in crate::city_layout) struct GardenClearanceGeometry {
    pub working: Vec<CityPlotBounds>,
    pub plants: Vec<Vec<crate::scene_coordinates::ScenePlanPoint>>,
}

impl CityGarden {
    pub(in crate::city_layout) fn translate(
        &mut self,
        delta: crate::scene_coordinates::PlanDisplacement,
        tangent: bevy::math::Dir2,
    ) -> adventuresim_building_generator::spatial_geometry::GeometryResult<()> {
        self.plot = self.plot.translated(delta)?;
        self.cultivated_bounds = self.cultivated_bounds.translated(delta)?;
        for bed in &mut self.beds {
            *bed = bed.translated(delta)?;
        }
        for plant in &mut self.plants {
            plant.centre_metres = plant.centre_metres.translated(delta)?;
        }
        super::super::compound::translate_property_access(&mut self.access, delta, tangent)?;
        Ok(())
    }

    pub(in crate::city_layout) fn clearance_geometry(
        &self,
    ) -> Result<GardenClearanceGeometry, GardenIssue> {
        let mut working = self.beds.clone();
        for segment in &self.access {
            working.push(corridor(*segment).ok_or(GardenIssue::InvalidAccess)?);
        }
        Ok(GardenClearanceGeometry {
            working,
            plants: self
                .plants
                .iter()
                .map(|plant| plant.world_hull())
                .collect::<Result<_, _>>()?,
        })
    }

    /// Includes projections and eaves; this is deliberately conservative in height.
    pub fn clears_building(&self, envelope: CityPlotBounds) -> bool {
        // Validated garden geometry stays in the plot, except its bounded
        // street approach. Reject distant pairs before constructing exact hulls.
        let reach = self.plot.dimensions_metres().length() * 0.5
            + MAX_STREET_APPROACH_METRES
            + envelope.dimensions_metres().length() * 0.5;
        if self
            .plot
            .centre_metres()
            .distance_squared(envelope.centre_metres())
            > reach * reach
        {
            return true;
        }
        self.beds.iter().all(|bed| !bed.intersects(envelope))
            && self
                .access
                .iter()
                .all(|segment| corridor(*segment).is_some_and(|route| !route.intersects(envelope)))
            && self.plants.iter().all(|plant| {
                plant.world_hull().is_ok_and(|hull| {
                    !overlaps(
                        &hull
                            .into_iter()
                            .map(crate::scene_coordinates::ScenePlanPoint::metres)
                            .collect::<Vec<_>>(),
                        &envelope.corners(),
                        GARDEN_LEAF_WIND_CLEARANCE_METRES,
                    )
                })
            })
    }

    pub fn validate_geometry(
        &self,
        streets: &[super::super::CityStreetPatch],
    ) -> Result<(), GardenIssue> {
        self.validate_bounds()?;
        self.validate_access(streets)?;
        self.validate_plants()
    }

    fn validate_bounds(&self) -> Result<(), GardenIssue> {
        if !self.plot.is_valid()
            || !self.cultivated_bounds.is_valid()
            || self.beds.is_empty()
            || self.access.is_empty()
            || self.plot.dimensions_metres().max_element() > MAX_PROPERTY_EXTENT_METRES
            || self.beds.len() > MAX_GARDEN_ELEMENTS
            || self.access.len() > MAX_GARDEN_ELEMENTS
            || self.plants.len() > MAX_GARDEN_ELEMENTS
            || !self
                .cultivated_bounds
                .corners()
                .iter()
                .all(|point| self.plot.contains(*point))
        {
            return Err(GardenIssue::InvalidBounds);
        }
        for (index, bed) in self.beds.iter().enumerate() {
            if !bed.is_valid()
                || self.beds[..index]
                    .iter()
                    .any(|other| other.intersects(*bed))
                || !bed
                    .corners()
                    .iter()
                    .all(|point| self.cultivated_bounds.contains(*point))
            {
                return Err(GardenIssue::InvalidBounds);
            }
        }
        Ok(())
    }

    fn validate_access(
        &self,
        streets: &[super::super::CityStreetPatch],
    ) -> Result<(), GardenIssue> {
        if !streets
            .iter()
            .any(|street| street.contains(self.access[0].start_metres()))
        {
            return Err(GardenIssue::StreetDisconnected);
        }
        for (index, segment) in self.access.iter().enumerate() {
            if !segment.start_metres().is_finite()
                || !segment.end_metres().is_finite()
                || !segment.half_width_metres().is_finite()
                || segment.half_width_metres() <= 0.0
                || segment.half_width_metres() > MAX_ACCESS_HALF_WIDTH_METRES
                || segment.start_metres().distance(segment.end_metres())
                    > MAX_PROPERTY_EXTENT_METRES
            {
                return Err(GardenIssue::InvalidAccess);
            }
            let route = corridor(*segment).ok_or(GardenIssue::InvalidAccess)?;
            if index > 0
                && !self.access[..index]
                    .iter()
                    .any(|earlier| corridor(*earlier).is_some_and(|other| other.intersects(route)))
            {
                return Err(GardenIssue::DisconnectedTendingLane);
            }
            // Only the street approach may leave the owned plot. Its width
            // stays within the side boundary and its end is inside the plot.
            let approach_corners = route.corners().map(|p| {
                self.plot
                    .orientation()
                    .world_to_local(p - self.plot.centre_metres())
            });
            let half = self.plot.dimensions_metres() * 0.5;
            if !self.plot.contains(segment.end_metres())
                || (index == 0
                    && approach_corners.iter().any(|p| {
                        p.x.abs() > half.x
                            || p.y < -half.y - MAX_STREET_APPROACH_METRES
                            || p.y > half.y
                    }))
                || (index > 0 && route.corners().iter().any(|p| !self.plot.contains(*p)))
            {
                return Err(GardenIssue::InvalidAccess);
            }
            if self.beds.iter().any(|bed| bed.intersects(route)) {
                return Err(GardenIssue::ObstructedAccess);
            }
        }
        Ok(())
    }

    fn validate_plants(&self) -> Result<(), GardenIssue> {
        let routes = self
            .access
            .iter()
            .map(|segment| corridor(*segment).ok_or(GardenIssue::InvalidAccess))
            .collect::<Result<Vec<_>, _>>()?;
        let hulls = self
            .plants
            .iter()
            .map(|plant| {
                plant.world_hull().map(|hull| {
                    hull.into_iter()
                        .map(crate::scene_coordinates::ScenePlanPoint::metres)
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (index, plant) in self.plants.iter().enumerate() {
            if !plant.centre_metres.metres().is_finite()
                || !plant.orientation.is_valid()
                || !plant.scale.is_valid()
                || plant.scale.value() > MAX_PLANT_SCALE
            {
                return Err(GardenIssue::InvalidPlant);
            }
            let hull = &hulls[index];
            if !hull.iter().all(|point| {
                let local = self
                    .cultivated_bounds
                    .orientation()
                    .world_to_local(*point - self.cultivated_bounds.centre_metres());
                (local.abs() + Vec2::splat(GARDEN_LEAF_WIND_CLEARANCE_METRES))
                    .cmple(self.cultivated_bounds.dimensions_metres() * 0.5)
                    .all()
            }) {
                return Err(GardenIssue::PlantOutsidePlot);
            }
            if self
                .beds
                .iter()
                .any(|bed| overlaps(&hull, &bed.corners(), GARDEN_LEAF_WIND_CLEARANCE_METRES))
                || routes.iter().any(|route| {
                    overlaps(&hull, &route.corners(), GARDEN_LEAF_WIND_CLEARANCE_METRES)
                })
            {
                return Err(GardenIssue::PlantObstructsWorkingSpace);
            }
            if self.plants[..index]
                .iter()
                .enumerate()
                .any(|(other_index, other)| {
                    other.id == plant.id
                        || overlaps(
                            &hull,
                            &hulls[other_index],
                            GARDEN_LEAF_WIND_CLEARANCE_METRES * 2.0,
                        )
                })
            {
                return Err(GardenIssue::OverlappingPlants);
            }
        }
        Ok(())
    }
}

pub(super) fn corridor(segment: CityAccessSegment) -> Option<CityPlotBounds> {
    let direction = segment.end_metres() - segment.start_metres();
    CityPlotBounds::new(
        crate::scene_coordinates::ScenePlanPoint::try_from(
            (segment.start_metres() + segment.end_metres()) * 0.5,
        )
        .ok()?,
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
            direction.length() + segment.half_width_metres() * 2.0,
            segment.half_width_metres() * 2.0,
        ))
        .ok()?,
        BuildingOrientation::from_frontage_tangent(direction)?,
    )
    .ok()
}

pub(super) fn overlaps(first: &[Vec2], second: &[Vec2], margin: f32) -> bool {
    [first, second]
        .into_iter()
        .flat_map(|polygon| {
            polygon
                .iter()
                .zip(polygon.iter().cycle().skip(1))
                .take(polygon.len())
        })
        .all(|(a, b)| {
            let axis = (*b - *a).perp().normalize_or_zero();
            let interval = |points: &[Vec2]| {
                points
                    .iter()
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), point| {
                        let p = point.dot(axis);
                        (min.min(p), max.max(p))
                    })
            };
            let (a, b) = interval(first);
            let (c, d) = interval(second);
            b + margin > c && d + margin > a
        })
}
