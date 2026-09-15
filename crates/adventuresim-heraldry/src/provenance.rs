//! Source credit follows adapted artwork through composition and export.
use crate::document::{ArmsDesign, ChargeKind, Document, Field};
pub const LION_CREDIT: &str = "Lion artwork adapted from Lion Rampant Or (16th century German) by Tom Lemmens (Tom-L, 2013), after Rinaldum (2009), Wikimedia Commons. Based on Sammelband mehrerer Wappenbuecher, southern Germany, c.1530, BSB Cod.icon. 391; this modern redraw is not a color facsimile. Source and adapted lion artwork: Creative Commons Attribution-ShareAlike 3.0 Unported (CC BY-SA 3.0). Source: https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg . License: https://creativecommons.org/licenses/by-sa/3.0/ . Modifications by Fabelgeist contributors: shared anatomical deformation, recoloring, separated painted tones, tail duplication, crown, line widths, heraldic composition and physical material rendering. Preserve this credit and the license when sharing adapted lion artwork. This notice applies to the adapted artwork; it does not relicense the software.";
pub fn attribution(d: &Document) -> Option<&'static str> {
    contains_lion(&d.arms).then_some(LION_CREDIT)
}
fn contains_lion(a: &ArmsDesign) -> bool {
    a.charges
        .iter()
        .any(|c| matches!(c.shape, ChargeKind::Lion { .. }))
        || a.inescutcheon.as_deref().is_some_and(contains_lion)
        || matches!(&a.field, Field::Quarterly { quarters } if quarters.iter().any(contains_lion))
}
