use super::*;
use crate::scene_input::{DistantBuildingPlacement, GeneratedBuildingRecipe};
use bevy::math::Vec3Swizzles;
use serde_json::Value;

mod enclosures;
mod foundations;
mod movement;

pub(super) struct Fixture {
    pub(super) property: CityCompound,
    levels: CompoundSupportLevels,
    document: Value,
}

fn elevation(metres: f32) -> SupportElevation {
    SupportElevation::from_metres(metres).unwrap()
}

fn terraced() -> CourtTreatment {
    CourtTreatment::Terraced(CourtStairLimits::new(
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.19)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.25)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.0)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
            .unwrap(),
    ))
}

impl Fixture {
    pub(super) fn selected_plan(&self, geographic: &GeographicSurface) -> CompoundSupportPlan {
        self.try_selected_plan(geographic).unwrap()
    }

    fn try_selected_plan(
        &self,
        geographic: &GeographicSurface,
    ) -> Result<CompoundSupportPlan, SupportDiagnostic> {
        let value = &self.document;
        let chosen = &value["doorway_solution"];
        let metres = |key| chosen[key].as_f64().unwrap() as f32;
        let threshold = serde_json::from_value(value["front_street_threshold"].clone()).unwrap();
        let mut observations = self.levels;
        let sample = |point| geographic.elevation_at(point).unwrap();
        observations.front.elevation = sample(threshold);
        observations.rear.elevation = sample(observations.rear.court_threshold_metres);
        observations.court = sample(
            crate::scene_coordinates::ScenePlanPoint::try_from(self.property.court.centre_metres())
                .unwrap(),
        );
        observations.gate = sample(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                self.property.boundary.gate.centre_metres,
            )
            .unwrap(),
        );
        observations.street = sample(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                self.property.access[0].start_metres(),
            )
            .unwrap(),
        );
        CompoundSupportRequest {
            property: &self.property,
            observations,
            street_threshold_metres: threshold,
            street_apron: serde_json::from_value(chosen["street_entry_apron"].clone()).unwrap(),
            geographic,
            limits: SupportLimits::new(
                crate::city_layout::grounding::SupportGrade::from_ratio(metres("maximum_grade"))
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("maximum_displacement_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("contact_tolerance_m"),
                )
                .unwrap(),
            ),
            stairs: CourtStairLimits::new(
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("maximum_riser_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("minimum_going_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("clear_stair_width_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("endpoint_landing_run_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("court_landing_run_m"),
                )
                .unwrap(),
            ),
            embedment: FoundationEmbedment::from_metres(metres("foundation_embedment_m")).unwrap(),
        }
        .select()
    }

    fn doorway_plan(&mut self) -> CompoundSupportPlan {
        let value = &self.document;
        let chosen = &value["doorway_solution"];
        let metres = |key| chosen[key].as_f64().unwrap() as f32;
        self.levels.front.elevation = elevation(metres("front_floor_m"));
        self.levels.rear.elevation = elevation(metres("rear_floor_m"));
        self.levels.court = elevation(metres("court_floor_m"));
        let plan = CompoundSupportPlan::compile(
            &self.property,
            self.levels,
            SupportLimits::new(
                crate::city_layout::grounding::SupportGrade::from_ratio(metres("maximum_grade"))
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("maximum_displacement_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("contact_tolerance_m"),
                )
                .unwrap(),
            ),
            CourtTreatment::Terraced(CourtStairLimits::new(
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("maximum_riser_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("minimum_going_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("clear_stair_width_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("endpoint_landing_run_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("court_landing_run_m"),
                )
                .unwrap(),
            )),
        )
        .unwrap();
        let threshold: Vec2 =
            serde_json::from_value(value["front_street_threshold"].clone()).unwrap();
        let apron: CityPlotBounds =
            serde_json::from_value(chosen["street_entry_apron"].clone()).unwrap();
        plan.bind_street_entry(
            crate::scene_coordinates::ScenePlanPoint::try_from(threshold).unwrap(),
            apron,
            &foundations::geographic_fixture(),
            CourtStairLimits::new(
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("maximum_riser_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("minimum_going_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("clear_stair_width_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("endpoint_landing_run_m"),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    metres("court_landing_run_m"),
                )
                .unwrap(),
            ),
        )
        .unwrap()
    }

    pub(super) fn load() -> Self {
        let value: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-grounding/goslar-1238.json"
        )))
        .unwrap();
        Self::from_document(value)
    }

    pub(super) fn load_965() -> Self {
        Self::from_document(
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../assets/tactical-grounding/goslar-965.json"
            )))
            .unwrap(),
        )
    }

    pub(super) fn source(&self) -> GeographicSurface {
        GeographicSurface::from_triangles(
            serde_json::from_value::<Vec<[Vec3; 3]>>(self.document["geographic_triangles"].clone())
                .unwrap(),
        )
        .unwrap()
    }

    fn from_document(value: Value) -> Self {
        let property: CityCompound = serde_json::from_value(value["compound"].clone()).unwrap();
        let front: DistantBuildingPlacement =
            serde_json::from_value(value["front_distant_placement"].clone()).unwrap();
        let member = |placement: DistantBuildingPlacement, threshold, height| {
            let recipe = GeneratedBuildingRecipe::generate(placement.occupied_program()).unwrap();
            let contact = recipe
                .collision
                .ground_floor_contact_bounds()
                .unwrap()
                .unwrap();
            MemberSupport {
                building_id: placement.id,
                contact: CityPlotBounds::new(
                    crate::scene_coordinates::ScenePlanPoint::try_from(
                        placement.centre_metres.metres()
                            + placement.orientation.local_to_world(
                                contact.centre().unwrap().metres().xz()
                                    - recipe.collision.bounds.centre().unwrap().metres().xz(),
                            ),
                    )
                    .unwrap(),
                    adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                        contact.plan_half_extents().unwrap().metres() * 2.0,
                    )
                    .unwrap(),
                    placement.orientation,
                )
                .unwrap(),
                court_threshold_metres: serde_json::from_value(value[threshold].clone()).unwrap(),
                elevation: elevation(value[height].as_f64().unwrap() as f32),
            }
        };
        let rear: DistantBuildingPlacement =
            serde_json::from_value(value["rear_distant_placement"].clone()).unwrap();
        Self {
            levels: CompoundSupportLevels {
                front: member(front, "front_threshold", "front_floor_m"),
                rear: member(rear, "rear_threshold", "rear_candidate_m"),
                court: elevation(value["court_candidate_m"].as_f64().unwrap() as f32),
                gate: elevation(value["gate_source_m"].as_f64().unwrap() as f32),
                street: elevation(value["street_approach_source_m"].as_f64().unwrap() as f32),
            },
            property,
            document: value,
        }
    }

    fn plan(&self, court: CourtTreatment) -> CompoundSupportPlan {
        CompoundSupportPlan::compile(
            &self.property,
            self.levels,
            SupportLimits::new(
                crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.001,
                )
                .unwrap(),
            ),
            court,
        )
        .unwrap()
    }
}

