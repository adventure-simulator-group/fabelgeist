//! Source evidence and appearance estimates travel with exported materials.
use crate::{
    document::Tincture,
    paint::{Paint, PaintPalette},
};

pub(super) fn manifest(palette: &PaintPalette) -> serde_json::Value {
    let records: Vec<_> = Tincture::ALL.into_iter().map(|tincture| {
        let selection = palette[tincture];
        let definition = match selection {
            Paint::Recipe { recipe } => serde_json::to_value(recipe.definition()).expect("recipe definition"),
            Paint::Mixed { .. } => crate::paint::mixing::provenance(),
        };
        serde_json::json!({"tincture": tincture, "selection": selection, "definition": definition})
    }).collect();
    serde_json::json!({
        "scope": "Palette definitions, including unused tinctures; not a bill of materials. Leaf is selected separately.",
        "evidence": "ConservationInference identifies pigments on German shields with a probable egg binder. WorkshopManual refers to Cennini's earlier Italian instructions, not proof of German use in 1544.",
        "appearance": "Recipe swatches are authored estimates. Mixed paints interpolate measured gum-Arabic/parchment spectra, with independent base/shadow/highlight stock recipes. Roughness and transfer to shield grounds remain estimates. Batch quotes are separate from this palette.",
        "preparation": "Pigments are prepared colors; binders identify the dried paint medium. Water, grinding, application and drying are not simulated. Historical parts have no inferred mass or volume unit.",
        "palette": records,
    })
}
