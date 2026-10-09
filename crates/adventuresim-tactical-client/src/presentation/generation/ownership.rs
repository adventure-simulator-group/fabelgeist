//! Separate mutable preparations from installed assets and other scene owners.
use super::{
    GeneratedTacticalScene, PreparationError, PreparationResult, PreparedProducts,
    PresentationOwner, PresentationOwners, TacticalSceneInput,
};
use adventuresim_building_generator::BuildingProgram;
use adventuresim_tactical_core::regional_city::RegionalCityInput;
use serde::{Deserialize, Serialize};
use std::{
    num::NonZeroU32,
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, OnceLock},
};

static RESIDENCY: OnceLock<Mutex<GenerationResidency>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparationTicket {
    owner: PresentationOwner,
    sequence: NonZeroU32,
}

#[derive(Default)]
struct GenerationResidency {
    owners: PresentationOwners<OwnerResidency>,
    next_sequence: u32,
    resident_facades: Vec<BuildingProgram>,
}

#[derive(Default)]
struct OwnerResidency {
    active: PreparedProducts,
    staged: PreparedProducts,
    preparing: Option<PreparationTicket>,
    completed: Option<PreparedSnapshot>,
}

struct PreparedSnapshot {
    ticket: PreparationTicket,
    input: PreparationInputIdentity,
    products: PreparedProducts,
}

/// Admitted from the canonical scene input digest, never an arbitrary string.
#[derive(PartialEq)]
enum PreparationInputIdentity {
    Scene(String),
    RegionalCity(Box<RegionalCityInput>),
}

pub(super) struct ProductAccess {
    residency: MutexGuard<'static, GenerationResidency>,
    selection: ProductSelection,
}

enum ProductSelection {
    Active(PresentationOwner),
    Staged(PreparationTicket),
}

impl GenerationResidency {
    fn owner(&self, owner: PresentationOwner) -> &OwnerResidency {
        self.owners.get(owner)
    }

    fn owner_mut(&mut self, owner: PresentationOwner) -> &mut OwnerResidency {
        self.owners.get_mut(owner)
    }
}

impl PreparationTicket {
    pub(super) fn require_owner(self, owner: PresentationOwner) -> PreparationResult<()> {
        if self.owner != owner {
            return Err(PreparationError::PreparationOwner);
        }
        Ok(())
    }
}

impl PreparationInputIdentity {
    fn from_input(input: &TacticalSceneInput) -> PreparationResult<Self> {
        input.validate()?;
        Ok(Self::Scene(input.digest()?))
    }

    fn product_kind(&self) -> super::ProductKind {
        match self {
            Self::Scene(_) => super::ProductKind::Scene,
            Self::RegionalCity(_) => super::ProductKind::RegionalCity,
        }
    }
}

impl OwnerResidency {
    fn finish(
        &mut self,
        ticket: PreparationTicket,
        input: PreparationInputIdentity,
    ) -> PreparationResult<()> {
        if self.preparing != Some(ticket) {
            return Err(PreparationError::StalePreparation);
        }
        let prepared = match &input {
            PreparationInputIdentity::Scene(digest) => self
                .staged
                .scenes
                .iter()
                .any(|scene| scene.digest == *digest),
            PreparationInputIdentity::RegionalCity(document) => self
                .staged
                .regional_city
                .as_ref()
                .is_some_and(|city| city.document == **document),
        };
        if !prepared {
            return Err(PreparationError::NotPrepared {
                product: input.product_kind(),
            });
        }
        self.completed = Some(PreparedSnapshot {
            ticket,
            input,
            products: std::mem::take(&mut self.staged),
        });
        self.preparing = None;
        Ok(())
    }
}

impl ProductAccess {
    pub(super) fn resident_facades(&self) -> &[BuildingProgram] {
        &self.residency.resident_facades
    }
}

impl Deref for ProductAccess {
    type Target = PreparedProducts;

    fn deref(&self) -> &Self::Target {
        match self.selection {
            ProductSelection::Active(owner) => &self.residency.owner(owner).active,
            ProductSelection::Staged(ticket) => &self.residency.owner(ticket.owner).staged,
        }
    }
}

impl DerefMut for ProductAccess {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.selection {
            ProductSelection::Active(owner) => &mut self.residency.owner_mut(owner).active,
            ProductSelection::Staged(ticket) => &mut self.residency.owner_mut(ticket.owner).staged,
        }
    }
}

pub(super) fn begin(owner: PresentationOwner) -> PreparationResult<PreparationTicket> {
    let mut residency = residency()?;
    let sequence = residency
        .next_sequence
        .checked_add(1)
        .and_then(NonZeroU32::new)
        .ok_or(PreparationError::SequenceExhausted)?;
    residency.next_sequence = sequence.get();
    let ticket = PreparationTicket { owner, sequence };
    let slot = residency.owner_mut(owner);
    // Large immutable scene and geometry buffers are shared. Preparation owns
    // its mutable bindings; installed assets remain available until activation.
    slot.staged = slot.active.clone();
    slot.staged.begin_generation();
    slot.preparing = Some(ticket);
    Ok(ticket)
}