#[test]
fn a_declared_street_apron_that_is_too_short_reports_its_constraint_without_expanding() {
    let mut fixture = Fixture::load_965();
    let mut apron: CityPlotBounds =
        serde_json::from_value(fixture.document["doorway_solution"]["street_entry_apron"].clone())
            .unwrap();
    let inner_edge = apron.centre_metres()
        + apron.orientation().local_to_world(Vec2::Y) * apron.dimensions_metres().y * 0.5;
    apron
        .resize(
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                bevy::math::Vec2::new(apron.dimensions_metres().x, 1.0),
            )
            .unwrap(),
        )
        .unwrap();
    apron
        .relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                inner_edge - apron.orientation().local_to_world(Vec2::Y) * 0.5,
            )
            .unwrap(),
        )
        .unwrap();
    fixture.document["doorway_solution"]["street_entry_apron"] =
        serde_json::to_value(apron).unwrap();
    let error = fixture.try_selected_plan(&fixture.source()).unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(965));
    assert_eq!(
        error.member_building_ids,
        [965, 17349].map(crate::scene_input::SceneBuildingId)
    );
    assert_eq!(error.constraint, SupportConstraint::StairGoing);
    assert_eq!(error.boundary, SupportBoundary::StreetLanding);
    assert_eq!(
        error.location_metres.attempted_metres(),
        apron.centre_metres()
    );
    println!("short_apron={}", serde_json::to_string(&error).unwrap());
    assert!(error.violation.actual_value() > error.violation.limit_value());
    assert!((error.violation.limit_value() - 1.26).abs() < 0.001);
    assert!(error.violation.discrepancy_value() > 0.1);
}

