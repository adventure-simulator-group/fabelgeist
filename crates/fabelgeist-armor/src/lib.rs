//! Deterministic, renderer-independent parametric armor fitted to canonical
//! anatomical surface samples, and the metal it is finished in.

mod construction;
pub mod engraving;
pub mod material;
pub mod ornament;
pub mod pattern;
mod skin;
mod tiling;
pub use construction::{Construction, ConstructionError, Lacing, Plate, Tiling};
pub use tiling::{Constructed, MAX_TILES};

mod helmet_crown;
pub use helmet_crown::HelmetCrown;

mod breastplate_design;
pub use breastplate_design::*;
mod plate_fluting;
pub use plate_fluting::{FluteCount, PlateFluting};
mod components;
mod plate_face;
pub use plate_face::PlateFace;
mod surface_grid;
pub use surface_grid::SurfaceGrid;
pub mod trim;
pub use trim::{ArmorSurface, ArmorTrim, TrimBand, TrimError};
mod design;
pub use components::{ArmorComponent, ArmorComponentRole, ArmorHinge};
mod error;
mod frame;
pub use frame::{BoundaryNormals, PartFrame};
mod garment_armor;
pub mod gpu;
pub use gpu::{
    ArmorGpu, BuiltPart, DevicePart, record_extremity_armor, record_helmet, record_limb_armor,
};
mod garment_plate_design;
mod gorget_chart;
pub use garment_plate_design::GarmentPlateShape;
pub use gorget_chart::gorget_control_angle;
mod helmets;
mod limb_armor;
pub use garment_armor::{
    GARMENT_ARMPIT_ROW, GARMENT_AXIAL_SEGMENTS, GARMENT_PANEL_ACROSS, GARMENT_PANEL_ALONG,
    GARMENT_RING_SEGMENTS, GARMENT_SHOULDER_DEPTH_SEGMENTS, GarmentArmorDesign, GarmentArmorKind,
};
pub use helmets::*;
pub use limb_armor::*;

pub use design::*;
pub use error::GenerateError;

pub const SCHEMA_VERSION: u16 = 1;
pub const GENERATOR_VERSION: u16 = 12;

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
    if let Some(fluting) = &design.fluting {
        fluting.validate()?;
    }
    if design.catalog_id.trim().is_empty() {
        return Err(DesignError::EmptyCatalogId);
    }
    if !(700..=1_300).contains(&design.neck_width.0)
        || !(600..=1_400).contains(&design.neck_depth.0)
        || !(700..=1_300).contains(&design.arm_opening_depth.0)
        || !(750..=1_200).contains(&design.waist_width.0)
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