pub(super) fn finish(
    ticket: PreparationTicket,
    input: &TacticalSceneInput,
) -> PreparationResult<()> {
    ticket.require_owner(PresentationOwner::Scene)?;
    let input = PreparationInputIdentity::from_input(input)?;
    let mut residency = residency()?;
    residency.owner_mut(ticket.owner).finish(ticket, input)
}

pub(super) fn finish_city(
    ticket: PreparationTicket,
    document: &RegionalCityInput,
) -> PreparationResult<()> {
    ticket.require_owner(PresentationOwner::RegionalMap)?;
    residency()?.owner_mut(ticket.owner).finish(
        ticket,
        PreparationInputIdentity::RegionalCity(Box::new(document.clone())),
    )
}

pub(super) fn cancel(ticket: PreparationTicket) -> PreparationResult<()> {
    let mut residency = residency()?;
    let slot = residency.owner_mut(ticket.owner);
    if slot.preparing == Some(ticket) {
        slot.preparing = None;
        slot.staged = Default::default();
    }
    if slot
        .completed
        .as_ref()
        .is_some_and(|snapshot| snapshot.ticket == ticket)
    {
        slot.completed = None;
    }
    Ok(())
}

/// Only the Bevy installation consumer activates a completed preparation.
/// Starting another request cannot replace dependencies of pending old meshes.
pub(crate) fn activate(
    ticket: PreparationTicket,
    input: &TacticalSceneInput,
) -> PreparationResult<GeneratedTacticalScene> {
    ticket.require_owner(PresentationOwner::Scene)?;
    let mut residency = residency()?;
    let slot = residency.owner_mut(PresentationOwner::Scene);
    let snapshot = slot
        .completed
        .as_ref()
        .filter(|snapshot| snapshot.ticket == ticket)
        .ok_or(PreparationError::StalePreparation)?;
    if snapshot.input != PreparationInputIdentity::from_input(input)? {
        return Err(PreparationError::PreparationInputMismatch);
    }
    let scene = snapshot.products.generated_scene(input)?;
    if let Some(snapshot) = slot.completed.take() {
        slot.active = snapshot.products;
    }
    Ok(scene)
}

pub(in crate::presentation) fn activate_city(
    ticket: PreparationTicket,
    document: &RegionalCityInput,
) -> PreparationResult<std::sync::Arc<super::city::PreparedCityProduct>> {
    ticket.require_owner(PresentationOwner::RegionalMap)?;
    let mut residency = residency()?;
    let slot = residency.owner_mut(ticket.owner);
    let snapshot = slot
        .completed
        .as_ref()
        .filter(|snapshot| snapshot.ticket == ticket)
        .ok_or(PreparationError::StalePreparation)?;
    if !matches!(&snapshot.input,
        PreparationInputIdentity::RegionalCity(input) if **input == *document)
    {
        return Err(PreparationError::PreparationInputMismatch);
    }
    let city =
        snapshot
            .products
            .regional_city
            .as_ref()
            .cloned()
            .ok_or(PreparationError::NotPrepared {
                product: super::ProductKind::RegionalCity,
            })?;
    if let Some(snapshot) = slot.completed.take() {
        slot.active = snapshot.products;
    }
    Ok(city)
}

pub(super) fn staged_products(ticket: PreparationTicket) -> PreparationResult<ProductAccess> {
    let residency = residency()?;
    if residency.owner(ticket.owner).preparing != Some(ticket) {
        return Err(PreparationError::StalePreparation);
    }
    Ok(ProductAccess {
        residency,
        selection: ProductSelection::Staged(ticket),
    })
}

pub(super) fn active_products(owner: PresentationOwner) -> PreparationResult<ProductAccess> {
    Ok(ProductAccess {
        residency: residency()?,
        selection: ProductSelection::Active(owner),
    })
}

pub(in crate::presentation) fn retain_facade(program: &BuildingProgram) -> PreparationResult<()> {
    let mut residency = residency()?;
    if !residency.resident_facades.contains(program) {
        residency.resident_facades.push(program.clone());
    }
    Ok(())
}

pub(in crate::presentation) fn clear_residency() -> PreparationResult<()> {
    let mut residency = residency()?;
    // Issued identities never repeat, even when all presentation assets reset.
    let next_sequence = residency.next_sequence;
    *residency = GenerationResidency {
        next_sequence,
        ..Default::default()
    };
    Ok(())
}

fn residency() -> PreparationResult<MutexGuard<'static, GenerationResidency>> {
    RESIDENCY
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| PreparationError::ResidencyPoisoned)
}
