//! Inner-to-outer geometry dependencies from authored equipment placements.
//! Channel order supplies the ordinary rule; explicit local precedence can
//! reverse it, as when a boot encloses padded hose.
use std::cmp::Ordering;

use anyhow::{Context, Result, ensure};

use crate::item_catalog_schema::{EquipmentLocation, EquipmentPlacement};

/// Acyclic fitting order and the lower surfaces each selection depends on.
#[derive(Debug)]
pub struct LayerPlan {
    order: Vec<usize>,
    supports: Vec<Vec<usize>>,
}

impl LayerPlan {
    /// Indices refer to `placements`; unrelated items retain input order.
    pub fn new(placements: &[&EquipmentPlacement]) -> Result<Self> {
        let mut supports = vec![Vec::new(); placements.len()];
        for a in 0..placements.len() {
            for b in a + 1..placements.len() {
                match precedence(placements[a], placements[b])? {
                    Some(Ordering::Greater) => supports[a].push(b),
                    Some(Ordering::Less) => supports[b].push(a),
                    _ => {}
                }
            }
        }
        let mut order = Vec::with_capacity(placements.len());
        let mut visited = vec![false; placements.len()];
        while order.len() < placements.len() {
            let next = (0..placements.len())
                .find(|&index| {
                    !visited[index] && supports[index].iter().all(|&inner| visited[inner])
                })
                .context("equipment layer dependencies contain a cycle")?;
            visited[next] = true;
            order.push(next);
        }
        Ok(Self { order, supports })
    }

    pub fn order(&self) -> &[usize] {
        &self.order
    }

    /// Actual support surfaces, as opposed to their transitive dependencies.
    pub fn supports(&self, outer: usize) -> impl Iterator<Item = usize> + '_ {
        self.order
            .iter()
            .copied()
            .filter(move |i| self.supports[outer].contains(i))
    }

    /// Everything needed to reconstruct this fit after cache eviction. A
    /// lower garment's own dependencies matter even outside the outer region.
    pub fn ancestors(&self, outer: usize) -> Vec<usize> {
        let mut needed = vec![false; self.supports.len()];
        needed[outer] = true;
        for &index in self.order.iter().rev() {
            if needed[index] {
                for &inner in &self.supports[index] {
                    needed[inner] = true;
                }
            }
        }
        self.order
            .iter()
            .copied()
            .filter(|&i| i != outer && needed[i])
            .collect()
    }
}

fn rank(placement: &EquipmentPlacement) -> Result<(u8, u16)> {
    let channel = placement
        .outermost_channel()
        .context("equipment placement has no layer channel")?;
    let order = placement
        .occupancy
        .iter()
        .filter(|entry| entry.channel == channel)
        .map(|entry| entry.order)
        .chain(
            placement
                .parents
                .iter()
                .filter(|entry| entry.channel == channel)
                .map(|entry| entry.order),
        )
        .max()
        .unwrap_or(0);
    Ok((channel.order(), order))
}

/// Greater means `a` lies geometrically outside `b`.
fn precedence(a: &EquipmentPlacement, b: &EquipmentPlacement) -> Result<Option<Ordering>> {
    let a_over_b = explicitly_over(a, b);
    let b_over_a = explicitly_over(b, a);
    ensure!(
        !(a_over_b && b_over_a),
        "equipment placements declare contradictory layer precedence"
    );
    if a_over_b {
        return Ok(Some(Ordering::Greater));
    }
    if b_over_a {
        return Ok(Some(Ordering::Less));
    }
    Ok(match rank(a)?.cmp(&rank(b)?) {
        Ordering::Greater if overlaps_as_outer(a, b) => Some(Ordering::Greater),
        Ordering::Less if overlaps_as_outer(b, a) => Some(Ordering::Less),
        _ => None,
    })
}

fn explicitly_over(outer: &EquipmentPlacement, inner: &EquipmentPlacement) -> bool {
    outer.layers_over.iter().any(|precedence| {
        inner.occupancy.iter().any(|entry| {
            entry.location == precedence.location
                && entry.channel == precedence.channel
                && entry.order == precedence.order
        })
    })
}

