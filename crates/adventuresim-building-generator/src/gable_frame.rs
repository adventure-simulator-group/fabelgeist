//! Exposed end trusses and their recessed gable infill share one physical frame.
use bevy::math::Vec3;

use crate::{
    RoofEnclosureFace, RoofMaterial, TimberFrameAssembly, TimberFrameMember, TimberMemberRole,
};

const FRAME_PLANE_TOLERANCE_METRES: f32 = 0.001;
const GABLE_WALL_BUILDUP_METRES: f32 = 0.40;

pub(crate) fn normal(face: &RoofEnclosureFace) -> Vec3 {
    (face.polygon[1] - face.polygon[0])
        .cross(face.polygon[2] - face.polygon[0])
        .normalize_or_zero()
}

/// Select the actual exterior truss, never an interior frame projected outward.
pub(crate) fn members<'a>(
    face: &RoofEnclosureFace,
    frame: &'a TimberFrameAssembly,
) -> Vec<&'a TimberFrameMember> {
    if face.material != RoofMaterial::TimberInfill || face.polygon.len() < 3 {
        return Vec::new();
    }
    let outward = normal(face);
    let plane = face.polygon[0].dot(outward);
    let base = face
        .polygon
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min);
    let tie = frame
        .members
        .iter()
        .filter(|member| {
            member.role == TimberMemberRole::GableTie
                && (member.start.y - base).abs() < FRAME_PLANE_TOLERANCE_METRES
                && (member.end.y - base).abs() < FRAME_PLANE_TOLERANCE_METRES
                && (member.start - member.end).dot(outward).abs() < FRAME_PLANE_TOLERANCE_METRES
                && (member.start.dot(outward) - plane).abs() < GABLE_WALL_BUILDUP_METRES
        })
        .min_by(|a, b| {
            (a.start.dot(outward) - plane)
                .abs()
                .total_cmp(&(b.start.dot(outward) - plane).abs())
        });
    let Some(tie) = tie else {
        return Vec::new();
    };
    let frame_plane = tie.start.dot(outward);
    frame
        .members
        .iter()
        .filter(|member| {
            matches!(
                member.role,
                TimberMemberRole::GableTie
                    | TimberMemberRole::GablePost
                    | TimberMemberRole::Rafter
                    | TimberMemberRole::Collar
                    | TimberMemberRole::Rail
            ) && (member.start.dot(outward) - frame_plane).abs() < FRAME_PLANE_TOLERANCE_METRES
                && (member.end.dot(outward) - frame_plane).abs() < FRAME_PLANE_TOLERANCE_METRES
                && member.start.y >= base - FRAME_PLANE_TOLERANCE_METRES
                && member.end.y >= base - FRAME_PLANE_TOLERANCE_METRES
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, generate};

    #[test]
    fn civilian_gable_infill_leaves_its_real_truss_faces_exposed() {
        for archetype in [
            BuildingArchetype::TownHouse,
            BuildingArchetype::FachwerkCottage,
            BuildingArchetype::FachwerkMerchantHouse,
            BuildingArchetype::StorageRange,
        ] {
            let plan = generate(&BuildingProgram::fixture(archetype, 42)).unwrap();
            let frame = plan.timber_frame.as_ref().unwrap();
            let mut checked = 0;
            for roof in plan.roof_assemblies.iter().filter(|r| r.parent.is_none()) {
                for face in &roof.enclosure_faces {
                    let outward = normal(face);
                    for member in members(face, frame) {
                        let reveal = member.start.dot(outward)
                            + member.section_metres.min_element() * 0.5
                            - face.polygon[0].dot(outward);
                        assert!((0.007..=0.05).contains(&reveal), "{archetype:?}: {reveal}");
                        checked += 1;
                    }
                }
            }
            assert!(checked >= 10, "missing gable trusses for {archetype:?}");
        }
    }
}
