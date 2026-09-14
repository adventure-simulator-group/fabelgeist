use super::*;
use crate::garment_material::MailWeave;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GarmentPreset {
    Shirt,
    FittedShirt,
    Trousers,
    Skirt,
    Dress,
}
impl GarmentPreset {
    pub const ALL: [Self; 5] = [
        Self::Shirt,
        Self::FittedShirt,
        Self::Trousers,
        Self::Skirt,
        Self::Dress,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Shirt => "Shirt",
            Self::FittedShirt => "Fitted shirt",
            Self::Trousers => "Trousers",
            Self::Skirt => "Skirt",
            Self::Dress => "Dress",
        }
    }
    pub fn design(self) -> Result<Design> {
        let design = Design::from_yaml_str(assets::DESIGNS[0].yaml)?;
        let upper = matches!(self, Self::Shirt | Self::FittedShirt | Self::Dress);
        design.set_v(
            "meta.upper",
            if matches!(self, Self::FittedShirt) {
                Value::Str("FittedShirt".into())
            } else if upper {
                Value::Str("Shirt".into())
            } else {
                Value::Null
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
        if matches!(self, Self::Dress) {
            design.set_f("shirt.length", 1.0);
        }
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
        let mut fabric = match self {
            Self::Chainmail => return Fabric::CHAINMAIL,
            Self::Cotton => Fabric::COTTON,
            Self::Silk => Fabric::SILK,
            Self::Denim => Fabric::DENIM,
            Self::Wool => Fabric::WOOL,
            Self::Jersey => Fabric::JERSEY,
        };
        fabric.bend_compliance *= 100.0;
        fabric
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GarmentSelection {
    pub preset: GarmentPreset,
    pub fabric: FabricPreset,
    pub resolution_cm: f32,
    pub drape: DrapeSettings,
    /// Ring geometry and finish when the fabric is chainmail. Appearance only:
    /// editing it never re-runs the simulation.
    pub mail: MailWeave,
}
impl Default for GarmentSelection {
    fn default() -> Self {
        Self {
            preset: GarmentPreset::Shirt,
            fabric: FabricPreset::Cotton,
            resolution_cm: 3.5,
            drape: DrapeSettings::for_fabric(FabricPreset::Cotton.fabric()),
            mail: MailWeave::STANDARD,
        }
    }
}
impl GarmentSelection {
    /// Target mesh edge range; finer is available at roughly squared cost.
    pub const RESOLUTION_CM: std::ops::RangeInclusive<f32> = 2.5..=6.0;

    pub fn chainmail() -> Self {
        Self {
            preset: GarmentPreset::FittedShirt,
            fabric: FabricPreset::Chainmail,
            drape: DrapeSettings::for_fabric(FabricPreset::Chainmail.fabric()),
            ..Default::default()
        }
    }
    /// Whether two selections simulate identically; appearance may differ.
    pub fn same_simulation(&self, other: &Self) -> bool {
        self.preset == other.preset
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
        self.drape.validate()?;
        self.mail.validate()
    }
}
