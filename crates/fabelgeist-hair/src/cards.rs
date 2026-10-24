//! Hair cards: the groom as a few thousand textured strips.
//!
//! Games far from the camera, and phones at any distance, cannot draw a
//! hundred thousand strands. They draw cards: strips following the groom's
//! locks, each showing a painted bundle of strands with transparency. Here
//! the cards come from the groom itself rather than from an artist -- the
//! strands are clustered into locks by the same guide map that clumps them
//! ([`crate::ops::GuideMap`]), and each lock becomes a strip along its members' mean
//! path, as wide as they spread across it, lying flat against the head
//! (its face turned out from the head's centre), starting behind its middle
//! root far enough to cover the scalp its lock grows from -- a card from
//! the middle would leave the half of the lock behind it bare, as partings
//! show -- and kept off the head where its few segments would cut through
//! the curve of the scalp. The bundle it shows is one
//! of a few variants in an atlas painted procedurally: wavy strands
//! converging towards the tip, each with its own shade and depth in the
//! bundle, mipmapped by coverage.

pub use atlas::{
    AtlasError, AtlasMip, AtlasResult, AtlasSettings, AtlasTileSize, AtlasTileSpan,
    AtlasVariantCount, CardAtlas,
};

pub use layout::{CardCount, CardError, CardLayout, CardResult, CardSegments, CardStripLayout};
pub use mesh::CardMesh;

use crate::parameters::StrandCount;

mod atlas;
mod build;
mod layout;
mod mesh;

#[derive(Clone, Copy, Debug)]
pub struct CardSettings {
    /// Width of a lock at the root, metres: smaller makes more cards.
    pub size: f32,
    /// Quads along each card.
    pub segments: CardSegments,
    /// Scale on each card's measured width.
    pub width_scale: f32,
    /// Locks of fewer strands than this are left out.
    pub min_strands: StrandCount,
}

/// One card vertex, as the card shader reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CardVertex {
    pub position: [f32; 3],
    /// Across the card, 0 to 1.
    pub u: f32,
    /// Along the lock, root to tip.
    pub tangent: [f32; 3],
    /// Along the card, 0 at the root.
    pub v: f32,
    /// The card's face.
    pub normal: [f32; 3],
    /// Which atlas variant.
    pub variant: f32,
    /// Per card, 0 to 1.
    pub random: f32,
    /// Baked sky visibility.
    pub occlusion: f32,
    pub pad: [f32; 2],
}

impl Default for CardSettings {
    fn default() -> Self {
        Self {
            size: 0.009,
            segments: CardSegments::from_count(12),
            width_scale: 1.3,
            min_strands: StrandCount::from_count(3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{PartingGroup, StrandId};
    use crate::strands::Strands;
    use fabelgeist_math::Vec3;
    use crate::strands::Attachment;

    #[test]
    fn a_comb_of_strands_becomes_a_few_cards() {
        let mut strands = Strands::new(16);
        for i in 0..60 {
            let x = (i % 30) as f32 * 0.001 + (i / 30) as f32 * 0.05;
            let positions: Vec<Vec3> = (0..16)
                .map(|j| Vec3::new(x, 0.1 - j as f32 * 0.01, 0.1))
                .collect();
            strands.push(
                &positions,
                Attachment {
                    id: StrandId::from_word(i),
                    group: PartingGroup::Unassigned,
                    radius: 3e-5,
                    ..Default::default()
                },
            );
        }
        let mesh = CardMesh::build(
            &strands,
            Vec3::default(),
            None,
            None,
            &CardSettings {
                size: 0.02,
                ..Default::default()
            },
            AtlasVariantCount::try_from(4).unwrap(),
        )
        .unwrap();
        assert!(mesh.strips().unwrap().cards().count() >= 2 && mesh.strips().unwrap().cards().count() <= 8, "{}", mesh.strips().unwrap().cards().count());
        assert_eq!(mesh.indices().len(), mesh.strips().unwrap().cards().count() * 12 * 6);
        let atlas = CardAtlas::paint(
            AtlasSettings {
                variants: AtlasVariantCount::try_from(4).unwrap(),
                tile: AtlasTileSize {
                    width: AtlasTileSpan::try_from(32).unwrap(),
                    height: AtlasTileSpan::try_from(128).unwrap(),
                },
            },
            fabelgeist_determinism::Seed::from_u64(1),
        )
        .unwrap();
        assert_eq!(
            atlas.base().pixel_bytes().len(),
            (atlas.base().width_texels() * atlas.base().height_texels() * 4) as usize
        );
        assert!(atlas.base().pixel_bytes().chunks(4).any(|t| t[0] > 200));
    }
}
