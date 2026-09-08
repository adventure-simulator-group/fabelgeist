//! Exports deterministic native vectors for the independent WebGPU reference comparison.
use super::*;
use bevy::math::Vec2;
#[test]
#[ignore = "requires FABELGEIST_LEAF_PARITY_OUTPUT and the WebGPU reference runner"]
fn export_reference_vectors() {
    let path = std::env::var("FABELGEIST_LEAF_PARITY_OUTPUT").expect("explicit evidence output");
    let mut shapes: Vec<_> = LeafShape::preset_names()
        .map(|n| (n.to_owned(), LeafShape::preset(n).unwrap()))
        .collect();
    let oak = LeafShape::default();
    let beech = LeafShape::preset("beech").unwrap();
    for i in 0..=10 {
        shapes.push((
            format!("oak-beech-{i}"),
            oak.interpolate(&beech, i as f32 / 10.0),
        ));
    }
    let mut cases = Vec::new();
    for (name, shape) in shapes {
        let uniform = uniform::LeafUniform::new(&shape, 128, 1);
        let packed = serde_json::to_value(&uniform).unwrap();
        let kernel = kernel::Kernel::from(uniform);
        let classes: Vec<u8> = (0..32 * 32)
            .map(|i| {
                kernel.class(
                    (Vec2::new((i % 32) as f32 + 0.5, (i / 32) as f32 + 0.5) / 32.0
                        - Vec2::splat(0.5))
                        * 2.0,
                )
            })
            .collect();
        cases.push(serde_json::json!({"name":name,"uniform":packed,"classes":classes}));
    }
    std::fs::write(path, serde_json::to_vec(&cases).unwrap()).unwrap();
}
