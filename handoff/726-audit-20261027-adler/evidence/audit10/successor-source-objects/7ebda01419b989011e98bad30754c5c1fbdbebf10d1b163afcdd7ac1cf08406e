//! Named attachments and connected guard/ornament assemblies.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mount {
    ShaftTop,
    ShaftTopCentered,
    ShaftTopSleeve,
    ComponentEnd,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentAnchor {
    #[default]
    Base,
    Center,
    Top,
    Origin,
    /// Center of a blade's receiving heel section, independent of asymmetry.
    #[serde(rename = "heel-center")]
    HeelCenter,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub to: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub at: Option<AttachmentAnchor>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub offset: Option<[Metres; 3]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub overlap: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum NodeBinding {
    Frame {
        frame: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        offset: Option<[Metres; 3]>,
    },
    Between {
        between: [String; 2],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        t: Option<Ratio>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        offset: Option<[Metres; 3]>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GuardMember {
    pub path: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub label: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub section: Option<GuardSection>,
    pub section_width: Metres,
    pub section_depth: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub section_twist: Option<Degrees>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub radial_segments: Option<Count>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub tip_scale: Option<Ratio>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub terminal_swell: Option<Ratio>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GuardPlate {
    pub outline: Vec<String>,
    pub cutout: Vec<String>,
    pub thickness: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub material: Option<Material>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub dish_depth: Option<Metres>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub rim_radius: Option<Metres>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "style", deny_unknown_fields)]
pub enum OrnamentGeometry {
    Crown {},
    Escutcheon {},
    Authored {
        positions: Vec<Metres>,
        indices: Vec<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        smooth: Option<bool>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ornament {
    pub socket: String,
    pub scale: Metres,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub rotation: Option<[Degrees; 3]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::recipe::deserialize_present"
    )]
    pub material: Option<Material>,
    #[serde(flatten)]
    pub geometry: OrnamentGeometry,
}

impl GuardAssemblyParameters {
    /// Order derived nodes by dependency, independently of their names.
    pub(crate) fn binding_order(&self) -> Result<Vec<String>, String> {
        let Some(bindings) = &self.node_bindings else {
            return Ok(Vec::new());
        };
        let mut pending: std::collections::BTreeSet<_> = bindings.keys().cloned().collect();
        let mut order = Vec::new();
        while !pending.is_empty() {
            let next = pending
                .iter()
                .find(|name| match &bindings[*name] {
                    NodeBinding::Frame { .. } => true,
                    NodeBinding::Between { between, .. } => between
                        .iter()
                        .all(|dependency| !pending.contains(dependency)),
                })
                .cloned()
                .ok_or("guard node bindings contain a cycle")?;
            pending.remove(&next);
            order.push(next);
        }
        Ok(order)
    }
}

/// Reserved assembly frame keys shared by placement and controlling-hand selection.
pub(crate) const WEAPON_ROOT_FRAME: &str = "weapon.root";
pub(crate) const WEAPON_GRIP_FRAME: &str = "weapon.grip";
pub(crate) const SHAFT_BOTTOM_FRAME: &str = "shaft.bottom";
pub(crate) const SHAFT_TOP_FRAME: &str = "shaft.top";
pub(crate) const SHIELD_GRIP_FRAME: &str = "shield.grip";
pub(crate) const GRIP_BASE_FRAME: &str = "grip.base";
pub(crate) const GRIP_TOP_FRAME: &str = "grip.top";
pub(crate) const GRIP_CENTER_FRAME: &str = "grip.center";
pub(crate) const GRIP_COMPONENT_ID: &str = "grip";
