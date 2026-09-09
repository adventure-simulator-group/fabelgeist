use super::*;
use crate::{BuildingPlan, OpeningUse, WallAssemblyId};
use bevy::math::Mat3;

const PANEL_WIDTH_METRES: f32 = 1.35;
const PANEL_HEIGHT_METRES: f32 = 0.62;
const MIN_PANEL_HEIGHT_METRES: f32 = 0.28;
const ENTRANCE_HEAD_GAP_METRES: f32 = 0.10;
const BRACKET_HEADROOM_METRES: f32 = 0.20;
const FACADE_OFFSET_METRES: f32 = 0.12;
const MAX_FACADE_PROJECTION_METRES: f32 = 0.8;
pub(super) const PANEL_THICKNESS_METRES: f32 = 0.06;
#[cfg(feature = "sign-render")]
pub(super) const PAINT_OFFSET_METRES: f32 = PANEL_THICKNESS_METRES * 0.5 + 0.001;

#[derive(Clone, Copy, Debug)]
pub struct SignSite {
    pub wall: WallAssemblyId,
    /// Exterior wall face directly above a public entrance.
    pub attachment: Vec3,
    pub outward: Vec3,
    pub panel_size: Vec2,
    pub mounting: SignMounting,
}

impl SignSite {
    pub fn for_plan(plan: &BuildingPlan) -> Option<Self> {
        let (point, outward, head, wall_id) = if let Some(work) = &plan.workplace {
            let passage = work.passages.first()?;
            let point = Vec2::new((passage.min.x + passage.max.x) * 0.5, 0.0);
            let wall = plan.wall_assemblies.iter().find(|wall| {
                wall.frame.outward == Vec2::NEG_Y
                    && wall.base_elevation_metres > 0.0
                    && (wall.frame.origin.x - point.x).abs() < 0.1
            })?;
            (
                point + wall.frame.outward * wall.thickness_metres * 0.5,
                wall.frame.outward,
                wall.base_elevation_metres,
                wall.id,
            )
        } else {
            let opening = plan
                .opening_assemblies
                .iter()
                .filter(|opening| {
                    matches!(opening.use_kind, OpeningUse::Door | OpeningUse::Gate)
                        && opening.sill_elevation_metres < 0.5
                        && plan.wall_assemblies.iter().any(|wall| {
                            wall.id == opening.host_wall
                                && wall.frame.outside_room.is_none()
                                && wall.storey_level == 0
                        })
                })
                .min_by(|a, b| a.frame.origin.y.total_cmp(&b.frame.origin.y))?;
            let wall = plan
                .wall_assemblies
                .iter()
                .find(|wall| wall.id == opening.host_wall)?;
            (
                opening.frame.origin + wall.frame.outward * wall.thickness_metres * 0.5,
                wall.frame.outward,
                opening.sill_elevation_metres + opening.profile.clear_height_metres(),
                wall.id,
            )
        };
        let bottom = (head + ENTRANCE_HEAD_GAP_METRES).max(SIGN_PEDESTRIAN_CLEARANCE_METRES);
        let height =
            (plan.storey_height_metres - bottom - BRACKET_HEADROOM_METRES).min(PANEL_HEIGHT_METRES);
        if height < MIN_PANEL_HEIGHT_METRES {
            return None;
        }
        let outward = Vec3::new(outward.x, 0.0, outward.y);
        let panel_size = Vec2::new(PANEL_WIDTH_METRES, height);
        let attachment = facade_attachment(
            plan,
            Vec3::new(point.x, bottom + height * 0.5, point.y),
            outward,
            panel_size,
        );
        let mounting = SignMounting::find(plan, attachment, outward, panel_size)?;
        let attachment =
            attachment + outward * (mounting.contact - attachment).dot(outward).max(0.0);
        Some(Self {
            wall: wall_id,
            attachment,
            outward,
            panel_size,
            mounting,
        })
    }

    pub fn board(self, mount: SignMount) -> SignBoard {
        let right = match mount {
            SignMount::Wall => Vec3::Y.cross(self.outward),
            SignMount::Projecting => self.outward,
        };
        let projection = match mount {
            SignMount::Wall => FACADE_OFFSET_METRES,
            SignMount::Projecting => self.panel_size.x * 0.5 + FACADE_OFFSET_METRES,
        };
        SignBoard {
            centre: self.attachment + self.outward * projection,
            rotation: Quat::from_mat3(&Mat3::from_cols(right, Vec3::Y, right.cross(Vec3::Y))),
            size: self.panel_size,
        }
    }

    /// Physical overlap checks apply to the panel; its metal mounting foot touches the host wall.
    pub fn supports(self, plan: &BuildingPlan, mount: SignMount) -> bool {
        let board = self.board(mount);
        let half = Vec3::new(board.size.x, board.size.y, PANEL_THICKNESS_METRES) * 0.5;
        let extent = (board.rotation * Vec3::X).abs() * half.x
            + Vec3::Y * half.y
            + (board.rotation * Vec3::Z).abs() * half.z;
        let min = board.centre - extent;
        let max = board.centre + extent;
        if min.y < SIGN_PEDESTRIAN_CLEARANCE_METRES
            || !self.mounting.is_supported(plan, self.outward)
        {
            return false;
        }
        !plan.resolved_geometry.solids.iter().any(|solid| {
            let extent = solid_extent(solid);
            (max.min(solid.centre + extent) - min.max(solid.centre - extent)).min_element() > 0.005
        })
    }
}

/// Keep the panel clear of visible timberwork and masonry.
fn facade_attachment(
    plan: &BuildingPlan,
    attachment: Vec3,
    outward: Vec3,
    panel_size: Vec2,
) -> Vec3 {
    let tangent = Vec3::Y.cross(outward);
    let mut projection = 0.0_f32;
    for solid in &plan.resolved_geometry.solids {
        let extent = solid_extent(solid);
        let offset = solid.centre - attachment;
        let side = offset.dot(tangent).abs();
        if side > panel_size.x * 0.5 + extent.dot(tangent.abs())
            || offset.y.abs() > panel_size.y * 0.5 + BRACKET_HEADROOM_METRES + extent.y
        {
            continue;
        }
        let face = offset.dot(outward) + extent.dot(outward.abs());
        if (0.0..=MAX_FACADE_PROJECTION_METRES).contains(&face) {
            projection = projection.max(face);
        }
    }
    attachment + outward * projection
}

pub(super) fn solid_extent(solid: &crate::ResolvedSolid) -> Vec3 {
    let rotation = solid_rotation(solid);
    let half = solid.size * 0.5;
    (rotation * Vec3::X).abs() * half.x
        + (rotation * Vec3::Y).abs() * half.y
        + (rotation * Vec3::Z).abs() * half.z
}

pub(super) fn solid_rotation(solid: &crate::ResolvedSolid) -> Quat {
    Quat::from_rotation_y(solid.yaw_radians)
        * Quat::from_rotation_x(solid.crossfall_radians)
        * Quat::from_rotation_z(solid.longfall_radians)
}
