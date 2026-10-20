//! Checked source selection/allocation planning, with whole-record partitions.

use anyhow::{Context, Result, ensure};
use fabelgeist_botany::identity::PlantId;
use fabelgeist_gpu::prelude::BufferByteLength;

use super::{FullExpansionBudget, geometry::RecordGeometry, gpu::NativeExpansion};
use crate::prepared::{RecordSpan, TreeBank, VariantIndex};

const MESH_VERTEX_BYTES: u64 =
    (size_of::<[f32; 3]>() * 2 + size_of::<[f32; 2]>() + size_of::<u32>()) as u64;

pub(super) struct ExpansionPlan {
    pub variant: VariantIndex,
    pub parts: Vec<PartPlan>,
    pub mesh_bytes: BufferByteLength,
}

pub(super) struct PartPlan {
    pub records: RecordSpan,
    pub geometry: RecordGeometry,
    pub vertices: u32,
    pub indices: u32,
}

struct SourceParts {
    variant: VariantIndex,
    records: [RecordShape; 2],
}

struct RecordShape {
    records: RecordSpan,
    geometry: RecordGeometry,
}

impl ExpansionPlan {
    pub fn new(
        bank: &TreeBank,
        source: PlantId,
        budget: FullExpansionBudget,
        gpu: &NativeExpansion,
    ) -> Result<Self> {
        let selected = SourceParts::from_bank(bank, source)?;
        let mut parts = Vec::new();
        let mut total = 0_u64;
        for shape in selected.records {
            let bytes = u64::from(shape.records.count)
                .checked_mul(
                    u64::from(shape.geometry.vertices_per_record()?) * MESH_VERTEX_BYTES
                        + u64::from(shape.geometry.indices_per_record()?) * size_of::<u32>() as u64,
                )
                .context("Full mesh byte overflow")?;
            total = total
                .checked_add(bytes)
                .context("Full total byte overflow")?;
            ensure!(
                total <= u64::from(budget.total_mesh_bytes),
                "Full mesh byte budget exceeded"
            );
            shape.partition(gpu, budget, &mut parts)?;
        }
        Ok(Self {
            variant: selected.variant,
            parts,
            mesh_bytes: total.into(),
        })
    }
}

impl SourceParts {
    fn from_bank(bank: &TreeBank, source: PlantId) -> Result<Self> {
        let library = bank.library();
        let mut matching = library
            .variant_indices()
            .zip(&library.models)
            .filter(|(_, model)| model.source == source);
        let (variant, model) = matching
            .next()
            .context("grown source is absent from bank")?;
        ensure!(matching.next().is_none(), "ambiguous grown source in bank");
        let species = library
            .species
            .get(model.species.word() as usize)
            .context("grown source species is absent")?;
        ensure!(
            model.branches.count > 0 || model.leaves.count > 0,
            "Full source is empty"
        );
        let [wood, leaves] = RecordGeometry::full(&library.detail, species.foliage);
        Ok(Self {
            variant,
            records: [
                RecordShape {
                    records: model.branches,
                    geometry: wood,
                },
                RecordShape {
                    records: model.leaves,
                    geometry: leaves,
                },
            ],
        })
    }
}

impl RecordShape {
    fn partition(
        self,
        gpu: &NativeExpansion,
        budget: FullExpansionBudget,
        parts: &mut Vec<PartPlan>,
    ) -> Result<()> {
        if self.records.count == 0 {
            return Ok(());
        }
        let vertices_per_record = self.geometry.vertices_per_record()?;
        let indices_per_record = self.geometry.indices_per_record()?;
        ensure!(
            vertices_per_record > 0 && indices_per_record > 0,
            "empty Full record topology"
        );
        let source_end = self
            .records
            .start
            .checked_add(self.records.count)
            .context("Full source span overflow")?;
        source_end
            .checked_mul(vertices_per_record)
            .context("Full integer address overflow")?;
        let cap = gpu
            .record_capacity(
                self.geometry.role(),
                vertices_per_record,
                self.geometry.record_stride(),
                budget.part_vertices.vertices(),
            )?
            .min(u32::MAX / indices_per_record);
        ensure!(cap > 0, "no complete Full record fits the part budget");
        let mut start = self.records.start;
        while start < source_end {
            let count = cap.min(source_end - start);
            parts.push(PartPlan {
                records: RecordSpan { start, count },
                geometry: self.geometry,
                vertices: count
                    .checked_mul(vertices_per_record)
                    .context("Full part vertex overflow")?,
                indices: count
                    .checked_mul(indices_per_record)
                    .context("Full part index overflow")?,
            });
            start = start
                .checked_add(count)
                .context("Full partition overflow")?;
        }
        ensure!(start == source_end, "incomplete Full partition coverage");
        Ok(())
    }
}

impl PartPlan {
    pub fn source_range(&self) -> Result<std::ops::Range<usize>> {
        let start = usize::try_from(self.records.start)?;
        let end = start
            .checked_add(usize::try_from(self.records.count)?)
            .context("Full source slice overflow")?;
        Ok(start..end)
    }
}
