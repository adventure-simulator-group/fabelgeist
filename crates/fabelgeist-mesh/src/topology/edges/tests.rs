use super::*;
#[derive(serde::Deserialize)]
struct TopologyFixture {
    vertices: u32,
    topology: PrimitiveTopology,
    indices: Vec<u32>,
    target: u32,
    wireframe: Vec<[u32; 2]>,
    neighbor_lines: Vec<[u32; 2]>,
}
#[test]
fn original_edge_filtering_target_policy_and_empty_sentinel() {
    let fixtures: Vec<TopologyFixture> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/topology_contracts.json"
    ))
    .unwrap();
    assert_eq!(fixtures.len(), 270);
    for fixture in fixtures {
        let vertices = DrawVertexCount::from(fixture.vertices);
        let target = DrawVertexIndex::from(fixture.target);
        let mut indices = Vec::new();
        for index in fixture.indices {
            indices.push(DrawVertexIndex::from(index));
        }
        let edges = MeshEdges::from_indices(indices, fixture.topology);
        let lines = MeshLineUpload::wireframe(vertices, &edges);
        let mut pairs = lines.0.as_chunks::<2>().0.to_vec();
        pairs.sort_unstable();
        assert_eq!(pairs, fixture.wireframe);
        let lines = MeshLineUpload::neighbors(vertices, &edges, target);
        let mut pairs = lines.0.as_chunks::<2>().0.to_vec();
        pairs.sort_unstable();
        assert_eq!(pairs, fixture.neighbor_lines);
    }
}
#[test]
fn nominal_edges_preserve_the_original_word_hash_framing() {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let edge = DrawEdge::new(3u32.into(), u32::MAX.into());
    let mut nominal = DefaultHasher::new();
    edge.hash(&mut nominal);
    let mut original = DefaultHasher::new();
    (3u32, u32::MAX).hash(&mut original);
    assert_eq!(nominal.finish(), original.finish());
}
#[tokio::test]
async fn native_neighbor_records_clamp_counts_and_retain_partial_row_sentinel() {
    use crate::neighborhood::NeighborRecords;
    use crate::{DerivedMeshBuffer, MeshNeighborhood, MeshTopologyTransferError, NeighborCapacity};
    use fabelgeist_gpu::prelude::{BufferCreationError, BufferDefinition};
    use std::error::Error;
    let context = WgpuContext::new().await.unwrap();
    let source = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[99u32, 7, 8, 1]),
        BufferDefinition::storage(),
    )
    .unwrap();
    let records = NeighborRecords::read(&context, &source).await.unwrap();
    let capacity = NeighborCapacity::from(2);
    assert_eq!(
        records.neighbor_lines(capacity, 0u32.into()).0,
        vec![0, 7, 0, 8]
    );
    assert_eq!(records.neighbor_lines(capacity, 1u32.into()).0, vec![0, 0]);
    for output in [
        DerivedMeshBuffer::Neighborhood,
        DerivedMeshBuffer::Wireframe,
        DerivedMeshBuffer::NeighborLines,
    ] {
        let error = output
            .upload(&context, BufferUpload::from_elements(&[] as &[u32]))
            .unwrap_err();
        assert!(
            matches!(error,MeshTopologyTransferError::Allocation{output:actual,source:BufferCreationError::Empty} if actual==output)
        );
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<BufferCreationError>()
                .is_some()
        );
    }
    let malformed = source.with_logical_length(1u64.into()).unwrap();
    let error = MeshNeighborhood::build(
        &context,
        3u32.into(),
        PrimitiveTopology::TriangleList,
        Some(&malformed),
        capacity,
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        MeshTopologyTransferError::ReadIndices(ReadbackError::PartialElement { .. })
    ));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<ReadbackError>()
            .is_some()
    );
}
