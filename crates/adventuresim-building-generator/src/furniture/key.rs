//! Valid authored combinations of furniture form, size and wood treatment.
use super::{FurnitureKind, FurnitureVariant};
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum FurnitureWoodState {
    #[default]
    Natural,
    Handled,
    Repaired,
    Painted,
}

impl FurnitureWoodState {
    pub const ALL: [Self; 4] = [Self::Natural, Self::Handled, Self::Repaired, Self::Painted];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinishableFurnitureKind {
    DiningTable,
    Bench,
    Chair,
    StorageChest,
    Workbench,
}

impl FinishableFurnitureKind {
    pub const ALL: [Self; 5] = [
        Self::DiningTable,
        Self::Bench,
        Self::Chair,
        Self::StorageChest,
        Self::Workbench,
    ];

    pub const fn kind(self) -> FurnitureKind {
        match self {
            Self::DiningTable => FurnitureKind::DiningTable,
            Self::Bench => FurnitureKind::Bench,
            Self::Chair => FurnitureKind::Chair,
            Self::StorageChest => FurnitureKind::StorageChest,
            Self::Workbench => FurnitureKind::Workbench,
        }
    }

    pub const fn from_kind(kind: FurnitureKind) -> Option<Self> {
        match kind {
            FurnitureKind::DiningTable => Some(Self::DiningTable),
            FurnitureKind::Bench => Some(Self::Bench),
            FurnitureKind::Chair => Some(Self::Chair),
            FurnitureKind::StorageChest => Some(Self::StorageChest),
            FurnitureKind::Workbench => Some(Self::Workbench),
            _ => None,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Reflect,
)]
#[reflect(opaque)]
#[serde(try_from = "KeyFields", into = "KeyFields")]
pub struct FurnitureKey {
    pub(super) kind: FurnitureKind,
    pub(super) variant: FurnitureVariant,
    pub(super) wood_state: FurnitureWoodState,
}

impl FurnitureKey {
    pub const fn natural(kind: FurnitureKind, variant: FurnitureVariant) -> Self {
        Self {
            kind,
            variant,
            wood_state: FurnitureWoodState::Natural,
        }
    }

    pub const fn wood(
        kind: FinishableFurnitureKind,
        variant: FurnitureVariant,
        wood_state: FurnitureWoodState,
    ) -> Self {
        Self {
            kind: kind.kind(),
            variant,
            wood_state,
        }
    }

    pub const fn kind(self) -> FurnitureKind {
        self.kind
    }
    pub const fn variant(self) -> FurnitureVariant {
        self.variant
    }
    pub const fn wood_state(self) -> FurnitureWoodState {
        self.wood_state
    }

    pub const COUNT: usize = FurnitureKind::ALL.len() * FurnitureVariant::ALL.len()
        + FinishableFurnitureKind::ALL.len()
            * FurnitureVariant::ALL.len()
            * (FurnitureWoodState::ALL.len() - 1);
    pub const ALL: [Self; Self::COUNT] = {
        let mut keys =
            [Self::natural(FurnitureKind::Barrel, FurnitureVariant::Compact); Self::COUNT];
        let mut cursor = 0;
        let mut kind = 0;
        while kind < FurnitureKind::ALL.len() {
            let mut variant = 0;
            while variant < FurnitureVariant::ALL.len() {
                keys[cursor] =
                    Self::natural(FurnitureKind::ALL[kind], FurnitureVariant::ALL[variant]);
                cursor += 1;
                variant += 1;
            }
            kind += 1;
        }
        kind = 0;
        while kind < FinishableFurnitureKind::ALL.len() {
            let mut variant = 0;
            while variant < FurnitureVariant::ALL.len() {
                let mut state = 1;
                while state < FurnitureWoodState::ALL.len() {
                    keys[cursor] = Self::wood(
                        FinishableFurnitureKind::ALL[kind],
                        FurnitureVariant::ALL[variant],
                        FurnitureWoodState::ALL[state],
                    );
                    cursor += 1;
                    state += 1;
                }
                variant += 1;
            }
            kind += 1;
        }
        keys
    };
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyFields {
    kind: FurnitureKind,
    variant: FurnitureVariant,
    wood_state: FurnitureWoodState,
}

impl From<FurnitureKey> for KeyFields {
    fn from(key: FurnitureKey) -> Self {
        Self {
            kind: key.kind,
            variant: key.variant,
            wood_state: key.wood_state,
        }
    }
}

impl TryFrom<KeyFields> for FurnitureKey {
    type Error = &'static str;
    fn try_from(fields: KeyFields) -> Result<Self, Self::Error> {
        if fields.wood_state == FurnitureWoodState::Natural {
            Ok(Self::natural(fields.kind, fields.variant))
        } else {
            FinishableFurnitureKind::from_kind(fields.kind)
                .map(|kind| Self::wood(kind, fields.variant, fields.wood_state))
                .ok_or("furniture kind has no authored wood treatment")
        }
    }
}
