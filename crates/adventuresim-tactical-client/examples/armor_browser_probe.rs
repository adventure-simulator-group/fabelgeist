//! Browser acceptance probe for the runtime generator, using an input body GLB.
mod armor_fixture;

fn main() {}

enum Probe {
    Catalog(&'static str, &'static str),
    Anime,
    Wrapped,
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn fit_probe(bytes: Vec<u8>) -> Result<String, String> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(&bytes).await.map_err(|error| format!("{error:#}"))
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn prepare_probe(bytes: Vec<u8>) -> Result<String, String> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let result = async {
        let (body, _) = armor_fixture::load(&bytes)?;
        let bracer = adventuresim_character_creator::design_input::load_bracer_design(None)?;
        let breastplate =
            adventuresim_character_creator::design_input::load_breastplate_design(None)?;
        let errors = capture_errors(adventuresim_character_creator::armor_gpu_async().await?);
        let report = adventuresim_character_creator::runtime_equipment::warm_up(
            &body,
            &bracer,
            &breastplate,
        )
        .await;
        let failures = errors.lock().unwrap().clone();
        anyhow::ensure!(
            failures.is_empty(),
            "WebGPU preparation validation: {failures:?}"
        );
        anyhow::Ok(serde_json::to_string(&report?)?)
    }
    .await;
    result.map_err(|error| format!("{error:#}"))
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
    let gpu = adventuresim_character_creator::armor_gpu_async().await?;
    let errors = capture_errors(gpu);
    let mut results = Vec::new();
    for round in 0..2 {
        for probe in [
            Probe::Catalog("vambrace", "left"),
            Probe::Catalog("breastplate", "worn"),
            Probe::Catalog("gorget", "worn"),
            Probe::Catalog("sallet", "worn"),
            Probe::Catalog("mail_standard", "worn"),
            Probe::Catalog("linen_tunic", "worn"),
            Probe::Catalog("leather_boot", "left"),
            Probe::Catalog("puffed_sleeve", "left"),
            Probe::Catalog("puffed_hose", "left"),
            Probe::Catalog("pauldron", "left"),
            Probe::Anime,
            Probe::Wrapped,
        ] {
            let (item, placement) = match probe {
                Probe::Catalog(item, placement) => (item, placement),
                Probe::Anime => ("anime", "worn"),
                Probe::Wrapped => ("wrapped_tassets", "worn"),
            };
            let started = web_time::Instant::now();
            let cached_kernels = gpu.cache().len();
            let mut breastplate = breastplate.clone();
            let armor = if matches!(probe, Probe::Wrapped) {
                wrapped(&body).await
            } else {
                let recipe = if matches!(probe, Probe::Anime) {
                    breastplate.construction =
                        fabelgeist_armor::BreastplateConstruction::Anime(Default::default());
                    "cuirass"
                } else {
                    item
                };
                runtime_equipment::generate(&body, recipe, placement, &bracer, &breastplate, &[])
                    .await
                    .with_context(|| format!("fitting {item}, round {round}"))
            };
            let failures = errors.lock().unwrap().clone();
            anyhow::ensure!(failures.is_empty(), "WebGPU validation: {failures:?}");
            let armor = armor?;
            anyhow::ensure!(
                gpu.cache().len() == cached_kernels,
                "post-readiness fit compiled new kernels: {item}"
            );
            anyhow::ensure!(
                armor.morphs.is_empty(),
                "runtime equipment generated morph targets"
            );
            anyhow::ensure!(
                !armor.positions.is_empty() && !armor.indices.is_empty(),
                "empty equipment"
            );
            anyhow::ensure!(
                armor
                    .positions
                    .iter()
                    .chain(&armor.normals)
                    .flatten()
                    .all(|v| v.is_finite()),
                "non-finite mesh"
            );
            anyhow::ensure!(
                armor
                    .indices
                    .iter()
                    .all(|&index| (index as usize) < armor.positions.len()),
                "invalid mesh index"
            );
            anyhow::ensure!(
                armor.indices.as_chunks::<3>().0.iter().any(|face| {
                    let [a, b, c] =
                        face.map(|i| bevy::math::Vec3::from(armor.positions[i as usize]));
                    (b - a).cross(c - a).length_squared() > 1e-14
                }),
                "collapsed equipment geometry: {item}"
            );
            results.push(serde_json::json!({"item":item,"round":round,"ms":started.elapsed().as_secs_f64()*1000.0,
                "vertices":armor.positions.len(),"triangles":armor.indices.len()/3,"morphs":armor.morphs.len(),
                "new_kernels":gpu.cache().len()-cached_kernels}));
        }
    }
    Ok(serde_json::to_string_pretty(&results)?)
}

#[cfg_attr(
    not(target_family = "wasm"),
    expect(dead_code, reason = "browser probe helper")
)]
fn capture_errors(
    gpu: &fabelgeist_armor::ArmorGpu,
) -> std::sync::Arc<std::sync::Mutex<Vec<String>>> {
    let errors = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = errors.clone();
    gpu.context()
        .device
        .on_uncaptured_error(std::sync::Arc::new(move |error| {
            captured.lock().unwrap().push(format!("{error}"));
        }));
    errors
}

#[cfg_attr(
    not(target_family = "wasm"),
    expect(dead_code, reason = "browser probe helper")
)]
async fn wrapped(
    body: &adventuresim_character_creator::runtime_equipment::RuntimeBody,
) -> anyhow::Result<fabelgeist_armor::GeneratedArmor> {
    use adventuresim_character_creator::{
        armor_recipes::ParametricDesign,
        device_fit::{self, FitBody, Fitted, Realization},
    };
    use fabelgeist_armor::{
        GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape, Permille, WrappedTassetDesign,
    };
    let mut garment = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
    garment.lame_count = 8;
    garment.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
        upper_edge_slope: Permille(400),
        ..Default::default()
    });
    let design = ParametricDesign::Garment(garment);
    let gpu = adventuresim_character_creator::armor_gpu_async().await?;
    let topology = FitBody {
        faces: &body.faces,
        texcoords: &body.texcoords,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
    };
    let wearer = Realization {
        positions: &body.positions,
        normals: &body.normals,
        joints: &body.global_joint_states,
        device: &body.device,
    };
    let part =
        device_fit::fit_recipe_async(gpu, &topology, &wearer, &[], &design, "worn", &[]).await;
    device_fit::assemble_recipe(
        &design,
        part?,
        &Fitted {
            placement: "worn",
            morphs: &[],
            domain: &body.domain,
            joint_names: &body.joint_names,
            joints: &body.global_joint_states,
        },
    )
}
