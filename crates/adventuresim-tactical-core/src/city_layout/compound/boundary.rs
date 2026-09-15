//! Fixed enclosure and operable gate geometry share one descriptor.
use super::*;
use adventuresim_building_generator::{DoorSpec, OpeningAssemblyId, ResolvedItemId};
use bevy::math::Vec3;

const BOUNDARY_GATE_OPENING_DOMAIN: u64 = 0xc300_0000_0000_0000;
const GATE_LEAF_THICKNESS_METRES: f32 = 0.055;
const GATE_GROUND_GAP_METRES: f32 = 0.05;
const GATE_POST_WIDTH_METRES: f32 = 0.3;
const GATE_POST_HEAD_METRES: f32 = 0.15;
const WALL_CAP_HEIGHT_METRES: f32 = 0.1;
const WALL_CAP_OVERHANG_METRES: f32 = 0.025;
const GATE_OPEN_ANGLE_RADIANS: f32 = core::f32::consts::FRAC_PI_2;
const GATE_HINGE_CLEARANCE_METRES: f32 = 0.025;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CityBoundaryMaterial {
    Masonry,
    Timber,
    Iron,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityBoundaryMember {
    pub centre_metres: Vec3,
    pub size_metres: Vec3,
    pub yaw_radians: f32,
    pub material: CityBoundaryMaterial,
}

impl CityBoundary {
    pub fn fixed_members(&self) -> Vec<CityBoundaryMember> {
        let mut members = Vec::new();
        for wall in &self.walls {
            let centre = (wall.start_metres + wall.end_metres) * 0.5;
            let length = wall.start_metres.distance(wall.end_metres);
            let orientation =
                BuildingOrientation::from_frontage_tangent(wall.end_metres - wall.start_metres)
                    .expect("validated nonzero boundary segment");
            for (height, thickness, elevation) in [
                (
                    wall.height_metres,
                    wall.thickness_metres,
                    wall.height_metres * 0.5,
                ),
                (
                    WALL_CAP_HEIGHT_METRES,
                    wall.thickness_metres + WALL_CAP_OVERHANG_METRES * 2.0,
                    wall.height_metres + WALL_CAP_HEIGHT_METRES * 0.5,
                ),
            ] {
                members.push(CityBoundaryMember {
                    centre_metres: Vec3::new(centre.x, elevation, centre.y),
                    size_metres: Vec3::new(length, height, thickness),
                    yaw_radians: orientation.yaw_radians(),
                    material: CityBoundaryMaterial::Masonry,
                });
            }
        }
        for side in [-1.0, 1.0] {
            let centre = self.gate.centre_metres
                + self.gate.orientation.local_to_world(
                    Vec2::X * side * (self.gate.width_metres + GATE_POST_WIDTH_METRES) * 0.5,
                );
            let height = self.gate.height_metres + GATE_POST_HEAD_METRES;
            members.push(CityBoundaryMember {
                centre_metres: Vec3::new(centre.x, height * 0.5, centre.y),
                size_metres: Vec3::new(GATE_POST_WIDTH_METRES, height, GATE_POST_WIDTH_METRES),
                yaw_radians: self.gate.orientation.yaw_radians(),
                material: CityBoundaryMaterial::Masonry,
            });
        }
        members
    }
}

impl CityGate {
    pub fn owns_opening(opening: OpeningAssemblyId) -> bool {
        opening.0 & BOUNDARY_GATE_OPENING_DOMAIN == BOUNDARY_GATE_OPENING_DOMAIN
    }

    pub fn door(self, property: CityPropertyId) -> DoorSpec {
        let opening = OpeningAssemblyId(BOUNDARY_GATE_OPENING_DOMAIN | property.0);
        let tangent = self.orientation.local_to_world(Vec2::X);
        let outward = self.orientation.local_to_world(-Vec2::Y);
        let leaf_centre = self.centre_metres
            - outward
                * ((GATE_POST_WIDTH_METRES + GATE_LEAF_THICKNESS_METRES) * 0.5
                    + GATE_HINGE_CLEARANCE_METRES);
        let closed_centre = Vec3::new(
            leaf_centre.x,
            self.height_metres * 0.5 + GATE_GROUND_GAP_METRES,
            leaf_centre.y,
        );
        DoorSpec {
            opening,
            source: ResolvedItemId(opening.0),
            closed_centre,
            hinge_centre: closed_centre
                - Vec3::new(tangent.x, 0.0, tangent.y) * self.width_metres * 0.5,
            size_metres: Vec3::new(
                self.width_metres,
                self.height_metres,
                GATE_LEAF_THICKNESS_METRES,
            ),
            closed_yaw_radians: self.orientation.yaw_radians(),
            tangent,
            outward,
            open_angle_radians: -GATE_OPEN_ANGLE_RADIANS,
        }
    }
}
