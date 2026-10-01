//! Browser acceptance probe for the runtime generator, using an input body GLB.
mod armor_fixture;

fn main() {}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn fit_probe(bytes: Vec<u8>) -> Result<String, String> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(&bytes).await.map_err(|error| format!("{error:#}"))
}

#[cfg_attr(
    not(target_family = "wasm"),
    expect(dead_code, reason = "called by the browser probe export")
)]
async fn run(bytes: &[u8]) -> anyhow::Result<String> {
    use adventuresim_character_creator::{design_input, runtime_equipment};
    use anyhow::Context;
    let (body, _) = armor_fixture::load(bytes)?;
    let bracer = design_input::load_bracer_design(None)?;
    let breastplate = design_input::load_breastplate_design(None)?;
    let mut results = Vec::new();
    for round in 0..2 {
        for (item, placement) in [
            ("vambrace", "left"),
            ("breastplate", "worn"),
            ("gorget", "worn"),
            ("sallet", "worn"),
            ("mail_standard", "worn"),
            ("linen_tunic", "worn"),
            ("leather_boot", "left"),
        ] {
            let started = web_time::Instant::now();
            let armor = runtime_equipment::generate(&body, item, placement, &bracer, &breastplate)
                .await
                .with_context(|| format!("fitting {item}, round {round}"))?;
            anyhow::ensure!(
                armor.morphs.is_empty(),
                "runtime equipment generated morph targets"
            );
            anyhow::ensure!(
                !armor.positions.is_empty() && !armor.indices.is_empty(),
                "empty equipment"
            );
            anyhow::ensure!(
                armor.positions.iter().flatten().all(|v| v.is_finite()),
                "non-finite mesh"
            );
            anyhow::ensure!(
                armor
                    .indices
                    .iter()
                    .all(|&index| (index as usize) < armor.positions.len()),
                "invalid mesh index"
            );
            results.push(serde_json::json!({"item":item,"round":round,"ms":started.elapsed().as_secs_f64()*1000.0,
                "vertices":armor.positions.len(),"triangles":armor.indices.len()/3,"morphs":armor.morphs.len()}));
        }
    }
    Ok(serde_json::to_string_pretty(&results)?)
}