#[test]
fn geographic_selection_prefers_a_level_court_when_complete_support_and_access_fit() {
    let mut fixture = Fixture::load();
    let footprint = fixture.doorway_plan();
    let source =
        GeographicSurface::from_triangles(foundations::flat_source(&footprint, 20.0)).unwrap();
    let plan = fixture.selected_plan(&source);
    assert_eq!(plan.treatment, CourtTreatment::Level);
    assert_eq!(plan.reservation(), fixture.property.plot);
    assert!(plan.stair_flights().next().is_none());
    assert!(
        plan.member_support()
            .iter()
            .all(|member| member.elevation.metres() == 20.0)
    );
    assert_eq!(plan.court_elevation().metres(), 20.0);
    plan.foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
}

#[test]
fn goslar_geographic_selection_seats_complete_triangles_and_retains_identity() {
    let fixture = Fixture::load();
    let source = foundations::geographic_fixture();
    let plan = fixture.selected_plan(&source);
    let mut triangles: Vec<_> = source.triangles().collect();
    triangles.reverse();
    for triangle in &mut triangles {
        triangle.swap(0, 1);
    }
    let reordered = fixture.selected_plan(&GeographicSurface::from_triangles(triangles).unwrap());
    assert_eq!(plan.mesh().unwrap(), reordered.mesh().unwrap());
    assert_eq!(plan.member_support(), reordered.member_support());
    assert_eq!(plan.reservation(), fixture.property.plot);
    assert_eq!(plan.property_id(), CityPropertyId(1238));
    let [front, rear] = plan.member_support();
    assert_eq!(
        [front.building_id, rear.building_id],
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert!((front.elevation.metres() - 21.042906).abs() < 0.001);
    assert!(front.elevation.metres() > plan.court_elevation().metres());
    assert!(plan.court_elevation().metres() > rear.elevation.metres());
    assert!(plan.mesh().unwrap().maximum_grade() <= 0.65);
    let foundation = plan
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
    println!(
        "selected_support={}",
        serde_json::json!({
            "property_id": plan.property_id(),
            "members": plan.member_support(),
            "court_m": plan.court_elevation(),
            "gate_m": plan.gate_elevation(),
            "regions": plan.support_regions(),
            "maximum_grade": plan.mesh().unwrap().maximum_grade(),
            "foundation_volume_m3": foundation.volume_cubic_metres(),
            "foundation_vertices": foundation.positions.len(),
            "support_triangles": foundation.support_triangles.len(),
            "stair_flights": plan.stair_flights().collect::<Vec<_>>()
        })
    );
}

#[test]
fn goslar_1238_retains_bearings_and_generates_graded_triangles_with_separate_gate_support() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let mesh = plan.mesh().unwrap();
    assert_eq!(mesh.property_id, CityPropertyId(1238));
    assert_eq!(
        mesh.member_building_ids,
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert!(
        mesh.maximum_grade() <= 0.6501,
        "triangle grade {}",
        mesh.maximum_grade()
    );
    assert!(!mesh.retaining_triangles.is_empty());
    assert!(plan.gate_elevation().metres() - plan.member_support()[0].elevation.metres() > 2.19);
    assert!((18.48..18.50).contains(&plan.member_support()[1].elevation.metres()));
    for member in plan.member_support() {
        let half = member.contact.dimensions_metres() * 0.5;
        for x in 0..=10 {
            for z in 0..=10 {
                let local = Vec2::new(x as f32 / 10.0, z as f32 / 10.0) * 2.0 * half - half;
                let point = member.contact.centre_metres()
                    + member.contact.orientation().local_to_world(local);
                assert!(
                    mesh.elevations_at(
                        crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap()
                    )
                    .iter()
                    .any(|h| (h.metres() - member.elevation.metres()).abs() < 0.001),
                    "building {} bearing {:?} is unsupported",
                    member.building_id,
                    point
                );
            }
        }
    }
    for route in &fixture.property.access {
        for station in 0..=100 {
            let point = route.start_metres()
                + (route.end_metres() - route.start_metres()) * station as f32 / 100.0;
            let expected = plan.elevations_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap(),
            );
            assert!(expected.iter().next().is_some());
            assert!(expected.iter().any(|expected| {
                mesh.elevations_at(
                    crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap(),
                )
                .iter()
                .any(|actual| (actual.metres() - expected.metres()).abs() < 0.001)
            }));
        }
    }
}

