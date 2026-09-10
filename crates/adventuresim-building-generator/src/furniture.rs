//! Bounded ground-centred outdoor furniture shared by city placement, rendering and collision.
//! Recipes contain local metre-space geometry only; world placement and gameplay live in the runtime.
use std::sync::OnceLock;

use crate::{CollisionBounds, CollisionCuboid, LodMesh};
use bevy::{math::Vec3, prelude::Reflect};
use serde::{Deserialize, Serialize};

mod builder;
mod containers;
mod domestic;
mod horse_stop;
mod seating;
mod spec;
mod stall;
mod trade;
pub use spec::{FurnitureAccessFace, InteriorFurnitureSpec};
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
    DiningTable,
    Bench,
    Chair,
    Stool,
    Bed,
    BunkBed,
    StorageChest,
    Cupboard,
    Shelving,
    WritingDesk,
    Lectern,
    ChurchBench,
    Altar,
    WardBed,
    BathTub,
    WashStand,
    Workbench,
    CuttingTable,
    ToolRack,
    WeaponRack,
    ArmourStand,
    GrainBin,
    StorageCrate,
    Counter,
    CounterLeftEnd,
    CounterRightEnd,
    CounterCorner,
    DisplayCounter,
    DryingRack,
    KneadingTrough,
    ButchersBlock,
    CaskRack,
    HayRack,
    FeedTrough,
}

impl FurnitureKind {
    pub const OUTDOOR: [Self; 5] = [
        Self::Barrel,
        Self::CargoStack,
        Self::TableBenchSet,
        Self::CanvasStall,
        Self::HitchingTrough,
    ];
    pub const INTERIOR: [Self; 34] = [
        Self::DiningTable,
        Self::Bench,
        Self::Chair,
        Self::Stool,
        Self::Bed,
        Self::BunkBed,
        Self::StorageChest,
        Self::Cupboard,
        Self::Shelving,
        Self::WritingDesk,
        Self::Lectern,
        Self::ChurchBench,
        Self::Altar,
        Self::WardBed,
        Self::BathTub,
        Self::WashStand,
        Self::Workbench,
        Self::CuttingTable,
        Self::ToolRack,
        Self::WeaponRack,
        Self::ArmourStand,
        Self::GrainBin,
        Self::StorageCrate,
        Self::Counter,
        Self::CounterLeftEnd,
        Self::CounterRightEnd,
        Self::CounterCorner,
        Self::DisplayCounter,
        Self::DryingRack,
        Self::KneadingTrough,
        Self::ButchersBlock,
        Self::CaskRack,
        Self::HayRack,
        Self::FeedTrough,
    ];
    pub const ALL: [Self; 39] = [
        Self::Barrel,
        Self::CargoStack,
        Self::TableBenchSet,
        Self::CanvasStall,
        Self::HitchingTrough,
        Self::DiningTable,
        Self::Bench,
        Self::Chair,
        Self::Stool,
        Self::Bed,
        Self::BunkBed,
        Self::StorageChest,
        Self::Cupboard,
        Self::Shelving,
        Self::WritingDesk,
        Self::Lectern,
        Self::ChurchBench,
        Self::Altar,
        Self::WardBed,
        Self::BathTub,
        Self::WashStand,
        Self::Workbench,
        Self::CuttingTable,
        Self::ToolRack,
        Self::WeaponRack,
        Self::ArmourStand,
        Self::GrainBin,
        Self::StorageCrate,
        Self::Counter,
        Self::CounterLeftEnd,
        Self::CounterRightEnd,
        Self::CounterCorner,
        Self::DisplayCounter,
        Self::DryingRack,
        Self::KneadingTrough,
        Self::ButchersBlock,
        Self::CaskRack,
        Self::HayRack,
        Self::FeedTrough,
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
    pub const ALL: [Self; FurnitureKind::ALL.len() * FurnitureVariant::ALL.len()] = {
        let mut keys = [Self {
            kind: FurnitureKind::Barrel,
            variant: FurnitureVariant::Compact,
        }; FurnitureKind::ALL.len() * FurnitureVariant::ALL.len()];
        let mut kind = 0;
        while kind < FurnitureKind::ALL.len() {
            let mut variant = 0;
            while variant < FurnitureVariant::ALL.len() {
                keys[kind * FurnitureVariant::ALL.len() + variant] = Self {
                    kind: FurnitureKind::ALL[kind],
                    variant: FurnitureVariant::ALL[variant],
                };
                variant += 1;
            }
            kind += 1;
        }
        keys
    };

    /// All authored recipes share one immutable cache, independent of placement seed.
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
            FurnitureKind::DiningTable => domestic::assemble(&mut builder, self),
            FurnitureKind::Bench => domestic::assemble(&mut builder, self),
            FurnitureKind::Chair => domestic::assemble(&mut builder, self),
            FurnitureKind::Stool => domestic::assemble(&mut builder, self),
            FurnitureKind::Bed => domestic::assemble(&mut builder, self),
            FurnitureKind::BunkBed => domestic::assemble(&mut builder, self),
            FurnitureKind::StorageChest => domestic::assemble(&mut builder, self),
            FurnitureKind::Cupboard => domestic::assemble(&mut builder, self),
            FurnitureKind::Shelving => domestic::assemble(&mut builder, self),
            FurnitureKind::WritingDesk => domestic::assemble(&mut builder, self),
            FurnitureKind::Lectern => domestic::assemble(&mut builder, self),
            FurnitureKind::ChurchBench => domestic::assemble(&mut builder, self),
            FurnitureKind::Altar => domestic::assemble(&mut builder, self),
            FurnitureKind::WardBed => domestic::assemble(&mut builder, self),
            FurnitureKind::BathTub => domestic::assemble(&mut builder, self),
            FurnitureKind::WashStand => domestic::assemble(&mut builder, self),
            FurnitureKind::Workbench => trade::assemble(&mut builder, self),
            FurnitureKind::CuttingTable => trade::assemble(&mut builder, self),
            FurnitureKind::ToolRack => trade::assemble(&mut builder, self),
            FurnitureKind::WeaponRack => trade::assemble(&mut builder, self),
            FurnitureKind::ArmourStand => trade::assemble(&mut builder, self),
            FurnitureKind::GrainBin => trade::assemble(&mut builder, self),
            FurnitureKind::StorageCrate => trade::assemble(&mut builder, self),
            FurnitureKind::Counter => trade::assemble(&mut builder, self),
            FurnitureKind::CounterLeftEnd => trade::assemble(&mut builder, self),
            FurnitureKind::CounterRightEnd => trade::assemble(&mut builder, self),
            FurnitureKind::CounterCorner => trade::assemble(&mut builder, self),
            FurnitureKind::DisplayCounter => trade::assemble(&mut builder, self),
            FurnitureKind::DryingRack => trade::assemble(&mut builder, self),
            FurnitureKind::KneadingTrough => trade::assemble(&mut builder, self),
            FurnitureKind::ButchersBlock => trade::assemble(&mut builder, self),
            FurnitureKind::CaskRack => trade::assemble(&mut builder, self),
            FurnitureKind::HayRack => trade::assemble(&mut builder, self),
            FurnitureKind::FeedTrough => trade::assemble(&mut builder, self),
        }
        if let Some(spec) = self.interior_spec() {
            for &face in spec.required_faces {
                let bounds = spec.access_bounds(face);
                builder.clearance(FurnitureClearanceKind::Access, bounds.min, bounds.max);
            }
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
