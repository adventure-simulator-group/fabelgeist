//! Successful process-local road topology loading and sharing.
//!
//! Generated SpacetimeDB rows are admitted at the native decoding boundary.
//! Routing retains the wire node IDs and metre lengths in its integer kernel.
//! Heap priority pairs are native distance/node keys, preserving tie ordering.

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::StatusCode, routing::post};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn failed_road_query_retries_then_concurrent_loads_share_success() {
        let requests = Arc::new(AtomicUsize::new(0));
        let recorded_requests = Arc::clone(&requests);
        let fixture = Router::new().route(
            "/v1/database/test/sql",
            post(move |body: String| {
                let requests = Arc::clone(&recorded_requests);
                async move {
                    assert_eq!(body, "SELECT * FROM travel_edge");
                    let status = if requests.fetch_add(1, Ordering::SeqCst) == 0 {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::OK
                    };
                    (status, axum::Json(serde_json::json!([])))
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, fixture).await.unwrap();
        });
        let database = SpacetimeClient::new(address, "test").unwrap();
        let cache = TravelEdgeCache::new();
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            assert!(cache.load(&database).await.is_none());
            let (first, second) = tokio::join!(cache.load(&database), cache.load(&database));
            let first = first.unwrap();
            let second = second.unwrap();
            assert!(first.is_empty());
            assert!(Arc::ptr_eq(&first, &second));
            assert!(Arc::ptr_eq(&first, &cache.load(&database).await.unwrap()));
            assert_eq!(requests.load(Ordering::SeqCst), 2);
            assert_eq!(database.query_metrics().requests, 2);
        })
        .await
        .unwrap();
        server.abort();
    }
}
use super::{TravelDestination, settlement_destination};
use crate::spacetimedb::{SettlementView, SpacetimeClient, SqlQuery};
use adventuresim_core::strategic_time::OVERLAND_WALKING_SPEED_KM_PER_HOUR;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
};

pub(crate) static TRAVEL_EDGE_CACHE: TravelEdgeCache = TravelEdgeCache::new();

#[derive(Clone, Copy)]
pub(crate) struct TravelEdgeTopology {
    pub(super) from_node_id: u64,
    pub(super) to_node_id: u64,
    pub(super) length_m: u32,
}

impl From<adventuresim_stdb_client::TravelEdge> for TravelEdgeTopology {
    fn from(edge: adventuresim_stdb_client::TravelEdge) -> Self {
        Self {
            from_node_id: edge.from_node_id,
            to_node_id: edge.to_node_id,
            length_m: edge.length_m,
        }
    }
}

/// One successful load is shared; failed initialization may be retried.
pub(crate) struct TravelEdgeCache {
    edges: tokio::sync::OnceCell<Arc<Vec<TravelEdgeTopology>>>,
}

impl TravelEdgeCache {
    const fn new() -> Self {
        Self {
            edges: tokio::sync::OnceCell::const_new(),
        }
    }

    pub(crate) async fn load(&self, db: &SpacetimeClient) -> Option<Arc<Vec<TravelEdgeTopology>>> {
        match self
            .edges
            .get_or_try_init(|| async {
                // Project only at the SQL/HTTP decoding port, keeping its query owner.
                db.query_sats::<adventuresim_stdb_client::TravelEdge>(
                    SqlQuery::travel_edges().as_str(),
                )
                .await
                .map(|edges| Arc::new(edges.into_iter().map(TravelEdgeTopology::from).collect()))
            })
            .await
        {
            Ok(edges) => Some(Arc::clone(edges)),
            Err(error) => {
                tracing::warn!(%error, "failed to load travel edge cache");
                None
            }
        }
    }
}

pub(crate) fn connected_destinations(
    origin: &SettlementView,
    settlements: &[SettlementView],
    edges: &[TravelEdgeTopology],
) -> Vec<TravelDestination> {
    let Some(origin_node) = origin.source_node_id else {
        return settlements
            .iter()
            .filter(|settlement| settlement.id != origin.id)
            .cloned()
            .map(|settlement| {
                let distance_km = ((origin.longitude - settlement.longitude).powi(2)
                    + (origin.latitude - settlement.latitude).powi(2))
                .sqrt()
                .ceil() as u64;
                let distance_m = distance_km.saturating_mul(1_000);
                // Preserve the inherited degree-distance estimate as kilometres.
                // Native arithmetic supplies metre/minute presentation fields.
                let journey_minutes = distance_m
                    .saturating_mul(60)
                    .div_ceil(OVERLAND_WALKING_SPEED_KM_PER_HOUR * 1_000)
                    .max(1);
                settlement_destination(settlement, distance_m, journey_minutes)
            })
            .collect();
    };
    let settlement_nodes: HashSet<u64> = settlements
        .iter()
        .filter_map(|settlement| settlement.source_node_id)
        .collect();
    let settlements_by_node: HashMap<u64, &SettlementView> = settlements
        .iter()
        .filter_map(|settlement| settlement.source_node_id.map(|node| (node, settlement)))
        .collect();
    let mut adjacency: HashMap<u64, Vec<(u64, u32)>> = HashMap::new();
    for edge in edges {
        adjacency
            .entry(edge.from_node_id)
            .or_default()
            .push((edge.to_node_id, edge.length_m));
        adjacency
            .entry(edge.to_node_id)
            .or_default()
            .push((edge.from_node_id, edge.length_m));
    }
    let mut distances = HashMap::from([(origin_node, 0_u64)]);
    let mut pending = BinaryHeap::from([std::cmp::Reverse((0_u64, origin_node))]);
    let mut destinations = Vec::new();
    while let Some(std::cmp::Reverse((distance_m, node))) = pending.pop() {
        if distances
            .get(&node)
            .is_some_and(|known| *known != distance_m)
        {
            continue;
        }
        if node != origin_node && settlement_nodes.contains(&node) {
            if let Some(settlement) = settlements_by_node.get(&node) {
                let settlement = (*settlement).clone();
                // Metres and kilometres per hour become rounded-up minutes.
                // Saturation and the one-minute minimum retain routing policy.
                let journey_minutes = distance_m
                    .saturating_mul(60)
                    .div_ceil(OVERLAND_WALKING_SPEED_KM_PER_HOUR * 1_000)
                    .max(1);
                destinations.push(settlement_destination(
                    settlement,
                    distance_m,
                    journey_minutes,
                ));
            }
            continue;
        }
        for (neighbor, edge_length_m) in adjacency.get(&node).into_iter().flatten() {
            let next_distance = distance_m.saturating_add(u64::from(*edge_length_m));
            if distances
                .get(neighbor)
                .is_none_or(|known| next_distance < *known)
            {
                distances.insert(*neighbor, next_distance);
                pending.push(std::cmp::Reverse((next_distance, *neighbor)));
            }
        }
    }
    destinations.sort_by_key(|destination| destination.distance_m);
    destinations
}
