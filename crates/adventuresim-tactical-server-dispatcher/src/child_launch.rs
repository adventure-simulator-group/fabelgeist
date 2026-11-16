//! A claimed scene starts without inheriting the gateway's credentials.

use adventuresim_stdb_client::{
    ReducerEventContext, record_tactical_scene_provisioning_failure, revoke_tactical_server_claim,
};
use std::{
    collections::HashSet,
    net::SocketAddr,
    process::Command,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

pub(crate) struct ChildLaunch {
    pub(crate) instance_id: String,
    pub(crate) scene_key: String,
    pub(crate) scene_input: String,
    pub(crate) addr: SocketAddr,
    pub(crate) bin: String,
    pub(crate) stdb_url: String,
    pub(crate) stdb_module: String,
    pub(crate) claim: String,
    pub(crate) arguments: Vec<String>,
}

pub(crate) struct ExitedLaunch {
    pub(crate) instance_id: String,
    pub(crate) outcome: ChildExitOutcome,
}

#[derive(PartialEq, Eq)]
pub(crate) enum ChildExitOutcome {
    Clean,
    Failed,
}

impl ChildLaunch {
    pub(crate) fn start(
        self,
        ctx: &ReducerEventContext,
        spawned: Arc<Mutex<HashSet<String>>>,
        exited: Arc<Mutex<Vec<ExitedLaunch>>>,
    ) {
        let mut command = self.command();
        match command.spawn() {
            Ok(mut child) => {
                info!(instance_id = %self.instance_id, pid = child.id(), "Spawned tactical scene");
                std::thread::spawn(move || {
                    let outcome = match child.wait() {
                        Ok(status) => {
                            info!(instance_id = %self.instance_id, %status, "Tactical scene exited");
                            if status.success() {
                                ChildExitOutcome::Clean
                            } else {
                                ChildExitOutcome::Failed
                            }
                        }
                        Err(error) => {
                            error!(%error, "Failed to observe child exit");
                            ChildExitOutcome::Failed
                        }
                    };
                    if let Ok(mut exited) = exited.lock() {
                        exited.push(ExitedLaunch {
                            instance_id: self.instance_id,
                            outcome,
                        });
                    }
                });
            }
            Err(error) => {
                error!(%error, "Failed to spawn tactical scene");
                if let Err(error) = ctx
                    .reducers
                    .record_tactical_scene_provisioning_failure(self.instance_id.clone())
                {
                    error!(%error, "Could not report scene launch failure");
                }
                if let Ok(mut spawned) = spawned.lock() {
                    spawned.remove(&self.instance_id);
                }
                if let Err(error) = ctx.reducers.revoke_tactical_server_claim(self.instance_id) {
                    error!(%error, "Failed to revoke unused tactical claim");
                }
            }
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.bin);
        command
            .env_remove("SPACETIMEDB_TOKEN")
            .env("ADVENTURESIM_TACTICAL_CLAIM", &self.claim)
            .args([
                "--instance-id",
                &self.instance_id,
                "--scene-key",
                &self.scene_key,
                "--scene-input",
                &self.scene_input,
                "--addr",
                &self.addr.to_string(),
                "--spacetimedb-url",
                &self.stdb_url,
                "--spacetimedb-module",
                &self.stdb_module,
            ])
            .args(&self.arguments);
        command
    }
}

pub(crate) fn reclaim_pending(ctx: &adventuresim_stdb_client::SubscriptionEventContext) {
    use adventuresim_stdb_client::{TacticalServerRequestTableAccess, spacetimedb_sdk::Table};
    // A previous dispatcher can disappear before its child consumes a
    // claim. Pending slots have no registered serving authority yet.
    for request in ctx.db.tactical_server_request().iter() {
        if let Err(error) = ctx
            .reducers
            .revoke_tactical_server_claim(request.instance_id)
        {
            error!(%error, "Could not reclaim a pending scene launch");
        }
    }
}
