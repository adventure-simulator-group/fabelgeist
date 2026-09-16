//! Paint regions of the two pinned Commons sources; see references/EAGLES.md.
use crate::{artwork::PaintRole, document::*};

pub(super) struct EaglePaint {
    pub body: Tincture,
    pub armed: Tincture,
    pub langued: Tincture,
    pub modeling: PaintedModeling,
}
pub(super) struct Layer {
    pub tincture: Tincture,
    pub role: PaintRole,
    pub coverage: Ratio,
}
impl Layer {
    fn new(tincture: Tincture, role: PaintRole, coverage: Ratio) -> Self {
        Self {
            tincture,
            role,
            coverage,
        }
    }
}
impl EaglePaint {
    pub fn fill(&self, paint: &usvg::Paint, id: &str) -> Vec<Layer> {
        let accent = self.accent(id);
        let (tincture, role, shadow) = match rgb(paint) {
            [26, 26, 26] => (self.body, PaintRole::Charge, Some(PaintRole::Shadow)),
            [45, 45, 45] => (self.body, PaintRole::Charge, None),
            [77, 77, 76] => (self.body, PaintRole::Highlight, None),
            [170, 0, 0] => (accent, PaintRole::Accent, Some(PaintRole::AccentShadow)),
            [204, 0, 0] => (accent, PaintRole::Accent, None),
            [219, 18, 18] => (accent, PaintRole::AccentHighlight, None),
            [120, 0, 0] => (accent, PaintRole::AccentShadow, None),
            [255, 255, 255] => (Tincture::Argent, PaintRole::Accent, None),
            [255, 204, 0] => (Tincture::Or, PaintRole::Accent, None),
            [0, 0, 0] | [17, 17, 17] => (Tincture::Sable, PaintRole::Accent, None),
            other => unreachable!("vendored eagle fill changed: {other:?}"),
        };
        let coverage = if matches!(role, PaintRole::Highlight | PaintRole::AccentHighlight) {
            self.modeling.highlights
        } else {
            Ratio(1.0)
        };
        let mut layers = vec![Layer::new(tincture, role, coverage)];
        // Dark underpainting also carries the silhouette; retain its base in Flat.
        if let Some(shadow) = shadow {
            layers.push(Layer::new(tincture, shadow, self.modeling.shadows));
        }
        layers.retain(|l| l.coverage.0 > 0.0);
        layers
    }
    pub fn stroke(&self, paint: &usvg::Paint, id: &str) -> Option<Layer> {
        let (tincture, role, coverage) = match rgb(paint) {
            [77, 77, 76] => (self.body, PaintRole::Highlight, self.modeling.highlights),
            [0, 0, 0] => (Tincture::Sable, PaintRole::Accent, Ratio(1.0)),
            [120, 0, 0] | [122, 0, 0] => (self.accent(id), PaintRole::AccentShadow, Ratio(1.0)),
            // The double source's red halos are outlines, not tonal modeling.
            [170, 0, 0] => (self.armed, PaintRole::Accent, Ratio(1.0)),
            other => unreachable!("vendored eagle stroke changed: {other:?}"),
        };
        (coverage.0 > 0.0).then_some(Layer::new(tincture, role, coverage))
    }
    fn accent(&self, id: &str) -> Tincture {
        if id.starts_with("tongue-") {
            self.langued
        } else {
            self.armed
        }
    }
}
fn rgb(paint: &usvg::Paint) -> [u8; 3] {
    let usvg::Paint::Color(c) = paint else {
        unreachable!("vendored eagles use solid paint")
    };
    [c.red, c.green, c.blue]
}
