//! Deterministic, renderer-independent parametric armor fitted to canonical
//! anatomical surface samples.

mod helmet_crown;
pub use helmet_crown::HelmetCrown;

#[path = "breastplate_carrier.rs"]
mod breastplate;
mod breastplate_design;
pub use breastplate_design::*;
mod anime_design;
pub use anime_design::{AnimeDesign, BreastplateConstruction};
mod pierced_plate_domain;
mod plate_fluting;
mod plate_patch;
pub use plate_fluting::{FluteCount, PlateFluting};
mod components;
mod design;
pub use components::{ArmorComponent, ArmorComponentMaterial, ArmorComponentRole, ArmorHinge};
mod mesh;
pub mod parametric;
pub use parametric::{BoundaryNormals, PartFrame, PartMesh, ShellExtrusion};
mod surface_relief;
pub use surface_relief::SurfaceRelief;
mod fauld_chart;
mod garment_armor;
pub use fauld_chart::FauldLameChart;
mod garment_plate_design;
mod gorget_plates;
pub use garment_plate_design::GarmentPlateShape;
pub use gorget_plates::{
    generate_gorget_plates, gorget_bib_direction, gorget_control_angle, gorget_surface_angle,
};
mod besagew;
mod helmets;
mod limb_armor;
mod pauldron;
mod radial_fluting;
mod waist_armor;
mod wrapped_tassets;
pub use radial_fluting::RadialFluting;
mod visor_bellows;
pub use besagew::{BesagewDesign, generate_besagew};
pub use garment_armor::{
    GARMENT_ARMPIT_ROW, GARMENT_AXIAL_SEGMENTS, GARMENT_LAME_SPACING_GAUGES, GARMENT_PANEL_ACROSS,
    GARMENT_PANEL_ALONG, GARMENT_RING_SEGMENTS, GARMENT_SHOULDER_DEPTH_SEGMENTS,
    GarmentArmorDesign, GarmentArmorKind, generate_garment_armor,
};
pub use helmets::*;
pub use limb_armor::*;
pub use pauldron::{PauldronCarrier, PauldronDesign, PauldronOutline};
pub use visor_bellows::VisorBellows;
pub use wrapped_tassets::{TassetSide, TassetSpan, WrappedTassetDesign, generate_wrapped_tasset};
mod joint_cup_design;
pub use joint_cup_design::{JointCupConstruction, JointCupDesign, JointFluteOrientation};
mod joint_extension;
mod raised_joint_cop;
pub use joint_extension::JointExtension;
pub use waist_armor::{
    TASSET_SUSPENSION_GAP_M, WaistArmorDesign, compose_waist, suspend_horizontal_tassets,
};

pub use breastplate::generate_breastplate;
pub use design::*;
pub use mesh::{GenerateError, generate_bracer};

pub const SCHEMA_VERSION: u16 = 1;
/// Identifies mesh-generation behavior in review and equipment manifests.
pub const GENERATOR_VERSION: u16 = 16;

/// Hash a serialized typed parametric recipe for exported asset provenance.
pub fn parametric_design_hash(encoded: &[u8]) -> [u8; 32] {
    *blake3::hash(encoded).as_bytes()
}

pub fn encode(design: &BracerDesign) -> Result<Vec<u8>, DesignError> {
    validate(design)?;
    postcard::to_allocvec(design).map_err(|_| DesignError::Encoding)
}

pub fn decode(bytes: &[u8]) -> Result<BracerDesign, DesignError> {
    let design = postcard::from_bytes(bytes).map_err(|_| DesignError::Encoding)?;
    validate(&design)?;
    Ok(design)
}

pub fn design_hash(design: &BracerDesign) -> Result<[u8; 32], DesignError> {
    Ok(*blake3::hash(&encode(design)?).as_bytes())
}

pub fn breastplate_design_hash(design: &BreastplateDesign) -> Result<[u8; 32], DesignError> {
    validate_breastplate(design)?;
    let bytes = postcard::to_allocvec(design).map_err(|_| DesignError::Encoding)?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

pub fn validate_breastplate(design: &BreastplateDesign) -> Result<(), DesignError> {
    design.profile.validate()?;
    if let BreastplateConstruction::Anime(anime) = &design.construction {
        anime.validate(design.wall_thickness)?;
    }
    if let Some(fluting) = &design.fluting {
        fluting.validate()?;
    }
    if design.catalog_id.trim().is_empty() {
        return Err(DesignError::EmptyCatalogId);
    }
    if !(700..=1_300).contains(&design.neck_width.0)
        || !(600..=1_400).contains(&design.neck_depth.0)
        || !(700..=1_300).contains(&design.arm_opening_depth.0)
        || !BreastplateDesign::WAIST_WIDTH_RANGE.contains(&design.waist_width.0)
        || !(700..=1100).contains(&design.back_depth.0)
        || !(650..=1_150).contains(&design.plate_length.0)
        || !(850..=1_080).contains(&design.side_return.0)
        || !(500..=1_600).contains(&design.skirt_length.0)
        || design.skirt_flare.0 > 70
        || !(1..=20).contains(&design.wall_thickness.0)
        || !(4..=30).contains(&design.front_clearance.0)
        || !(6..=35).contains(&design.back_clearance.0)
    {
        return Err(DesignError::BreastplateEdges);
    }
    Ok(())
}

pub fn validate(design: &BracerDesign) -> Result<(), DesignError> {
    if design.elbow_flare.0 > 15 || design.wrist_flare.0 > 15 || design.center_ridge.0 > 8 {
        return Err(DesignError::Clearance);
    }
    if let Some(fluting) = &design.fluting {
        fluting.validate()?;
    }
    if design.catalog_id.trim().is_empty() {
        return Err(DesignError::EmptyCatalogId);
    }
    if !(50..=1_000).contains(&design.coverage.0) {
        return Err(DesignError::Coverage);
    }
    if design.wrist_offset.0 > 950
        || u32::from(design.coverage.0) + u32::from(design.wrist_offset.0) > 1_000
    {
        return Err(DesignError::Placement);
    }
    if !(1..=20).contains(&design.wall_thickness.0) {
        return Err(DesignError::WallThickness);
    }
    if !(1..=30).contains(&design.clearance.0) {
        return Err(DesignError::Clearance);
    }
    Ok(())
}
