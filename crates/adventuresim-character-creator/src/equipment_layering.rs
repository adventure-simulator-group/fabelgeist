//! Dependency ordering and support surfaces for a selected equipment outfit.

use super::*;
use adventuresim_character_creator::{
    armor_layer::ArmorLayerSurface,
    item_catalog_schema::{
        EquipmentLayerPrecedence, EquipmentLocation, EquipmentPlacement, OccupancyRequirement,
    },
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LayerRank {
    channel: u8,
    order: u16,
}

#[derive(Clone, Debug)]
struct LayerMetadata {
    occupancy: Vec<OccupancyRequirement>,
    layers_over: Vec<EquipmentLayerPrecedence>,
    rank: LayerRank,
}

#[derive(Clone, Debug)]
pub(super) struct PlannedSelection {
    pub item_id: String,
    pub placement_id: String,
    metadata: LayerMetadata,
}

pub(super) struct FittedLayer {
    pub plan: PlannedSelection,
    pub generated: GeneratedArmor,
}

impl PlannedSelection {
    fn new(item_id: &str, placement_id: &str, placement: &EquipmentPlacement) -> Result<Self> {
        let channel = placement
            .outermost_channel()
            .with_context(|| format!("equipment {item_id} {placement_id} has no layer channel"))?;
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
        Ok(Self {
            item_id: item_id.into(),
            placement_id: placement_id.into(),
            metadata: LayerMetadata {
                occupancy: placement.occupancy.clone(),
                layers_over: placement.layers_over.clone(),
                rank: LayerRank {
                    channel: channel.order(),
                    order,
                },
            },
        })
    }
}

pub(super) fn plan(
    selections: &[&ClothingSelection],
    catalog: &EquipmentCatalog,
) -> Result<Vec<PlannedSelection>> {
    let plans = selections
        .iter()
        .map(|selection| {
            let placement = catalog
                .placement(&selection.item_id, &selection.placement_id)
                .with_context(|| {
                    format!(
                        "missing equipment placement {} {}",
                        selection.item_id, selection.placement_id
                    )
                })?;
            PlannedSelection::new(&selection.item_id, &selection.placement_id, placement)
        })
        .collect::<Result<Vec<_>>>()?;
    order(plans)
}

pub(super) fn plan_batches(
    selections: &[&ClothingSelection],
    catalog: &EquipmentCatalog,
) -> Result<Vec<Vec<PlannedSelection>>> {
    batch(plan(selections, catalog)?)
}

fn batch(plans: Vec<PlannedSelection>) -> Result<Vec<Vec<PlannedSelection>>> {
    let mut batches = Vec::<Vec<PlannedSelection>>::new();
    for plan in plans {
        let mut level = 0;
        for (candidate, batch) in batches.iter().enumerate() {
            for inner in batch {
                if precedence(&plan.metadata, &inner.metadata)? == Some(std::cmp::Ordering::Greater)
                {
                    level = level.max(candidate + 1);
                }
            }
        }
        if level == batches.len() {
            batches.push(Vec::new());
        }
        batches[level].push(plan);
    }
    Ok(batches)
}

fn order(mut plans: Vec<PlannedSelection>) -> Result<Vec<PlannedSelection>> {
    let mut dependencies = vec![BTreeSet::new(); plans.len()];
    for a in 0..plans.len() {
        for b in a + 1..plans.len() {
            match precedence(&plans[a].metadata, &plans[b].metadata)? {
                Some(std::cmp::Ordering::Greater) => {
                    dependencies[a].insert(b);
                }
                Some(std::cmp::Ordering::Less) => {
                    dependencies[b].insert(a);
                }
                _ => {}
            }
        }
    }
    let mut ordered = Vec::with_capacity(plans.len());
    while !plans.is_empty() {
        let next = (0..plans.len())
            .filter(|index| dependencies[*index].is_empty())
            .min_by_key(|index| {
                (
                    plans[*index].metadata.rank,
                    &plans[*index].item_id,
                    &plans[*index].placement_id,
                )
            })
            .context("equipment layer dependencies contain a cycle")?;
        ordered.push(plans.remove(next));
        dependencies.remove(next);
        for entries in &mut dependencies {
            *entries = entries
                .iter()
                .filter_map(|index| {
                    if *index == next {
                        None
                    } else {
                        Some(if *index > next { index - 1 } else { *index })
                    }
                })
                .collect();
        }
    }
    Ok(ordered)
}

impl FittedLayer {
    pub(super) fn supports<'a>(
        &'a self,
        outer: &PlannedSelection,
        target: Option<&str>,
    ) -> Result<Option<ArmorLayerSurface<'a>>> {
        if precedence(&outer.metadata, &self.plan.metadata)? != Some(std::cmp::Ordering::Greater) {
            return Ok(None);
        }
        let positions = if let Some(name) = target {
            &self
                .generated
                .morphs
                .iter()
                .find(|morph| morph.name == name)
                .with_context(|| {
                    format!(
                        "support {} {} has no morph {name}",
                        self.plan.item_id, self.plan.placement_id
                    )
                })?
                .direct_positions
        } else {
            &self.generated.positions
        };
        Ok(Some(ArmorLayerSurface {
            relief: adventuresim_armor_model::Millimeters(0),
            positions,
            faces: self.generated.indices.as_chunks::<3>().0,
            joint_indices: &self.generated.joint_indices,
            joint_weights: &self.generated.joint_weights,
        }))
    }
}

