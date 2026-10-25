//! Random purposes shared by authoritative terrain and its presentation.
//!
//! These names are part of the replay contract. Keeping them here prevents
//! the server terrain and client presentation from assigning different streams
//! to the same spatial field.

use fabelgeist_determinism::StreamId;

pub const BROAD: StreamId = StreamId::new("terrain.broad");
pub const CLOD: StreamId = StreamId::new("terrain.clod");
pub const CREEP_WARP: StreamId = StreamId::new("terrain.creep-warp");
pub const DETAIL: StreamId = StreamId::new("terrain.detail");
pub const FINE: StreamId = StreamId::new("terrain.fine");
pub const FRACTURE_A: StreamId = StreamId::new("terrain.fracture-a");
pub const FRACTURE_B: StreamId = StreamId::new("terrain.fracture-b");
pub const GROUND_MASK_LATTICE: StreamId = StreamId::new("terrain.ground-mask-lattice");
pub const MICRORELIEF_FINE: StreamId = StreamId::new("terrain.microrelief-fine");
pub const RILL_SPACING: StreamId = StreamId::new("terrain.rill-spacing");
pub const RILL_WARP: StreamId = StreamId::new("terrain.rill-warp");
pub const ROAD_RUT: StreamId = StreamId::new("terrain.road-rut");
pub const ROCK_DEBRIS: StreamId = StreamId::new("terrain.rock-debris");
pub const ROCK_DOWNHILL: StreamId = StreamId::new("terrain.rock-downhill");
pub const ROCK_INFLUENCE: StreamId = StreamId::new("terrain.rock-influence");
pub const ROCK_TAIL: StreamId = StreamId::new("terrain.rock-tail");
pub const ROOT_SHAPE: StreamId = StreamId::new("terrain.root-shape");
pub const STRATA_DIRECTION: StreamId = StreamId::new("terrain.strata-direction");
pub const STRATA_WARP: StreamId = StreamId::new("terrain.strata-warp");
pub const TREE_ROOTS: StreamId = StreamId::new("terrain.tree-roots");
pub const VALUE_LATTICE: StreamId = StreamId::new("terrain.value-lattice");
