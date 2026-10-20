//! Exact shader-codec decoding of bounded actual storage snapshots.
use super::*;
use bevy::{
    pbr::MeshUniform,
    render::{
        batching::gpu_preprocessing::{
            IndirectBatchSet, IndirectParametersIndexed, IndirectParametersNonIndexed,
        },
        render_resource::{ShaderType, encase},
    },
};
use snapshots::SnapshotBytes;
use types::{ColorPhase, SnapshotKind};

pub(super) struct PhaseData {
    pub phase: ColorPhase,
    pub meshes: Vec<MeshUniform>,
    pub indexed: Vec<IndirectParametersIndexed>,
    pub non_indexed: Vec<IndirectParametersNonIndexed>,
    pub indexed_sets: Vec<IndirectBatchSet>,
    pub non_indexed_sets: Vec<IndirectBatchSet>,
}

impl PhaseData {
    pub(super) fn decode(snapshots: &[SnapshotBytes]) -> Result<Vec<Self>> {
        let mut phases = Vec::new();
        for phase in [ColorPhase::Opaque, ColorPhase::AlphaMask] {
            let snapshots: Vec<_> = snapshots
                .iter()
                .filter(|snapshot| snapshot.key.phase == phase)
                .collect();
            if snapshots.is_empty() {
                continue;
            }
            for (index, snapshot) in snapshots.iter().enumerate() {
                ensure!(
                    snapshots[..index]
                        .iter()
                        .all(|earlier| std::mem::discriminant(&earlier.key.kind)
                            != std::mem::discriminant(&snapshot.key.kind)),
                    "duplicate decoded storage kind"
                );
            }
            let mut meshes = None;
            let mut indexed = Vec::new();
            let mut non_indexed = Vec::new();
            let mut indexed_sets = Vec::new();
            let mut non_indexed_sets = Vec::new();
            for snapshot in snapshots {
                match snapshot.key.kind {
                    SnapshotKind::MeshOutput { logical_instances } => {
                        let stride = <Vec<MeshUniform> as ShaderType>::METADATA.stride().get();
                        let required = u64::try_from(logical_instances)?
                            .checked_mul(stride)
                            .context("mesh decode size overflow")?;
                        ensure!(
                            logical_instances > 0
                                && required == u64::try_from(snapshot.data.len())?,
                            "partial/wrong mesh output length"
                        );
                        let decoded: Vec<MeshUniform> =
                            encase::StorageBuffer::new(snapshot.data.as_slice()).create()?;
                        ensure!(
                            decoded.len() == logical_instances,
                            "mesh output count mismatch"
                        );
                        meshes = Some(decoded);
                    }
                    SnapshotKind::IndexedArguments => indexed = pod_records(&snapshot.data)?,
                    SnapshotKind::NonIndexedArguments => non_indexed = pod_records(&snapshot.data)?,
                    SnapshotKind::IndexedBatchSets => indexed_sets = pod_records(&snapshot.data)?,
                    SnapshotKind::NonIndexedBatchSets => {
                        non_indexed_sets = pod_records(&snapshot.data)?
                    }
                }
            }
            phases.push(Self {
                phase,
                meshes: meshes.context("actual mesh output missing")?,
                indexed,
                non_indexed,
                indexed_sets,
                non_indexed_sets,
            });
        }
        ensure!(!phases.is_empty(), "no decoded GPU phases");
        Ok(phases)
    }
}

fn pod_records<T: bytemuck::Pod>(bytes: &[u8]) -> Result<Vec<T>> {
    ensure!(
        !bytes.is_empty() && bytes.len().is_multiple_of(size_of::<T>()),
        "partial GPU argument record"
    );
    Ok(bytes
        .chunks_exact(size_of::<T>())
        .map(bytemuck::pod_read_unaligned)
        .collect())
}
