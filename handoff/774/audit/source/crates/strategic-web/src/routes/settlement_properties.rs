//! Authoritative property provisioning for ordinary residence actions.
use super::{AppState, SettlementView};
use adventuresim_core::settlement_property::GeneratedHomeCatalog;
use adventuresim_stdb_client::SettlementPropertyManifest;
use std::sync::{Arc, Mutex, OnceLock};

const CACHED_CATALOG_LIMIT: usize = 64;
const GENERATION_CONCURRENCY: usize = 2;

pub(super) async fn ensure(state: &AppState, settlement: &SettlementView) -> anyhow::Result<()> {
    // Cache immutable native products per profile, never legal or occupancy state.
    static CATALOGS: OnceLock<
        Mutex<std::collections::BTreeMap<String, Arc<GeneratedHomeCatalog>>>,
    > = OnceLock::new();
    let key = serde_json::to_string(&(
        settlement.id.as_str(),
        settlement.population_level,
        settlement.population_estimate,
        &settlement.economy,
    ))?;
    let cache = CATALOGS.get_or_init(Mutex::default);
    let cached = cache
        .lock()
        .map_err(|_| anyhow::anyhow!("Property cache unavailable"))?
        .get(&key)
        .cloned();
    let catalog = if let Some(catalog) = cached {
        catalog
    } else {
        let settlement = settlement.clone();
        static PERMITS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
        let permit = Arc::clone(
            PERMITS.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(GENERATION_CONCURRENCY))),
        )
        .acquire_owned()
        .await?;
        let generated = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            adventuresim_tactical_server_dispatcher::settlement_properties::generated_homes(
                &settlement.id,
                settlement.population_level,
                settlement.population_estimate,
                &settlement.economy,
            )
        })
        .await?
        .map_err(anyhow::Error::msg)?;
        let catalog = Arc::new(generated);
        let mut cache = cache
            .lock()
            .map_err(|_| anyhow::anyhow!("Property cache unavailable"))?;
        if cache.len() >= CACHED_CATALOG_LIMIT {
            cache.clear();
        }
        cache.insert(key, Arc::clone(&catalog));
        catalog
    };
    ensure_catalog(state, &catalog).await
}

pub(super) async fn ensure_catalog(
    state: &AppState,
    catalog: &GeneratedHomeCatalog,
) -> anyhow::Result<()> {
    let existing = state
        .db
        .query_one_sats::<SettlementPropertyManifest>(&format!(
            "SELECT * FROM settlement_property_manifest WHERE settlement_id = {}",
            crate::spacetimedb::sql_string_literal(&catalog.settlement_id)
        ))
        .await?;
    if let Some(existing) = existing {
        anyhow::ensure!(
            existing.digest == catalog.digest()?,
            "Registered physical properties differ from the generated catalog"
        );
    } else {
        state
            .db
            .call(
                "register_settlement_properties",
                &[serde_json::json!(serde_json::to_string(catalog)?)],
            )
            .await?;
    }
    Ok(())
}
