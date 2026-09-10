//! Export a deterministic close-helmet candidate using a saved anatomical frame.
use adventuresim_armor_model::{
    CloseHelmetDesign, CloseHelmetProfile, HelmetDesign, PartFrame, generate_close_helmet,
    generate_helmet,
};
use std::{collections::BTreeMap, env, fs};

fn main() {
    let args: Vec<_> = env::args().collect();
    let input: serde_json::Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let design: CloseHelmetDesign = if let Some(path) = args.get(3) {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        CloseHelmetDesign::default()
    };
    let frame = PartFrame {
        origin: serde_json::from_value(input["frame"]["origin"].clone()).unwrap(),
        axes: serde_json::from_value(input["frame"]["axes"].clone()).unwrap(),
        half_extents: serde_json::from_value(input["frame"]["half_extents"].clone()).unwrap(),
    };
    let profile = args.get(4).map(|path| {
        let body: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let points: Vec<[f32; 3]> = serde_json::from_value(body["positions"].clone()).unwrap();
        CloseHelmetProfile::from_samples(&design, &frame, &points).unwrap()
    });
    let mesh = match profile {
        Some(p) => generate_close_helmet(&design, &frame, &p).unwrap(),
        None => generate_helmet(&HelmetDesign::CloseHelmet(design), &frame).unwrap(),
    };
    let mut welded = BTreeMap::new();
    let mapping: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| {
            let next = welded.len() as u32;
            *welded
                .entry(p.map(|v| (f64::from(v) * 1e7).round() as i64))
                .or_insert(next)
        })
        .collect();
    for part in &mesh.components {
        let mut edges = BTreeMap::<(u32, u32), usize>::new();
        for triangle in mesh.indices[part.indices.clone()].as_chunks::<3>().0 {
            let [a, b, c] = triangle.map(|i| mapping[i as usize]);
            for (a, b) in [(a, b), (b, c), (c, a)] {
                *edges.entry((a.min(b), a.max(b))).or_default() += 1;
            }
        }
        for ((a, b), count) in edges.iter().filter(|(_, count)| **count != 2).take(8) {
            eprintln!("{:?} physical edge {a}/{b}: {count}", part.role);
        }
    }
    fs::write(
        &args[2],
        serde_json::to_vec(&serde_json::json!({
            "id":"close_helmet", "placement":"worn", "design":design,
            "positions":mesh.positions, "indices":mesh.indices, "normals":mesh.normals().unwrap(),
            "components":mesh.components, "frame":input["frame"], "profile":profile,
            "generator_version":adventuresim_armor_model::GENERATOR_VERSION,
        }))
        .unwrap(),
    )
    .unwrap();
}
