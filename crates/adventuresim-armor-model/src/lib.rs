//! Deterministic, renderer-independent parametric armor fitted to canonical
//! anatomical surface samples.

#[path = "breastplate_carrier.rs"]
mod breastplate;
#[path = "breastplate_topology_surface.rs"]
pub mod breastplate_topology;
mod design;
mod mesh;

pub use breastplate::generate_breastplate;
pub use design::*;
pub use mesh::{GenerateError, generate_bracer};

pub const SCHEMA_VERSION: u16 = 1;
pub const GENERATOR_VERSION: u16 = 8;

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
    if design.catalog_id.trim().is_empty() {
        return Err(DesignError::EmptyCatalogId);
    }
    if !(700..=1_300).contains(&design.neck_width.0)
        || !(600..=1_400).contains(&design.neck_depth.0)
        || !(700..=1_300).contains(&design.arm_opening_depth.0)
        || !(750..=1_200).contains(&design.waist_width.0)
        || !(650..=1_150).contains(&design.plate_length.0)
        || !(850..=1_080).contains(&design.side_return.0)
        || design.front_crown.0 > 30
        || !(18..=55).contains(&design.shoulder_band_width.0)
        || !(500..=1_600).contains(&design.skirt_length.0)
        || design.skirt_flare.0 > 70
        || !(1..=20).contains(&design.wall_thickness.0)
        || !(4..=30).contains(&design.front_clearance.0)
        || !(6..=35).contains(&design.back_clearance.0)
        || !(4..=30).contains(&design.plate_gap.0)
    {
        return Err(DesignError::BreastplateEdges);
    }
    Ok(())
}

pub fn validate(design: &BracerDesign) -> Result<(), DesignError> {
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