fn overlaps_as_outer(outer: &EquipmentPlacement, inner: &EquipmentPlacement) -> bool {
    outer.occupancy.iter().any(|outer| {
        inner
            .occupancy
            .iter()
            .any(|inner| supports_location(outer.location, inner.location))
    })
}

fn supports_location(outer: EquipmentLocation, inner: EquipmentLocation) -> bool {
    use EquipmentLocation as L;
    outer == inner
        || matches!(
            (outer, inner),
            (L::LeftShoulder, L::LeftArm | L::Chest | L::Back | L::Neck)
                | (L::RightShoulder, L::RightArm | L::Chest | L::Back | L::Neck)
                | (L::LeftArm, L::LeftHand)
                | (L::RightArm, L::RightHand)
                | (L::LeftLeg, L::LeftFoot | L::Stomach)
                | (L::RightLeg, L::RightFoot | L::Stomach)
                | (L::LeftFoot, L::LeftLeg)
                | (L::RightFoot, L::RightLeg)
                | (L::Stomach, L::Chest | L::LeftLeg | L::RightLeg)
                | (L::Head | L::Face, L::Neck | L::Chest)
                | (L::Neck, L::Chest | L::Back)
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_catalog_schema::{
        EquipmentChannel, EquipmentLayerPrecedence, OccupancyRequirement,
    };

    fn placement(location: EquipmentLocation, channel: EquipmentChannel) -> EquipmentPlacement {
        EquipmentPlacement {
            id: "worn".into(),
            occupancy: vec![OccupancyRequirement {
                location,
                channel,
                fit_zone: None,
                order: 0,
            }],
            parents: vec![],
            layers_over: vec![],
            protection: vec![],
            surface: vec![],
        }
    }

    #[test]
    fn unrelated_limbs_do_not_invalidate_a_fit() {
        use EquipmentChannel as C;
        use EquipmentLocation as L;
        let plates = placement(L::LeftArm, C::RigidArmor);
        let shirt = placement(L::LeftArm, C::BaseClothing);
        let other = placement(L::RightArm, C::BaseClothing);
        let plan = LayerPlan::new(&[&plates, &shirt, &other]).unwrap();
        assert_eq!(plan.ancestors(0), [1]);
        assert_eq!(plan.supports(0).collect::<Vec<_>>(), [1]);
        assert!(plan.ancestors(2).is_empty());
    }

    #[test]
    fn support_keys_include_transitive_dependencies() {
        use EquipmentChannel as C;
        use EquipmentLocation as L;
        let collar = placement(L::Neck, C::RigidArmor);
        let shirt = placement(L::Chest, C::Padding);
        let lining = placement(L::Chest, C::BaseClothing);
        let plan = LayerPlan::new(&[&collar, &shirt, &lining]).unwrap();
        assert_eq!(plan.order(), [2, 1, 0]);
        assert_eq!(plan.ancestors(0), [2, 1]);
    }

    #[test]
    fn explicit_precedence_overrides_channels_and_cycles_fail() {
        use EquipmentChannel as C;
        use EquipmentLocation as L;
        let mut boot = placement(L::LeftFoot, C::BaseClothing);
        let mut hose = placement(L::LeftLeg, C::Padding);
        boot.layers_over.push(EquipmentLayerPrecedence {
            location: L::LeftLeg,
            channel: C::Padding,
            order: 0,
        });
        let plan = LayerPlan::new(&[&boot, &hose]).unwrap();
        assert_eq!(plan.order(), [1, 0]);
        hose.layers_over.push(EquipmentLayerPrecedence {
            location: L::LeftFoot,
            channel: C::BaseClothing,
            order: 0,
        });
        assert!(LayerPlan::new(&[&boot, &hose]).is_err());
    }

    #[test]
    fn nonlocal_cycles_are_rejected_before_fitting() {
        use EquipmentChannel as C;
        use EquipmentLocation as L;
        let mut a = placement(L::LeftArm, C::BaseClothing);
        let b = placement(L::LeftArm, C::Padding);
        let c = placement(L::LeftArm, C::FlexibleArmor);
        a.layers_over.push(EquipmentLayerPrecedence {
            location: L::LeftArm,
            channel: C::FlexibleArmor,
            order: 0,
        });
        assert!(
            LayerPlan::new(&[&a, &b, &c])
                .unwrap_err()
                .to_string()
                .contains("cycle")
        );
    }
}