#[test]
fn a_level_court_is_compared_without_lowering_the_separate_gate_to_the_front_floor() {
    let fixture = Fixture::load();
    let level = fixture.plan(CourtTreatment::Level);
    let stepped = fixture.plan(terraced());
    assert_eq!(level.court_elevation(), level.member_support()[0].elevation);
    assert_eq!(
        level.member_support()[0].elevation,
        level.member_support()[1].elevation
    );
    assert!(stepped.court_elevation().metres() < level.court_elevation().metres() - 1.3);
    assert_eq!(level.gate_elevation(), stepped.gate_elevation());
    assert!(level.mesh().unwrap().maximum_grade() <= 0.6501);
}

#[test]
fn support_retains_horizontal_identity_and_is_independent_of_route_iteration_order() {
    let mut fixture = Fixture::load();
    let original = fixture.plan(terraced());
    fixture.property.access.reverse();
    let reordered = fixture.plan(terraced());
    assert_eq!(original.mesh().unwrap(), reordered.mesh().unwrap());
    assert_eq!(original.member_support(), reordered.member_support());
    assert_eq!(original.reservation(), fixture.property.plot);
}

#[test]
fn intentionally_invalid_gate_approach_reports_exact_identity_location_and_shortfall() {
    let fixture = Fixture::load();
    let mut levels = fixture.levels;
    levels.gate = elevation(levels.street.metres() - 1.0);
    let error = CompoundSupportPlan::compile(
        &fixture.property,
        levels,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        terraced(),
    )
    .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(1238));
    assert_eq!(
        error.member_building_ids,
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert_eq!(error.constraint, SupportConstraint::AccessGrade);
    assert_eq!(error.boundary, SupportBoundary::GateLanding);
    assert_eq!(error.violation.actual_value(), 1.0);
    assert!((error.violation.limit_value() - 0.455).abs() < 0.001);
    assert!((error.violation.discrepancy_value() - 0.545).abs() < 0.001);
    assert!(
        error
            .location_metres
            .attempted_metres()
            .distance(fixture.property.access[0].start_metres())
            < 0.401
    );
}

#[test]
fn missing_exact_threshold_binding_is_rejected_instead_of_selecting_a_nearby_route() {
    let mut fixture = Fixture::load();
    fixture.levels.front.court_threshold_metres = fixture
        .levels
        .front
        .court_threshold_metres
        .translated(crate::scene_coordinates::PlanDisplacement::try_from(Vec2::X).unwrap())
        .unwrap();
    let error = CompoundSupportPlan::compile(
        &fixture.property,
        fixture.levels,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        terraced(),
    )
    .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::ThresholdBinding);
    assert_eq!(
        error.location_metres.attempted_metres(),
        fixture.levels.front.court_threshold_metres.metres()
    );
}

