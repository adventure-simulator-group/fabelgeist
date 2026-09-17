//! Where each article sits on the body, and whether a set of worn articles fits together.

use super::{Article, EquipConflict, InventoryItemId};
use crate::{
    equipment_catalog::ItemCatalog,
    garment::{Construction, FabricPreset, GarmentSelection},
    item_catalog_schema::{
        EquipmentChannel, EquipmentDefinition, EquipmentLocation, OccupancyRequirement,
        ParentRequirement,
    },
};
use adventuresim_core::equipment::{EquipmentGraph, EquipmentGraphEdge, EquipmentGraphPlacement};
use std::collections::BTreeMap;

/// The body cells an article fills and the attachment points it hangs from.
#[derive(Clone, Debug, PartialEq)]
pub struct Occupancy {
    pub body: Vec<OccupancyRequirement>,
    pub parents: Vec<ParentRequirement>,
}

impl Occupancy {
    fn on_body(channel: EquipmentChannel, locations: &[EquipmentLocation]) -> Self {
        Self {
            body: locations
                .iter()
                .map(|&location| OccupancyRequirement {
                    location,
                    channel,
                    fit_zone: None,
                    order: 0,
                })
                .collect(),
            parents: Vec::new(),
        }
    }

    /// The outermost layer the article reaches.
    pub fn layer(&self) -> EquipmentChannel {
        self.body
            .iter()
            .map(|requirement| requirement.channel)
            .chain(self.parents.iter().map(|requirement| requirement.channel))
            .max_by_key(|channel| channel.order())
            .unwrap_or(EquipmentChannel::Accessory)
    }

    pub fn locations(&self) -> impl Iterator<Item = EquipmentLocation> + '_ {
        self.body
            .iter()
            .map(|requirement| requirement.location)
            .chain(
                self.parents
                    .iter()
                    .filter_map(|requirement| requirement.location),
            )
    }
}

impl Article {
    pub fn occupancy(&self, catalog: &ItemCatalog) -> Result<Occupancy, EquipConflict> {
        match self {
            Self::Catalog(article) => {
                let (_, placement) = article.resolve(catalog)?;
                Ok(Occupancy {
                    body: placement.occupancy.clone(),
                    parents: placement.parents.clone(),
                })
            }
            Self::Draped(selection) => Ok(Occupancy::on_body(
                garment_channel(selection),
                &garment_locations(selection),
            )),
            Self::Plate(armor) => {
                use EquipmentLocation::{Chest, Stomach};
                let locations: &[_] = if armor.fauld.layer_count > 0 {
                    &[Chest, Stomach]
                } else {
                    &[Chest]
                };
                Ok(Occupancy::on_body(EquipmentChannel::RigidArmor, locations))
            }
        }
    }
}

/// Mail is armor however it is cut; cloth is worn in its chosen layer.
fn garment_channel(selection: &GarmentSelection) -> EquipmentChannel {
    match selection.fabric {
        FabricPreset::Chainmail => EquipmentChannel::FlexibleArmor,
        FabricPreset::Cotton
        | FabricPreset::Silk
        | FabricPreset::Denim
        | FabricPreset::Wool
        | FabricPreset::Jersey => selection.layer.channel(),
    }
}

/// The body cells a garment covers: the torso for an upper, the arms for its
/// sleeves and the legs for a lower garment.
fn garment_locations(selection: &GarmentSelection) -> Vec<EquipmentLocation> {
    use EquipmentLocation::*;
    let pattern = match &selection.construction {
        Construction::Coif(_) => return vec![Head, Neck],
        Construction::Sewn(pattern) => pattern,
    };
    let mut locations = Vec::new();
    if pattern.upper.is_some() {
        locations.extend([Chest, Stomach]);
    }
    if pattern.sleeves.is_some() {
        locations.extend([LeftArm, RightArm]);
    }
    if pattern.lower.is_some() {
        locations.extend([LeftLeg, RightLeg]);
    }
    locations
}

/// One worn article, ready to be fitted into the equipment graph.
pub(super) struct Candidate<'a> {
    pub id: InventoryItemId,
    pub occupancy: Occupancy,
    pub equipment: Option<&'a EquipmentDefinition>,
    pub attachment_tags: &'a [String],
}

/// Worn articles joined into one conflict-free equipment graph.
#[derive(Debug, Default)]
pub struct WornGraph {
    graph: EquipmentGraph,
    /// The articles each worn article hangs from.
    pub supports: BTreeMap<InventoryItemId, Vec<InventoryItemId>>,
}

