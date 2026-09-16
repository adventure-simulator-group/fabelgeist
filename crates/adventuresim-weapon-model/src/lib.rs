//! Deterministic, renderer-independent melee weapon designs and triangle meshes.

pub mod authoring;
mod catalog;
mod codec;
mod construction;
mod derive;
mod derived_properties;
mod design;
mod editor;
mod evaluation;
pub use evaluation::{EvaluatedHolder, EvaluatedWeapon};
mod hash;
mod holders;
mod icon;
mod mesh;
mod model;
pub mod recipe;
mod validation;

pub use catalog::{
    MELEE_CATALOG_IDS, PRESET_IDS, default_design, default_holder_design, preset_design,
    recommended_holder,
};
pub use codec::{CodecError, decode, decode_holder, encode, encode_holder};
pub use construction::Detail;
pub use derive::{derive_holder_properties, derive_material_masses, derive_properties};
pub use derived_properties::{DerivedMaterialMass, DerivedProperties};
pub use design::*;
pub use editor::{EditorField, NumericEditorField, editor_fields, numeric_editor_fields};
pub use hash::{DesignHash, design_hash, holder_design_hash};
pub use icon::{
    ICON_RENDERER_VERSION, IconBounds, IconError, WeaponIcon, WeaponIconLayout, WeaponIconSpec,
    environment_hdr, generate_holder_icon, generate_icon, icon_layout,
};
pub use mesh::{GenerateError, generate, generate_holder};
pub use model::{GeneratedModel, ModelPart, ModelStats, PhysicalProperties, generate_model};
pub use recipe::Material;
pub use validation::{ValidationError, validate, validate_holder};

pub const SCHEMA_VERSION: u16 = 16;
pub const GENERATOR_VERSION: u16 = 19;
pub const HOLDER_SCHEMA_VERSION: u16 = 10;
pub const HOLDER_GENERATOR_VERSION: u16 = 10;
pub const MAX_ENCODED_RECIPE_BYTES: usize = 128 * 1024;

/// Maximum cylindrical grip radius compatible with a full-hand power grip.
pub const MAX_ROUND_GRIP_RADIUS_MM: u32 = 22;
pub const MAX_SWORD_GRIP_WIDTH_MM: u32 = 38;
pub const MAX_SWORD_GRIP_THICKNESS_MM: u32 = 28;
