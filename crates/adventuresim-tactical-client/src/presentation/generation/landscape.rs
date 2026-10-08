//! Client-built landscape products retained independently of scene entities.
use super::*;
use crate::presentation::vista::grass::PreparedGrass;
use crate::presentation::vista::streets::prepared::PreparedCityGround;
use adventuresim_tactical_core::{prelude::SceneTerrain, scene_input::furniture::FurnitureGroup};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub(super) struct GroundDependencies {
    pub terrain: SceneTerrain,
    pub groups: Vec<FurnitureGroup>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct GroundProduct {
    pub digest: String,
    pub graphics: String,
    pub ground: Arc<PreparedCityGround>,
}

impl GroundProduct {
    pub(super) fn generate(
        input: &TacticalSceneInput,
        graphics: String,
        dependencies: GroundDependencies,
    ) -> PreparationResult<Self> {
        let config = crate::presentation::config::TacticalGraphicsConfig::parse(&graphics)
            .map_err(|message| PreparationError::GraphicsConfiguration { message })?;
        Ok(Self {
            digest: input.digest()?,
            ground: Arc::new(PreparedCityGround::from_scene(
                input,
                &dependencies.terrain,
                &dependencies.groups,
                config.rendering.vista.maximum_lods,
            )),
            graphics,
        })
    }
}

pub(super) fn jobs(input_json: &str, graphics: &str) -> PreparationResult<Vec<String>> {
    crate::presentation::config::TacticalGraphicsConfig::parse(graphics)
        .map_err(|message| PreparationError::GraphicsConfiguration { message })?;
    let input: TacticalSceneInput = serde_json::from_str(input_json)?;
    let digest = input.digest()?;
    let mut products = products()?;
    products.active_ground = Some((digest.clone(), graphics.to_owned()));
    let mut jobs = Vec::new();
    if let Some(index) = products
        .ground
        .iter()
        .position(|p| p.digest == digest && p.graphics == graphics)
    {
        let resident = products.ground.remove(index);
        products.ground.push(resident);
    } else {
        jobs.push(GenerationJob::Ground {
            input: Box::new(input.clone()),
            graphics: graphics.to_owned(),
        });
    }
    if let Some(index) = products
        .grass
        .iter()
        .position(|p| p.digest == digest && p.graphics == graphics)
    {
        let resident = products.grass.remove(index);
        products.grass.push(resident);
    } else {
        jobs.push(GenerationJob::Grass {
            input: Box::new(input),
            graphics: graphics.to_owned(),
        });
    }
    jobs.into_iter()
        .map(|job| serde_json::to_string(&job).map_err(PreparationError::from))
        .collect()
}

pub(super) fn retain(ground: GroundProduct) -> PreparationResult<()> {
    let mut products = products()?;
    products.ground.push(ground);
    if products.ground.len() > RETAINED_SCENE_PRODUCTS {
        products.ground.remove(0);
    }
    Ok(())
}

pub(in crate::presentation) fn ground(
    digest: &str,
) -> PreparationResult<Option<Arc<PreparedCityGround>>> {
    let products = products()?;
    let Some((_, graphics)) = products
        .active_ground
        .as_ref()
        .filter(|(active, _)| active == digest)
    else {
        return Ok(None);
    };
    Ok(products
        .ground
        .iter()
        .find(|p| p.digest == digest && p.graphics == *graphics)
        .map(|p| p.ground.clone()))
}

#[derive(Serialize, Deserialize)]
pub(super) struct GrassDependencies {
    pub terrain: SceneTerrain,
    pub ground: adventuresim_tactical_core::prelude::SceneGround,
}

#[derive(Serialize, Deserialize)]
pub(super) struct GrassProduct {
    pub digest: String,
    pub graphics: String,
    pub grass: Arc<PreparedGrass>,
}

impl GrassProduct {
    pub(super) fn generate(
        input: &TacticalSceneInput,
        graphics: String,
        dependencies: GrassDependencies,
    ) -> PreparationResult<Self> {
        let config = crate::presentation::config::TacticalGraphicsConfig::parse(&graphics)
            .map_err(|message| PreparationError::GraphicsConfiguration { message })?;
        Ok(Self {
            digest: input.digest()?,
            grass: Arc::new(PreparedGrass::new(
                input,
                &dependencies.terrain,
                &dependencies.ground,
                &config,
            )),
            graphics,
        })
    }
}

pub(in crate::presentation) fn grass(
    digest: &str,
) -> PreparationResult<Option<Arc<PreparedGrass>>> {
    let products = products()?;
    let Some((_, graphics)) = products
        .active_ground
        .as_ref()
        .filter(|(active, _)| active == digest)
    else {
        return Ok(None);
    };
    Ok(products
        .grass
        .iter()
        .find(|p| p.digest == digest && p.graphics == *graphics)
        .map(|p| p.grass.clone()))
}
