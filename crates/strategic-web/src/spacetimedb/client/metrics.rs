//! Cumulative SQL accounting and the shared database-request latency policy.

use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct QueryRequestCount(u64);

impl From<u64> for QueryRequestCount {
    fn from(requests: u64) -> Self {
        Self(requests)
    }
}

impl QueryRequestCount {
    fn delta(self, before: Self) -> Self {
        Self(self.0.saturating_sub(before.0))
    }
}

impl fmt::Debug for QueryRequestCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct QueryElapsedMicros(u64);

impl From<u64> for QueryElapsedMicros {
    fn from(micros: u64) -> Self {
        Self(micros)
    }
}

impl From<DatabaseRequestLatency> for QueryElapsedMicros {
    fn from(latency: DatabaseRequestLatency) -> Self {
        // Preserve the native counter's truncation and unsigned narrowing.
        Self(latency.0.as_micros() as u64)
    }
}

impl QueryElapsedMicros {
    fn delta(self, before: Self) -> Self {
        Self(self.0.saturating_sub(before.0))
    }
}

impl fmt::Debug for QueryElapsedMicros {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct QueryMetricsSnapshot {
    requests: QueryRequestCount,
    elapsed_micros: QueryElapsedMicros,
}

impl QueryMetricsSnapshot {
    pub(crate) fn delta(self, before: Self) -> Self {
        Self {
            requests: self.requests.delta(before.requests),
            elapsed_micros: self.elapsed_micros.delta(before.elapsed_micros),
        }
    }
}

#[derive(Default)]
pub(super) struct QueryMetrics {
    requests: AtomicU64,
    elapsed_micros: AtomicU64,
}

impl QueryMetrics {
    pub(super) fn begin_query(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_latency(&self, latency: DatabaseRequestLatency) {
        let elapsed = QueryElapsedMicros::from(latency);
        self.elapsed_micros.fetch_add(elapsed.0, Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> QueryMetricsSnapshot {
        QueryMetricsSnapshot {
            requests: QueryRequestCount::from(self.requests.load(Ordering::Acquire)),
            elapsed_micros: QueryElapsedMicros::from(self.elapsed_micros.load(Ordering::Acquire)),
        }
    }
}

/// Send-to-response-headers latency, including unsuccessful sends.
#[derive(Clone, Copy)]
pub(super) struct DatabaseRequestLatency(Duration);

impl From<Duration> for DatabaseRequestLatency {
    fn from(elapsed: Duration) -> Self {
        Self(elapsed)
    }
}

impl DatabaseRequestLatency {
    pub(super) fn class(self) -> RequestLatencyClass {
        if self.0 >= Duration::from_millis(250) {
            RequestLatencyClass::Slow
        } else {
            RequestLatencyClass::Ordinary
        }
    }
}

impl fmt::Debug for DatabaseRequestLatency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RequestLatencyClass {
    Ordinary,
    Slow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread};

    #[tokio::test]
    async fn failed_sql_sends_count_once_and_reducer_sends_do_not_count_as_queries() {
        use crate::spacetimedb::{SpacetimeClient, SpacetimeError};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let client = SpacetimeClient::new(format!("http://{address}"), "test").unwrap();
        let before = client.query_metrics();
        let result = client
            .query_sats::<adventuresim_stdb_client::SettlementAlias>(
                "SELECT * FROM settlement_alias".into(),
            )
            .await;
        assert!(matches!(result, Err(SpacetimeError::Http(_))));
        let after = client.query_metrics();
        assert_eq!(after.requests, QueryRequestCount::from(1));
        assert_eq!(after.delta(before).requests, QueryRequestCount::from(1));
        assert!(matches!(
            client.call("unused_reducer", &[]).await,
            Err(SpacetimeError::Http(_))
        ));
        assert_eq!(client.query_metrics(), after);
    }

    #[test]
    fn query_metrics_are_monotonic_and_clone_safe() {
        let metrics = Arc::new(QueryMetrics::default());
        for _ in 0..3 {
            metrics.begin_query();
        }
        metrics.record_latency(DatabaseRequestLatency::from(Duration::from_micros(125)));
        let before = QueryMetricsSnapshot {
            requests: QueryRequestCount::from(3),
            elapsed_micros: QueryElapsedMicros::from(125),
        };
        assert_eq!(metrics.snapshot(), before);
        assert_eq!(
            metrics.snapshot().delta(before),
            QueryMetricsSnapshot::default()
        );
        let clone = Arc::clone(&metrics);
        clone.begin_query();
        assert_eq!(
            metrics.snapshot().delta(before).requests,
            QueryRequestCount::from(1)
        );
    }

    #[test]
    fn injected_latency_measurement_delta_is_deterministic() {
        let before = QueryMetricsSnapshot::default();
        let after = QueryMetricsSnapshot {
            requests: QueryRequestCount::from(3),
            elapsed_micros: QueryElapsedMicros::from(425_000),
        };
        assert_eq!(after.delta(before), after);
    }

    #[test]
    fn backwards_snapshots_saturate_each_quantity_independently() {
        let before = QueryMetricsSnapshot {
            requests: QueryRequestCount::from(5),
            elapsed_micros: QueryElapsedMicros::from(100),
        };
        let after = QueryMetricsSnapshot {
            requests: QueryRequestCount::from(3),
            elapsed_micros: QueryElapsedMicros::from(120),
        };
        assert_eq!(
            after.delta(before),
            QueryMetricsSnapshot {
                requests: QueryRequestCount::default(),
                elapsed_micros: QueryElapsedMicros::from(20),
            }
        );
        assert_eq!(
            before.delta(after),
            QueryMetricsSnapshot {
                requests: QueryRequestCount::from(2),
                elapsed_micros: QueryElapsedMicros::default(),
            }
        );
    }

    #[test]
    fn latency_counts_complete_micros_and_preserves_native_unsigned_words() {
        let metrics = QueryMetrics::default();
        for nanos in [999, 1_001, 2_999] {
            metrics.record_latency(DatabaseRequestLatency::from(Duration::from_nanos(nanos)));
        }
        assert_eq!(
            metrics.snapshot().elapsed_micros,
            QueryElapsedMicros::from(3)
        );
        let wide = DatabaseRequestLatency::from(Duration::from_secs(u64::MAX));
        assert_eq!(
            QueryElapsedMicros::from(wide),
            QueryElapsedMicros::from(18_446_744_073_708_551_616)
        );
        metrics.requests.store(u64::MAX, Ordering::Relaxed);
        metrics.elapsed_micros.store(u64::MAX, Ordering::Relaxed);
        metrics.begin_query();
        metrics.record_latency(DatabaseRequestLatency::from(Duration::from_micros(2)));
        assert_eq!(
            metrics.snapshot(),
            QueryMetricsSnapshot {
                requests: QueryRequestCount::default(),
                elapsed_micros: QueryElapsedMicros::from(1),
            }
        );
    }

    #[test]
    fn concurrent_accounting_retains_every_request_and_completed_microsecond() {
        let metrics = Arc::new(QueryMetrics::default());
        let mut workers = Vec::new();
        for _ in 0..4 {
            let metrics = Arc::clone(&metrics);
            workers.push(thread::spawn(move || -> QueryMetricsSnapshot {
                for _ in 0..100 {
                    metrics.begin_query();
                    metrics.record_latency(DatabaseRequestLatency::from(Duration::from_micros(7)));
                }
                metrics.snapshot()
            }));
        }
        for worker in workers {
            assert_ne!(worker.join().unwrap(), QueryMetricsSnapshot::default());
        }
        assert_eq!(
            metrics.snapshot(),
            QueryMetricsSnapshot {
                requests: QueryRequestCount::from(400),
                elapsed_micros: QueryElapsedMicros::from(2_800),
            }
        );
    }

    #[test]
    fn warning_admission_uses_header_latency_at_the_exact_threshold() {
        assert_eq!(
            DatabaseRequestLatency::from(Duration::from_nanos(249_999_999)).class(),
            RequestLatencyClass::Ordinary
        );
        assert_eq!(
            DatabaseRequestLatency::from(Duration::from_millis(250)).class(),
            RequestLatencyClass::Slow
        );
        assert_eq!(
            DatabaseRequestLatency::from(Duration::from_millis(251)).class(),
            RequestLatencyClass::Slow
        );
    }

    #[test]
    fn tracing_keeps_existing_numeric_and_duration_diagnostics() {
        assert_eq!(
            format!(
                "{:?}",
                QueryMetricsSnapshot {
                    requests: QueryRequestCount::from(3),
                    elapsed_micros: QueryElapsedMicros::from(125),
                }
            ),
            "QueryMetricsSnapshot { requests: 3, elapsed_micros: 125 }"
        );
        assert_eq!(
            format!(
                "{:?}",
                DatabaseRequestLatency::from(Duration::from_millis(250))
            ),
            "250ms"
        );
    }
}
