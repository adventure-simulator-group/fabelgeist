//! One request-owned bank admission task, then one population installation.

use std::sync::Arc;

use adventuresim_environment::{
    prepared::{PreparedError, SharedForest, TreeBank},
    render::{
        ForestDetail,
        benchmark::{CompletionPolicy, ForestBenchmark, ForestRequestId, ValidationPolicy},
        trees::TreeFeed,
    },
};
use anyhow::{Result, ensure};
use bevy::prelude::*;

use super::super::{
    bridge::{self, Command},
    wire::BankTransfer,
};
use super::{REQUEST_DEADLINE, Session, State};
use crate::browser_runtime::queue_foliage;

impl Session {
    pub(super) fn begin_payload(
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
        ensure!(
            arrival.started.elapsed() < REQUEST_DEADLINE,
            "foliage request deadline exceeded"
        );
        match arrival.document.transfer {
            BankTransfer::Download => {
                ensure!(
                    self.bank.is_none() && !bank.is_empty(),
                    "bank already resident or missing download"
                );
                arrival.state = State::BankAdmitting;
                wasm_bindgen_futures::spawn_local(async move {
                    let outcome = TreeBank::from_bytes(&bank).await.map(Arc::new);
                    if queue_foliage(Command::BankAdmitted {
                        request,
                        outcome,
                        population,
                    })
                    .is_err()
                    {
                        bridge::renderer_failed();
                    }
                });
                Ok(())
            }
            BankTransfer::Resident => {
                ensure!(bank.is_empty(), "resident request supplied bank bytes");
                let bank = self
                    .bank
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("missing resident bank"))?;
                ensure!(
                    bank.id() == arrival.document.bank,
                    "resident bank identity mismatch"
                );
                self.install_population(world, request, bank, population)
            }
        }
    }

    pub(super) fn complete_bank(
        &mut self,
        world: &mut World,
        request: ForestRequestId,
        outcome: std::result::Result<Arc<TreeBank>, PreparedError>,
        population: Vec<u8>,
    ) -> Result<()> {
        let Some(arrival) = self.arrival.as_ref().filter(|arrival| {
            arrival.request == request && matches!(arrival.state, State::BankAdmitting)
        }) else {
            return Ok(());
        };
        ensure!(
            arrival.started.elapsed() < REQUEST_DEADLINE,
            "foliage request deadline exceeded"
        );
        let bank = outcome?;
        self.install_population(world, request, bank, population)
    }

    fn install_population(
        &mut self,
        world: &mut World,
        request: ForestRequestId,
        bank: Arc<TreeBank>,
        population: Vec<u8>,
    ) -> Result<()> {
        let arrival = self
            .arrival
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("missing request"))?;
        ensure!(
            arrival.request == request
                && matches!(arrival.state, State::Fetching | State::BankAdmitting),
            "stale population installation"
        );
        ensure!(
            arrival.started.elapsed() < REQUEST_DEADLINE,
            "foliage request deadline exceeded"
        );
        ensure!(
            bank.id() == arrival.document.bank,
            "bank content identity mismatch"
        );
        let forest = Arc::new(SharedForest::from_population_bytes(
            Arc::clone(&bank),
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
        let mut instances = forest.expected_instances();
        if !forest.bank().library().detail.leaves {
            instances.leaves = 0;
        }
        probe.expect_instances(instances);
        // CPU validation may cross the original deadline. Nothing from this
        // candidate may enter the World or session residency after expiry.
        ensure!(
            arrival.started.elapsed() < REQUEST_DEADLINE,
            "foliage request deadline exceeded"
        );
        world.insert_resource(ForestDetail {
            ..forest.bank().library().detail
        });
        // Publish residency only after both complete payload identities and
        // population semantics have passed against this exact admitted bank.
        self.bank = Some(bank);
        probe.activate(world);
        world.resource_mut::<TreeFeed>().show(forest);
        arrival.probe = Some(probe);
        arrival.state = State::Warming;
        Ok(())
    }
}
