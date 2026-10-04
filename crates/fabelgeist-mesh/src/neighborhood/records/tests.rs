use super::*;
use crate::PrimitiveTopology;
#[derive(serde::Deserialize)]
struct AdjacencyFixture {
    vertices: u32,
    topology: PrimitiveTopology,
    indices: Vec<u32>,
    adjacency: Vec<Vec<u32>>,
}
#[test]
fn original_adjacency_sets_capacity_counts_and_zero_padding() {
    let fixtures: Vec<AdjacencyFixture> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/topology_contracts.json"
    ))
    .unwrap();
    assert_eq!(fixtures.len(), 270);
    for fixture in fixtures {
        let mut indices = Vec::new();
        for word in fixture.indices {
            indices.push(DrawVertexIndex::from(word));
        }
        let edges = MeshEdges::from_indices(indices, fixture.topology);
        let rows = MeshAdjacency::from_edges(fixture.vertices.into(), &edges);
        let mut decoded = Vec::new();
        for row in &rows.0 {
            let mut words = Vec::new();
            for &index in row {
                words.push(u32::from(index));
            }
            words.sort_unstable();
            decoded.push(words);
        }
        assert_eq!(decoded, fixture.adjacency);
        for capacity in [0usize, 1, 3] {
            let records = NeighborRecords::from_adjacency(&rows, NeighborCapacity::from(capacity));
            let stride = 1 + capacity;
            assert_eq!(records.0.len(), rows.0.len() * stride);
            for (vertex, row) in decoded.iter().enumerate() {
                let base = vertex * stride;
                let count = row.len().min(capacity);
                assert_eq!(records.0[base] as usize, count);
                for &word in &records.0[base + 1..base + 1 + count] {
                    assert!(row.contains(&word));
                }
                for word in &records.0[base + 1 + count..base + stride] {
                    assert_eq!(*word, 0);
                }
            }
        }
    }
}
