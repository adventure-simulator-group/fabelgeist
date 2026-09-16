//! Authoritative assembly and generation of precise weapon recipes.
mod ammunition;
mod archery;
mod bent_bar;
mod blade_reduction;
mod blade_sections;
mod blades;
mod bolts;
mod contoured_plate;
#[cfg(test)]
mod contoured_plate_tests;
mod crossbows;
mod extents;
mod firearms;
mod firelocks;
mod generic_blade;
#[cfg(test)]
mod generic_blade_tests;
mod grip;
#[cfg(test)]
mod groove_body_tests;
mod groove_sections;
mod guard_nodes;
mod guards;
mod lofted_blade;
#[cfg(test)]
mod longsword_tests;
#[cfg(test)]
mod mace_tests;
mod maces;
mod mortised_guard;
mod mounts;
pub(crate) mod output;
mod placement;
mod polls;
mod pommels;
mod profile_body;
#[cfg(test)]
mod profile_tests;
#[cfg(test)]
mod ranged_layout_tests;
#[cfg(test)]
mod seated_hilt_tests;
mod shaft_wrapping;
mod shapes;
mod shields;
mod spear_socket;
#[cfg(test)]
mod spear_tests;
mod spears;
mod wheel;

use crate::{construction::*, recipe::*};
pub use output::{GeneratedModel, ModelPart, ModelStats, PhysicalProperties};
use output::{PartSource, ResolvedRecipe};
use placement::ResolvedComponent;

/// Generate renderer geometry while keeping physical properties independent of
/// the selected display detail. High detail is the fixed integration budget.
pub fn generate_model(recipe: &Recipe, detail: Detail) -> Result<GeneratedModel, String> {
    Construction::new(recipe)?.render(detail)
}

/// Validated material solids shared by physics, holders and renderer output.
pub(crate) struct Construction {
    resolved: placement::Resolved,
    pub(crate) sources: Vec<PartSource>,
    pub(crate) physical: PhysicalProperties,
}
impl Construction {
    pub(crate) fn frames(&self) -> &std::collections::BTreeMap<String, Point> {
        &self.resolved.output.frames
    }
    pub(crate) fn new(recipe: &Recipe) -> Result<Self, String> {
        recipe.validate().map_err(|error| error.to_string())?;
        let mut resolved = placement::resolve(recipe)?;
        if resolved
            .output
            .frames
            .values()
            .flatten()
            .any(|value| value.abs() > MODEL_FRAME_EXTENT)
        {
            return Err("resolved assembly exceeds the supported world extent".into());
        }
        let sources = construct(&resolved, Detail::High)?;
        let physical = PhysicalProperties::from_sources(&sources, resolved.grip);
        let tip = sources
            .iter()
            .flat_map(|part| part.solid.positions.iter().copied())
            .fold(resolved.grip, |tip, point| {
                if magnitude(sub(point, resolved.grip)) > magnitude(sub(tip, resolved.grip)) + 1e-12
                {
                    point
                } else {
                    tip
                }
            });
        resolved.output.frames.insert("weapon.tip".into(), tip);
        Ok(Self {
            resolved,
            sources,
            physical,
        })
    }
    pub(crate) fn render(self, detail: Detail) -> Result<GeneratedModel, String> {
        let sources = if detail == Detail::High {
            self.sources
        } else {
            construct(&self.resolved, detail)?
        };
        Ok(GeneratedModel::from_sources(
            sources,
            self.resolved.output,
            self.physical,
        ))
    }
}

