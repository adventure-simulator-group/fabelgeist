use adventuresim_weapon_model::*;

#[test]
fn refined_partisan_retains_winding_after_float32_attachment_translation() {
    let study: serde_json::Value =
        serde_json::from_str(include_str!("../review/museum/met-08.261.2.json")).unwrap();
    let recipe = serde_json::from_value(study["definition"].clone()).unwrap();
    let model = generate_model(&recipe, Detail::High).unwrap();
    let translation = [1.0, 2.0, 3.0];
    let grip = model.physical.control_point;
    for part in &model.parts {
        for (triangle, face) in part.indices.as_chunks::<3>().0.iter().enumerate() {
            let points: [[f64; 3]; 3] = std::array::from_fn(|corner| {
                std::array::from_fn(|axis| {
                    (part.positions[face[corner] as usize * 3 + axis] - grip[axis]
                        + translation[axis]) as f32 as f64
                })
            });
            let u: [f64; 3] = std::array::from_fn(|axis| points[1][axis] - points[0][axis]);
            let v: [f64; 3] = std::array::from_fn(|axis| points[2][axis] - points[0][axis]);
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for &index in face {
                let dot: f64 = (0..3)
                    .map(|axis| cross[axis] * part.normals[index as usize * 3 + axis])
                    .sum();
                assert!(dot > 0.0, "{} triangle {triangle} lost winding", part.label);
            }
        }
    }
}
