use super::travel;
use crate::{live::LiveState, session::SessionCodec, spacetimedb::SpacetimeClient};

/// Application state shared across routes
#[derive(Clone)]
pub struct AppState {
    pub tactical_proxy_origin: Option<crate::tactical_proxy::TacticalProxyOrigin>,
    pub db: SpacetimeClient,
    pub live: LiveState,
    pub strategic_map: Option<std::sync::Arc<crate::strategic_map::StrategicMap>>,
    pub terrain: Option<std::sync::Arc<travel::TerrainPlanner>>,
    pub session_codec: std::sync::Arc<SessionCodec>,
}
