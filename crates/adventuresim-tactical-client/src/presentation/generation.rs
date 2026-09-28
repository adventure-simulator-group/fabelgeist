//! Pure generation jobs shared by browser workers and the renderer.
//! Products contain immutable scene assets, never tactical simulation state.
use adventuresim_building_generator::BuildingProgram;
use adventuresim_tactical_core::scene_input::{GeneratedTacticalScene, TacticalSceneInput};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

mod building;
pub(super) use building::PreparedFacade;
#[cfg(test)]
mod tests;
#[cfg(target_family = "wasm")]
mod wasm;

#[derive(Serialize, Deserialize)]
enum GenerationJob {
    Scene(Box<TacticalSceneInput>),
    Building(Box<BuildingProgram>),
}

#[derive(Serialize, Deserialize)]
enum GenerationProduct {
    Scene(Box<GeneratedTacticalScene>),
    Building(Box<PreparedFacade>),
}

#[derive(Default)]
struct PreparedProducts {
    scenes: Vec<GeneratedTacticalScene>,
    facades: Vec<PreparedFacade>,
}
static PRODUCTS: OnceLock<Mutex<PreparedProducts>> = OnceLock::new();

fn products() -> std::sync::MutexGuard<'static, PreparedProducts> {
    PRODUCTS
        .get_or_init(Default::default)
        .lock()
        .expect("generation product queue")
}

pub(crate) fn take_scene(input: &TacticalSceneInput) -> Result<GeneratedTacticalScene, String> {
    let digest = input.digest().map_err(|error| error.to_string())?;
    let mut products = products();
    let index = products
        .scenes
        .iter()
        .position(|scene| scene.digest == digest)
        .ok_or("scene generation was not completed before installation")?;
    Ok(products.scenes.swap_remove(index))
}

pub(super) fn take_facade(program: &BuildingProgram) -> Result<PreparedFacade, String> {
    let mut products = products();
    let index = products
        .facades
        .iter()
        .position(|facade| facade.program == *program)
        .ok_or("building generation was not completed before installation")?;
    Ok(products.facades.swap_remove(index))
}

pub(super) fn release_unused_facades() {
    products().facades.clear();
}

fn jobs(input_json: &str) -> Result<Vec<String>, String> {
    let input: TacticalSceneInput = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    input.validate().map_err(|e| e.to_string())?;
    let mut programs = Vec::new();
    for placement in &input.distant_buildings {
        let program = placement.exterior_program();
        if !programs.contains(&program) {
            programs.push(program);
        }
    }
    std::iter::once(GenerationJob::Scene(Box::new(input)))
        .chain(
            programs
                .into_iter()
                .map(|program| GenerationJob::Building(Box::new(program))),
        )
        .map(|job| serde_json::to_string(&job).map_err(|e| e.to_string()))
        .collect()
}

fn generate(job_json: &str) -> Result<Vec<u8>, String> {
    let job: GenerationJob = serde_json::from_str(job_json).map_err(|e| e.to_string())?;
    let product = match job {
        GenerationJob::Scene(input) => {
            GenerationProduct::Scene(Box::new(input.generate().map_err(|e| e.to_string())?))
        }
        GenerationJob::Building(program) => {
            GenerationProduct::Building(Box::new(PreparedFacade::generate(*program)?))
        }
    };
    let mut bytes = Vec::new();
    ciborium::into_writer(&product, &mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn receive(job_json: &str, bytes: &[u8]) -> Result<(), String> {
    let job: GenerationJob = serde_json::from_str(job_json).map_err(|e| e.to_string())?;
    let product: GenerationProduct = ciborium::from_reader(bytes).map_err(|e| e.to_string())?;
    match (job, product) {
        (GenerationJob::Scene(input), GenerationProduct::Scene(scene))
            if input.digest().map_err(|e| e.to_string())? == scene.digest =>
        {
            products().scenes.push(*scene);
        }
        (GenerationJob::Building(program), GenerationProduct::Building(facade))
            if *program == facade.program =>
        {
            products().facades.push(*facade);
        }
        _ => return Err("generation product does not match its requested input".into()),
    }
    Ok(())
}
