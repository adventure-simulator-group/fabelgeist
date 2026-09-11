//! Authored fastening layouts, distinct from plate shape and decoration.
use super::StrapDesign;
use crate::armor_frames::{FitRegion, Side};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AttachmentRegion {
    Torso,
    UpperArm,
    Forearm,
    Thigh,
    LowerLeg,
    Knee,
    Elbow,
    Hand,
    Foot,
}

impl AttachmentRegion {
    pub fn fitted(self, placement: &str) -> Result<FitRegion> {
        if matches!(self, Self::Torso) {
            return Ok(FitRegion::Torso);
        }
        let side = Side::from_placement(placement)?;
        Ok(match self {
            Self::Torso => unreachable!(),
            Self::UpperArm => FitRegion::UpperArm(side),
            Self::Forearm => FitRegion::Forearm(side),
            Self::Thigh => FitRegion::Thigh(side),
            Self::LowerLeg => FitRegion::LowerLeg(side),
            Self::Knee => FitRegion::Knee(side),
            Self::Elbow => FitRegion::Elbow(side),
            Self::Hand => FitRegion::Hand(side),
            Self::Foot => FitRegion::Foot(side),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionDesign {
    pub region: AttachmentRegion,
    pub closure: StrapDesign,
    pub support: ClosureSupport,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ClosureSupport {
    Wearer,
    Greave,
    Rerebrace,
}

impl RetentionDesign {
    pub fn frame(
        &self,
        wearer: &crate::armor_frames::Wearer<'_>,
        placement: &str,
    ) -> Result<adventuresim_armor_model::PartFrame> {
        let mut frame = wearer.frame(self.region.fitted(placement)?)?;
        if !matches!(self.region, AttachmentRegion::Torso) {
            let outward = match Side::from_placement(placement)? {
                Side::Left => 1.0,
                Side::Right => -1.0,
            };
            if frame.axes[0][0] * outward < 0.0 {
                frame.axes[0] = frame.axes[0].map(|v| -v);
            }
        }
        Ok(frame)
    }
    pub fn validate(&self) -> Result<()> {
        self.closure.validate()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FastenerRecipe {
    Retention(RetentionDesign),
    TassetSuspension(super::suspension::SuspensionDesign),
}

impl FastenerRecipe {
    pub fn support_item(&self) -> Option<&'static str> {
        match self {
            Self::Retention(RetentionDesign {
                support: ClosureSupport::Greave,
                ..
            }) => Some("greave"),
            Self::Retention(RetentionDesign {
                support: ClosureSupport::Rerebrace,
                ..
            }) => Some("rerebrace"),
            _ => None,
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Retention(d) => d.validate(),
            Self::TassetSuspension(d) => d.validate(),
        }
    }

    pub fn generate(
        &self,
        plate: &adventuresim_armor_model::PartMesh,
        wearer: &crate::armor_frames::Wearer<'_>,
        placement: &str,
        support: Option<&adventuresim_armor_model::PartMesh>,
    ) -> Result<adventuresim_armor_model::PartMesh> {
        match self {
            Self::Retention(d) => super::rear_closures(
                plate,
                wearer,
                &d.frame(wearer, placement)?,
                d.region.fitted(placement)?,
                support,
                &d.closure,
            ),
            Self::TassetSuspension(d) => d.generate(
                &component(plate, adventuresim_armor_model::ArmorComponentRole::Tassets)?,
                &component(plate, adventuresim_armor_model::ArmorComponentRole::Fauld)?,
            ),
        }
    }
}

fn component(
    mesh: &adventuresim_armor_model::PartMesh,
    role: adventuresim_armor_model::ArmorComponentRole,
) -> Result<adventuresim_armor_model::PartMesh> {
    let part = mesh
        .components
        .iter()
        .find(|c| c.role == role)
        .context("waist assembly is missing an attachment plate")?;
    let mut result = adventuresim_armor_model::PartMesh::new();
    result.positions = mesh.positions[part.vertices.clone()].to_vec();
    result.indices = mesh.indices[part.indices.clone()]
        .iter()
        .map(|i| i - part.vertices.start as u32)
        .collect();
    Ok(result)
}

pub type FastenerRecipes = BTreeMap<String, FastenerRecipe>;
const SOURCE: &str = include_str!("../../../../assets_src/equipment/armor-fasteners.json");

pub fn load(path: Option<&Path>) -> Result<FastenerRecipes> {
    let mut recipes: FastenerRecipes = serde_json::from_str(SOURCE)?;
    if let Some(path) = path {
        let overrides: FastenerRecipes = serde_json::from_slice(&std::fs::read(path)?)?;
        for (id, recipe) in overrides {
            ensure!(recipes.contains_key(&id), "unknown fastening recipe {id}");
            recipes.insert(id, recipe);
        }
    }
    for (id, recipe) in &recipes {
        recipe
            .validate()
            .with_context(|| format!("fasteners for {id}"))?;
    }
    Ok(recipes)
}
