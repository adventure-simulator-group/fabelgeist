use crate::data::gpu::shader::{ShaderLanguage, parse_naga};
fn parser_observation() -> String {
    let cases = [
        ("empty", "", wgpu::naga::ShaderStage::Compute),
        (
            "wgsl",
            "@compute @workgroup_size(1) fn main() {}",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "glsl",
            "#version 450\nlayout(local_size_x=1) in; void main() {}",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "comment-marker",
            "// #version\n@compute @workgroup_size(1) fn main() {}",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "uppercase",
            "// #VERSION\n@compute @workgroup_size(1) fn main() {}",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "embedded",
            "abc#version xyz",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "invalid-wgsl",
            "@compute fn broken( {",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "invalid-glsl",
            "#version 450\nvoid main( {",
            wgpu::naga::ShaderStage::Compute,
        ),
        (
            "vertex",
            "#version 450\nvoid main(){gl_Position=vec4(0.0);}",
            wgpu::naga::ShaderStage::Vertex,
        ),
        (
            "fragment",
            "#version 450\nlayout(location=0) out vec4 color; void main(){color=vec4(1.0);}",
            wgpu::naga::ShaderStage::Fragment,
        ),
        (
            "unicode",
            "// β\n@compute @workgroup_size(2,3,4) fn entry() {}",
            wgpu::naga::ShaderStage::Compute,
        ),
        ("nul", "\0", wgpu::naga::ShaderStage::Compute),
    ];
    let mut out = String::new();
    for (label, code, stage) in cases {
        let result = parse_naga(code, stage)
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
            "{label}:language={};result={result:?}\n",
            ShaderLanguage::from_source_text(code)
        ));
    }
    out
}

#[test]
fn classification_and_native_parser_results_match_main() {
    assert_eq!(parser_observation(), include_str!("fixtures/parser.txt"));
}
