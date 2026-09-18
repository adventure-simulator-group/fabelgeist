//! Ringed puff-and-slash textile carriers for limbs.

use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::{
    ArmorComponentMaterial, ArmorComponentRole, BoundaryNormals, GenerateError, Millimeters,
    PartFrame, PartMesh, Permille, ShellExtrusion,
};

const PANEL_SEAM_GAP_M: f32 = 0.000_1;
const MINIMUM_TUBE_COLUMNS: usize = 8;
const MINIMUM_SLEEVE_ROWS: usize = 8;
const MINIMUM_HOSE_ROWS: usize = 6;
const MINIMUM_SLEEVE_PANEL_ROWS: usize = 3;
const MINIMUM_HOSE_PANEL_ROWS: usize = 2;
const MINIMUM_PATCH_COLUMNS: usize = 8;

#[derive(Clone, Copy)]
struct PuffSpan {
    low: f32,
    high: f32,
    scale: f32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum PuffAndSlashKind {
    Sleeve,
    Hose,
}

/// Serialized sRGB textile color.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextileColor(pub [u8; 3]);

impl TextileColor {
    pub fn material(self) -> ArmorComponentMaterial {
        ArmorComponentMaterial {
            base_color: [
                f32::from(self.0[0]) / 255.0,
                f32::from(self.0[1]) / 255.0,
                f32::from(self.0[2]) / 255.0,
                1.0,
            ],
            metallic: 0.0,
            roughness: 0.92,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffAndSlashDesign {
    pub kind: PuffAndSlashKind,
    pub puff_count: u8,
    pub puff_fullness: Millimeters,
    /// Fullness at the distal end relative to the proximal end.
    pub distal_fullness: Permille,
    /// Bulb profile exponent: 1000 is a sine profile; lower values broaden it.
    pub puff_roundness: Permille,
    pub slash_count: u8,
    /// Fraction of each circumferential repeat occupied by the opening.
    pub slash_width: Permille,
    /// Fraction of each puff length occupied by open slashes.
    pub slash_length: Permille,
    /// Fraction of each puff course reserved for fitted constriction bands.
    pub constriction_width: Permille,
    pub length: Permille,
    /// Placement within the limb span: zero distal, 500 centered, 1000 proximal.
    pub proximal_position: Permille,
    pub rotation: Permille,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    pub outer_color: TextileColor,
    pub undercloth_color: TextileColor,
}

impl Default for PuffAndSlashDesign {
    fn default() -> Self {
        Self {
            kind: PuffAndSlashKind::Sleeve,
            puff_count: 3,
            puff_fullness: Millimeters(45),
            distal_fullness: Permille(650),
            puff_roundness: Permille(850),
            slash_count: 8,
            slash_width: Permille(300),
            slash_length: Permille(700),
            constriction_width: Permille(120),
            length: Permille(950),
            proximal_position: Permille(750),
            rotation: Permille(0),
            clearance: Millimeters(4),
            thickness: Millimeters(2),
            outer_color: TextileColor([150, 32, 28]),
            undercloth_color: TextileColor([224, 184, 58]),
        }
    }
}

impl PuffAndSlashDesign {
    pub fn validate(&self) -> Result<(), GenerateError> {
        let valid = (1..=8).contains(&self.puff_count)
            && (5..=90).contains(&self.puff_fullness.0)
            && (250..=1_500).contains(&self.distal_fullness.0)
            && (400..=2_500).contains(&self.puff_roundness.0)
            && (self.slash_count == 0 || (3..=16).contains(&self.slash_count))
            && (50..=650).contains(&self.slash_width.0)
            && (300..=900).contains(&self.slash_length.0)
            && (40..=350).contains(&self.constriction_width.0)
            && (350..=1_000).contains(&self.length.0)
            && self.proximal_position.0 <= 1_000
            && self.rotation.0 <= 1_000
            && (1..=15).contains(&self.clearance.0)
            && (1..=6).contains(&self.thickness.0);
        if !valid {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

/// Generate a stable ring-and-panel chart. Anatomical fitting replaces its
/// elliptical carrier while preserving the authored radial textile allowance.
pub fn generate_puff_and_slash(
    design: &PuffAndSlashDesign,
    frame: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    design.validate()?;
    frame.validate()?;
    let mut undercloth = tube(design, frame)?.with_component(ArmorComponentRole::Undercloth, None);
    undercloth.components[0].material = Some(design.undercloth_color.material());

    let mut outer = PartMesh::new();
    let [garment_low, garment_high] = axial_span(design, frame);
    let course = (garment_high - garment_low) / f32::from(design.puff_count);
    let constriction = course * design.constriction_width.unit();
    for band in 0..=design.puff_count {
        let center = garment_low + f32::from(band) * course;
        let low = (center - constriction * 0.5).max(garment_low);
        let high = (center + constriction * 0.5).min(garment_high);
        outer.append(constriction_patch(design, frame, low, high)?);
    }
    for puff in 0..design.puff_count {
        let low = garment_low + f32::from(puff) * course + constriction * 0.5;
        let high = garment_low + f32::from(puff + 1) * course - constriction * 0.5;
        let slash_margin = (high - low) * (1.0 - design.slash_length.unit()) * 0.5;
        let slash_low = low + slash_margin;
        let slash_high = high - slash_margin;
        let scale = if design.puff_count == 1 {
            1.0
        } else {
            let proximal = f32::from(puff) / f32::from(design.puff_count - 1);
            design.distal_fullness.unit() * (1.0 - proximal) + proximal
        };
        let puff_span = PuffSpan { low, high, scale };
        outer.append(cuff_patch(
            design,
            frame,
            low,
            slash_low - PANEL_SEAM_GAP_M,
            puff_span.low,
            puff_span.high,
            puff_span.scale,
        )?);
        for panel in 0..design.slash_count {
            outer.append(panel_patch(
                design,
                frame,
                [slash_low + PANEL_SEAM_GAP_M, slash_high - PANEL_SEAM_GAP_M],
                puff_span,
                panel,
            )?);
        }
        outer.append(cuff_patch(
            design,
            frame,
            slash_high + PANEL_SEAM_GAP_M,
            high,
            puff_span.low,
            puff_span.high,
            puff_span.scale,
        )?);
    }
    outer = outer.with_component(ArmorComponentRole::OuterFabric, None);
    outer.components[0].material = Some(design.outer_color.material());
    undercloth.append(outer);
    Ok(undercloth.transformed(frame))
}

fn tube(design: &PuffAndSlashDesign, frame: &PartFrame) -> Result<PartMesh, GenerateError> {
    let columns = frame.detail.segments(64, MINIMUM_TUBE_COLUMNS);
    let minimum_rows = match design.kind {
        PuffAndSlashKind::Sleeve => MINIMUM_SLEEVE_ROWS,
        PuffAndSlashKind::Hose => MINIMUM_HOSE_ROWS,
    };
    let rows = frame.detail.segments(64, minimum_rows);
    let [low, high] = axial_span(design, frame);
    let allowance = design.clearance.metres() + design.thickness.metres();
    let mut positions = Vec::with_capacity(columns * (rows + 1));
    for row in 0..=rows {
        let y = low + (high - low) * row as f32 / rows as f32;
        for column in 0..columns {
            let theta = TAU * column as f32 / columns as f32;
            positions.push(point(
                frame,
                theta,
                y,
                allowance + garment_fullness(design, frame, y),
            ));
        }
    }
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..rows {
        for column in 0..columns {
            let next = (column + 1) % columns;
            let a = (row * columns + column) as u32;
            let b = (row * columns + next) as u32;
            let c = ((row + 1) * columns + column) as u32;
            let d = ((row + 1) * columns + next) as u32;
            indices.extend([a, b, d, a, d, c]);
        }
    }
    PartMesh::from_surface(
        positions,
        indices,
        design.thickness.metres(),
        BoundaryNormals::Smooth,
        ShellExtrusion::Normal,
    )
}

fn panel_patch(
    design: &PuffAndSlashDesign,
    frame: &PartFrame,
    slash: [f32; 2],
    puff: PuffSpan,
    panel: u8,
) -> Result<PartMesh, GenerateError> {
    let columns = frame.detail.segments(8, 1);
    let minimum_rows = match design.kind {
        PuffAndSlashKind::Sleeve => MINIMUM_SLEEVE_PANEL_ROWS,
        PuffAndSlashKind::Hose => MINIMUM_HOSE_PANEL_ROWS,
    };
    let rows = frame.detail.segments(16, minimum_rows);
    let repeat = TAU / f32::from(design.slash_count);
    let width = repeat * (1.0 - design.slash_width.unit());
    let center = repeat * (f32::from(panel) + design.rotation.unit());
    let mut positions = Vec::with_capacity((columns + 1) * (rows + 1));
    for row in 0..=rows {
        let y = slash[0] + (slash[1] - slash[0]) * row as f32 / rows as f32;
        let fullness = puff_fullness(design, puff.low, puff.high, y, puff.scale);
        for column in 0..=columns {
            let theta = center - width * 0.5 + width * column as f32 / columns as f32;
            positions.push(point(
                frame,
                theta,
                y,
                design.clearance.metres() + 2.0 * design.thickness.metres() + fullness,
            ));
        }
    }
    let stride = columns + 1;
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..rows {
        for column in 0..columns {
            let a = (row * stride + column) as u32;
            let b = a + 1;
            let c = ((row + 1) * stride + column) as u32;
            let d = c + 1;
            indices.extend([a, b, d, a, d, c]);
        }
    }
    PartMesh::from_surface(
        positions,
        indices,
        design.thickness.metres(),
        BoundaryNormals::Smooth,
        ShellExtrusion::Normal,
    )
}

fn cuff_patch(
    design: &PuffAndSlashDesign,
    frame: &PartFrame,
    low: f32,
    high: f32,
    puff_low: f32,
    puff_high: f32,
    scale: f32,
) -> Result<PartMesh, GenerateError> {
    let columns = frame.detail.segments(64, MINIMUM_PATCH_COLUMNS);
    let rows = frame.detail.segments(4, 1);
    let mut positions = Vec::with_capacity(columns * (rows + 1));
    for row in 0..=rows {
        let y = low + (high - low) * row as f32 / rows as f32;
        let allowance = design.clearance.metres()
            + 2.0 * design.thickness.metres()
            + puff_fullness(design, puff_low, puff_high, y, scale);
        for column in 0..columns {
            positions.push(point(
                frame,
                TAU * column as f32 / columns as f32,
                y,
                allowance,
            ));
        }
    }
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..rows {
        for column in 0..columns {
            let next = (column + 1) % columns;
            let a = (row * columns + column) as u32;
            let b = (row * columns + next) as u32;
            let c = ((row + 1) * columns + column) as u32;
            let d = ((row + 1) * columns + next) as u32;
            indices.extend([a, b, d, a, d, c]);
        }
    }
    PartMesh::from_surface(
        positions,
        indices,
        design.thickness.metres(),
        BoundaryNormals::Smooth,
        ShellExtrusion::Normal,
    )
}

fn constriction_patch(
    design: &PuffAndSlashDesign,
    frame: &PartFrame,
    low: f32,
    high: f32,
) -> Result<PartMesh, GenerateError> {
    let columns = frame.detail.segments(64, MINIMUM_PATCH_COLUMNS);
    let rows = frame.detail.segments(4, 1);
    let allowance = design.clearance.metres() + 2.0 * design.thickness.metres();
    let mut positions = Vec::with_capacity(columns * (rows + 1));
    for row in 0..=rows {
        let y = low + (high - low) * row as f32 / rows as f32;
        for column in 0..columns {
            positions.push(point(
                frame,
                TAU * column as f32 / columns as f32,
                y,
                allowance,
            ));
        }
    }
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..rows {
        for column in 0..columns {
            let next = (column + 1) % columns;
            let a = (row * columns + column) as u32;
            let b = (row * columns + next) as u32;
            let c = ((row + 1) * columns + column) as u32;
            let d = ((row + 1) * columns + next) as u32;
            indices.extend([a, b, d, a, d, c]);
        }
    }
    PartMesh::from_surface(
        positions,
        indices,
        design.thickness.metres(),
        BoundaryNormals::Smooth,
        ShellExtrusion::Normal,
    )
}

fn puff_fullness(design: &PuffAndSlashDesign, low: f32, high: f32, y: f32, scale: f32) -> f32 {
    let t = ((y - low) / (high - low)).clamp(0.0, 1.0);
    design.puff_fullness.metres()
        * scale
        * (std::f32::consts::PI * t)
            .sin()
            .max(0.0)
            .powf(design.puff_roundness.unit())
}

fn garment_fullness(design: &PuffAndSlashDesign, frame: &PartFrame, y: f32) -> f32 {
    let [low, high] = axial_span(design, frame);
    let course = (high - low) / f32::from(design.puff_count);
    let puff = (((y - low) / course).floor() as usize).min(usize::from(design.puff_count) - 1);
    let puff_low = low + puff as f32 * course;
    let scale = if design.puff_count == 1 {
        1.0
    } else {
        let proximal = puff as f32 / f32::from(design.puff_count - 1);
        design.distal_fullness.unit() * (1.0 - proximal) + proximal
    };
    puff_fullness(design, puff_low, puff_low + course, y, scale)
}

fn axial_span(design: &PuffAndSlashDesign, frame: &PartFrame) -> [f32; 2] {
    let full = 2.0 * frame.half_extents[1];
    let length = full * design.length.unit();
    let margin = full - length;
    let low = -frame.half_extents[1] + margin * design.proximal_position.unit();
    [low, low + length]
}

fn point(frame: &PartFrame, theta: f32, y: f32, allowance: f32) -> [f32; 3] {
    [
        (frame.half_extents[0] + allowance) * theta.sin(),
        y,
        (frame.half_extents[2] + allowance) * theta.cos(),
    ]
}
