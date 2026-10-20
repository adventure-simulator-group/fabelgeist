//! Checked source selection/allocation planning, with whole-record partitions.

use anyhow::{Context, Result, ensure};
use fabelgeist_botany::identity::PlantId;

use super::{GrownMeshBudget, GrownMeshBytes, geometry::RecordGeometry};
use crate::{
    prepared::{RecordSpan, TreeBank, VariantIndex},
    render::Level,
};

const MESH_VERTEX_BYTES: u64 =
    (size_of::<[f32; 3]>() * 2 + size_of::<[f32; 2]>() + size_of::<u32>()) as u64;

pub(super) struct GeometryPlan {
    pub level: Level,
    pub variant: VariantIndex,
    pub parts: Vec<PartPlan>,
    pub mesh_bytes: GrownMeshBytes,
}

pub(super) struct PartPlan {
    pub level: Level,
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
    level: Level,
    records: RecordSpan,
    geometry: RecordGeometry,
}

impl GeometryPlan {
    pub fn new(
        bank: &TreeBank,
        source: PlantId,
        level: Level,
        budget: GrownMeshBudget,
        capacity: impl Fn(RecordGeometry) -> Result<u32>,
    ) -> Result<Self> {
        let selected = SourceParts::from_bank(bank, source, level)?;
        let mut parts = Vec::new();
        let mut total = 0_u64;
        for shape in selected.records {
            let bytes = u64::from(shape.records.count)
                .checked_mul(
                    u64::from(shape.geometry.vertices_per_record()?) * MESH_VERTEX_BYTES
                        + u64::from(shape.geometry.indices_per_record()?) * size_of::<u32>() as u64,
                )
                .context("grown mesh byte overflow")?;
            total = total
                .checked_add(bytes)
                .context("grown total byte overflow")?;
            ensure!(
                total <= budget.total_mesh_bytes.bytes(),
                "grown mesh byte budget exceeded"
            );
            shape.partition(&capacity, &mut parts)?;
        }
        Ok(Self {
            level,
            variant: selected.variant,
            parts,
            mesh_bytes: GrownMeshBytes::from_bytes(total),
        })
    }
}

impl SourceParts {
    fn from_bank(bank: &TreeBank, source: PlantId, level: Level) -> Result<Self> {
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
            "grown source is empty"
        );
        let [wood, leaves] = RecordGeometry::at_level(&library.detail, species.foliage, level);
        Ok(Self {
            variant,
            records: [
                RecordShape {
                    level,
                    records: model.branches,
                    geometry: wood,
                },
                RecordShape {
                    level,
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
        capacity: &impl Fn(RecordGeometry) -> Result<u32>,
        parts: &mut Vec<PartPlan>,
    ) -> Result<()> {
        if self.records.count == 0 {
            return Ok(());
        }
        let vertices_per_record = self.geometry.vertices_per_record()?;
        let indices_per_record = self.geometry.indices_per_record()?;
        ensure!(
            vertices_per_record > 0 && indices_per_record > 0,
            "empty grown record topology"
        );
        let source_end = self
            .records
            .start
            .checked_add(self.records.count)
            .context("grown source span overflow")?;
        source_end
            .checked_mul(vertices_per_record)
            .context("grown integer address overflow")?;
        let cap = capacity(self.geometry)?.min(u32::MAX / indices_per_record);
        ensure!(cap > 0, "no complete grown record fits the part budget");
        let mut start = self.records.start;
        while start < source_end {
            let count = cap.min(source_end - start);
            parts.push(PartPlan {
                level: self.level,
                records: RecordSpan { start, count },
                geometry: self.geometry,
                vertices: count
                    .checked_mul(vertices_per_record)
                    .context("grown part vertex overflow")?,
                indices: count
                    .checked_mul(indices_per_record)
                    .context("grown part index overflow")?,
            });
            start = start
                .checked_add(count)
                .context("grown partition overflow")?;
        }
        ensure!(start == source_end, "incomplete grown partition coverage");
        Ok(())
    }
}

#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
impl PartPlan {
    pub fn source_range(&self) -> Result<std::ops::Range<usize>> {
        let start = usize::try_from(self.records.start)?;
        let end = start
            .checked_add(usize::try_from(self.records.count)?)
            .context("grown source slice overflow")?;
        Ok(start..end)
    }
}
