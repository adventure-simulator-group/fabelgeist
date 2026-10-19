//! Serial arrivals with immutable banks and fresh request-owned probes.

use super::{
    bridge::{self, Command},
    capture::{self, CaptureRole, PendingCapture, Readback},
    fault::Injection,
    scene,
    wire::{BankTransfer, Document},
};
use adventuresim_environment::{
    prepared::{SharedForest, TreeBank},
    render::{
        ForestDetail,
        benchmark::{
            CompletionPolicy, ForestBenchmark, ForestRequestId, ValidationPhase, ValidationPolicy,
            WindowStart,
        },
        trees::{TreeFeed, TreeMeshStats},
    },
};
use anyhow::{Result, ensure};
use bevy::prelude::*;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use web_time::Instant;

const REQUEST_DEADLINE: Duration = Duration::from_secs(90);
const RETIREMENT_TIME: Duration = Duration::from_secs(1);
const RETIREMENT_FRAMES: usize = 60;

#[derive(Resource, Default)]
pub(super) struct Session {
    pub bank: Option<Arc<TreeBank>>,
    pub camera: Option<Entity>,
    pub arrival: Option<Arrival>,
    error: Option<String>,
    baseline: Option<super::inventory::AssetInventory>,
}

pub(super) struct Arrival {
    pub request: ForestRequestId,
    pub document: Document,
    pub started: Instant,
    pub probe: Option<ForestBenchmark>,
    pub pending: Option<PendingCapture>,
    pub readbacks: Vec<Readback>,
    pub injection: Injection,
    inventory: Option<super::inventory::AssetInventory>,
    state: State,
}

enum State {
    Fetching,
    Warming,
    Draining { started: Instant, frames: usize },
    Finalizing,
    Capturing,
    Complete,
}

