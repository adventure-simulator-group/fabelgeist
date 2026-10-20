//! Select a construction explicitly; installation cannot reinterpret its channels.

use adventuresim_environment::prepared::GrownAuthoringContext;

use super::*;

pub(super) struct ConstructedGeometry {
    pub geometry: GrownGeometry,
    pub authoring_adapter: Option<AuthoringAdapter>,
}

#[derive(Serialize)]
pub(super) struct AuthoringAdapter {
    name: String,
    backend: String,
    device_type: String,
    driver: String,
    driver_info: String,
}

impl Role {
    pub(super) fn construct(
        self,
        bank: &Arc<TreeBank>,
        source: PlantId,
        level: Level,
        budget: GrownMeshBudget,
    ) -> Result<ConstructedGeometry> {
        match self {
            Self::Reference => Ok(ConstructedGeometry {
                geometry: bank.address_grown(source, level, budget)?,
                authoring_adapter: None,
            }),
            Self::Candidate => {
                let mut author = pollster::block_on(GrownAuthoringContext::new())?;
                let adapter = author.adapter_info();
                let geometry =
                    pollster::block_on(bank.expand_grown(&mut author, source, level, budget))?;
                // Release this device before the tactical renderer starts.
                drop(author);
                Ok(ConstructedGeometry {
                    geometry,
                    authoring_adapter: Some(AuthoringAdapter {
                        name: adapter.name,
                        backend: format!("{:?}", adapter.backend),
                        device_type: format!("{:?}", adapter.device_type),
                        driver: adapter.driver,
                        driver_info: adapter.driver_info,
                    }),
                })
            }
        }
    }
}
