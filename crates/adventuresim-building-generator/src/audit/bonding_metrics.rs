use super::{BuildingPlan, ResolvedSolid, Result, resolved_solid_bounds};
use crate::spatial_geometry::{Area as MeasuredArea, SignedLength};
use bevy::math::{Vec2, Vec3};

const CONTACT_TOLERANCE_METRES: f32 = 0.025;
const INTERFACE_AREA_TOLERANCE_SQUARE_METRES: f32 = 0.005;

pub(super) fn resolved_plan_overlap_area(
    left: &ResolvedSolid,
    right: &ResolvedSolid,
) -> Result<MeasuredArea> {
    let local_x = Vec2::new(
        left.yaw_radians.radians().cos(),
        -left.yaw_radians.radians().sin(),
    );
    let local_z = Vec2::new(
        left.yaw_radians.radians().sin(),
        left.yaw_radians.radians().cos(),
    );
    let delta = Vec2::new(
        right.centre.metres().x - left.centre.metres().x,
        right.centre.metres().z - left.centre.metres().z,
    );
    let right_x = Vec2::new(
        right.yaw_radians.radians().cos(),
        -right.yaw_radians.radians().sin(),
    );
    let right_z = Vec2::new(
        right.yaw_radians.radians().sin(),
        right.yaw_radians.radians().cos(),
    );
    let overlap = |axis: Vec2, left_extent: f32| {
        let right_extent = right.size.metres().x * 0.5 * right_x.dot(axis).abs()
            + right.size.metres().z * 0.5 * right_z.dot(axis).abs();
        (left_extent + right_extent - delta.dot(axis).abs()).max(0.0)
    };
    Ok(MeasuredArea::from_square_metres(
        overlap(local_x, left.size.metres().x * 0.5) * overlap(local_z, left.size.metres().z * 0.5),
    )?)
}

struct BondedInterfaceMetrics {
    // A tolerated gap can put the measured lower plane above the upper plane.
    // These are sampled planes, not an ordered physical bounds interval.
    contact_min: crate::spatial_geometry::Position<crate::Architectural>,
    contact_max: crate::spatial_geometry::Position<crate::Architectural>,
    area: MeasuredArea,
    penetration: SignedLength,
}

impl BondedInterfaceMetrics {
    /// Classify contact from signed overlaps of architectural axis-aligned
    /// bounds.
    /// One axis permits a gap up to `CONTACT_TOLERANCE_METRES`; the other two
    /// require positive overlap. Native metre arithmetic orders these overlaps.
    fn between(a: &ResolvedSolid, b: &ResolvedSolid) -> Result<Option<Self>> {
        let (a_min, a_max) = resolved_solid_bounds(a);
        let (b_min, b_max) = resolved_solid_bounds(b);
        let signed = a_max.min(b_max) - a_min.max(b_min);
        let mut axes = [(signed.x, 0_usize), (signed.y, 1), (signed.z, 2)];
        axes.sort_by(|left, right| left.0.total_cmp(&right.0));
        if axes[0].0 < -CONTACT_TOLERANCE_METRES || axes[1].0 <= 0.0 || axes[2].0 <= 0.0 {
            return Ok(None);
        }
        let contact_min = a_min.max(b_min);
        let mut contact_max = a_max.min(b_max);
        if axes[0].0 < 0.0 {
            let axis = axes[0].1;
            let midpoint = (contact_min[axis] + contact_max[axis]) * 0.5;
            contact_max[axis] = midpoint;
        }
        Ok(Some(Self {
            contact_min: crate::spatial_geometry::Position::from_metres(contact_min)?,
            contact_max: crate::spatial_geometry::Position::from_metres(contact_max)?,
            area: MeasuredArea::from_square_metres(axes[1].0 * axes[2].0)?,
            penetration: SignedLength::from_metres(axes[0].0.max(0.0))?,
        }))
    }

    fn fits(&self, bond: &crate::JunctionBond) -> bool {
        let contact_min = self.contact_min.metres();
        let contact_max = self.contact_max.metres();
        // Order sampled planes only for containment, including tolerated gaps.
        contact_min
            .min(contact_max)
            .cmpge(bond.bounds.min().metres() - Vec3::splat(CONTACT_TOLERANCE_METRES))
            .all()
            && contact_min
                .max(contact_max)
                .cmple(bond.bounds.max().metres() + Vec3::splat(CONTACT_TOLERANCE_METRES))
                .all()
            && self.area.square_metres() + INTERFACE_AREA_TOLERANCE_SQUARE_METRES
                >= bond.minimum_interface_area_square_metres
            && self.penetration.metres()
                <= bond.maximum_penetration_metres + CONTACT_TOLERANCE_METRES
    }
}
pub(super) fn bonded_geometry_matches(
    plan: &BuildingPlan,
    bond: &crate::JunctionBond,
) -> Result<bool> {
    crate::geometry_index::try_any(
        plan.resolved_geometry
            .solids
            .iter()
            .filter(|a| a.owner == bond.owners[0]),
        |a| {
            crate::geometry_index::try_any(
                plan.resolved_geometry
                    .solids
                    .iter()
                    .filter(|b| b.owner == bond.owners[1]),
                |b| {
                    Ok::<_, crate::GenerationError>(
                        BondedInterfaceMetrics::between(a, b)?
                            .is_some_and(|metrics| metrics.fits(bond)),
                    )
                },
            )
        },
    )
}

#[cfg(test)]
#[path = "interface_contract_tests.rs"]
mod tests;
