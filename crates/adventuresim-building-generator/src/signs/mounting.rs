//! A bracket foot must bear on a complete, planar structural face.
use super::*;
use crate::{BuildingPlan, ResolvedItemId, ResolvedSolid, ResolvedSolidShape, SolidRole};

pub const MOUNTING_PLATE_THICKNESS_METRES: f32 = 0.04;
const PLATE_LONG_EDGE_METRES: f32 = 0.32;
const PLATE_SHORT_EDGE_METRES: f32 = 0.09;
const SUPPORT_EDGE_MARGIN_METRES: f32 = 0.015;
const MIN_HANGER_LENGTH_METRES: f32 = 0.12;
const MAX_BRACKET_RISE_METRES: f32 = 0.75;
const CONTACT_TOLERANCE_METRES: f32 = 0.001;
const MAX_ANCHOR_RECESS_METRES: f32 = 0.8;

#[derive(Clone, Copy, Debug)]
pub struct SignMounting {
    /// Centre of the plate's back face, in contact with its structural support.
    pub contact: Vec3,
    pub size: Vec2,
    pub support: ResolvedItemId,
}

impl SignMounting {
    pub(super) fn find(
        plan: &BuildingPlan,
        attachment: Vec3,
        outward: Vec3,
        panel: Vec2,
    ) -> Option<Self> {
        let tangent = Vec3::Y.cross(outward);
        let minimum_y = attachment.y + panel.y * 0.5 + MIN_HANGER_LENGTH_METRES;
        let mut candidates = Vec::new();
        for solid in &plan.resolved_geometry.solids {
            if !structural_face(plan, solid, outward) {
                continue;
            }
            let extent = super::site::solid_extent(solid);
            let face = solid.centre.dot(outward) + extent.dot(outward.abs());
            if (face - attachment.dot(outward)).abs() > MAX_ANCHOR_RECESS_METRES {
                continue;
            }
            for size in [
                Vec2::new(PLATE_SHORT_EDGE_METRES, PLATE_LONG_EDGE_METRES),
                Vec2::new(PLATE_LONG_EDGE_METRES, PLATE_SHORT_EDGE_METRES),
            ] {
                if (attachment - solid.centre).dot(tangent).abs()
                    + size.x * 0.5
                    + SUPPORT_EDGE_MARGIN_METRES
                    > extent.dot(tangent.abs())
                {
                    continue;
                }
                let lower = (solid.centre.y - extent.y + size.y * 0.5 + SUPPORT_EDGE_MARGIN_METRES)
                    .max(minimum_y);
                let upper = (solid.centre.y + extent.y - size.y * 0.5 - SUPPORT_EDGE_MARGIN_METRES)
                    .min(minimum_y + MAX_BRACKET_RISE_METRES);
                if lower > upper {
                    continue;
                }
                let contact = attachment
                    + Vec3::Y * (lower - attachment.y)
                    + outward * (face - attachment.dot(outward));
                candidates.push(Self {
                    contact,
                    size,
                    support: solid.id,
                });
            }
        }
        candidates
            .into_iter()
            .min_by(|a, b| a.contact.y.total_cmp(&b.contact.y))
    }

    /// Verify the whole back plate, including its edges, against the actual support solid.
    pub fn is_supported(self, plan: &BuildingPlan, outward: Vec3) -> bool {
        let Some(solid) = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.id == self.support)
        else {
            return false;
        };
        if !structural_face(plan, solid, outward) {
            return false;
        }
        let extent = super::site::solid_extent(solid);
        let tangent = Vec3::Y.cross(outward);
        let offset = self.contact - solid.centre;
        (offset.dot(outward) - extent.dot(outward.abs())).abs() <= CONTACT_TOLERANCE_METRES
            && offset.dot(tangent).abs() + self.size.x * 0.5 + SUPPORT_EDGE_MARGIN_METRES
                <= extent.dot(tangent.abs()) + CONTACT_TOLERANCE_METRES
            && offset.y.abs() + self.size.y * 0.5 + SUPPORT_EDGE_MARGIN_METRES
                <= extent.y + CONTACT_TOLERANCE_METRES
    }
}

fn structural_face(plan: &BuildingPlan, solid: &ResolvedSolid, outward: Vec3) -> bool {
    let rotation = super::site::solid_rotation(solid);
    let local_normal = rotation.inverse() * outward;
    matches!(solid.shape, ResolvedSolidShape::Cuboid)
        && [Vec3::X, Vec3::Y, Vec3::Z]
            .into_iter()
            .all(|axis| (rotation * axis).abs().max_element() > 1.0 - CONTACT_TOLERANCE_METRES)
        && local_normal.abs().max_element() > 1.0 - CONTACT_TOLERANCE_METRES
        && (matches!(
            solid.role,
            SolidRole::LoadBearing
                | SolidRole::WallHost
                | SolidRole::FramePost
                | SolidRole::FramePlate
                | SolidRole::FrameRail
                | SolidRole::FrameJettyBeam
                | SolidRole::FrameMember
                | SolidRole::OpeningHead
        ) || plan.workplace.as_ref().is_some_and(|work| {
            work.parts.iter().any(|part| {
                part.solid == solid.id
                    && matches!(
                        part.feature,
                        crate::WorkplaceFeature::Wall
                            | crate::WorkplaceFeature::Post
                            | crate::WorkplaceFeature::Beam
                    )
            })
        }))
}
