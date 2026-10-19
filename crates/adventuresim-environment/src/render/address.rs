//! Shared integer vertex-address ABI for grown geometry.

/// Explicit Bevy vertex ABI: one integer address, decoded by the forest shader.
pub const VERTEX_ADDRESS: bevy::mesh::MeshVertexAttribute = bevy::mesh::MeshVertexAttribute::new(
    "ForestAddress",
    0x4652_5354_4144_4452,
    bevy::render::render_resource::VertexFormat::Uint32,
);
