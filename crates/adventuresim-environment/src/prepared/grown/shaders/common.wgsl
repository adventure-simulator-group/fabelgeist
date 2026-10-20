// Native authoring boundary. Positions retain the source record's local metres.
#define_import_path adventuresim_environment::grown_expansion

const GROWN_EXPANSION_GROUP_WIDTH: u32 = 64u;

struct Params {
    shape: vec4<u32>,
    span: vec4<u32>,
};

struct ExpandedVertex {
    position: vec4<f32>,
    normal: vec4<f32>,
    uv: vec2<f32>,
    address: u32,
    pad: u32,
};
