use super::*;
use crate::garment_material::MailWeave;
use crate::item_catalog_schema::EquipmentChannel;
use adventuresim_armor_model::{CoifDesign, HelmetDesign};
use std::ops::RangeInclusive;

/// Garments the creator can drape, named for the medieval wardrobe they stand
/// in for. Each sewn preset is a cut from the GarmentCodeData design space (see
/// [`cut`](super::cut)); the coif is fitted around the wearer instead.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GarmentPreset {
    /// The straight T-tunic block with short sleeves; a hauberk in mail.
    Shirt,
    /// The darted bodice, cut at the waist.
    FittedShirt,
    /// The cotte: a straight, flared, long-sleeved tunic to the thigh.
    Tunic,
    /// The fitted bodice with long, close sleeves and a standing collar.
    Doublet,
    /// A hip-length, straight, long-sleeved padded coat worn under mail.
    Gambeson,
    Trousers,
    /// Chausses: full-length legs tapering to the ankle.
    Hose,
    /// Loose breeches to above the knee, worn beneath hose.
    Braies,
    Skirt,
    Dress,
    /// The fitted bodice with long sleeves over a flared ankle-length skirt.
    Kirtle,
    /// A sleeveless, flared, knee-length overgarment worn over armor.
    Surcoat,
    /// A voluminous floor-length gown with wide sleeves and a standing collar.
    Houppelande,
    Coif,
}

/// How a garment hangs on the body, which decides how it follows the limbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarmentForm {
    /// Hangs from the shoulders and moves with the torso and arms.
    Upper,
    /// Each leg moves with its own limb.
    Legged,
    /// Hangs between the legs and blends their motion.
    Skirted,
    /// Fitted to the wearer as a surface instead of sewn from a pattern.
    Fitted,
}

impl GarmentPreset {
    pub const ALL: [Self; 14] = [
        Self::Shirt,
        Self::FittedShirt,
        Self::Tunic,
        Self::Doublet,
        Self::Gambeson,
        Self::Trousers,
        Self::Hose,
        Self::Braies,
        Self::Skirt,
        Self::Dress,
        Self::Kirtle,
        Self::Surcoat,
        Self::Houppelande,
        Self::Coif,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Shirt => "Shirt",
            Self::FittedShirt => "Fitted shirt",
            Self::Tunic => "Tunic",
            Self::Doublet => "Doublet",
            Self::Gambeson => "Gambeson",
            Self::Trousers => "Trousers",
            Self::Hose => "Hose",
            Self::Braies => "Braies",
            Self::Skirt => "Skirt",
            Self::Dress => "Dress",
            Self::Kirtle => "Kirtle",
            Self::Surcoat => "Surcoat",
            Self::Houppelande => "Houppelande",
            Self::Coif => "Coif",
        }
    }
    pub fn form(self) -> GarmentForm {
        match self {
            Self::Shirt | Self::FittedShirt | Self::Tunic | Self::Doublet | Self::Gambeson => {
                GarmentForm::Upper
            }
            Self::Trousers | Self::Hose | Self::Braies => GarmentForm::Legged,
            Self::Skirt | Self::Dress | Self::Kirtle | Self::Surcoat | Self::Houppelande => {
                GarmentForm::Skirted
            }
            Self::Coif => GarmentForm::Fitted,
        }
    }
    /// Fitted around the wearer as a surface instead of sewn from a pattern.
    pub fn is_fitted(self) -> bool {
        self.form() == GarmentForm::Fitted
    }
    /// The layer the preset is worn in when its fabric is cloth: padding goes
    /// under mail and outerwear over plate; everything else is base clothing.
    pub fn layer(self) -> EquipmentChannel {
        match self {
            Self::Gambeson => EquipmentChannel::Padding,
            Self::Surcoat | Self::Houppelande => EquipmentChannel::Outerwear,
            _ => EquipmentChannel::BaseClothing,
        }
    }
    /// The fabric the preset is usually made in.
    pub fn default_fabric(self) -> FabricPreset {
        match self {
            // Heavy and stiff, as quilted layers are.
            Self::Gambeson => FabricPreset::Denim,
            Self::Surcoat | Self::Houppelande => FabricPreset::Wool,
            Self::Hose => FabricPreset::Jersey,
            _ => FabricPreset::Cotton,
        }
    }
    /// Adjustable length below the shoulders, in multiples of the
    /// neck-to-waist length. The fitted bodice is always cut at the waist.
    pub fn length_range(self) -> Option<RangeInclusive<f32>> {
        self.cut().and_then(|cut| cut.length_range())
    }
    pub fn default_length(self) -> f32 {
        self.cut().map_or(1.0, |cut| cut.default_length())
    }
    /// The pattern at this preset's default length.
    pub fn design(self) -> Result<Design> {
        self.design_with_length(self.default_length())
    }
    pub fn design_with_length(self, length: f32) -> Result<Design> {
        let Some(cut) = self.cut() else {
            bail!(
                "{} is fitted to the wearer, not cut from a pattern",
                self.label()
            );
        };
        cut.design(length)
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
    GarmentPreset::Shirt.default_length()
}
impl Default for GarmentSelection {
    fn default() -> Self {
        Self::for_preset(GarmentPreset::Shirt)
    }
}
impl GarmentSelection {
    /// Target mesh edge range; finer is available at roughly squared cost.
    pub const RESOLUTION_CM: RangeInclusive<f32> = 2.5..=6.0;

    /// A preset in its usual fabric, at its default length.
    pub fn for_preset(preset: GarmentPreset) -> Self {
        let fabric = preset.default_fabric();
        Self {
            preset,
            fabric,
            resolution_cm: 3.5,
            length: preset.default_length(),
            coif: CoifDesign::default(),
            drape: DrapeSettings::for_fabric(fabric.fabric()),
            mail: MailWeave::STANDARD,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_wear_their_usual_fabric_at_their_default_length() {
        let surcoat = GarmentSelection::for_preset(GarmentPreset::Surcoat);
        assert_eq!(surcoat.fabric, FabricPreset::Wool);
        assert_eq!(surcoat.length, GarmentPreset::Surcoat.default_length());
        assert_eq!(
            surcoat.drape.settling.damping,
            FabricPreset::Wool.fabric().damping
        );
        surcoat.validate().unwrap();
        assert_eq!(GarmentSelection::default().preset, GarmentPreset::Shirt);
        assert_eq!(GarmentSelection::default().fabric, FabricPreset::Cotton);
    }

    #[test]
    fn lengths_outside_a_presets_range_are_rejected() {
        let mut gown = GarmentSelection::for_preset(GarmentPreset::Houppelande);
        gown.length = GarmentPreset::Shirt.default_length();
        assert!(gown.validate().is_err());
        gown.length = GarmentPreset::Houppelande.default_length();
        gown.validate().unwrap();
    }

    #[test]
    fn layers_follow_the_wardrobe() {
        assert_eq!(GarmentPreset::Gambeson.layer(), EquipmentChannel::Padding);
        for outer in [GarmentPreset::Surcoat, GarmentPreset::Houppelande] {
            assert_eq!(outer.layer(), EquipmentChannel::Outerwear);
        }
        for base in [
            GarmentPreset::Tunic,
            GarmentPreset::Doublet,
            GarmentPreset::Hose,
            GarmentPreset::Kirtle,
        ] {
            assert_eq!(base.layer(), EquipmentChannel::BaseClothing);
        }
    }
}
