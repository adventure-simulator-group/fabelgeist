//! Ingredient catalog, not a fabrication or spectral paint simulation.
//! All swatches (including painted tones) and roughness values are authored.
use super::{PaintAppearance, PaintColors};
use crate::document::{Ratio, Tincture};
use serde::{Deserialize, Serialize};

pub const BEHAIM_SOURCE: &str = "https://resources.metmuseum.org/resources/metpublications/pdf/Appendix_Notes_on_the_Restoration_of_the_Behaim_Shields_The_Metropolitan_Museum_Journal_v_30_1995.pdf";
pub const CENNINI_PIGMENTS: &str = "https://noteaccess.com/Texts/Cennini/2.htm";
pub const CENNINI_PAINTING: &str = "https://noteaccess.com/Texts/Cennini/6TM.htm";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Binder {
    EggYolk,
    AnimalGlueSize,
}
impl Binder {
    pub fn label(self) -> &'static str {
        match self {
            Self::EggYolk => "Egg-yolk tempera",
            Self::AnimalGlueSize => "Animal-glue size",
        }
    }
    /// A display starting point, not a measured or universal binder property.
    fn estimated_roughness(self) -> Ratio {
        match self {
            Self::EggYolk => Ratio(0.58),
            Self::AnimalGlueSize => Ratio(0.72),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Pigment {
    LeadTinYellow,
    YellowOchre,
    Orpiment,
    LeadWhite,
    Vermilion,
    RedOchre,
    Azurite,
    Indigo,
    Lampblack,
    VineBlack,
    Malachite,
    Lac,
    NaturalUltramarine,
}
impl Pigment {
    pub fn label(self) -> &'static str {
        match self {
            Self::LeadTinYellow => "Lead-tin yellow",
            Self::YellowOchre => "Yellow ochre",
            Self::Orpiment => "Orpiment",
            Self::LeadWhite => "Lead white",
            Self::Vermilion => "Vermilion",
            Self::RedOchre => "Red ochre",
            Self::Azurite => "Azurite",
            Self::Indigo => "Indigo",
            Self::Lampblack => "Lampblack",
            Self::VineBlack => "Vine black",
            Self::Malachite => "Malachite",
            Self::Lac => "Lac",
            Self::NaturalUltramarine => "Natural ultramarine",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Ingredient {
    pub pigment: Pigment,
    /// Historical "parts", when specified. No mass or volume unit is inferred.
    pub historical_parts: Option<u8>,
}
impl Ingredient {
    const fn new(pigment: Pigment) -> Self {
        Self {
            pigment,
            historical_parts: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RecipeEvidence {
    /// Pigment identified on the object; egg is the conservators' probable binder.
    ConservationInference,
    /// An earlier Italian workshop instruction, not proof of German shield use.
    WorkshopManual,
}
impl RecipeEvidence {
    pub fn label(self) -> &'static str {
        match self {
            Self::ConservationInference => "German shield evidence; egg binder probable",
            Self::WorkshopManual => "Italian workshop manual, c. 1400; comparative evidence",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct RecipeDefinition {
    pub label: &'static str,
    pub ingredients: &'static [Ingredient],
    pub binder: Binder,
    pub evidence: RecipeEvidence,
    pub source_url: &'static str,
    pub source_location: &'static str,
    pub appearance: PaintAppearance,
}

/// IDs persist the selected historical construction, not just its preview RGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaintRecipe {
    LeadTinYellowTempera,
    YellowOchreTempera,
    OrpimentSize,
    LeadWhiteTempera,
    VermilionTempera,
    RedOchreTempera,
    AzuriteTempera,
    IndigoWhiteSize,
    LampblackTempera,
    VineBlackTempera,
    MalachiteTempera,
    OrpimentIndigoSize,
    LacUltramarineTempera,
}
impl PaintRecipe {
    pub const ALL: [Self; 13] = [
        Self::LeadTinYellowTempera,
        Self::YellowOchreTempera,
        Self::OrpimentSize,
        Self::LeadWhiteTempera,
        Self::VermilionTempera,
        Self::RedOchreTempera,
        Self::AzuriteTempera,
        Self::IndigoWhiteSize,
        Self::LampblackTempera,
        Self::VineBlackTempera,
        Self::MalachiteTempera,
        Self::OrpimentIndigoSize,
        Self::LacUltramarineTempera,
    ];
    pub fn for_tincture(t: Tincture) -> Self {
        match t {
            Tincture::Or => Self::LeadTinYellowTempera,
            Tincture::Argent => Self::LeadWhiteTempera,
            Tincture::Gules => Self::VermilionTempera,
            Tincture::Azure => Self::AzuriteTempera,
            Tincture::Sable => Self::LampblackTempera,
            Tincture::Vert => Self::MalachiteTempera,
            Tincture::Purpure => Self::LacUltramarineTempera,
        }
    }
    /// Suggested tincture; authors may deliberately choose another mapping.
    pub fn tincture(self) -> Tincture {
        match self {
            Self::LeadTinYellowTempera | Self::YellowOchreTempera | Self::OrpimentSize => {
                Tincture::Or
            }
            Self::LeadWhiteTempera => Tincture::Argent,
            Self::VermilionTempera | Self::RedOchreTempera => Tincture::Gules,
            Self::AzuriteTempera | Self::IndigoWhiteSize => Tincture::Azure,
            Self::LampblackTempera | Self::VineBlackTempera => Tincture::Sable,
            Self::MalachiteTempera | Self::OrpimentIndigoSize => Tincture::Vert,
            Self::LacUltramarineTempera => Tincture::Purpure,
        }
    }
    pub fn definition(self) -> RecipeDefinition {
        let (binder, evidence, source_url, source_location) = self.source();
        RecipeDefinition {
            label: self.label(),
            ingredients: self.ingredients(),
            binder,
            evidence,
            source_url,
            source_location,
            appearance: PaintAppearance {
                colors: self.colors(),
                roughness: binder.estimated_roughness(),
            },
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::LeadTinYellowTempera => "Lead-tin yellow",
            Self::YellowOchreTempera => "Yellow ochre",
            Self::OrpimentSize => "Orpiment",
            Self::LeadWhiteTempera => "Lead white",
            Self::VermilionTempera => "Vermilion",
            Self::RedOchreTempera => "Red ochre / sinoper",
            Self::AzuriteTempera => "Azurite",
            Self::IndigoWhiteSize => "Indigo + lead white",
            Self::LampblackTempera => "Lampblack",
            Self::VineBlackTempera => "Vine black",
            Self::MalachiteTempera => "Malachite",
            Self::OrpimentIndigoSize => "Orpiment + indigo",
            Self::LacUltramarineTempera => "Lac + ultramarine + lead white",
        }
    }
    fn ingredients(self) -> &'static [Ingredient] {
        use Pigment::*;
        match self {
            Self::LeadTinYellowTempera => const { &[Ingredient::new(LeadTinYellow)] },
            Self::YellowOchreTempera => const { &[Ingredient::new(YellowOchre)] },
            Self::OrpimentSize => const { &[Ingredient::new(Orpiment)] },
            Self::LeadWhiteTempera => const { &[Ingredient::new(LeadWhite)] },
            Self::VermilionTempera => const { &[Ingredient::new(Vermilion)] },
            Self::RedOchreTempera => const { &[Ingredient::new(RedOchre)] },
            Self::AzuriteTempera => const { &[Ingredient::new(Azurite)] },
            Self::IndigoWhiteSize => {
                const { &[Ingredient::new(Indigo), Ingredient::new(LeadWhite)] }
            }
            Self::LampblackTempera => const { &[Ingredient::new(Lampblack)] },
            Self::VineBlackTempera => const { &[Ingredient::new(VineBlack)] },
            Self::MalachiteTempera => const { &[Ingredient::new(Malachite)] },
            Self::OrpimentIndigoSize => {
                const {
                    &[
                        Ingredient {
                            pigment: Orpiment,
                            historical_parts: Some(2),
                        },
                        Ingredient {
                            pigment: Indigo,
                            historical_parts: Some(1),
                        },
                    ]
                }
            }
            Self::LacUltramarineTempera => {
                const {
                    &[
                        Ingredient::new(Lac),
                        Ingredient::new(NaturalUltramarine),
                        Ingredient::new(LeadWhite),
                    ]
                }
            }
        }
    }
    fn colors(self) -> PaintColors {
        let [base, shadow, highlight] = match self {
            Self::LeadTinYellowTempera => [[222, 188, 72], [143, 101, 37], [242, 220, 137]],
            Self::YellowOchreTempera => [[185, 136, 52], [116, 76, 31], [219, 181, 111]],
            Self::OrpimentSize => [[232, 185, 39], [159, 107, 25], [246, 213, 104]],
            Self::LeadWhiteTempera => [[234, 229, 214], [148, 142, 131], [245, 243, 234]],
            Self::VermilionTempera => [[191, 57, 41], [117, 30, 40], [222, 111, 81]],
            Self::RedOchreTempera => [[148, 67, 47], [88, 38, 32], [189, 122, 89]],
            Self::AzuriteTempera => [[43, 91, 136], [26, 48, 84], [112, 155, 182]],
            Self::IndigoWhiteSize => [[65, 86, 116], [32, 44, 70], [128, 145, 163]],
            Self::LampblackTempera => [[27, 26, 25], [15, 14, 14], [86, 83, 78]],
            Self::VineBlackTempera => [[36, 38, 41], [20, 22, 26], [87, 92, 98]],
            Self::MalachiteTempera => [[59, 114, 79], [34, 67, 47], [132, 158, 99]],
            Self::OrpimentIndigoSize => [[76, 107, 54], [40, 63, 36], [146, 157, 80]],
            Self::LacUltramarineTempera => [[109, 64, 108], [65, 33, 69], [163, 120, 156]],
        };
        PaintColors {
            base,
            shadow,
            highlight,
        }
    }
    fn source(self) -> (Binder, RecipeEvidence, &'static str, &'static str) {
        use Binder::*;
        use RecipeEvidence::*;
        match self {
            Self::LeadTinYellowTempera | Self::VermilionTempera | Self::AzuriteTempera => (
                EggYolk,
                ConservationInference,
                BEHAIM_SOURCE,
                "Faltermeier & Meyer 1995, p. 54; original paint on 25.26.1",
            ),
            Self::IndigoWhiteSize => (
                AnimalGlueSize,
                WorkshopManual,
                CENNINI_PAINTING,
                "Cennini, ch. CXLIV; panel or shield",
            ),
            Self::LacUltramarineTempera => (
                EggYolk,
                WorkshopManual,
                CENNINI_PAINTING,
                "Cennini, ch. CXLV; violet panel paint",
            ),
            Self::OrpimentSize => (
                AnimalGlueSize,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. XLVII; shields and lances",
            ),
            Self::OrpimentIndigoSize => (
                AnimalGlueSize,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. LIII; 2:1 historical parts; shields and lances",
            ),
            Self::MalachiteTempera => (
                EggYolk,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. LII; egg tempera, light grinding",
            ),
            Self::LeadWhiteTempera => (
                EggYolk,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. LIX; egg binder from panel method, ch. CXLV",
            ),
            Self::LampblackTempera | Self::VineBlackTempera => (
                EggYolk,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. XXXVII; egg binder from panel method, ch. CXLV",
            ),
            Self::YellowOchreTempera => (
                EggYolk,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. XLV; egg binder from panel method, ch. CXLV",
            ),
            Self::RedOchreTempera => (
                EggYolk,
                WorkshopManual,
                CENNINI_PIGMENTS,
                "Cennini, ch. XXXVIII; egg binder from panel method, ch. CXLV",
            ),
        }
    }
}
