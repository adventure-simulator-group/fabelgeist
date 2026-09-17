//! One draped garment's construction, fabric, layer and drape settings.
use super::pattern::{Pattern, shapes};
use super::*;
use crate::garment_material::MailWeave;
use crate::item_catalog_schema::EquipmentChannel;
use adventuresim_armor_model::{CoifDesign, HelmetDesign};
use std::ops::RangeInclusive;

/// How a garment hangs on the body, which decides how it follows the limbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarmentForm {
    /// Hangs from the shoulders and moves with the torso and arms.
    Upper,
    /// Each leg moves with its own limb.
    Legged,
    /// Hangs across both legs and blends their motion.
    Skirted,
    /// Fitted to the wearer as a surface instead of sewn from a pattern.
    Fitted,
}

/// How the garment is made.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Construction {
    /// Sewn from flat panels cut to the wearer's measurements.
    Sewn(Pattern),
    /// A mail coif's hood, neck and flaps, fitted around the head.
    Coif(CoifDesign),
}

impl Construction {
    pub fn form(&self) -> GarmentForm {
        match self {
            Self::Sewn(pattern) => pattern.form(),
            Self::Coif(_) => GarmentForm::Fitted,
        }
    }
}

/// The layer cloth is worn in; chainmail is always the mail layer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClothLayer {
    Clothing,
    /// Between clothing and mail.
    Padding,
    /// Over mail and plate.
    Outerwear,
}

impl ClothLayer {
    pub const ALL: [Self; 3] = [Self::Clothing, Self::Padding, Self::Outerwear];
    pub fn label(self) -> &'static str {
        match self {
            Self::Clothing => "Clothing",
            Self::Padding => "Padding",
            Self::Outerwear => "Outerwear",
        }
    }
    pub fn channel(self) -> EquipmentChannel {
        match self {
            Self::Clothing => EquipmentChannel::BaseClothing,
            Self::Padding => EquipmentChannel::Padding,
            Self::Outerwear => EquipmentChannel::Outerwear,
        }
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
#[serde(deny_unknown_fields)]
pub struct GarmentSelection {
    pub name: String,
    pub construction: Construction,
    pub fabric: FabricPreset,
    /// Ignored for chainmail, which is always the mail layer.
    pub layer: ClothLayer,
    pub resolution_cm: f32,
    pub drape: DrapeSettings,
    /// Ring geometry and finish when the fabric is chainmail. Appearance only:
    /// editing it never re-runs the simulation.
    pub mail: MailWeave,
}

impl Default for GarmentSelection {
    /// A cotton shirt.
    fn default() -> Self {
        Self {
            name: "Cloth".into(),
            construction: Construction::Sewn(Pattern::default()),
            fabric: FabricPreset::Cotton,
            layer: ClothLayer::Clothing,
            resolution_cm: 3.5,
            drape: DrapeSettings::for_fabric(FabricPreset::Cotton.fabric()),
            mail: MailWeave::STANDARD,
        }
    }
}

impl GarmentSelection {
    /// Target mesh edge range; finer is available at roughly squared cost.
    pub const RESOLUTION_CM: RangeInclusive<f32> = 2.5..=6.0;

    /// A cotton garment cut from a named shape.
    pub fn from_shape(shape: &shapes::Shape) -> Self {
        Self {
            name: shape.name.into(),
            construction: Construction::Sewn(shape.pattern),
            layer: shape.layer,
            ..Self::default()
        }
    }
    /// A mail shirt on the straight T-tunic block, as hauberks were cut.
    pub fn chainmail() -> Self {
        Self {
            name: "Chainmail shirt".into(),
            fabric: FabricPreset::Chainmail,
            drape: DrapeSettings::for_fabric(FabricPreset::Chainmail.fabric()),
            ..Self::from_shape(&shapes::SHIRT)
        }
    }
    /// A mail coif: hood, neck and breast and back flaps, fitted to the head.
    pub fn chainmail_coif() -> Self {
        Self {
            name: "Chainmail coif".into(),
            construction: Construction::Coif(CoifDesign::default()),
            ..Self::chainmail()
        }
    }
    pub fn form(&self) -> GarmentForm {
        self.construction.form()
    }
    pub fn is_fitted(&self) -> bool {
        self.form() == GarmentForm::Fitted
    }
    /// The sewing pattern's design tree.
    pub fn design(&self) -> Result<Design> {
        match &self.construction {
            Construction::Sewn(pattern) => pattern.design(),
            Construction::Coif(_) => {
                bail!("a coif is fitted to the wearer, not cut from a pattern")
            }
        }
    }
    /// Whether two selections simulate identically; name, layer and
    /// appearance may differ.
    pub fn same_simulation(&self, other: &Self) -> bool {
        self.construction == other.construction
            && self.fabric == other.fabric
            && self.resolution_cm == other.resolution_cm
            && self.drape == other.drape
    }
    pub fn validate(&self) -> Result<()> {
        if !self.resolution_cm.is_finite() || !Self::RESOLUTION_CM.contains(&self.resolution_cm) {
            bail!(
                "cloth resolution must be within {:?} cm",
                Self::RESOLUTION_CM
            );
        }
        match &self.construction {
            Construction::Sewn(pattern) => pattern
                .validate()
                .with_context(|| format!("invalid pattern for {}", self.name))?,
            Construction::Coif(coif) => HelmetDesign::MailCoif(*coif)
                .validate()
                .map_err(|error| anyhow::anyhow!("invalid coif: {error:?}"))?,
        }
        self.drape.validate()?;
        self.mail.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_garment_is_a_valid_cotton_shirt() {
        let cloth = GarmentSelection::default();
        cloth.validate().unwrap();
        assert_eq!(cloth.form(), GarmentForm::Upper);
        assert_eq!(cloth.fabric, FabricPreset::Cotton);
        assert_eq!(cloth.layer, ClothLayer::Clothing);
        assert_eq!(
            cloth.drape.settling.damping,
            FabricPreset::Cotton.fabric().damping
        );
    }

    #[test]
    fn garments_round_trip_and_reject_invalid_patterns() {
        for garment in [
            GarmentSelection::from_shape(&shapes::KIRTLE),
            GarmentSelection::chainmail_coif(),
        ] {
            let parsed: GarmentSelection =
                serde_json::from_slice(&serde_json::to_vec(&garment).unwrap()).unwrap();
            assert_eq!(parsed, garment);
            parsed.validate().unwrap();
        }
        let sleeves_only = GarmentSelection {
            construction: Construction::Sewn(Pattern {
                upper: None,
                ..Pattern::default()
            }),
            ..GarmentSelection::default()
        };
        assert!(sleeves_only.validate().is_err());
    }

    #[test]
    fn coifs_are_fitted_and_have_no_pattern() {
        let coif = GarmentSelection::chainmail_coif();
        assert!(coif.is_fitted());
        assert!(coif.design().is_err());
    }
}
