//! Place carry sockets on the generated garment's actual surface.
use super::*;
use adventuresim_core::item_catalog::{EquipmentLocation, definition};
use fabelgeist_rig::{RigJointName, RigJointOrdinal};

pub(super) fn garment_sockets(
    body: &RuntimeBody,
    garment: &GeneratedArmor,
    item: &str,
) -> BTreeMap<String, Transform> {
    let Some(equipment) = definition(item).and_then(|item| item.equipment.as_ref()) else {
        return BTreeMap::new();
    };
    let Some(pelvis) = RigJointName::ROOT.index_in(&body.joint_names) else {
        return BTreeMap::new();
    };
    let state = body.global_joint_states[usize::from(pelvis)];
    let bind = GlobalTransform::from(Transform {
        translation: Vec3::new(state[0], state[1], state[2]),
        rotation: Quat::from_xyzw(state[3], state[4], state[5], state[6]),
        scale: Vec3::splat(state[7]),
    });
    let points = garment
        .indices
        .iter()
        .map(|index| Vec3::from_array(garment.positions[*index as usize]))
        .collect::<Vec<_>>();
    if points.is_empty() {
        return BTreeMap::new();
    }
    let lower = points.iter().fold(f32::INFINITY, |y, point| y.min(point.y));
    let upper = points
        .iter()
        .fold(f32::NEG_INFINITY, |y, point| y.max(point.y));
    let waist = (lower + upper) * 0.5;
    let left = RigJointName::L_UPLEG.index_in(&body.joint_names).map_or(
        1.0,
        |index: RigJointOrdinal| -> f32 {
            (body.global_joint_states[usize::from(index)][0] - state[0]).signum()
        },
    );
    equipment
        .attachment_points
        .iter()
        .filter_map(|socket| {
            let direction = match socket.locations.first()? {
                EquipmentLocation::LeftBelt => Vec3::X * left,
                EquipmentLocation::RightBelt => -Vec3::X * left,
                EquipmentLocation::FrontBelt => Vec3::Z,
                EquipmentLocation::BackBelt => -Vec3::Z,
                _ => return None,
            };
            let position = surface_socket(&points, direction, waist)?;
            let tangent = Vec3::from_array(socket.tangent_direction?).try_normalize()?;
            let world = GlobalTransform::from(
                Transform::from_translation(position)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, tangent)),
            );
            Some((socket.id.clone(), world.reparented_to(&bind)))
        })
        .collect()
}

fn surface_socket(points: &[Vec3], direction: Vec3, height: f32) -> Option<Vec3> {
    // Prefer the belt's midline rather than the upper/lower garment cut edge.
    const HEIGHT_PREFERENCE: f32 = 4.0;
    points.iter().copied().max_by(|a, b| {
        let score =
            |point: Vec3| point.dot(direction) - (point.y - height).abs() * HEIGHT_PREFERENCE;
        score(*a).total_cmp(&score(*b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opposite_carry_points_follow_garment_width_and_avoid_cut_edges() {
        let points = [
            Vec3::new(-0.2, 1.0, 0.0),
            Vec3::new(0.2, 1.0, 0.0),
            Vec3::new(0.22, 1.1, 0.0),
        ];
        assert_eq!(surface_socket(&points, Vec3::X, 1.0), Some(points[1]));
        assert_eq!(surface_socket(&points, -Vec3::X, 1.0), Some(points[0]));
    }
}
