mod controls;
pub use controls::validate_controls;

use crate::recipe::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HeadTemplate {
    id: String,
    components: Vec<Component>,
    controls: Vec<Value>,
    radius_bindings: Vec<RadiusBinding>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RadiusBinding {
    component: String,
    parameter: RadiusParameter,
    factor: Ratio,
    #[serde(default)]
    minimum: Option<Metres>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum RadiusParameter {
    #[serde(rename = "sleeveRadius")]
    SleeveBase,
    #[serde(rename = "sleeveTopRadius")]
    SleeveTop,
    #[serde(rename = "maceRootRadius")]
    MaceRoot,
    #[serde(rename = "maceShoulderRadius")]
    MaceShoulder,
    #[serde(rename = "maceCuspRadius")]
    MaceCusp,
}
#[derive(Deserialize)]
struct Haft {
    id: String,
    shaft: Shaft,
    components: Vec<Component>,
}

pub fn authoring_catalog() -> &'static Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../catalog/authoring.json"))
            .expect("authored weapon catalog is valid JSON")
    })
}
fn templates() -> &'static Vec<HeadTemplate> {
    static HEADS: OnceLock<Vec<HeadTemplate>> = OnceLock::new();
    HEADS.get_or_init(|| {
        serde_json::from_value(authoring_catalog()["headTemplates"].clone())
            .expect("authored heads have typed constructions")
    })
}
impl RadiusBinding {
    fn apply(&self, components: &mut [Component], radius: Metres) -> Result<(), String> {
        let component = components
            .iter_mut()
            .find(|c| c.id.as_ref() == Some(&self.component))
            .ok_or("missing bound head component")?;
        let value = Metres::new(
            (radius.get() * self.factor.get()).max(self.minimum.map_or(0.0, Metres::get)),
        )?;
        match (&mut component.shape, &self.parameter) {
            (Shape::Sleeve(p), RadiusParameter::SleeveBase) => p.radius = value,
            (Shape::Sleeve(p), RadiusParameter::SleeveTop) => p.top_radius = Some(value),
            (Shape::Mace(p), RadiusParameter::MaceRoot) => p.root_radius = value,
            (Shape::Mace(p), RadiusParameter::MaceShoulder) => p.shoulder_radius = value,
            (Shape::Mace(p), RadiusParameter::MaceCusp) => p.cusp_radius = value,
            _ => return Err("radius binding does not match component construction".into()),
        }
        Ok(())
    }
}
pub fn compose_weapon(haft_id: &str, head_id: &str) -> Result<Recipe, String> {
    let hafts: Vec<Haft> =
        serde_json::from_value(authoring_catalog()["hafts"].clone()).map_err(|e| e.to_string())?;
    let haft = hafts
        .into_iter()
        .find(|h| h.id == haft_id)
        .ok_or("unknown haft")?;
    let template = templates()
        .iter()
        .find(|h| h.id == head_id)
        .ok_or("unknown head")?;
    let mut head = template.components.clone();
    for binding in &template.radius_bindings {
        binding.apply(&mut head, haft.shaft.radius)?;
    }
    for component in &mut head {
        if let Shape::Box(p) = &component.shape
            && p.fit_shaft_side == Some(true)
        {
            let offset = component
                .offset
                .as_mut()
                .ok_or("side-mounted langet needs offset")?;
            offset[0] = Metres::new(
                offset[0].get().signum()
                    * (haft.shaft.radius.get() * haft.shaft.top_scale.map_or(0.92, Ratio::get)
                        + p.size[0].get() / 2.0
                        - 0.002),
            )?;
        }
    }
    let recipe = Recipe {
        shaft: Some(haft.shaft),
        components: haft.components.into_iter().chain(head).collect(),
        grip_clearance: None,
    };
    recipe.validate().map_err(|e| e.to_string())?;
    Ok(recipe)
}
pub fn composition_controls(recipe: &Recipe) -> Result<Vec<Value>, String> {
    let shaft = recipe.shaft.as_ref().ok_or("composition requires a haft")?;
    let scale = shaft
        .bottom_scale
        .map_or(1.0, Ratio::get)
        .max(shaft.top_scale.map_or(0.92, Ratio::get));
    let maximum = (0.022 / scale * 1000.0 + 1e-9).floor() / 1000.0;
    let mut controls = vec![
        serde_json::json!({"label":"Haft length","target":"shaft","key":"length","min":0.45,"max":3.2,"step":0.01,"unit":"m"}),
        serde_json::json!({"label":"Haft radius","target":"shaft","key":"radius","min":0.006,"max":maximum,"step":0.001,"unit":"m"}),
    ];
    if let Some(primary) = recipe
        .components
        .iter()
        .find(|c| {
            c.id.as_ref()
                .is_some_and(|id| id.starts_with(COMPOSED_HEAD_PREFIX))
        })
        .or_else(|| {
            recipe
                .components
                .iter()
                .find(|c| c.id.as_deref() == Some(SINGLE_HEAD_ID))
        })
        && let Some(template) = templates().iter().find(|h| {
            h.components
                .iter()
                .find(|c| {
                    c.id.as_deref() == Some(SINGLE_HEAD_ID)
                        || c.id.as_deref() == Some(FIRST_COMPOSED_HEAD_ID)
                })
                .is_some_and(|c| {
                    std::mem::discriminant(&c.shape) == std::mem::discriminant(&primary.shape)
                })
        })
    {
        for control in &template.controls {
            let mut control = control.clone();
            control["componentId"] =
                serde_json::to_value(&primary.id).map_err(|e| e.to_string())?;
            controls.push(control);
        }
    }
    Ok(controls)
}

#[derive(Serialize)]
pub struct AuthoringSelection {
    pub definition: Recipe,
    pub controls: Vec<Value>,
}

const COMPOSED_HEAD_PREFIX: &str = "head-primary";

const FIRST_COMPOSED_HEAD_ID: &str = "head-primary-0";

const SINGLE_HEAD_ID: &str = "head";

/// Museum studies are separate from the general authoring preset catalog.
pub fn museum_studies() -> &'static Value {
    static STUDIES: OnceLock<Value> = OnceLock::new();
    STUDIES.get_or_init(|| {
        serde_json::json!([
            serde_json::from_str::<Value>(include_str!("../review/museum/met-14.25.394.json"))
                .expect("museum study catalog must be valid JSON"),
            serde_json::from_str::<Value>(include_str!("../review/museum/cma-1921.1253.json"))
                .expect("museum study catalog must be valid JSON"),
            serde_json::from_str::<Value>(include_str!("../review/museum/london-80.157.json"))
                .expect("museum study catalog must be valid JSON"),
            serde_json::from_str::<Value>(include_str!("../review/museum/cma-1916.1589.json"))
                .expect("museum study catalog must be valid JSON")
        ])
    })
}
