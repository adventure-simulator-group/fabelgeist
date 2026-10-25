//! Deterministic, renderer-independent parametric armor fitted to canonical
//! anatomical surface samples, and the metal it is finished in.

pub use cloth_pleats::ClothPleats;
pub use cloth_pleat_profile::ClothPleatProfile;
pub use cloth_lacing::{ClothLacing, LaceRouting, LacingEyelet};
pub use cloth_slit::ClothSlit;
pub use cloth_closures::{ClothClosures, ClothFastener};
pub use cloth_tape::ClothTape;
pub use cloth_ties::ClothTies;
pub use gown::{FittedSleeves, GownBodyFit, GownDesign};
pub use gown_dressing_slit::{DressingOpeningSide, GownDressingSlit};
pub use gown_waist_support::GownWaistSupport;
pub use partlet::{
    ClothBinding, ClothNeckCurve, CollarBand, CollarSection, PartletDesign, StandingClothCollar,
};
pub use waist_band_fasteners::WaistBandFasteners;
pub use bodice::{
    BodiceDesign, BodiceInterlining, BodiceLining, BodiceNeckline, BodiceOpening,
    InterliningCoverage, NecklineShape,
};
pub use textile_needlework::{
    TextileFoldProfile, TextileNeedlework, TextilePathClosure, TextilePathPoint, TextileStroke,
    TextileStrokeStyle,
};
pub use textile_pleat_zone::TextilePleatZone;
pub use textile_pleat_relief::{
    ShirringPattern, TextileChartSize, TextilePleatLayout, TextilePleatRelief, TextileShirring,
};
pub use skirt::{
    DressBodice, SKIRT_AXIAL_SEGMENTS, SKIRT_MESH_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign,
    SkirtProfile,
};
pub use skirt_structure::{
    PleatStyle, SkirtFrontPleats, SkirtHoops, SkirtPleats, SkirtRepeatCount, SkirtStays,
    SkirtSupport,
};
pub use textile_pattern::{
    MAX_TEXTILE_LAYERS, TextileAppearance, TextileAxis, TextileChartBoundary, TextileFill,
    TextileFinish, TextileLayer, TextileLayers, TextileMotif, TextileOffset, TextilePatternSpace,
    TextileRegion, TextileRepeatCount, TextileSpan, TextileThreadCount,
};
pub use textile_raster::TextilePixels;
pub use construction::{Construction, ConstructionError, ConstructionResult, Lacing, Plate, Tiling};
pub use tiling::{Constructed, MAX_TILES};
pub use helmet_crown::HelmetCrown;
pub use sampling::{ArmorDetail, ArmorLod, EQUIPMENT_TRIANGLE_BUDGET};
pub use breastplate_design::*;
pub use anime_design::{AnimeDesign, BreastplateConstruction};
pub use plate_fluting::{FluteCount, PlateFluting};
pub use surface_material::SurfaceMaterial;
pub use plate_mount::{PlateCourse, PlateGridEnd, PlateJointMotion, PlateMount, PlateParent};
pub use plate_face::PlateFace;
pub use surface_grid::{SurfaceColumn, SurfaceEdgeDistance, SurfaceGrid, SurfaceSample};
pub use trim::{ArmorSurface, ArmorTrim, TrimBand, TrimError};
pub use components::{ArmorComponent, ArmorComponentRole, ArmorHinge, ComponentAppearance};
pub use fauld_chart::FauldLameChart;
pub use frame::{BoundaryNormals, PartFrame};
pub use gpu::{
    ArmorGpu, BuiltPart, DevicePart, record_extremity_armor, record_helmet, record_limb_armor,
};
pub use garment_plate_design::GarmentPlateShape;
pub use gorget_chart::{
    GORGET_COLLAR_HEIGHT_NECK_RATIO, GORGET_FORMED_COLLAR_FRACTION, gorget_control_angle,
};
pub use textile::TextileColor;
pub use radial_fluting::RadialFluting;
pub use besagew::BesagewDesign;
pub use garment_armor::{
    GARMENT_ARMPIT_ROW, GARMENT_AXIAL_SEGMENTS, GARMENT_LAME_SPACING_GAUGES, GARMENT_PANEL_ACROSS,
    GARMENT_PANEL_ALONG, GARMENT_RING_SEGMENTS, GARMENT_SHOULDER_DEPTH_SEGMENTS,
    GarmentArmorDesign, GarmentArmorKind,
};
pub use helmets::*;
pub use limb_armor::*;
pub use pauldron::{PauldronDesign, PauldronOutline};
pub use puff_and_slash::{PuffAndSlashDesign, PuffAndSlashKind, PuffAndSlashPattern};
pub use trunk_hose::TrunkHoseDesign;
pub use visor_bellows::VisorBellows;
pub use wrapped_tassets::{TassetSide, WrappedTassetDesign};
pub use joint_cup_design::{JointCupConstruction, JointCupDesign, JointFluteOrientation};
pub use joint_extension::JointExtension;
pub use waist_armor::{TASSET_SUSPENSION_GAP_M, WaistArmorDesign};
pub use design::*;
pub use error::{GenerateError, PlateShellFailure};

