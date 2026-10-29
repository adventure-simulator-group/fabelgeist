//! Lacing design uses the shared physical Boolean opening query.
use crate::armor_frames::Wearer;
use crate::bodice_opening_edges::BodiceOpeningEdges;
use crate::cloth_opening::ClothOpening;
use crate::surface_cut::SurfaceCut;
use anyhow::Result;
use fabelgeist_armor::{ClothLacing, GownDesign, PartFrame};
pub(crate) struct BodiceLacingPlan {
    pub design: ClothLacing,
    pub attachment: LacingAttachment,
}
pub(crate) enum LacingAttachment {
    Metal(ClothOpening),
    Sewn(Vec<crate::sewn_eyelet_plan::SewnEyeletPair>),
}
impl BodiceLacingPlan {
    pub fn new(
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        head: &PartFrame,
        gown: &GownDesign,
    ) -> Result<Option<Self>> {
        let Some(design) = &gown.lacing else {
            return Ok(None);
        };
        Ok(Some(Self {
            design: design.clone(),
            attachment: match design.eyelet {
                fabelgeist_armor::LacingEyelet::Metal { .. } => {
                    LacingAttachment::Metal(BodiceOpeningEdges::new(cut, body, head, gown)?.opening)
                }
                fabelgeist_armor::LacingEyelet::Sewn { .. } => LacingAttachment::Sewn(
                    crate::sewn_eyelet_plan::SewnEyeletPair::from_cut(cut, body, design)?,
                ),
            },
        }))
    }
}
