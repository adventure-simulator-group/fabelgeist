//! Sealed card geometry keeps generic triangles distinct from generated strips.

use super::{CardLayout, CardResult, CardStripLayout, CardVertex};

/// Cards as a checked triangle list, supplied directly or generated as strips.
#[derive(Clone, Debug, Default)]
pub struct CardMesh {
    pub(super) vertices: Vec<CardVertex>,
    pub(super) indices: Vec<u32>,
    pub(super) topology: CardTopology,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum CardTopology {
    Triangles(CardLayout),
    Strips(CardStripLayout),
}

impl Default for CardTopology {
    fn default() -> Self {
        Self::Triangles(CardLayout::default())
    }
}

impl CardMesh {
    /// Admit a complete triangle list and every index's membership in its vertices.
    pub fn from_triangles(vertices: Vec<CardVertex>, indices: Vec<u32>) -> CardResult<Self> {
        let layout = CardLayout::for_triangles(&vertices, &indices)?;
        Ok(Self { vertices, indices, topology: CardTopology::Triangles(layout) })
    }

    pub fn vertices(&self) -> &[CardVertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn layout(&self) -> CardLayout {
        match self.topology {
            CardTopology::Triangles(layout) => layout,
            CardTopology::Strips(strips) => strips.layout(),
        }
    }

    /// Generated strip metadata; directly supplied triangles have no card count.
    pub fn strips(&self) -> Option<CardStripLayout> {
        match self.topology {
            CardTopology::Triangles(_) => None,
            CardTopology::Strips(strips) => Some(strips),
        }
    }
}
