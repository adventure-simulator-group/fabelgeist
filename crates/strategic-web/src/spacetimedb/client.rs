//! SpacetimeDB HTTP client wrapper

use reqwest::Client;
use serde_json::Value;
use spacetimedb_sats::de::DeserializeOwned as SatsDeserializeOwned;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use super::types::QueryResponse;

mod sats;
use sats::decode_sats_query_response;

#[derive(Debug, thiserror::Error)]
pub enum SpacetimeError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("SpacetimeDB error: {0}")]
    Spacetime(String),
}

impl SpacetimeError {
    pub(crate) fn reducer_code(
        &self,
    ) -> Option<adventuresim_core::reducer_error::ReducerErrorCode> {
        let Self::Spacetime(message) = self else {
            return None;
        };
        adventuresim_core::reducer_error::parse_reducer_error(message)
    }
}

pub type Result<T> = std::result::Result<T, SpacetimeError>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryMetricsSnapshot {
    pub requests: u64,
    pub elapsed_micros: u64,
}

impl QueryMetricsSnapshot {
    pub fn delta(self, before: Self) -> Self {
        Self {
            requests: self.requests.saturating_sub(before.requests),
            elapsed_micros: self.elapsed_micros.saturating_sub(before.elapsed_micros),
        }
    }
}

#[derive(Default)]
struct QueryMetrics {
    requests: AtomicU64,
    elapsed_micros: AtomicU64,
}

/// HTTP client for SpacetimeDB
#[derive(Clone)]
pub struct SpacetimeClient {
    http: Client,
    base_url: String,
    database: String,
    token: Option<String>,
    metrics: Arc<QueryMetrics>,
}

impl SpacetimeClient {
    /// Create a new SpacetimeDB client
    pub fn new(base_url: impl Into<String>, database: impl Into<String>) -> Result<Self> {
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
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    /// Return monotonic SQL counters. Take a snapshot before and after one
    /// controlled request, then call `after.delta(before)`. This is safe for
    /// concurrent requests; it deliberately does not destructively reset a
    /// process-global counter.
    pub fn query_metrics(&self) -> QueryMetricsSnapshot {
        QueryMetricsSnapshot {
            requests: self.metrics.requests.load(Ordering::Acquire),
            elapsed_micros: self.metrics.elapsed_micros.load(Ordering::Acquire),
        }
    }

    async fn query_response(&self, sql: &str) -> Result<QueryResponse> {
        self.metrics.requests.fetch_add(1, Ordering::Relaxed);
        let url = format!("{}/v1/database/{}/sql", self.base_url, self.database);
        let mut request = self.http.post(&url).body(sql.to_owned());
        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let started = Instant::now();
        let response = request.send().await;
        let elapsed = started.elapsed();
        self.metrics
            .elapsed_micros
            .fetch_add(elapsed.as_micros() as u64, Ordering::Relaxed);
        if elapsed >= Duration::from_millis(250) {
            tracing::warn!(?elapsed, query = %sql, "slow SpacetimeDB query");
        }
        let response = response?;
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SpacetimeError::Spacetime(error_text));
        }

        let text = response.text().await?;
        serde_json::from_str(&text).map_err(Into::into)
    }

    /// Run a SQL query and decode exact generated rows through SATS rather
    /// than through hand-maintained serde mirrors.
    pub async fn query_sats<T: SatsDeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let query_response = self.query_response(sql).await?;
        decode_sats_query_response(&query_response)
    }

    /// Run a generated-row query that should return at most one row.
    pub async fn query_one_sats<T: SatsDeserializeOwned>(&self, sql: &str) -> Result<Option<T>> {
        let mut rows = self.query_sats(sql).await?;
        if rows.len() > 1 {
            return Err(SpacetimeError::Spacetime(format!(
                "query expected at most one row but returned {}: {sql}",
                rows.len()
            )));
        }
        Ok(rows.pop())
    }

    /// Decode exact generated rows first, then apply an explicit presentation
    /// projection. This keeps schema validation on the generated SATS owner.
    pub async fn query_sats_into<T, U>(&self, sql: &str) -> Result<Vec<U>>
    where
        T: SatsDeserializeOwned,
        U: TryFrom<T>,
        U::Error: std::fmt::Display,
    {
        self.query_sats(sql)
            .await?
            .into_iter()
            .map(|row| {
                U::try_from(row).map_err(|error| {
                    SpacetimeError::Spacetime(format!(
                        "generated-row presentation conversion failed: {error}"
                    ))
                })
            })
            .collect()
    }

    /// Decode and project a generated query that should return at most one row.
    pub async fn query_one_sats_into<T, U>(&self, sql: &str) -> Result<Option<U>>
    where
        T: SatsDeserializeOwned,
        U: TryFrom<T>,
        U::Error: std::fmt::Display,
    {
        self.query_one_sats(sql)
            .await?
            .map(U::try_from)
            .transpose()
            .map_err(|error| {
                SpacetimeError::Spacetime(format!(
                    "generated-row presentation conversion failed: {error}"
                ))
            })
    }

    /// Call a reducer with JSON arguments
    pub async fn call(&self, reducer: &str, args: &[Value]) -> Result<()> {
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
        let elapsed = started.elapsed();
        if elapsed >= Duration::from_millis(250) {
            tracing::warn!(?elapsed, reducer, "slow SpacetimeDB reducer call");
        }
        let response = response?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SpacetimeError::Spacetime(error_text));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_metrics_are_monotonic_and_clone_safe() {
        let client = SpacetimeClient::new("http://localhost:3000", "test").unwrap();
        client.metrics.requests.store(3, Ordering::Relaxed);
        client.metrics.elapsed_micros.store(125, Ordering::Relaxed);
        assert_eq!(
            client.query_metrics(),
            QueryMetricsSnapshot {
                requests: 3,
                elapsed_micros: 125
            }
        );
        assert_eq!(
            client.query_metrics().delta(QueryMetricsSnapshot {
                requests: 3,
                elapsed_micros: 125
            }),
            QueryMetricsSnapshot::default()
        );
        let clone = client.clone();
        clone.metrics.requests.fetch_add(1, Ordering::Relaxed);
        assert_eq!(
            client
                .query_metrics()
                .delta(QueryMetricsSnapshot {
                    requests: 3,
                    elapsed_micros: 125
                })
                .requests,
            1
        );
    }

    #[test]
    fn injected_latency_measurement_delta_is_deterministic() {
        let before = QueryMetricsSnapshot::default();
        let after = QueryMetricsSnapshot {
            requests: 3,
            elapsed_micros: 425_000,
        };
        assert_eq!(
            after.delta(before),
            QueryMetricsSnapshot {
                requests: 3,
                elapsed_micros: 425_000
            }
        );
    }
}
