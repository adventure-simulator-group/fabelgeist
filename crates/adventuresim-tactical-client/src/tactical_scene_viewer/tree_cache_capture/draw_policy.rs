//! Explicit content-fidelity and ordinary GPU proof requests.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(super) enum DrawMethod {
    ContentSingletons,
    OrdinaryGpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum BatchControl {
    Ordinary,
    ForcedSingletons,
    HiddenLastRoot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "control", rename_all = "snake_case")]
pub(super) enum DrawPolicy {
    #[default]
    ContentSingletons,
    OrdinaryGpu(BatchControl),
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RootVisibility {
    #[default]
    Normal,
    LastRootHidden,
}

impl RootVisibility {
    pub(super) fn for_view(policy: DrawPolicy, view: usize) -> Self {
        if policy.hides_last_root() && view > 0 {
            Self::LastRootHidden
        } else {
            Self::Normal
        }
    }
}

impl DrawPolicy {
    pub(super) fn admit(method: DrawMethod, control: Option<BatchControl>) -> Result<Self> {
        match (method, control) {
            (DrawMethod::ContentSingletons, None) => Ok(Self::ContentSingletons),
            (DrawMethod::OrdinaryGpu, Some(control)) => Ok(Self::OrdinaryGpu(control)),
            (DrawMethod::ContentSingletons, Some(_)) => {
                anyhow::bail!("batch control requires ordinary GPU proof")
            }
            (DrawMethod::OrdinaryGpu, None) => {
                anyhow::bail!("ordinary GPU proof requires explicit batch control")
            }
        }
    }

    pub(super) fn singleton_instrumentation(self) -> bool {
        matches!(
            self,
            Self::ContentSingletons | Self::OrdinaryGpu(BatchControl::ForcedSingletons)
        )
    }

    pub(super) fn retains_cpu_content(self) -> bool {
        self == Self::ContentSingletons
    }

    pub(super) fn hides_last_root(self) -> bool {
        self == Self::OrdinaryGpu(BatchControl::HiddenLastRoot)
    }
}
