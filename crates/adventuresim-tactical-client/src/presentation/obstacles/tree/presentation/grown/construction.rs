//! Preflight the complete CPU catalogue before inserting its first asset.

use super::*;
use adventuresim_environment::prepared::GrownGeometry;

struct PendingModel {
    source: GrownSource,
    levels: GrownLevels<GrownGeometry>,
}

struct RemainingMeshBudget {
    budget: GrownMeshBudget,
    remaining: GrownMeshBytes,
}

impl GrownCatalogueAssets {
    pub(in super::super) fn build(
        request: &GrownSceneRequest,
        reuse_bank: Option<Arc<GrownBankAssets>>,
        world: &mut World,
    ) -> Result<Arc<Self>> {
        GrownAssets::validate_target(world)?;
        if let Some(bank) = &reuse_bank {
            bank.validate_world(world)?;
            ensure!(
                bank.bank().id() == request.bank.id(),
                "grown reuse bank mismatch"
            );
        }
        let mut allowance = RemainingMeshBudget {
            budget: request.budget,
            remaining: request.budget.total_mesh_bytes,
        };
        let mut pending = Vec::with_capacity(request.sources.len());
        for source in &request.sources {
            let full = allowance.construct(&request.bank, source.plant, Level::Full)?;
            let middle = allowance.construct(&request.bank, source.plant, Level::Middle)?;
            let coarse = allowance.construct(&request.bank, source.plant, Level::Coarse)?;
            pending.push(PendingModel {
                source: *source,
                levels: GrownLevels {
                    full,
                    middle,
                    coarse,
                },
            });
        }
        let logical_mesh_bytes = request
            .budget
            .total_mesh_bytes
            .checked_sub(allowance.remaining)
            .context("grown catalogue byte accounting")?;
        // No buffer/material/mesh insertion occurs until every CPU closure fits.
        let bank = match reuse_bank {
            Some(bank) => bank,
            None => GrownBankAssets::upload(request.bank.clone(), world)?,
        };
        let mut models = Vec::with_capacity(pending.len());
        for model in pending {
            let full = GrownAssets::install(model.levels.full, bank.clone(), world)?;
            let middle = GrownAssets::install(model.levels.middle, bank.clone(), world)?;
            let coarse = GrownAssets::install(model.levels.coarse, bank.clone(), world)?;
            models.push(GrownModelAssets {
                source: model.source,
                levels: GrownLevels {
                    full,
                    middle,
                    coarse,
                },
            });
        }
        Ok(Arc::new(Self {
            bank,
            sources: request.sources.clone(),
            part_vertices: request.budget.part_vertices,
            logical_mesh_bytes,
            models,
        }))
    }
}

impl RemainingMeshBudget {
    fn construct(
        &mut self,
        bank: &Arc<TreeBank>,
        source: PlantId,
        level: Level,
    ) -> Result<GrownGeometry> {
        let geometry = bank.address_grown(
            source,
            level,
            GrownMeshBudget {
                part_vertices: self.budget.part_vertices,
                total_mesh_bytes: self.remaining,
            },
        )?;
        self.remaining = self
            .remaining
            .checked_sub(geometry.mesh_bytes())
            .context("grown catalogue byte accounting")?;
        Ok(geometry)
    }
}
