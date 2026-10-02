//! Source credit follows adapted artwork through composition and export.
use crate::document::{ArmsDesign, ChargeKind, Document, EagleHeads, Field};
pub const LION_CREDIT: &str = "Lion artwork adapted from Lion Rampant Or (16th century German) by Tom Lemmens (Tom-L, 2013), after Rinaldum (2009), Wikimedia Commons. Based on Sammelband mehrerer Wappenbuecher, southern Germany, c.1530, BSB Cod.icon. 391; this modern redraw is not a color facsimile. Source and adapted lion artwork: Creative Commons Attribution-ShareAlike 3.0 Unported (CC BY-SA 3.0). Source: https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg . License: https://creativecommons.org/licenses/by-sa/3.0/ . Modifications by Fabelgeist contributors: shared anatomical deformation, recoloring, separated painted tones, tail duplication, line widths, heraldic composition and physical material rendering. Preserve this credit and the license when sharing adapted lion artwork. This notice applies to the adapted artwork; it does not relicense the software.";
pub const SINGLE_EAGLE_CREDIT: &str = include_str!("../references/eagles/single.credit.txt");
pub const DOUBLE_EAGLE_CREDIT: &str = include_str!("../references/eagles/double.credit.txt");

pub fn attribution(d: &Document) -> Option<String> {
    let mut credits = Vec::new();
    collect(&d.arms, &mut credits);
    (!credits.is_empty()).then(|| credits.join("\n\n"))
}
fn collect(a: &ArmsDesign, credits: &mut Vec<&'static str>) {
    for charge in &a.charges {
        let credit = match charge.shape {
            ChargeKind::Lion { .. } => LION_CREDIT,
            ChargeKind::Eagle {
                heads: EagleHeads::One,
                ..
            } => SINGLE_EAGLE_CREDIT,
            ChargeKind::Eagle {
                heads: EagleHeads::Two,
                ..
            } => DOUBLE_EAGLE_CREDIT,
            _ => continue,
        };
        if !credits.contains(&credit) {
            credits.push(credit);
        }
    }
    if let Field::Quarterly { quarters } = &a.field {
        for quarter in quarters.iter() {
            collect(quarter, credits);
        }
    }
    if let Some(inset) = &a.inescutcheon {
        collect(inset, credits);
    }
}
