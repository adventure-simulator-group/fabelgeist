//! A local jetty connection and its actual masonry bearing solids.
use super::*;
use adventuresim_building_generator::{TimberMemberRole as Role, WallMaterialClass};

const CONTACT_TOLERANCE_METRES: f32 = 0.02;
const LOCAL_MEMBER_REACH_METRES: f32 = 2.0;
pub(super) const OBSERVER_EYE_HEIGHT_METRES: f32 = 1.6;

pub(super) fn item_ids(plan: &BuildingPlan) -> Vec<u64> {
    let Some(frame) = &plan.timber_frame else {
        return Vec::new();
    };
    let Some(bracket) = frame
        .members
        .iter()
        .find(|member| member.role == Role::Knagge)
    else {
        return Vec::new();
    };
    let mut ids = vec![bracket.solid.0];
    for member in &frame.members {
        let near_tip = member.start.distance(bracket.end) <= CONTACT_TOLERANCE_METRES
            || member.end.distance(bracket.end) <= CONTACT_TOLERANCE_METRES;
        let upper_sill = member.role == Role::Sill
            && (member.start.y - plan.storey_height_metres).abs() <= CONTACT_TOLERANCE_METRES
            && member.start.distance(member.end) <= LOCAL_MEMBER_REACH_METRES
            && (member.start.xz().distance(bracket.end.xz()) <= CONTACT_TOLERANCE_METRES
                || member.end.xz().distance(bracket.end.xz()) <= CONTACT_TOLERANCE_METRES);
        if (near_tip
            && matches!(
                member.role,
                Role::JettyBeam | Role::PrimaryPost | Role::Knagge
            ))
            || upper_sill
        {
            ids.push(member.solid.0);
        }
    }
    let interface = plan
        .resolved_geometry
        .support_interfaces
        .iter()
        .find(|interface| interface.id == bracket.support_interfaces[0])
        .expect("jetty bracket has a bearing interface");
    for wall in &plan.wall_assemblies {
        if wall.storey_level != 0 || wall.material != WallMaterialClass::CivilianMasonry {
            continue;
        }
        for solid in plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|solid| wall.host_solids.contains(&solid.id))
        {
            let half = solid.size * 0.5 + Vec3::splat(CONTACT_TOLERANCE_METRES);
            if interface.bounds.max.cmpge(solid.centre - half).all()
                && interface.bounds.min.cmple(solid.centre + half).all()
            {
                ids.push(solid.id.0);
            }
        }
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}
