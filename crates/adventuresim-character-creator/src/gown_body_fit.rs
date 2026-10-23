//! Apply authored loose-cloth release without changing its sewn topology.
use crate::armor_frames::Wearer;
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::SurfaceCut;
use anyhow::Result;
use fabelgeist_armor::{GownDesign, PartFrame};

impl SkirtJoinPlan {
    pub fn release_body(
        &mut self,
        design: &GownDesign,
        surface: &SurfaceCut,
        body: &Wearer<'_>,
        hips: &PartFrame,
        head: &PartFrame,
        enclosure_table: Vec<u32>,
    ) -> Result<()> {
        if let fabelgeist_armor::GownBodyFit::Loose { ease, release } = design.body_fit {
            self.enclosure_table = enclosure_table;
            self.ease = ease;
            self.release = release;
            let ease_plan = crate::gown_ease::GownEasePlan::new(
                surface,
                body,
                hips,
                head,
                design.bodice.hem_rise,
            )?;
            self.release_weights = ease_plan.weights;
            self.neckline_samples = ease_plan.neckline_samples;
            if design.is_straight_hanging() {
                // A straight hanging sheet has no intermediate silhouette bend.
                const STRAIGHT_CLOTH_INTERVALS: usize = 2;
                self.patch.hanging_rows = fabelgeist_armor::gpu::SkirtHangingRows::uniform(
                    self.patch.first_row,
                    STRAIGHT_CLOTH_INTERVALS,
                )?;
                self.patch.seam_layout = fabelgeist_armor::gpu::SkirtSeamLayout::StraightFall;
            }
        }
        Ok(())
    }
}