fn construct(resolved: &placement::Resolved, detail: Detail) -> Result<Vec<PartSource>, String> {
    let mut parts = Vec::new();
    if let Some(shaft) = &resolved.output.recipe.shaft {
        let profile = shaft.profile();
        parts.push(PartSource::new(
            Solid::lathe(
                &profile,
                shaft.segments.map_or(16, |n| n.0 as usize),
                1.0,
                true,
                detail,
            )?,
            shaft.material.unwrap_or(Material::Wood),
            "shaft",
            "shaft",
        ));
        parts.extend(shaft_wrapping::parts(shaft, "shaft", "shaft", detail)?);
    }
    for component in &resolved.components {
        let local = shapes::construct(component, detail)?;
        for mut part in local {
            part.solid = part.solid.transform(component.rotation, component.offset);
            if let Some(pivot) = part.animation_pivot {
                part.animation_pivot =
                    Some(add(rotate(pivot, component.rotation), component.offset));
            }
            if let Some(bore) = &mut part.bore {
                bore.touchhole_centerline = bore
                    .touchhole_centerline
                    .map(|p| add(rotate(p, component.rotation), component.offset));
                bore.bore_center = add(
                    rotate(bore.bore_center, component.rotation),
                    component.offset,
                );
            }
            parts.push(part);
        }
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_gameplay_catalog_constructs_every_chassis() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../catalog/gameplay.json")).unwrap();
        for design in catalog.as_array().unwrap() {
            let recipe: Recipe = serde_json::from_value(design["recipe"].clone())
                .unwrap_or_else(|e| panic!("{}: {e}", design["catalog_id"]));
            let model = generate_model(&recipe, Detail::Medium)
                .unwrap_or_else(|e| panic!("{}: {e}", design["catalog_id"]));
            assert!(model.physical.mass_kg > 0.0);
            println!("{}: {} kg", design["catalog_id"], model.physical.mass_kg);
        }
    }
    #[test]
    fn spear_is_fully_constructed_and_display_lod_does_not_change_physics() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../catalog/authoring.json")).unwrap();
        let preset = catalog["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "short-spear")
            .unwrap();
        let recipe: Recipe = serde_json::from_value(preset["definition"].clone()).unwrap();
        let low = generate_model(&recipe, Detail::Low).unwrap();
        let high = generate_model(&recipe, Detail::High).unwrap();
        assert_eq!(low.parts.len(), 4);
        assert_eq!(low.physical, high.physical);
        assert!(low.stats.triangles < high.stats.triangles);
        assert!(low.physical.mass_kg > 0.5 && low.physical.mass_kg < 3.0);
        assert!((low.stats.bounds.max[1] - 2.03).abs() < 1e-12);
    }

    #[test]
    fn supported_catalog_constructs_closed_finite_parts() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../catalog/authoring.json")).unwrap();
        for preset in catalog["presets"].as_array().unwrap() {
            let recipe: Recipe = serde_json::from_value(preset["definition"].clone()).unwrap();

            let model = generate_model(&recipe, Detail::Medium)
                .unwrap_or_else(|error| panic!("{}: {error}", preset["id"]));
            assert!(
                model.physical.mass_kg.is_finite() && model.physical.mass_kg > 0.0,
                "{}",
                preset["id"]
            );
            assert!(
                model.positions.iter().all(|v| v.is_finite()),
                "{}",
                preset["id"]
            );
            for part in &model.physical.components {
                assert!(part.mass_kg > 0.0, "{}: {}", preset["id"], part.label);
            }
            println!(
                "{}: {} parts, {} triangles",
                preset["id"],
                model.parts.len(),
                model.stats.triangles
            );
        }
    }

    #[test]
    fn centered_head_mount_uses_receiving_face_and_excludes_crown_extension() {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../../catalog/authoring.json")).unwrap();
        let preset = catalog["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "gothic-flanged-mace")
            .unwrap();
        let recipe: Recipe = serde_json::from_value(preset["definition"].clone()).unwrap();
        let model = generate_model(&recipe, Detail::Low).unwrap();
        assert!((model.stats.bounds.max[1] - 0.693).abs() < 1e-12);
    }

    #[test]
    fn rotated_assembly_contact_uses_its_nonzero_named_node() {
        let recipe = serde_json::json!({"components":[
            {"id":"parent","kind":"box","size":[0.1,0.1,0.1],
             "attach":{"to":"weapon.root","at":"origin"}},
            {"id":"branch","kind":"guardAssembly","rotation":[180,0,0],
             "attach":{"to":"parent.top","at":"center"},
             "anchorNode":"seat","nodes":{"seat":[0.02,0.03,0],"tip":[0.02,0.43,0]},
             "members":[{"path":["seat","tip"],"sectionWidth":0.006,"sectionDepth":0.006}]}
        ]});
        for detail in [Detail::Low, Detail::Medium, Detail::High] {
            let model =
                generate_model(&serde_json::from_value(recipe.clone()).unwrap(), detail).unwrap();
            let component = &model.resolved_definition.recipe.components[1];
            let offset = component.offset.unwrap().map(Metres::get);
            // The half-turn maps the named seat to [0.02,-0.03,0].
            let contact = [offset[0] + 0.02, offset[1] - 0.03, offset[2]];
            let target = model.resolved_definition.frames["parent.top"];
            for axis in 0..3 {
                assert!((contact[axis] - target[axis]).abs() < 1e-9);
            }
        }
    }
}