#[test]
fn a_cut_fill_shortfall_does_not_enlarge_the_property_reservation() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let point = fixture.property.court.centre_metres();
    let height = plan
        .elevations_at(crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap())
        .iter()
        .next()
        .unwrap();
    let error = plan
        .validate_displacement_at(
            crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap(),
            elevation(height.metres() - 6.2),
        )
        .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::CutFill);
    assert_eq!(error.boundary, SupportBoundary::GeographicSurface);
    assert!((error.violation.actual_value() - 6.2).abs() < 0.001);
    assert_eq!(error.violation.limit_value(), 6.0);
    assert!((error.violation.discrepancy_value() - 0.2).abs() < 0.001);
    assert_eq!(plan.reservation(), fixture.property.plot);
    assert!(
        plan.elevations_at(
            crate::scene_coordinates::ScenePlanPoint::from_metres(
                fixture.property.plot.centre_metres() + Vec2::splat(100.0)
            )
            .unwrap()
        )
        .iter()
        .next()
        .is_none()
    );
}

#[test]
fn gate_leaf_sweep_and_both_fixed_posts_have_level_support_above_the_front_floor() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let mesh = plan.mesh().unwrap();
    let door = fixture
        .property
        .boundary
        .gate
        .door(fixture.property.id)
        .unwrap();
    let mut points = Vec::new();
    for step in 0..=180 {
        let angle = door.open_angle_radians.radians() * step as f32 / 180.0;
        let pivot = bevy::math::Quat::from_rotation_y(angle);
        let rotation = bevy::math::Quat::from_rotation_y(door.closed_yaw_radians.radians() + angle);
        let centre = door.hinge_centre.metres()
            + pivot * (door.closed_centre.metres() - door.hinge_centre.metres());
        for x in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                points.push(
                    (centre + rotation * (door.size_metres.metres() * Vec3::new(x, 0.0, z) * 0.5))
                        .xz(),
                );
            }
        }
    }
    for side in [
        super::super::PropertySide::Left,
        super::super::PropertySide::Right,
    ] {
        let post = fixture.property.boundary.gate.post(side).unwrap();
        for x in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                points.push(
                    (post
                        .packing_cuboid(adventuresim_building_generator::ResolvedItemId(0))
                        .unwrap()
                        .centre
                        .metres()
                        + bevy::math::Quat::from_rotation_y(post.orientation.yaw_radians())
                            * (post.size_metres.metres() * Vec3::new(x, 0.0, z) * 0.5))
                        .xz(),
                );
            }
        }
    }
    for point in points {
        assert!(
            mesh.elevations_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap()
            )
            .iter()
            .any(|h| (h.metres() - plan.gate_elevation().metres()).abs() < 0.001),
            "unsupported gate swing or post at {:?}",
            point
        );
    }
}

#[test]
fn a_retaining_edge_exposes_both_levels_instead_of_inventing_one_shared_terrain_datum() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let post = fixture
        .property
        .boundary
        .gate
        .post(fixture.property.boundary.gate.hinge)
        .unwrap();
    let point = (post
        .packing_cuboid(adventuresim_building_generator::ResolvedItemId(0))
        .unwrap()
        .centre
        .metres()
        + bevy::math::Quat::from_rotation_y(post.orientation.yaw_radians())
            * Vec3::new(-post.size_metres.metres().x * 0.5, 0.0, 0.0))
    .xz();
    let heights = plan
        .mesh()
        .unwrap()
        .elevations_at(crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap())
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(heights.len(), 2);
    assert!((heights[0].metres() - plan.member_support()[0].elevation.metres()).abs() < 0.001);
    assert!((heights[1].metres() - plan.gate_elevation().metres()).abs() < 0.001);
}

#[test]
fn a_complete_bearing_rejects_an_interior_ramp_even_when_its_corners_are_level() {
    let mut fixture = Fixture::load();
    fixture.levels.front.elevation = elevation(0.0);
    fixture.levels.rear.elevation = elevation(0.0);
    fixture.levels.court = elevation(1.0);
    fixture.levels.gate = elevation(0.0);
    fixture.levels.street = elevation(0.0);
    let contact = &mut fixture.levels.front.contact;
    let x = fixture
        .property
        .plot
        .orientation()
        .world_to_local(contact.centre_metres() - fixture.property.plot.centre_metres())
        .x;
    contact
        .relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                fixture.property.plot.centre_metres()
                    + fixture
                        .property
                        .plot
                        .orientation()
                        .local_to_world(Vec2::new(x, 0.0)),
            )
            .unwrap(),
        )
        .unwrap();
    contact
        .resize(
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                bevy::math::Vec2::new(contact.dimensions_metres().x, 30.0),
            )
            .unwrap(),
        )
        .unwrap();
    let error = CompoundSupportPlan::compile(
        &fixture.property,
        fixture.levels,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        terraced(),
    )
    .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::Bearing);
    assert!(error.violation.actual_value() > 0.99);
}

