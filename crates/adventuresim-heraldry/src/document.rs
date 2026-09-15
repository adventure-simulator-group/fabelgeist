//! Portable identity, interpretation, construction and viewing documents.
use serde::{Deserialize, Serialize};

/// A dimensionless construction coefficient; bounds depend on its control.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ratio(pub f32);
/// Physical millimetres, independent of output resolution.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Millimeters(pub f32);
impl Millimeters {
    pub fn metres(self) -> f32 {
        self.0 / 1000.0
    }
}
/// Angles measured in degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Degrees(pub f32);
/// Deterministic variation in material workmanship, never in heraldic identity.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seed(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tincture {
    Or,
    Argent,
    Gules,
    Azure,
    Sable,
    Vert,
    Purpure,
}
impl Tincture {
    pub const ALL: [Self; 7] = [
        Self::Or,
        Self::Argent,
        Self::Gules,
        Self::Azure,
        Self::Sable,
        Self::Vert,
        Self::Purpure,
    ];
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|v| *v == self).unwrap()
    }
    pub fn metal(self) -> bool {
        matches!(self, Self::Or | Self::Argent)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Division {
    Pale,
    Fess,
    Bend,
    BendSinister,
    Chevron,
    Saltire,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Boundary {
    Straight,
    Wavy,
    Indented,
    Embattled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern {
    Stripes,
    Checks,
    Lozenges,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Field {
    Solid {
        tincture: Tincture,
    },
    Divided {
        division: Division,
        boundary: Boundary,
        tinctures: [Tincture; 2],
    },
    Patterned {
        pattern: Pattern,
        repeats: u8,
        tinctures: [Tincture; 2],
    },
    Quarterly {
        quarters: Box<[ArmsDesign; 4]>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrdinaryKind {
    Pale,
    Fess,
    Bend,
    BendSinister,
    Chevron,
    Cross,
    Saltire,
    Chief,
    Bordure,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ordinary {
    pub kind: OrdinaryKind,
    pub width: Ratio,
    pub boundary: Boundary,
    pub tincture: Tincture,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Coloring {
    Solid { tincture: Tincture },
    Counterchanged { tinctures: [Tincture; 2] },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Facing {
    Dexter,
    Sinister,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EagleHeads {
    One,
    Two,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LionTails {
    One,
    Two,
}
/// Charge identity. Fine artistic proportions live in DrawingStyle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ChargeKind {
    Roundel,
    Lozenge,
    Star {
        points: u8,
    },
    Eagle {
        heads: EagleHeads,
        facing: Facing,
        crowned: bool,
    },
    Lion {
        tails: LionTails,
        facing: Facing,
        crowned: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Charge {
    pub shape: ChargeKind,
    pub color: Coloring,
    /// Normalized artwork position, from the observer's upper left.
    pub center: [f32; 2],
    pub size: [Ratio; 2],
    pub rotation: Degrees,
    pub armed: Tincture,
    pub langued: Tincture,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmsDesign {
    pub field: Field,
    pub ordinaries: Vec<Ordinary>,
    pub charges: Vec<Charge>,
    pub inescutcheon: Option<Box<ArmsDesign>>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EagleDrawing {
    pub body_width: Ratio,
    pub wing_span: Ratio,
    pub wing_lift: Ratio,
    pub feather_length: Ratio,
    pub feather_count: u8,
    pub neck_length: Ratio,
    pub head_size: Ratio,
    pub leg_spread: Ratio,
    pub tail_spread: Ratio,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LionDrawing {
    pub body_width: Ratio,
    pub spine_arch: Ratio,
    pub head_size: Ratio,
    pub foreleg_reach: Ratio,
    pub hindleg_spread: Ratio,
    pub mane_fullness: Ratio,
    pub tail_curl: Ratio,
    pub paw_size: Ratio,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrawingStyle {
    /// Strength of supplemental eagle line work, from absent to fully drawn.
    pub detail: Ratio,
    pub painted_modeling: PaintedModeling,
    pub stroke_width: Ratio,
    pub contour_character: Ratio,
    pub asymmetry: Ratio,
    pub eagle: EagleDrawing,
    pub lion: LionDrawing,
}

/// Coverage of the lion's shadow and highlight paint, independent of anatomy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintedModeling {
    pub shadows: Ratio,
    pub highlights: Ratio,
}
impl PaintedModeling {
    pub const FLAT: Self = Self {
        shadows: Ratio(0.0),
        highlights: Ratio(0.0),
    };
    /// A restrained authored treatment, not a measured historical recipe.
    pub const MODELED: Self = Self {
        shadows: Ratio(0.30),
        highlights: Ratio(0.20),
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Covering {
    None,
    Canvas,
    Hide,
}
/// Application methods, with controls attached to the technique they affect.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "technique", deny_unknown_fields)]
pub enum MetalFinish {
    Pigment,
    WaterGilding {
        burnish: Ratio,
    },
    OilGilding,
    /// Raised wax/resin adhesive, as documented on the Behaim shields.
    MordantGilding {
        relief: Millimeters,
    },
    /// Oil-adhered silver under a yellow glaze, used to represent Or.
    YellowGlazedSilver {
        depth: Ratio,
    },
}
impl MetalFinish {
    pub const BURNISHED: Self = Self::WaterGilding {
        burnish: Ratio(0.85),
    };
    pub const RAISED_MORDANT: Self = Self::MordantGilding {
        relief: Millimeters(0.015),
    };
    pub const GLAZED_SILVER: Self = Self::YellowGlazedSilver { depth: Ratio(0.65) };
    pub const ALL: [Self; 5] = [
        Self::Pigment,
        Self::BURNISHED,
        Self::OilGilding,
        Self::RAISED_MORDANT,
        Self::GLAZED_SILVER,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayShape {
    Panel,
    Shield,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintedSurface {
    pub shape: DisplayShape,
    pub width: Millimeters,
    pub height: Millimeters,
    pub thickness: Millimeters,
    pub curvature: Millimeters,
    pub shoulder: Ratio,
    pub point: Ratio,
    pub covering: Covering,
    pub ground: Millimeters,
    pub pigment: Millimeters,
    pub substrate_relief: Millimeters,
    pub brush_relief: Millimeters,
    pub brush_width: Millimeters,
    pub brush_angle: Degrees,
    pub gold: MetalFinish,
    pub silver: MetalFinish,
    pub glaze: Ratio,
    pub glaze_roughness: Ratio,
    pub palette: crate::paint::PaintPalette,
    pub seed: Seed,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewing {
    pub yaw: Degrees,
    pub pitch: Degrees,
    pub light: Degrees,
    pub exposure: f32,
    pub zoom: Ratio,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub name: String,
    pub arms: ArmsDesign,
    pub drawing: DrawingStyle,
    pub surface: PaintedSurface,
    pub view: Viewing,
}
impl Document {
    pub fn from_json(source: &str) -> Result<Self, crate::Error> {
        const MAX_DOCUMENT_BYTES: usize = 512 * 1024;
        if source.len() > MAX_DOCUMENT_BYTES {
            return Err(crate::Error::Invalid("document exceeds 512 KiB".into()));
        }
        let document: Self = serde_json::from_str(source)?;
        document.validate()?;
        Ok(document)
    }
    pub fn to_json(&self) -> Result<String, crate::Error> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }
}
