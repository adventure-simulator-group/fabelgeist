//! Immutable application owners shared by strategic route handlers.
use super::{map_environment::roads::RegionalRoadSource, travel::TerrainPlanner};
use crate::{live::LiveState, session::SessionCodec, spacetimedb::SpacetimeClient};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: SpacetimeClient,
    pub live: LiveState,
    pub terrain: Option<Arc<TerrainPlanner>>,
    pub(crate) regional_roads: Arc<RegionalRoadSource>,
    pub session_codec: Arc<SessionCodec>,
}