/// `Greater` means `a` is geometrically outside `b`.
fn precedence(a: &LayerMetadata, b: &LayerMetadata) -> Result<Option<std::cmp::Ordering>> {
    let a_over_b = explicitly_over(a, b);
    let b_over_a = explicitly_over(b, a);
    anyhow::ensure!(
        !(a_over_b && b_over_a),
        "equipment placements declare contradictory layer precedence"
    );
    if a_over_b {
        return Ok(Some(std::cmp::Ordering::Greater));
    }
    if b_over_a {
        return Ok(Some(std::cmp::Ordering::Less));
    }
    match a.rank.cmp(&b.rank) {
        std::cmp::Ordering::Greater if overlaps_as_outer(a, b) => {
            Ok(Some(std::cmp::Ordering::Greater))
        }
        std::cmp::Ordering::Less if overlaps_as_outer(b, a) => Ok(Some(std::cmp::Ordering::Less)),
        _ => Ok(None),
    }
}

fn overlaps_as_outer(outer: &LayerMetadata, inner: &LayerMetadata) -> bool {
    outer.occupancy.iter().any(|outer| {
        inner
            .occupancy
            .iter()
            .any(|inner| supports_location(outer.location, inner.location))
    })
}

fn explicitly_over(outer: &LayerMetadata, inner: &LayerMetadata) -> bool {
    outer.layers_over.iter().any(|precedence| {
        inner.occupancy.iter().any(|entry| {
            entry.location == precedence.location
                && entry.channel == precedence.channel
                && entry.order == precedence.order
        })
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
    use adventuresim_character_creator::item_catalog_schema::EquipmentChannel;

    fn placement(
        location: EquipmentLocation,
        channel: EquipmentChannel,
        order: u16,
    ) -> EquipmentPlacement {
        EquipmentPlacement {
            id: "worn".into(),
            occupancy: vec![OccupancyRequirement {
                location,
                channel,
                fit_zone: None,
                order,
            }],
            parents: Vec::new(),
            layers_over: Vec::new(),
            protection: Vec::new(),
            surface: Vec::new(),
        }
    }

    #[test]
    fn rigid_arm_plate_is_outside_base_clothing_on_the_same_limb() {
        let sleeve = PlannedSelection::new(
            "sleeve",
            "left",
            &placement(
                EquipmentLocation::LeftArm,
                EquipmentChannel::BaseClothing,
                0,
            ),
        )
        .unwrap();
        let rerebrace = PlannedSelection::new(
            "rerebrace",
            "left",
            &placement(EquipmentLocation::LeftArm, EquipmentChannel::RigidArmor, 0),
        )
        .unwrap();
        assert_eq!(
            precedence(&rerebrace.metadata, &sleeve.metadata).unwrap(),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            precedence(
                &rerebrace.metadata,
                &PlannedSelection::new(
                    "right sleeve",
                    "right",
                    &placement(
                        EquipmentLocation::RightArm,
                        EquipmentChannel::BaseClothing,
                        0,
                    ),
                )
                .unwrap()
                .metadata
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn explicit_local_precedence_can_reverse_channel_order() {
        let hose = PlannedSelection::new(
            "hose",
            "left",
            &placement(EquipmentLocation::LeftLeg, EquipmentChannel::Padding, 0),
        )
        .unwrap();
        let mut boot_placement = placement(
            EquipmentLocation::LeftFoot,
            EquipmentChannel::BaseClothing,
            0,
        );
        boot_placement.layers_over.push(EquipmentLayerPrecedence {
            location: EquipmentLocation::LeftLeg,
            channel: EquipmentChannel::Padding,
            order: 0,
        });
        let boot = PlannedSelection::new("boot", "left", &boot_placement).unwrap();
        assert_eq!(
            precedence(&boot.metadata, &hose.metadata).unwrap(),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn dependency_order_does_not_depend_on_recipe_order() {
        let sleeve = PlannedSelection::new(
            "sleeve",
            "left",
            &placement(
                EquipmentLocation::LeftArm,
                EquipmentChannel::BaseClothing,
                0,
            ),
        )
        .unwrap();
        let rerebrace = PlannedSelection::new(
            "rerebrace",
            "left",
            &placement(EquipmentLocation::LeftArm, EquipmentChannel::RigidArmor, 0),
        )
        .unwrap();
        for plans in [
            vec![rerebrace.clone(), sleeve.clone()],
            vec![sleeve.clone(), rerebrace.clone()],
        ] {
            let ids = order(plans)
                .unwrap()
                .into_iter()
                .map(|plan| plan.item_id)
                .collect::<Vec<_>>();
            assert_eq!(ids, ["sleeve", "rerebrace"]);
        }
    }

    #[test]
    fn batches_separate_dependencies_and_keep_independent_items_parallel() {
        let left_sleeve = PlannedSelection::new(
            "sleeve",
            "left",
            &placement(
                EquipmentLocation::LeftArm,
                EquipmentChannel::BaseClothing,
                0,
            ),
        )
        .unwrap();
        let left_rerebrace = PlannedSelection::new(
            "rerebrace",
            "left",
            &placement(EquipmentLocation::LeftArm, EquipmentChannel::RigidArmor, 0),
        )
        .unwrap();
        let right_sleeve = PlannedSelection::new(
            "sleeve",
            "right",
            &placement(
                EquipmentLocation::RightArm,
                EquipmentChannel::BaseClothing,
                0,
            ),
        )
        .unwrap();

        let batches =
            batch(order(vec![left_rerebrace, right_sleeve, left_sleeve]).unwrap()).unwrap();
        let ids = batches
            .iter()
            .map(|batch| {
                batch
                    .iter()
                    .map(|plan| (plan.item_id.as_str(), plan.placement_id.as_str()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            [
                vec![("sleeve", "left"), ("sleeve", "right")],
                vec![("rerebrace", "left")],
            ]
        );
    }
}