mod bodice;
mod cloth_closures;
mod cloth_pleats;
mod cloth_pleat_profile;
mod cloth_lacing;
mod cloth_slit;
mod cloth_tape;
mod cloth_ties;
mod gown;
mod gown_dressing_slit;
mod gown_waist_support;
mod partlet;
mod waist_band_fasteners;
mod construction;
pub mod engraving;
pub mod material;
pub mod ornament;
pub mod pattern;
mod skin;
mod skirt;
mod skirt_structure;
mod textile_botanical;
mod textile_needlework;
mod textile_pattern;
mod textile_stroke_raster;
mod textile_pleat_relief;
mod textile_pleat_zone;
mod textile_raster;
mod textile_relief;
mod textile_sampling;
mod textile_seams;
mod textile_weave;
mod tiling;
mod helmet_crown;
mod sampling;
mod breastplate_design;
mod anime_design;
mod pierced_plate_domain;
mod plate_fluting;
mod components;
mod surface_material;
mod plate_mount;
mod plate_face;
mod surface_grid;
pub mod trim;
mod design;
mod error;
mod fauld_chart;
mod frame;
mod garment_armor;
pub mod gpu;
mod garment_plate_design;
mod gorget_chart;
mod besagew;
mod helmets;
mod limb_armor;
mod pauldron;
mod puff_and_slash;
mod textile;
mod radial_fluting;
mod trunk_hose;
mod waist_armor;
mod wrapped_tassets;
mod visor_bellows;
mod joint_cup_design;
mod joint_extension;

pub type DesignResult<T> = Result<T, DesignError>;

pub const SCHEMA_VERSION: u16 = 1;
/// Identifies mesh-generation behavior in review and equipment manifests.
pub const GENERATOR_VERSION: u16 = 33;

/// Hash a serialized typed parametric recipe for exported asset provenance.
pub fn parametric_design_hash(encoded: &[u8]) -> [u8; 32] {
    *blake3::hash(encoded).as_bytes()
}

pub fn encode(design: &BracerDesign) -> DesignResult<Vec<u8>> {
    validate(design)?;
    postcard::to_allocvec(design).map_err(|_| DesignError::Encoding)
}

pub fn decode(bytes: &[u8]) -> DesignResult<BracerDesign> {
    let design = postcard::from_bytes(bytes).map_err(|_| DesignError::Encoding)?;
    validate(&design)?;
    Ok(design)
}

pub fn design_hash(design: &BracerDesign) -> DesignResult<[u8; 32]> {
    Ok(*blake3::hash(&encode(design)?).as_bytes())
}

pub fn breastplate_design_hash(design: &BreastplateDesign) -> DesignResult<[u8; 32]> {
    validate_breastplate(design)?;
    let bytes = postcard::to_allocvec(design).map_err(|_| DesignError::Encoding)?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

pub fn validate_breastplate(design: &BreastplateDesign) -> DesignResult<()> {
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
        || !BreastplateDesign::ARM_OPENING_WIDTH_RANGE.contains(&design.arm_opening_width.0)
        || !BreastplateDesign::WAIST_WIDTH_RANGE.contains(&design.waist_width.0)
        || !(700..=1100).contains(&design.back_depth.0)
        || !(650..=1_150).contains(&design.plate_length.0)
        || !BreastplateDesign::SIDE_RETURN_RANGE.contains(&design.side_return.0)
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

pub fn validate(design: &BracerDesign) -> DesignResult<()> {
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
