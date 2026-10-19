//! Request/view/frame receipts for completed queue prefixes.

use std::{
    num::NonZeroU64,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use bevy::{prelude::Entity, render::view::RetainedViewEntity};

use super::{
    ForestBenchmark, ForestRequestId, NativeResidency, ValidationPhase, validation::RenderCheck,
};

/// Ordinal within one request; zero is never an observed render frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
pub struct ForestFrameId(NonZeroU64);

/// Queue completion is not pixel coverage or compositor presentation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
pub struct ForestFrameReceipt {
    request: ForestRequestId,
    frame: ForestFrameId,
    #[cfg_attr(
        feature = "prepared",
        serde(rename = "camera_entity_bits", serialize_with = "serialize_entity")
    )]
    camera: Entity,
    validation_phase: ValidationPhase,
    elapsed_from_request_micros: u64,
}

pub(super) struct CompletionTracker {
    policy: CompletionPolicy,
    next: AtomicU64,
    pub(super) receipts: Mutex<Receipts>,
}

#[derive(Default)]
pub(super) struct Receipts {
    first: Option<ForestFrameReceipt>,
    latest: Option<ForestFrameReceipt>,
    pub(super) final_validated: Option<CompletedValidation>,
}

/// Inventory and completion receipt from the exact same submitted render frame.
#[derive(Clone)]
pub(super) struct CompletedValidation {
    receipt: ForestFrameReceipt,
    pub(super) inventory: NativeResidency,
}

#[derive(Clone, Copy)]
pub(super) struct RenderObservation {
    pub request: ForestRequestId,
    pub frame: ForestFrameId,
    pub view: Option<RetainedViewEntity>,
    pub validation: RenderCheck,
}

/// Waited diagnostics serialize native frames; queued receipts let the event
/// loop run and acknowledge submitted work asynchronously on either backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
pub enum CompletionPolicy {
    WaitedDiagnostic,
    Queued,
}

impl CompletionTracker {
    pub(super) fn new(policy: CompletionPolicy) -> Self {
        Self {
            policy,
            next: AtomicU64::new(1),
            receipts: Mutex::new(Receipts::default()),
        }
    }

    pub(super) fn next(&self) -> Option<ForestFrameId> {
        NonZeroU64::new(self.next.fetch_add(1, Ordering::Relaxed)).map(ForestFrameId)
    }
}

impl ForestFrameReceipt {
    pub fn request(&self) -> ForestRequestId {
        self.request
    }
    pub fn frame(&self) -> ForestFrameId {
        self.frame
    }
    pub fn camera(&self) -> Entity {
        self.camera
    }
    pub fn validation_phase(&self) -> ValidationPhase {
        self.validation_phase
    }
}

impl ForestBenchmark {
    pub fn completion_policy(&self) -> CompletionPolicy {
        self.inner.completion.policy
    }

    pub fn first_completed_frame(&self) -> Option<ForestFrameReceipt> {
        self.inner
            .completion
            .receipts
            .lock()
            .ok()
            .and_then(|receipts| receipts.first)
    }

    pub fn latest_completed_frame(&self) -> Option<ForestFrameReceipt> {
        self.inner
            .completion
            .receipts
            .lock()
            .ok()
            .and_then(|receipts| receipts.latest)
    }

    /// Retained endpoint receipt cannot be overwritten by subsequent frames.
    pub fn final_completed_frame(&self) -> Option<ForestFrameReceipt> {
        self.inner.completion.receipts.lock().ok().and_then(|r| {
            r.final_validated
                .as_ref()
                .map(|completed| completed.receipt)
        })
    }

    pub(super) fn acknowledge_frame(
        &self,
        observation: RenderObservation,
        valid: bool,
        inventory: Option<NativeResidency>,
    ) {
        if observation.request != self.request_id() {
            self.failure();
            return;
        }
        let valid = valid && self.inner.failures.load(Ordering::Acquire) == 0;
        if !valid {
            return;
        }
        let Some(view) = observation.view else {
            self.failure();
            return;
        };
        let micros = self
            .inner
            .started
            .elapsed()
            .as_micros()
            .min(u64::MAX as u128) as u64;
        let receipt = ForestFrameReceipt {
            request: observation.request,
            frame: observation.frame,
            camera: view.main_entity.id(),
            validation_phase: observation.validation.phase(),
            elapsed_from_request_micros: micros.max(1),
        };
        let Ok(mut receipts) = self.inner.completion.receipts.lock() else {
            self.failure();
            return;
        };
        if !self.validation().acknowledge(observation.validation, valid) {
            return;
        }
        if receipt.validation_phase == ValidationPhase::FinalRequested {
            let Some(inventory) = inventory.filter(|inventory| inventory.frame == receipt.frame)
            else {
                self.failure();
                return;
            };
            if receipts.final_validated.is_none() {
                receipts.final_validated = Some(CompletedValidation { receipt, inventory });
            }
        }
        if receipts.first.is_none() {
            receipts.first = Some(receipt);
            self.inner
                .completed_micros
                .store(receipt.elapsed_from_request_micros, Ordering::Release);
        }
        if receipts
            .latest
            .is_none_or(|latest| latest.frame < receipt.frame)
        {
            receipts.latest = Some(receipt);
        }
    }
}

#[cfg(feature = "prepared")]
fn serialize_entity<S: serde::Serializer>(
    entity: &Entity,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(entity.to_bits())
}
