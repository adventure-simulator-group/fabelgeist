//! Bounded ground-centred outdoor furniture shared by city placement, rendering and collision.
//! Recipes contain local metre-space geometry only; world placement and gameplay live in the runtime.
use std::sync::OnceLock;

use crate::{CollisionBounds, CollisionCuboid, LodMesh};
use bevy::{math::Vec3, prelude::Reflect};
use serde::{Deserialize, Serialize};

mod builder;
mod containers;
mod horse_stop;
mod seating;
mod stall;
#[cfg(test)]
mod tests;

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Reflect,
)]
#[serde(rename_all = "snake_case")]
pub enum FurnitureKind {
    Barrel,
    CargoStack,
    TableBenchSet,
    CanvasStall,
    HitchingTrough,
}

impl FurnitureKind {
    pub const ALL: [Self; 5] = [
        Self::Barrel,
        Self::CargoStack,
        Self::TableBenchSet,
        Self::CanvasStall,
        Self::HitchingTrough,
    ];
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Reflect,
)]
#[serde(rename_all = "snake_case")]
pub enum FurnitureVariant {
    Compact,
    Broad,
}

impl FurnitureVariant {
    pub const ALL: [Self; 2] = [Self::Compact, Self::Broad];
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Reflect,
)]
#[serde(deny_unknown_fields)]
pub struct FurnitureKey {
    pub kind: FurnitureKind,
    pub variant: FurnitureVariant,
}

impl FurnitureKey {
    pub const ALL: [Self; 10] = [
        Self {
            kind: FurnitureKind::Barrel,
            variant: FurnitureVariant::Compact,
        },
        Self {
            kind: FurnitureKind::Barrel,
            variant: FurnitureVariant::Broad,
        },
        Self {
            kind: FurnitureKind::CargoStack,
            variant: FurnitureVariant::Compact,
        },
        Self {
            kind: FurnitureKind::CargoStack,
            variant: FurnitureVariant::Broad,
        },
        Self {
            kind: FurnitureKind::TableBenchSet,
            variant: FurnitureVariant::Compact,
        },
        Self {
            kind: FurnitureKind::TableBenchSet,
            variant: FurnitureVariant::Broad,
        },
        Self {
            kind: FurnitureKind::CanvasStall,
            variant: FurnitureVariant::Compact,
        },
        Self {
            kind: FurnitureKind::CanvasStall,
            variant: FurnitureVariant::Broad,
        },
        Self {
            kind: FurnitureKind::HitchingTrough,
            variant: FurnitureVariant::Compact,
        },
        Self {
            kind: FurnitureKind::HitchingTrough,
            variant: FurnitureVariant::Broad,
        },
    ];

    /// All ten authored recipes share one immutable cache, independent of placement seed.
    pub fn recipe(self) -> &'static FurnitureRecipe {
        static RECIPES: OnceLock<[FurnitureRecipe; FurnitureKey::ALL.len()]> = OnceLock::new();
        let recipes = RECIPES.get_or_init(|| Self::ALL.map(Self::compile));
        &recipes[self.kind as usize * FurnitureVariant::ALL.len() + self.variant as usize]
    }

    fn compile(self) -> FurnitureRecipe {
        let mut builder = builder::Builder::default();
        match self.kind {
            FurnitureKind::Barrel => containers::barrel(&mut builder, self.variant),
            FurnitureKind::CargoStack => containers::cargo(&mut builder, self.variant),
            FurnitureKind::TableBenchSet => seating::table_and_benches(&mut builder, self.variant),
            FurnitureKind::CanvasStall => stall::canopy(&mut builder, self.variant),
            FurnitureKind::HitchingTrough => horse_stop::assemble(&mut builder, self.variant),
        }
        builder.finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FurnitureClearanceKind {
    Access,
    WorkingSpace,
}

#[derive(Clone, Copy, Debug)]
pub struct FurnitureClearance {
    pub kind: FurnitureClearanceKind,
    pub bounds: CollisionBounds,
}

/// Whole supported object in a local horizontal frame, centred at x/z=0 and grounded at y=0.
/// Bounds include every visible vertex; clearances reserve additional usable space.
/// Collider source IDs are local to a recipe and must be scoped by the runtime instance identity.
#[derive(Clone, Debug)]
pub struct FurnitureRecipe {
    pub meshes: Vec<LodMesh>,
    pub colliders: Vec<CollisionCuboid>,
    pub bounds: CollisionBounds,
    pub support_points_metres: Vec<Vec3>,
    pub clearances: Vec<FurnitureClearance>,
    #[cfg(test)]
    members: Vec<CollisionCuboid>,
}
