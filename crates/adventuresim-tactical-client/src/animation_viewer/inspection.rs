//! Optional capture-only views expose armor boundaries without changing fitting.
use super::*;
use crate::equipment::ProceduralEquipmentPart;
use crate::presentation::interior_lighting::InteriorMaterial;

const SUPPORTING_CUIRASS_ALPHA: f32 = 0.2;

#[derive(Resource)]
pub(crate) struct CaptureInspection {
    pub orbit: Quat,
    pub diffuse_armor: bool,
}

impl CaptureInspection {
    pub(crate) fn new(orbit_degrees: f32, diffuse_armor: bool, output: &std::path::Path) -> Self {
        fs::create_dir_all(output).expect("create inspection capture directory");
        fs::write(
            output.join("inspection.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "orbit_degrees": orbit_degrees,
                "diffuse_armor": diffuse_armor,
                "cuirass_alpha": if diffuse_armor { SUPPORTING_CUIRASS_ALPHA } else { 1.0 },
            }))
            .expect("serialize capture inspection"),
        )
        .expect("write capture inspection");
        Self {
            orbit: Quat::from_rotation_y(orbit_degrees.to_radians()),
            diffuse_armor,
        }
    }
}

pub(super) fn inspect_materials(
    inspection: Res<CaptureInspection>,
    parts: Query<(&ProceduralEquipmentPart, &MeshMaterial3d<InteriorMaterial>)>,
    items: Query<&ItemProperties>,
    mut materials: ResMut<Assets<InteriorMaterial>>,
) {
    if !inspection.diffuse_armor {
        return;
    }
    for (part, handle) in &parts {
        let (Ok(item), Some(mut material)) = (items.get(part.item), materials.get_mut(&handle.0))
        else {
            continue;
        };
        let alpha = if item.id == "cuirass" {
            SUPPORTING_CUIRASS_ALPHA
        } else {
            1.0
        };
        if material.base.metallic == 0.0 && material.base.base_color.alpha() == alpha {
            continue;
        }
        material.base.metallic = 0.0;
        material.base.perceptual_roughness = 1.0;
        material.base.base_color.set_alpha(alpha);
        if alpha < 1.0 {
            material.base.alpha_mode = AlphaMode::Blend;
        }
    }
}