impl Session {
    pub fn is_active(&self) -> bool {
        self.arrival.is_some()
    }
    pub fn fail(&mut self, error: String) {
        self.error = Some(error.clone());
        bridge::publish(
            json!({"state":"failed", "request":self.arrival.as_ref().map(|a| a.request),
            "error":error, "fault":self.arrival.as_ref().map(|a| a.injection),
            "readbacks":self.arrival.as_ref().map(|a| &a.readbacks),
            "pending_capture":self.arrival.as_ref().and_then(|a| a.pending.as_ref()).map(PendingCapture::evidence)}),
            true,
        );
    }
    pub fn apply(&mut self, world: &mut World, command: Command) -> Result<()> {
        ensure!(
            self.error.is_none() && !bridge::has_render_failure(),
            "sticky renderer/review failure"
        );
        match command {
            Command::Begin {
                request,
                document,
                started,
            } => {
                ensure!(
                    self.arrival
                        .as_ref()
                        .is_none_or(|a| matches!(a.state, State::Complete)),
                    "overlapping foliage request"
                );
                let view = document.camera.clone();
                if self.camera.is_none() {
                    scene::lighting(world);
                }
                self.camera = Some(scene::place(world, self.camera, view));
                self.arrival = Some(Arrival {
                    request,
                    document: *document,
                    started,
                    probe: None,
                    pending: None,
                    readbacks: Vec::new(),
                    injection: Injection::default(),
                    inventory: None,
                    state: State::Fetching,
                });
            }
            Command::Payload {
                request,
                bank,
                population,
            } => self.admit(world, request, bank, population)?,
            Command::Fault { request, fault } => {
                let arrival = self
                    .arrival
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("ownership fault has no arrival"))?;
                ensure!(
                    arrival.request == request
                        && matches!(arrival.state, State::Draining { .. })
                        && arrival.pending.is_none()
                        && arrival.readbacks.len() == 1,
                    "ownership fault requires the current post-early drain"
                );
                arrival.injection.arm(fault)?;
            }
            Command::Failure { request, message } => {
                ensure!(
                    self.arrival.as_ref().is_some_and(|a| a.request == request),
                    "stale failure"
                );
                anyhow::bail!("browser transport/device failure: {message}");
            }
        }
        Ok(())
    }
    fn admit(
        &mut self,
        world: &mut World,
        request: ForestRequestId,
        bank: Vec<u8>,
        population: Vec<u8>,
    ) -> Result<()> {
        let arrival = self
            .arrival
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("missing request"))?;
        ensure!(
            arrival.request == request && matches!(arrival.state, State::Fetching),
            "stale payload"
        );
        match arrival.document.transfer {
            BankTransfer::Download => {
                ensure!(
                    self.bank.is_none() && !bank.is_empty(),
                    "bank already resident or missing download"
                );
                let admitted = TreeBank::from_bytes(&bank)?;
                ensure!(
                    admitted.id() == arrival.document.bank,
                    "bank content identity mismatch"
                );
                self.bank = Some(Arc::new(admitted));
            }
            BankTransfer::Resident => {
                ensure!(bank.is_empty(), "resident request supplied bank bytes")
            }
        }
        let bank = self
            .bank
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing resident bank"))?;
        ensure!(
            bank.id() == arrival.document.bank,
            "resident bank identity mismatch"
        );
        let forest = Arc::new(SharedForest::from_population_bytes(
            Arc::clone(bank),
            &population,
        )?);
        ensure!(
            forest.id() == arrival.document.population,
            "population content identity mismatch"
        );
        ensure!(
            forest.population().frame.terrain.is_some(),
            "complete diagnostic ground required"
        );
        let probe = ForestBenchmark::new(
            arrival.started,
            ValidationPolicy::SamplingEndpoints,
            request,
            CompletionPolicy::Queued,
        );
        world.insert_resource(ForestDetail {
            ..forest.bank().library().detail
        });
        let mut instances = forest.expected_instances();
        if !forest.bank().library().detail.leaves {
            instances.leaves = 0;
        }
        probe.expect_instances(instances);
        probe.activate(world);
        world.resource_mut::<TreeFeed>().show(forest);
        arrival.probe = Some(probe);
        arrival.state = State::Warming;
        Ok(())
    }
    fn progress(&mut self, world: &mut World) -> Result<()> {
        let Some(arrival) = &mut self.arrival else {
            bridge::booted();
            return Ok(());
        };
        ensure!(
            !bridge::has_render_failure(),
            "renderer reported a pipeline failure"
        );
        if matches!(arrival.state, State::Complete) {
            return Ok(());
        }
        ensure!(
            arrival.started.elapsed() < REQUEST_DEADLINE,
            "foliage request deadline exceeded"
        );
        let Some(probe) = arrival.probe.clone() else {
            return Ok(());
        };
        ensure!(
            !probe.status().failed,
            "forest GPU ownership/readiness failed"
        );
        let camera = self
            .camera
            .ok_or_else(|| anyhow::anyhow!("missing review camera"))?;
        if let Some(receipt) = probe.latest_completed_frame() {
            ensure!(
                receipt.request() == arrival.request && receipt.camera() == camera,
                "completed frame belongs to another request/view"
            );
        }
        match &mut arrival.state {
            State::Warming if probe.first_completed_frame().is_some() => {
                if arrival.readbacks.is_empty() {
                    capture::request(world, arrival, camera, CaptureRole::Early)?;
                } else if matches!(probe.validation().begin_sampling(), WindowStart::Opened) {
                    arrival.state = State::Draining {
                        started: Instant::now(),
                        frames: 0,
                    };
                }
            }
            State::Draining { started, frames } => {
                *frames += 1;
                if *frames >= RETIREMENT_FRAMES && started.elapsed() >= RETIREMENT_TIME {
                    probe.validation().finish_sampling();
                    arrival.state = State::Finalizing;
                }
            }
            State::Finalizing if probe.validation().finish_sampling() => {
                ensure!(
                    probe
                        .final_completed_frame()
                        .is_some_and(|r| r.validation_phase() == ValidationPhase::FinalRequested),
                    "missing final-phase receipt"
                );
                capture::request(world, arrival, camera, CaptureRole::SettledFirst)?;
                arrival.state = State::Capturing;
            }
            State::Capturing if arrival.pending.is_none() => {
                if arrival.readbacks.len() == 2 {
                    capture::request(world, arrival, camera, CaptureRole::SettledSecond)?;
                } else if arrival.readbacks.len() == 3 {
                    capture::validate_inventory(world, arrival)?;
                    let inventory = super::inventory::AssetInventory::snapshot(world, arrival)?;
                    if let Some(baseline) = &self.baseline {
                        inventory.within(baseline)?;
                    }
                    self.baseline.get_or_insert(inventory.clone());
                    arrival.inventory = Some(inventory);
                    arrival.state = State::Complete;
                }
            }
            _ => {}
        }
        self.publish(world);
        Ok(())
    }
    fn publish(&self, world: &World) {
        let Some(arrival) = &self.arrival else {
            return;
        };
        let complete = matches!(arrival.state, State::Complete);
        let probe = arrival.probe.as_ref();
        let counts = probe.map(ForestBenchmark::counts);
        bridge::publish(
            json!({"state": if complete { "complete" } else { "rendering" },
            "request":arrival.request, "bank":arrival.document.bank,
            "pending_capture":arrival.pending.as_ref().map(PendingCapture::evidence),
            "fault":arrival.injection,
            "population":arrival.document.population, "readbacks":arrival.readbacks,
            "assets":arrival.inventory,
            "camera":arrival.document.camera.evidence(),
            "first_completed_frame":probe.and_then(ForestBenchmark::first_completed_frame),
            "latest_completed_frame":probe.and_then(ForestBenchmark::latest_completed_frame),
            "final_completed_frame":probe.and_then(ForestBenchmark::final_completed_frame),
            "validation":probe.and_then(|p|p.validation().evidence()),
            "residency":world.get_resource::<TreeMeshStats>().and_then(|s|s.residency.as_ref()),
            "native_residency":probe.and_then(ForestBenchmark::native_residency),
            "queued_writes":counts.as_ref().map(|c|c.queued_writes),
            "submitted_writes":counts.as_ref().map(|c|c.submitted_writes),
            "uploaded_bytes":counts.as_ref().map(|c|c.uploaded_bytes),
            "gpu_failures":counts.as_ref().map(|c|c.gpu_failures)}),
            complete,
        );
    }
}

pub(super) fn advance(world: &mut World) {
    world.resource_scope(|world, mut session: Mut<Session>| {
        if session.error.is_some() {
            return;
        }
        if let Err(error) = session.progress(world) {
            session.fail(error.to_string());
        }
    });
}
