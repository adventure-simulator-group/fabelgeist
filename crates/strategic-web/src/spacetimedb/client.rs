//! SpacetimeDB HTTP client wrapper

use reqwest::Client;
use serde_json::Value;
use spacetimedb_sats::de::DeserializeOwned as SatsDeserializeOwned;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use super::projection::ViewProjectionError;
use super::{queries::SqlQuery, types::QueryResponse};

mod cardinality;
mod error;
mod metrics;
mod sats;
use cardinality::{QueryCardinality, QueryRowCount};
use error::SingleRowQueryError;
pub(crate) use error::{DatabaseOperation, RemoteDatabaseFailure, SpacetimeError};
use metrics::{DatabaseRequestLatency, QueryMetrics, QueryMetricsSnapshot, RequestLatencyClass};
use sats::decode_sats_query_response;

pub(crate) type Result<T> = std::result::Result<T, SpacetimeError>;

/// HTTP client for SpacetimeDB
#[derive(Clone)]
pub(crate) struct SpacetimeClient {
    http: Client,
    base_url: String,
    database: String,
    token: Option<String>,
    metrics: Arc<QueryMetrics>,
}

impl SpacetimeClient {
    /// Create a new SpacetimeDB client
    pub(crate) fn new(base_url: impl Into<String>, database: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(3))
                .build()?,
            base_url: base_url.into(),
            database: database.into(),
            token: None,
            metrics: Arc::new(QueryMetrics::default()),
        })
    }

    /// Set the auth token
    pub(crate) fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    /// Snapshot cumulative wrapping SQL counters without resetting shared state.
    /// After a controlled request, `after.delta(before)` reports a saturated
    /// difference. Concurrent requests contribute to the same counters.
    pub(crate) fn query_metrics(&self) -> QueryMetricsSnapshot {
        self.metrics.snapshot()
    }

    async fn query_response(&self, sql: &SqlQuery) -> Result<QueryResponse> {
        self.metrics.begin_query();
        let url = format!("{}/v1/database/{}/sql", self.base_url, self.database);
        let mut request = self.http.post(&url).body(sql.request_body());
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let started = Instant::now();
        let response = request.send().await;
        let elapsed = DatabaseRequestLatency::from(started.elapsed());
        self.metrics.record_latency(elapsed);
        if elapsed.class() == RequestLatencyClass::Slow {
            tracing::warn!(?elapsed, query = %sql, "slow SpacetimeDB query");
        }
        let response = response?;
        let status = response.status();
        if !status.is_success() {
            return Err(RemoteDatabaseFailure::from_response(
                DatabaseOperation::Query,
                status,
                response.text().await,
            )
            .into());
        }

        let text = response.text().await?;
        serde_json::from_str(&text).map_err(SpacetimeError::QueryResponseDecode)
    }

    /// Run a SQL query and decode exact generated rows through SATS rather
    /// than through hand-maintained serde mirrors.
    pub(crate) async fn query_sats<T: SatsDeserializeOwned>(
        &self,
        sql: SqlQuery,
    ) -> Result<Vec<T>> {
        let query_response = self.query_response(&sql).await?;
        decode_sats_query_response(&query_response).map_err(SpacetimeError::Sats)
    }

    /// Run a generated-row query that should return at most one row.
    pub(crate) async fn query_one_sats<T: SatsDeserializeOwned>(
        &self,
        sql: SqlQuery,
    ) -> Result<Option<T>> {
        let response = self.query_response(&sql).await?;
        let mut rows = decode_sats_query_response(&response).map_err(SpacetimeError::Sats)?;
        let received = QueryRowCount::from(rows.len());
        if received.cardinality() == QueryCardinality::Multiple {
            return Err(SingleRowQueryError::from_query(sql, received).into());
        }
        Ok(rows.pop())
    }

    /// Decode exact generated rows first, then apply an explicit presentation
    /// projection. This keeps schema validation on the generated SATS owner.
    pub(crate) async fn query_sats_into<T, U>(&self, sql: SqlQuery) -> Result<Vec<U>>
    where
        T: SatsDeserializeOwned,
        U: TryFrom<T>,
        U::Error: Into<ViewProjectionError>,
    {
        self.query_sats(sql)
            .await?
            .into_iter()
            .map(|row| U::try_from(row).map_err(|error| SpacetimeError::Projection(error.into())))
            .collect()
    }

    /// Decode and project a generated query that should return at most one row.
    pub(crate) async fn query_one_sats_into<T, U>(&self, sql: SqlQuery) -> Result<Option<U>>
    where
        T: SatsDeserializeOwned,
        U: TryFrom<T>,
        U::Error: Into<ViewProjectionError>,
    {
        self.query_one_sats(sql)
            .await?
            .map(U::try_from)
            .transpose()
            .map_err(|error| SpacetimeError::Projection(error.into()))
    }

    /// Call a reducer with JSON arguments
    pub(crate) async fn call(&self, reducer: &str, args: &[Value]) -> Result<()> {
        let url = format!(
            "{}/v1/database/{}/call/{}",
            self.base_url, self.database, reducer
        );

        let mut request = self.http.post(&url).json(args);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let started = Instant::now();
        let response = request.send().await;
        let elapsed = DatabaseRequestLatency::from(started.elapsed());
        if elapsed.class() == RequestLatencyClass::Slow {
            tracing::warn!(?elapsed, reducer, "slow SpacetimeDB reducer call");
        }
        let response = response?;
        let status = response.status();
        if !status.is_success() {
            return Err(RemoteDatabaseFailure::from_response(
                DatabaseOperation::Reducer,
                status,
                response.text().await,
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, extract::State, routing::post};
    use std::future::IntoFuture;

    #[derive(Clone, Copy)]
    enum AliasFixture {
        Empty,
        Singleton,
        Multiple,
    }

    async fn alias_sql_fixture(
        State(case): State<AliasFixture>,
        body: axum::body::Bytes,
    ) -> Json<Value> {
        assert_eq!(body.as_ref(), b"SELECT * FROM settlement_alias");
        let option = serde_json::json!({"Sum": {"variants": [
            {"name": {"some": "some"}, "algebraic_type": {"String": []}},
            {"name": {"some": "none"}, "algebraic_type": {"Product": {"elements": []}}}
        ]}});
        let row = serde_json::json!(["alias-1", "settlement-1", "Harbour", [1, []], [1, []]]);
        let rows = match case {
            AliasFixture::Empty => vec![],
            AliasFixture::Singleton => vec![row],
            AliasFixture::Multiple => vec![row.clone(), row],
        };
        Json(serde_json::json!([{
            "schema": {"elements": [
                {"name": {"some": "id"}, "algebraic_type": {"String": []}},
                {"name": {"some": "settlement_id"}, "algebraic_type": {"String": []}},
                {"name": {"some": "name"}, "algebraic_type": {"String": []}},
                {"name": {"some": "prefix"}, "algebraic_type": option},
                {"name": {"some": "language"}, "algebraic_type": option}
            ]},
            "rows": rows
        }]))
    }

    #[tokio::test]
    async fn http_generated_queries_preserve_empty_single_and_multiple_row_contracts() {
        for case in [
            AliasFixture::Empty,
            AliasFixture::Singleton,
            AliasFixture::Multiple,
        ] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let app = Router::new()
                .route("/v1/database/test/sql", post(alias_sql_fixture))
                .with_state(case);
            let server = tokio::spawn(axum::serve(listener, app).into_future());
            let client = SpacetimeClient::new(format!("http://{address}"), "test").unwrap();
            let result = client
                .query_one_sats::<adventuresim_stdb_client::SettlementAlias>(
                    "SELECT * FROM settlement_alias".into(),
                )
                .await;
            match case {
                AliasFixture::Empty => assert!(result.unwrap().is_none()),
                AliasFixture::Singleton => assert_eq!(result.unwrap().unwrap().name, "Harbour"),
                AliasFixture::Multiple => {
                    let error = result.unwrap_err();
                    assert!(matches!(error, SpacetimeError::Cardinality(_)));
                    assert_eq!(
                        error.to_string(),
                        "query expected at most one row but returned 2: SELECT * FROM settlement_alias"
                    );
                    let rows = client
                        .query_sats::<adventuresim_stdb_client::SettlementAlias>(
                            "SELECT * FROM settlement_alias".into(),
                        )
                        .await
                        .unwrap();
                    assert_eq!(rows.len(), 2);
                    assert!(rows.iter().all(|row| row.name == "Harbour"));
                }
            }
            server.abort();
        }
    }

    #[test]
    fn cloned_clients_share_accounting_without_resetting_snapshots() {
        let client = SpacetimeClient::new("http://localhost:3000", "test").unwrap();
        let before = client.query_metrics();
        let clone = client.clone();
        clone.metrics.begin_query();
        clone
            .metrics
            .record_latency(DatabaseRequestLatency::from(Duration::from_micros(125)));
        assert_eq!(client.query_metrics(), clone.query_metrics());
        assert_ne!(
            client.query_metrics().delta(before),
            QueryMetricsSnapshot::default()
        );
        assert_eq!(
            client.query_metrics().delta(clone.query_metrics()),
            QueryMetricsSnapshot::default()
        );
    }
}
