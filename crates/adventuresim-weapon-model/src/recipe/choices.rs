//! Precise manufacturing parameters owned by the canonical weapon kernel.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirearmFirearmFamily {
    #[serde(rename = "pistol")]
    Pistol,
    #[serde(rename = "arquebus")]
    Arquebus,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirearmStockStyle {
    #[serde(rename = "pistol")]
    Pistol,
    #[serde(rename = "shoulder")]
    Shoulder,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirearmLockType {
    #[serde(rename = "wheellock")]
    Wheellock,
    #[serde(rename = "matchlock")]
    Matchlock,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirearmMuzzleStyle {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "ringed")]
    Ringed,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirearmSightStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "bead")]
    Bead,
    #[serde(rename = "notch")]
    Notch,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirearmFacingStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "horn")]
    Horn,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BallPouchClosureStyle {
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "buckle")]
    Buckle,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowStockStyle {
    #[serde(rename = "straight")]
    Straight,
    #[serde(rename = "hunting")]
    Hunting,
    #[serde(rename = "swollen")]
    Swollen,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowFacingStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "horn")]
    Horn,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowProdConstruction {
    #[serde(rename = "steel")]
    Steel,
    #[serde(rename = "composite")]
    Composite,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowSpanningMode {
    #[serde(rename = "cranequin")]
    Cranequin,
    #[serde(rename = "goatsFoot")]
    GoatsFoot,
    #[serde(rename = "beltHook")]
    BeltHook,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowSightStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "peep")]
    Peep,
    #[serde(rename = "post")]
    Post,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowBoltHeadStyle {
    #[serde(rename = "bodkin")]
    Bodkin,
    #[serde(rename = "broadhead")]
    Broadhead,
    #[serde(rename = "hunting")]
    Hunting,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossbowBoltBoltUse {
    #[serde(rename = "war")]
    War,
    #[serde(rename = "hunting")]
    Hunting,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BoltQuiverCarrierStyle {
    #[serde(rename = "rigid")]
    Rigid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArcheryBowConstruction {
    #[serde(rename = "self")]
    SelfWood,
    #[serde(rename = "composite")]
    Composite,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArcheryBowLimbSection {
    #[serde(rename = "dShape")]
    DShape,
    #[serde(rename = "oval")]
    Oval,
    #[serde(rename = "flat")]
    Flat,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArrowHeadStyle {
    #[serde(rename = "broadhead")]
    Broadhead,
    #[serde(rename = "bodkin")]
    Bodkin,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArrowNockStyle {
    #[serde(rename = "self")]
    SelfWood,
    #[serde(rename = "reinforced")]
    Reinforced,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArrowQuiverCarrierStyle {
    #[serde(rename = "rigid")]
    Rigid,
    #[serde(rename = "bag")]
    Bag,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SectionBladeSection {
    #[serde(rename = "diamond")]
    Diamond,
    #[serde(rename = "fullered")]
    Fullered,
    #[serde(rename = "hexagonal")]
    Hexagonal,
    #[serde(rename = "lenticular")]
    Lenticular,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardMirrorMode {
    #[serde(rename = "symmetric")]
    Symmetric,
    #[serde(rename = "opposed")]
    Opposed,
    #[serde(rename = "independent")]
    Independent,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardSection {
    #[serde(rename = "round")]
    Round,
    #[serde(rename = "oval")]
    Oval,
    #[serde(rename = "diamond")]
    Diamond,
    #[serde(rename = "flat")]
    Flat,
    #[serde(rename = "triangular")]
    Triangular,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardTerminal {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ball")]
    Ball,
    #[serde(rename = "disk")]
    Disk,
    #[serde(rename = "pyramidal")]
    Pyramidal,
    #[serde(rename = "scroll")]
    Scroll,
    #[serde(rename = "fishtail")]
    Fishtail,
    #[serde(rename = "vase")]
    Vase,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardLeftTerminal {
    #[serde(rename = "shared")]
    Shared,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ball")]
    Ball,
    #[serde(rename = "disk")]
    Disk,
    #[serde(rename = "pyramidal")]
    Pyramidal,
    #[serde(rename = "scroll")]
    Scroll,
    #[serde(rename = "fishtail")]
    Fishtail,
    #[serde(rename = "vase")]
    Vase,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardRightTerminal {
    #[serde(rename = "shared")]
    Shared,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "ball")]
    Ball,
    #[serde(rename = "disk")]
    Disk,
    #[serde(rename = "pyramidal")]
    Pyramidal,
    #[serde(rename = "scroll")]
    Scroll,
    #[serde(rename = "fishtail")]
    Fishtail,
    #[serde(rename = "vase")]
    Vase,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PommelConstruction {
    #[serde(rename = "lathed")]
    Lathed,
    #[serde(rename = "plate")]
    Plate,
    #[serde(rename = "faceted")]
    Faceted,
    #[serde(rename = "writhen")]
    Writhen,
    #[serde(rename = "outline")]
    Outline,
    #[serde(rename = "composite")]
    Composite,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PommelBaseConstruction {
    #[serde(rename = "lathed")]
    Lathed,
    #[serde(rename = "plate")]
    Plate,
    #[serde(rename = "faceted")]
    Faceted,
    #[serde(rename = "writhen")]
    Writhen,
    #[serde(rename = "outline")]
    Outline,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PommelOutlineStyle {
    #[serde(rename = "fishtail")]
    Fishtail,
    #[serde(rename = "fan")]
    Fan,
    #[serde(rename = "pear")]
    Pear,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoundShieldFittingMode {
    #[serde(rename = "grip")]
    Grip,
    #[serde(rename = "grip-and-strap")]
    GripAndStrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapedShieldTopShape {
    #[serde(rename = "flat")]
    Flat,
    #[serde(rename = "rounded")]
    Rounded,
    #[serde(rename = "singlePeak")]
    SinglePeak,
    #[serde(rename = "doublePeak")]
    DoublePeak,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapedShieldBottomShape {
    #[serde(rename = "flat")]
    Flat,
    #[serde(rename = "rounded")]
    Rounded,
    #[serde(rename = "point")]
    Point,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapedShieldFittingMode {
    #[serde(rename = "grip")]
    Grip,
    #[serde(rename = "grip-and-strap")]
    GripAndStrap,
}
