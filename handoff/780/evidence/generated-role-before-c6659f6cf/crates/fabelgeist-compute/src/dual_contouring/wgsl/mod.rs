//! Built-in dual-contouring programs and shared vertex acceptance.

pub(super) use count::{index_count, vertex_count};
pub(super) use index::index_stream;
pub(super) use vertex::vertex_stream;

mod count;
mod index;
mod vertex;

const PARAMS: &str = r#"
struct Params {
    grid_x: u32,
    grid_y: u32,
    grid_z: u32,
    max_vertices: u32,
    max_indices: u32,
    front_mode: u32,
    threshold: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
}
"#;
