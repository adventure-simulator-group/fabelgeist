//! Check every bearing path once, including shared paths through stacked working equipment.
use std::collections::HashMap;

use crate::{StructuralNode, StructuralNodeId};

#[derive(Clone, Copy)]
enum GroundingState {
    Visiting,
    Supported,
    Unsupported,
}

pub(super) struct GroundSupport<'a> {
    nodes: &'a HashMap<StructuralNodeId, &'a StructuralNode>,
    states: HashMap<StructuralNodeId, GroundingState>,
}

impl<'a> GroundSupport<'a> {
    pub(super) fn new(nodes: &'a HashMap<StructuralNodeId, &'a StructuralNode>) -> Self {
        Self {
            nodes,
            states: HashMap::new(),
        }
    }

    pub(super) fn reaches_ground(&mut self, id: StructuralNodeId) -> bool {
        if let Some(state) = self.states.get(&id) {
            // An active node means this path contains a cycle; a completed node is reusable.
            return matches!(state, GroundingState::Supported);
        }
        let Some(node) = self.nodes.get(&id).copied() else {
            self.states.insert(id, GroundingState::Unsupported);
            return false;
        };
        self.states.insert(id, GroundingState::Visiting);
        let supported = node.grounded
            || (!node.supported_by.is_empty()
                && node
                    .supported_by
                    .iter()
                    .all(|parent| self.reaches_ground(*parent)));
        self.states.insert(
            id,
            if supported {
                GroundingState::Supported
            } else {
                GroundingState::Unsupported
            },
        );
        supported
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GeometryOwnerId, StructuralNodeKind};
    use bevy::math::Vec3;

    fn node(id: u64, grounded: bool, parents: &[u64]) -> StructuralNode {
        StructuralNode {
            id: StructuralNodeId(id),
            owner: GeometryOwnerId(1),
            kind: StructuralNodeKind::WallBearing,
            position: Vec3::ZERO,
            supported_by: parents.iter().copied().map(StructuralNodeId).collect(),
            grounded,
        }
    }

    #[test]
    fn many_shared_bearing_paths_are_validated_without_expanding_each_path() {
        let mut structure = vec![node(0, true, &[])];
        for tier in 0..48 {
            let parents = if tier == 0 {
                vec![0]
            } else {
                vec![tier * 2 - 1, tier * 2]
            };
            structure.push(node(tier * 2 + 1, false, &parents));
            structure.push(node(tier * 2 + 2, false, &parents));
        }
        let nodes = structure.iter().map(|node| (node.id, node)).collect();
        let mut support = GroundSupport::new(&nodes);
        // Starting at the crown exercises all 2^48 shared ancestry paths of a naive traversal.
        assert!(support.reaches_ground(StructuralNodeId(96)));
        assert!(structure.iter().all(|node| support.reaches_ground(node.id)));
    }

    #[test]
    fn every_ungrounded_parent_must_be_present_acyclic_and_supported() {
        let structure = [
            node(0, true, &[99]),
            node(1, false, &[]),
            node(2, false, &[0, 99]),
            node(3, false, &[0, 4]),
            node(4, false, &[3]),
            node(5, false, &[0, 1]),
            node(6, false, &[0]),
        ];
        let nodes = structure.iter().map(|node| (node.id, node)).collect();
        for order in [[3, 4, 1, 2, 5], [5, 2, 1, 4, 3]] {
            let mut support = GroundSupport::new(&nodes);
            assert!(
                support.reaches_ground(StructuralNodeId(0)),
                "grounded foundations terminate a path"
            );
            for id in order {
                assert!(
                    !support.reaches_ground(StructuralNodeId(id)),
                    "unsupported node {id}"
                );
            }
            assert!(support.reaches_ground(StructuralNodeId(6)));
        }
    }
}
