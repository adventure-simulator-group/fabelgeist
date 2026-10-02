//! Native renderer boundary for the shared double-precision weapon kernel.
use crate::*;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("invalid weapon design")]
    Invalid(Vec<ValidationError>),
    #[error("weapon construction failed: {0}")]
    Construction(String),
    #[error("weapon cannot provide required {0} holder geometry")]
    MissingHolderGeometry(&'static str),
}
pub(crate) fn bounds(points: &[[f32; 3]]) -> Bounds {
    let mut bounds = Bounds {
        min: [f32::INFINITY; 3],
        max: [f32::NEG_INFINITY; 3],
    };
    for point in points {
        for (axis, coordinate) in point.iter().enumerate() {
            bounds.min[axis] = bounds.min[axis].min(*coordinate);
            bounds.max[axis] = bounds.max[axis].max(*coordinate);
        }
    }
    bounds
}
pub(crate) fn native_part(part: ModelPart) -> MeshPart {
    let positions: Vec<_> = part
        .positions
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    MeshPart {
        component_id: part.component_id,
        material: part.material_id,
        bounds: bounds(&positions),
        positions,
        normals: part
            .normals
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
            .collect(),
        indices: part.indices,
    }
}
pub(crate) fn physical(design: &WeaponDesign, model: &GeneratedModel) -> DerivedProperties {
    derived(
        design,
        &model.physical,
        model.frames(),
        model.parts.iter().map(|part| {
            (
                part.component_id.as_str(),
                part.positions.as_chunks::<3>().0.iter().copied(),
            )
        }),
    )
}
pub(crate) fn derived<'a>(
    design: &WeaponDesign,
    p: &PhysicalProperties,
    frames: &std::collections::BTreeMap<String, [f64; 3]>,
    parts: impl Iterator<Item = (&'a str, impl Iterator<Item = [f64; 3]>)>,
) -> DerivedProperties {
    let head_ids: Vec<_> = design
        .recipe
        .components
        .iter()
        .filter(|c| c.role == Some(ComponentRole::Head))
        .filter_map(|c| c.id.as_deref())
        .collect();
    let (mut head_min, mut head_max) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
    for (id, points) in parts {
        let working_section = design
            .recipe
            .components
            .iter()
            .find(|c| c.id.as_deref() == Some(id))
            .and_then(|c| match &c.shape {
                recipe::Shape::Spear(parameters) if parameters.socket.is_some() => {
                    let base = frames.get(&format!("{id}.bladeBase"))?;
                    let tip = frames.get(&format!("{id}.tip"))?;
                    Some((base, tip))
                }
                _ => None,
            });
        for point in points {
            min = min.min(point[1]);
            max = max.max(point[1]);
            let within_working_section = working_section.is_none_or(|(base, tip)| {
                construction::dot(
                    construction::sub(point, *base),
                    construction::sub(*tip, *base),
                ) >= -1e-12
            });
            if head_ids.contains(&id) && within_working_section {
                head_min = head_min.min(point[1]);
                head_max = head_max.max(point[1]);
            }
        }
    }
    DerivedProperties {
        mass_kg: p.mass_kg as f32,
        length_m: (max - min) as f32,
        grip_to_tip_m: p.grip_to_tip_m as f32,
        striking_head_length_m: if head_min.is_finite() {
            (head_max - head_min) as f32
        } else {
            0.0
        },
        center_of_mass_from_grip_m: p.center_of_mass_from_grip_m as f32,
        moment_of_inertia_kg_m2: p.moment_of_inertia_kg_m2 as f32,
        balance: p.balance as f32,
    }
}

pub fn generate(design: &WeaponDesign) -> Result<GeneratedWeapon, GenerateError> {
    crate::validation::identity(design).map_err(GenerateError::Invalid)?;
    let model =
        generate_model(&design.recipe, Detail::High).map_err(GenerateError::Construction)?;
    let derived = physical(design, &model);
    let anchors = model
        .frames()
        .iter()
        .map(|(name, p)| Anchor {
            name: name.clone(),
            position: p.map(|n| n as f32),
        })
        .collect();
    Ok(GeneratedWeapon {
        design_hash: design_hash(design),
        parts: model.parts.into_iter().map(native_part).collect(),
        bounds: Bounds {
            min: model.stats.bounds.min.map(|n| n as f32),
            max: model.stats.bounds.max.map(|n| n as f32),
        },
        anchors,
        derived,
    })
}
pub use crate::holders::generate_holder;
