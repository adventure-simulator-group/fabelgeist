use super::*;
use crate::garment_material::MailWeave;
use adventuresim_armor_model::{CoifDesign, HelmetDesign};
use std::ops::RangeInclusive;

/// Upper garment length, in multiples of the neck-to-waist length.
const SHIRT_LENGTH: f32 = 1.2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GarmentPreset {
    Shirt,
    FittedShirt,
    Trousers,
    Skirt,
    Dress,
    Coif,
}
impl GarmentPreset {
    pub const ALL: [Self; 6] = [
        Self::Shirt,
        Self::FittedShirt,
        Self::Trousers,
        Self::Skirt,
        Self::Dress,
        Self::Coif,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Shirt => "Shirt",
            Self::FittedShirt => "Fitted shirt",
            Self::Trousers => "Trousers",
            Self::Skirt => "Skirt",
            Self::Dress => "Dress",
            Self::Coif => "Coif",
        }
    }
    /// Fitted around the wearer as a surface instead of sewn from a pattern.
    pub fn is_fitted(self) -> bool {
        matches!(self, Self::Coif)
    }
    /// Adjustable length below the shoulders, in multiples of the
    /// neck-to-waist length. The fitted bodice is always cut at the waist.
    pub fn length_range(self) -> Option<RangeInclusive<f32>> {
        match self {
            Self::Shirt => Some(0.5..=3.5),
            _ => None,
        }
    }
    pub fn default_length(self) -> f32 {
        SHIRT_LENGTH
    }
    /// The pattern at this preset's default length.
    pub fn design(self) -> Result<Design> {
        self.design_with_length(self.default_length())
    }
    pub fn design_with_length(self, length: f32) -> Result<Design> {
        if self.is_fitted() {
            bail!(
                "{} is fitted to the wearer, not cut from a pattern",
                self.label()
            );
        }
        let design = Design::from_yaml_str(assets::DESIGNS[0].yaml)?;
        let upper = matches!(self, Self::Shirt | Self::FittedShirt | Self::Dress);
        design.set_v(
            "meta.upper",
            match self {
                Self::FittedShirt => Value::Str("FittedShirt".into()),
                _ if upper => Value::Str("Shirt".into()),
                _ => Value::Null,
            },
        );
        design.set_v(
            "meta.bottom",
            match self {
                Self::Shirt | Self::FittedShirt => Value::Null,
                Self::Trousers => Value::Str("Pants".into()),
                _ => Value::Str("Skirt2".into()),
            },
        );
        design.set_v(
            "meta.wb",
            if upper {
                Value::Null
            } else {
                Value::Str("FittedWB".into())
            },
        );
        // Author each selectable style explicitly: the T-shirt asset's dormant
        // lower-body parameters otherwise describe shorts and a broad circle skirt.
        design.set_f("pants.length", 0.9);
        design.set_f("skirt.length", 0.45);
        design.set_f("skirt.ruffle", 1.0);
        design.set_f("skirt.flare", 1.0);
        design.set_f("waistband.waist", 1.03);
        design.set_f(
            "shirt.length",
            match self {
                Self::Dress => 1.0,
                _ => length as f64,
            },
        );
        Ok(design)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FabricPreset {
    Chainmail,
    Cotton,
    Silk,
    Denim,
    Wool,
    Jersey,
}
impl FabricPreset {
    /// Ease against the wearer, excluding the material's half thickness.
    pub fn body_ease_cm(self, armor: Option<&fabelgeist_armor::Armor>) -> f32 {
        const ARMORED_MAIL_EASE_CM: f32 = 0.2;
        if self == Self::Chainmail && armor.is_some() {
            ARMORED_MAIL_EASE_CM
        } else {
            FitSettings::default().body_offset_cm
        }
    }
    pub const ALL: [Self; 6] = [
        Self::Chainmail,
        Self::Cotton,
        Self::Silk,
        Self::Denim,
        Self::Wool,
        Self::Jersey,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Chainmail => "Chainmail",
            Self::Cotton => "Cotton",
            Self::Silk => "Silk",
            Self::Denim => "Denim",
            Self::Wool => "Wool",
            Self::Jersey => "Jersey",
        }
    }
    pub fn fabric(self) -> Fabric {
        match self {
            Self::Chainmail => Fabric::CHAINMAIL,
            Self::Cotton => Fabric::COTTON,
            Self::Silk => Fabric::SILK,
            Self::Denim => Fabric::DENIM,
            Self::Wool => Fabric::WOOL,
            Self::Jersey => Fabric::JERSEY,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GarmentSelection {
    pub preset: GarmentPreset,
    pub fabric: FabricPreset,
    pub resolution_cm: f32,
    /// Length below the shoulders for presets with
    /// [`GarmentPreset::length_range`], in neck-to-waist lengths.
    #[serde(default = "shirt_length")]
    pub length: f32,
    /// Coverage and flap shape when the preset is the fitted coif.
    #[serde(default)]
    pub coif: CoifDesign,
    pub drape: DrapeSettings,
    /// Ring geometry and finish when the fabric is chainmail. Appearance only:
    /// editing it never re-runs the simulation.
    pub mail: MailWeave,
}
fn shirt_length() -> f32 {
    SHIRT_LENGTH
}
impl Default for GarmentSelection {
    fn default() -> Self {
        Self {
            preset: GarmentPreset::Shirt,
            fabric: FabricPreset::Cotton,
            resolution_cm: 3.5,
            length: SHIRT_LENGTH,
            coif: CoifDesign::default(),
            drape: DrapeSettings::for_fabric(FabricPreset::Cotton.fabric()),
            mail: MailWeave::STANDARD,
        }
    }
}
impl GarmentSelection {
    /// Target mesh edge range; finer is available at roughly squared cost.
    pub const RESOLUTION_CM: RangeInclusive<f32> = 2.5..=6.0;

    /// A mail shirt on the straight T-tunic block, as hauberks were cut, so
    /// its length can reach anywhere from the waist to the knee.
    pub fn chainmail() -> Self {
        Self {
            preset: GarmentPreset::Shirt,
            fabric: FabricPreset::Chainmail,
            drape: DrapeSettings::for_fabric(FabricPreset::Chainmail.fabric()),
            ..Default::default()
        }
    }
    /// A mail coif: hood, neck and breast and back flaps, fitted to the head.
    pub fn chainmail_coif() -> Self {
        Self {
            preset: GarmentPreset::Coif,
            ..Self::chainmail()
        }
    }
    pub fn design(&self) -> Result<Design> {
        self.preset.design_with_length(self.length)
    }
    /// Whether two selections simulate identically; appearance may differ.
    pub fn same_simulation(&self, other: &Self) -> bool {
        self.preset == other.preset
            && self.fabric == other.fabric
            && self.resolution_cm == other.resolution_cm
            && self.length == other.length
            && self.coif == other.coif
            && self.drape == other.drape
    }
    pub fn validate(&self) -> Result<()> {
        if !self.resolution_cm.is_finite() || !Self::RESOLUTION_CM.contains(&self.resolution_cm) {
            bail!(
                "cloth resolution must be within {:?} cm",
                Self::RESOLUTION_CM
            );
        }
        if let Some(range) = self.preset.length_range()
            && !range.contains(&self.length)
        {
            bail!(
                "{} length must be within {range:?} neck-to-waist lengths",
                self.preset.label()
            );
        }
        if self.preset.is_fitted() {
            HelmetDesign::MailCoif(self.coif)
                .validate()
                .map_err(|error| anyhow::anyhow!("invalid coif: {error:?}"))?;
        }
        self.drape.validate()?;
        self.mail.validate()
    }
}
