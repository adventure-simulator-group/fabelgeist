//! Interpret only the actual submitted indirect range and live counted prefix.
use super::*;
use bindings::Geometry;
use commands::{CommandCount, Dispatch, Receipt};
use decode::PhaseData;

const MAX_SUBMITTED_COMMANDS: usize = 4096;
const MAX_SUBMITTED_INSTANCES: usize = 16_384;

pub(super) struct WorkBudget {
    commands: usize,
    instances: usize,
}

pub(super) struct GpuCommand {
    // Absolute ordinal in this phase's actual indexed/nonindexed argument ABI.
    pub parameter: u32,
    pub geometry: Geometry,
    pub first_instance: u32,
    pub instance_count: u32,
}

impl WorkBudget {
    pub(super) fn new() -> Self {
        Self {
            commands: MAX_SUBMITTED_COMMANDS,
            instances: MAX_SUBMITTED_INSTANCES,
        }
    }

    fn commands(&mut self, count: usize) -> Result<()> {
        self.commands = self
            .commands
            .checked_sub(count)
            .context("submitted-command fixture budget")?;
        Ok(())
    }

    pub(super) fn instances(&mut self, count: u32) -> Result<()> {
        self.instances = self
            .instances
            .checked_sub(usize::try_from(count)?)
            .context("submitted-instance fixture budget")?;
        Ok(())
    }
}

impl GpuCommand {
    pub(super) fn read(
        receipt: &Receipt,
        data: &PhaseData,
        budget: &mut WorkBudget,
    ) -> Result<Vec<Self>> {
        let Dispatch::Indirect { parameters, count } = &receipt.dispatch else {
            anyhow::bail!("GPU command decoder requires an actual indirect dispatch");
        };
        let maximum = parameters
            .end
            .checked_sub(parameters.start)
            .context("reversed indirect range")?;
        ensure!(maximum > 0, "empty submitted indirect range");
        let indexed = receipt.binding.geometry.indexed();
        let live = match count {
            CommandCount::Range => maximum,
            CommandCount::BatchSet(index) => {
                let sets = if indexed {
                    &data.indexed_sets
                } else {
                    &data.non_indexed_sets
                };
                let set = sets
                    .get(usize::try_from(*index)?)
                    .context("batch-set read bounds")?;
                ensure!(
                    set.indirect_parameters_base == parameters.start,
                    "actual batch-set base differs from draw offset"
                );
                ensure!(
                    set.indirect_parameters_count <= maximum,
                    "actual count exceeds submitted maximum"
                );
                set.indirect_parameters_count
            }
        };
        budget.commands(usize::try_from(live)?)?;
        let end = parameters
            .start
            .checked_add(live)
            .context("live argument range overflow")?;
        let mut commands = Vec::with_capacity(usize::try_from(live)?);
        for parameter in parameters.start..end {
            let ordinal = usize::try_from(parameter)?;
            let command = match receipt.binding.geometry {
                Geometry::Indexed { format, .. } => {
                    let actual = data
                        .indexed
                        .get(ordinal)
                        .context("indexed argument read bounds")?;
                    Self {
                        parameter,
                        geometry: Geometry::Indexed {
                            format,
                            count: actual.index_count,
                            first_index: actual.first_index,
                            first_vertex: actual.base_vertex,
                        },
                        first_instance: actual.first_instance,
                        instance_count: actual.instance_count,
                    }
                }
                Geometry::NonIndexed { .. } => {
                    let actual = data
                        .non_indexed
                        .get(ordinal)
                        .context("nonindexed argument read bounds")?;
                    Self {
                        parameter,
                        geometry: Geometry::NonIndexed {
                            count: actual.vertex_count,
                            first_vertex: actual.base_vertex,
                        },
                        first_instance: actual.first_instance,
                        instance_count: actual.instance_count,
                    }
                }
            };
            commands.push(command);
        }
        Ok(commands)
    }
}