#[test]
fn a_terraced_court_has_level_open_ground_and_narrow_discrete_stairs() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let flights: Vec<_> = plan.stair_flights().collect();
    assert_eq!(flights.len(), 2);
    for flight in flights {
        let riser = (flight.top_elevation_metres - flight.bottom_elevation_metres)
            / f32::from(flight.riser_count);
        assert!(riser <= 0.19);
        assert!(flight.going_metres >= 0.25);
        assert!(flight.riser_count == 8);
    }
    for x in [-4.0, 2.0] {
        for z in [3.0, 4.0, 5.0, 6.0, 7.0] {
            let point = fixture.property.plot.centre_metres()
                + fixture
                    .property
                    .plot
                    .orientation()
                    .local_to_world(Vec2::new(x, z));
            assert!(
                plan.mesh()
                    .unwrap()
                    .elevations_at(
                        crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap()
                    )
                    .iter()
                    .all(
                        |height| (height.metres() - plan.court_elevation().metres()).abs() < 0.001
                    )
            );
        }
    }
}

#[test]
fn stair_run_failure_reports_the_required_going_without_changing_floor_levels() {
    let fixture = Fixture::load();
    let treatment = CourtTreatment::Terraced(CourtStairLimits::new(
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.19)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.35)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.0)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
            .unwrap(),
    ));
    let error = CompoundSupportPlan::compile(
        &fixture.property,
        fixture.levels,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        treatment,
    )
    .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::StairGoing);
    assert_eq!(error.violation.unit(), SupportDiagnosticUnit::Metres);
    assert_eq!(
        *error.attempted_treatment,
        SupportGradingAttempt::Compound(treatment)
    );
    assert!((error.violation.actual_value() - 2.8).abs() < 0.001);
    assert!((error.violation.limit_value() - 2.060).abs() < 0.001);
    assert!((error.violation.discrepancy_value() - 0.740).abs() < 0.001);
    assert_eq!(
        error.member_building_ids,
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
}

#[test]
fn full_landing_failure_reports_the_old_floor_constraints_without_relaxing_grade() {
    let mut fixture = Fixture::load();
    fixture.levels.court = elevation(19.723_96);
    fixture.levels.rear.elevation = elevation(18.229_752);
    let error = CompoundSupportPlan::compile(
        &fixture.property,
        fixture.levels,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        terraced(),
    )
    .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::AccessGrade);
    assert_eq!(
        error.member_building_ids,
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert!((error.violation.actual_value() - 1.469_004).abs() < 0.001);
    assert!((error.violation.limit_value() - 1.339).abs() < 0.001);
    assert!((error.violation.discrepancy_value() - 0.130_004).abs() < 0.001);
    assert_eq!(error.violation.unit(), SupportDiagnosticUnit::Metres);
}

#[test]
fn insufficient_stair_width_reports_its_reserved_boundary_instead_of_clipping() {
    let fixture = Fixture::load();
    let treatment = CourtTreatment::Terraced(CourtStairLimits::new(
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.19)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.25)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(20.0)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
            .unwrap(),
    ));
    let error = CompoundSupportPlan::compile(
        &fixture.property,
        fixture.levels,
        SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        treatment,
    )
    .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::StairClearance);
    assert_eq!(error.boundary, SupportBoundary::PropertyReservation);
    assert_eq!(error.property_id, CityPropertyId(1238));
    assert_eq!(
        error.member_building_ids,
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert_eq!(error.violation.limit_value(), 0.0);
    assert!(error.violation.discrepancy_value() > 1.0);
    assert_eq!(error.violation.unit(), SupportDiagnosticUnit::Metres);
    assert_eq!(
        *error.attempted_treatment,
        SupportGradingAttempt::Compound(treatment)
    );
}