impl WornGraph {
    /// Fit candidates in order, deferring attachments until their support is worn.
    pub(super) fn build(
        candidates: Vec<Candidate<'_>>,
    ) -> Result<Self, (InventoryItemId, EquipConflict)> {
        let mut worn = Self::default();
        let mut placed: Vec<&Candidate<'_>> = Vec::new();
        let mut pending: Vec<&Candidate<'_>> = candidates.iter().collect();
        while !pending.is_empty() {
            let before = pending.len();
            let mut deferred = Vec::new();
            for candidate in pending {
                match worn.attach(candidate, &placed) {
                    Some(edges) => {
                        worn.place(candidate, edges, &placed)?;
                        placed.push(candidate);
                    }
                    None => deferred.push(candidate),
                }
            }
            if deferred.len() == before {
                let candidate = deferred[0];
                let requirement = candidate
                    .occupancy
                    .parents
                    .iter()
                    .find(|requirement| worn.support(candidate, requirement, &placed).is_none())
                    .expect("a deferred candidate has an unsupported attachment");
                return Err((
                    candidate.id,
                    EquipConflict::Unsupported {
                        channel: requirement.channel,
                        location: requirement.location,
                    },
                ));
            }
            pending = deferred;
        }
        Ok(worn)
    }

    fn attach(
        &self,
        candidate: &Candidate<'_>,
        placed: &[&Candidate<'_>],
    ) -> Option<Vec<EquipmentGraphEdge>> {
        let mut edges: Vec<EquipmentGraphEdge> = Vec::new();
        for requirement in &candidate.occupancy.parents {
            let mut edge = self.support(candidate, requirement, placed)?;
            // Two requirements on one point use successive capacity.
            edge.capacity_index += edges
                .iter()
                .filter(|other| {
                    other.parent_inventory_item_id == edge.parent_inventory_item_id
                        && other.attachment_point_id == edge.attachment_point_id
                })
                .count() as u16;
            edges.push(edge);
        }
        Some(edges)
    }

    /// The first worn article with a free attachment point that accepts `candidate`.
    fn support(
        &self,
        candidate: &Candidate<'_>,
        requirement: &ParentRequirement,
        placed: &[&Candidate<'_>],
    ) -> Option<EquipmentGraphEdge> {
        placed.iter().find_map(|parent| {
            if requirement
                .location
                .is_some_and(|location| !parent.occupancy.locations().any(|l| l == location))
            {
                return None;
            }
            parent
                .equipment?
                .attachment_points
                .iter()
                .find_map(|point| {
                    let accepts = point.channel == requirement.channel
                        && point.order == requirement.order
                        && (point.accepts_tags.is_empty()
                            || point
                                .accepts_tags
                                .iter()
                                .any(|tag| candidate.attachment_tags.contains(tag)));
                    let used = self.children_on(parent.id, &point.id);
                    (accepts && used < point.capacity).then(|| EquipmentGraphEdge {
                        parent_inventory_item_id: parent.id.0,
                        attachment_point_id: point.id.clone(),
                        capacity_index: used,
                    })
                })
        })
    }

    fn children_on(&self, parent: InventoryItemId, point: &str) -> u16 {
        self.graph
            .nodes
            .values()
            .flat_map(|node| &node.parents)
            .filter(|edge| {
                edge.parent_inventory_item_id == parent.0 && edge.attachment_point_id == point
            })
            .count() as u16
    }

    fn place(
        &mut self,
        candidate: &Candidate<'_>,
        edges: Vec<EquipmentGraphEdge>,
        placed: &[&Candidate<'_>],
    ) -> Result<(), (InventoryItemId, EquipConflict)> {
        let supports = edges
            .iter()
            .map(|edge| InventoryItemId(edge.parent_inventory_item_id))
            .collect();
        let placement = EquipmentGraphPlacement {
            body: candidate.occupancy.body.clone(),
            parents: edges,
        };
        self.graph
            .equip(candidate.id.0, placement)
            .map_err(|reason| {
                let occupied = placed.iter().find_map(|other| {
                    candidate.occupancy.body.iter().find_map(|requirement| {
                        other
                            .occupancy
                            .body
                            .iter()
                            .any(|cell| requirement.conflicts_with(*cell))
                            .then_some(EquipConflict::Occupied {
                                location: requirement.location,
                                channel: requirement.channel,
                                by: other.id,
                            })
                    })
                });
                (
                    candidate.id,
                    occupied.unwrap_or_else(|| EquipConflict::Invalid(reason.into())),
                )
            })?;
        self.supports.insert(candidate.id, supports);
        Ok(())
    }

    /// Worn articles that hang from `id`.
    pub fn dependents(&self, id: InventoryItemId) -> impl Iterator<Item = InventoryItemId> + '_ {
        self.supports
            .iter()
            .filter(move |(_, supports)| supports.contains(&id))
            .map(|(child, _)| *child)
    }
}
