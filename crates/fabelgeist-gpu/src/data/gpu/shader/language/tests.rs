use crate::data::gpu::shader::{ShaderSource, parse_naga};

#[derive(Clone, Copy, strum::Display)]
#[strum(serialize_all = "kebab-case")]
enum ParserFixture {
    Empty,
    Wgsl,
    Glsl,
    CommentMarker,
    Uppercase,
    Embedded,
    InvalidWgsl,
    InvalidGlsl,
    Vertex,
    Fragment,
    Unicode,
    Nul,
}

impl ParserFixture {
    const ALL: [Self; 12] = [
        Self::Empty,
        Self::Wgsl,
        Self::Glsl,
        Self::CommentMarker,
        Self::Uppercase,
        Self::Embedded,
        Self::InvalidWgsl,
        Self::InvalidGlsl,
        Self::Vertex,
        Self::Fragment,
        Self::Unicode,
        Self::Nul,
    ];

    fn source(self) -> ShaderSource<'static> {
        match self {
            Self::Empty => "",
            Self::Wgsl => "@compute @workgroup_size(1) fn main() {}",
            Self::Glsl => "#version 450\nlayout(local_size_x=1) in; void main() {}",
            Self::CommentMarker => "// #version\n@compute @workgroup_size(1) fn main() {}",
            Self::Uppercase => "// #VERSION\n@compute @workgroup_size(1) fn main() {}",
            Self::Embedded => "abc#version xyz",
            Self::InvalidWgsl => "@compute fn broken( {",
            Self::InvalidGlsl => "#version 450\nvoid main( {",
            Self::Vertex => "#version 450\nvoid main(){gl_Position=vec4(0.0);}",
            Self::Fragment => {
                "#version 450\nlayout(location=0) out vec4 color; void main(){color=vec4(1.0);}"
            }
            Self::Unicode => "// β\n@compute @workgroup_size(2,3,4) fn entry() {}",
            Self::Nul => "\0",
        }
        .into()
    }

    fn stage(self) -> wgpu::naga::ShaderStage {
        match self {
            Self::Vertex => wgpu::naga::ShaderStage::Vertex,
            Self::Fragment => wgpu::naga::ShaderStage::Fragment,
            _ => wgpu::naga::ShaderStage::Compute,
        }
    }
}

fn parser_observation() -> String {
    let mut out = String::new();
    for fixture in ParserFixture::ALL {
        let code = fixture.source();
        let result = parse_naga(&code, fixture.stage())
            .map(|module| {
                let entries = module
                    .entry_points
                    .iter()
                    .map(|e| (&e.name, e.stage, e.workgroup_size))
                    .collect::<Vec<_>>();
                let globals = module
                    .global_variables
                    .iter()
                    .map(|(_, g)| (&g.name, &g.binding, g.space))
                    .collect::<Vec<_>>();
                format!(
                    "entries={entries:?};globals={globals:?};types={};functions={}",
                    module.types.len(),
                    module.functions.len()
                )
            })
            .map_err(|e| format!("{e:#}"));
        out.push_str(&format!(
            "{fixture}:language={};result={result:?}\n",
            code.language()
        ));
    }
    out
}

#[test]
fn classification_and_native_parser_results_match_main() {
    assert_eq!(parser_observation(), include_str!("fixtures/parser.txt"));
}
