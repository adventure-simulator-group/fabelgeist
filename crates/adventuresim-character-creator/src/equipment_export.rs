use super::character_morphs::{
    CharacterMorphs, MorphDelta, armor_targets, rigged_armor, rigged_clothing,
};
use super::*;
use adventuresim_character_creator::item_catalog_schema::EquipmentPlacement;

pub(super) fn generate_equipment_assets(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    breastplate_design: &BreastplateDesign,
    item_filter: &[String],
) -> Result<()> {
    if !item_filter.is_empty() {
        for filter in item_filter {
            if !procedural_items(catalog).any(|item| item.id == *filter) {
                anyhow::bail!("unknown procedural equipment item {filter}");
            }
        }
        if output.exists()
            && std::fs::read_dir(output)
                .with_context(|| format!("reading equipment output {}", output.display()))?
                .next()
                .is_some()
        {
            anyhow::bail!(
                "filtered equipment export requires a new or empty staging directory: {}",
                output.display()
            );
        }
    }
    std::fs::create_dir_all(output)
        .with_context(|| format!("creating equipment output {}", output.display()))?;
    let generated = generate_character(model, recipe)?;
    let morphs = CharacterMorphs::generate(model, recipe, &generated)?;
    let exporter = EquipmentExporter {
        model,
        recipe,
        generated: &generated,
        morphs: &morphs,
        breastplate_design,
        catalog,
    };
    let mut assets = Vec::new();
    let mut generated_files = std::collections::BTreeSet::new();
    for item in procedural_items(catalog) {
        if !item_filter.is_empty() && !item_filter.contains(&item.id) {
            continue;
        }
        let equipment = item.equipment.as_ref().expect("filtered equipment");
        for placement in &equipment.placements {
            if placement.surface.is_empty() {
                continue;
            }
            let asset = if adventuresim_character_creator::armor_recipes::is_parametric(&item.id) {
                exporter.armor(output, item, placement)?
            } else {
                anyhow::ensure!(
                    !matches!(
                        item.kind,
                        adventuresim_character_creator::item_catalog_schema::ItemKind::Armor { .. }
                    ),
                    "armor {} has no parametric generator",
                    item.id
                );
                exporter.garment(output, item, placement)?
            };
            generated_files.insert(format!("{}--{}.glb", item.id, placement.id));
            assets.push(asset);
        }
    }
    for entry in std::fs::read_dir(output)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "glb")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !generated_files.contains(name))
        {
            std::fs::remove_file(&path)
                .with_context(|| format!("removing stale generated asset {}", path.display()))?;
        }
    }
    let manifest = serde_json::json!({
        "schema_version": 1,
        "mhr_release": "v1.0.1",
        "lod": model.lod,
        "assets": assets,
    });
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}

struct EquipmentExporter<'a> {
    catalog: &'a EquipmentCatalog,
    model: &'a BodyModel,
    recipe: &'a CharacterRecipe,
    generated: &'a GeneratedCharacter,
    morphs: &'a CharacterMorphs,
    breastplate_design: &'a BreastplateDesign,
}

impl EquipmentExporter<'_> {
    fn mesh(&self) -> RiggedMesh<'_> {
        let character = &self.model.mhr.character;
        RiggedMesh {
            joint_proportions: &self.generated.joint_proportions,
            morph_targets: &[],
            positions: &self.generated.positions,
            normals: &self.generated.normals,
            faces: &character.mesh.faces,
            export_body: false,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            joint_parents: &character.skeleton.parents,
            global_joint_states: &self.generated.global_joint_states,
        }
    }
    fn armor(
        &self,
        output: &std::path::Path,
        item: &ItemDefinition,
        placement: &EquipmentPlacement,
    ) -> Result<serde_json::Value> {
        let model = self.model;
        let recipe = self.recipe;
        let generated = self.generated;
        let morphs = self.morphs;
        let equipment = item.equipment.as_ref().expect("filtered equipment");
        let breastplate_design = self.breastplate_design;
        let (armor, parametric_coverage) = if item.id == "vambrace" {
            let side = match placement.id.as_str() {
                "left" => ForearmSide::Left,
                "right" => ForearmSide::Right,
                _ => {
                    anyhow::bail!("vambrace placement {} has no forearm side", placement.id)
                }
            };
            let design = BracerDesign::default();
            (
                fitted_bracer(model, generated, &design, side, &morphs.samples)?,
                design.coverage.unit(),
            )
        } else if matches!(item.id.as_str(), "breastplate" | "cuirass") {
            (
                fitted_breastplate(model, generated, breastplate_design, &morphs.samples)?,
                placement_coverage(placement),
            )
        } else {
            (
                parametric_equipment::fitted_design(
                    model,
                    generated,
                    &self
                        .catalog
                        .design(&item.id)
                        .context("missing parametric recipe")?,
                    &placement.id,
                    &morphs.samples,
                )?,
                placement_coverage(placement),
            )
        };
        let faces = armor.indices.as_chunks::<3>().0.to_vec();
        let morph_targets = armor_targets(&armor);
        let file_name = format!("{}--{}.glb", item.id, placement.id);
        let path = output.join(&file_name);
        let mut rigged_shell = rigged_armor(&item.display_name, &armor, &faces, &morph_targets);
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(
            equipment.material.context("armor material missing")?,
        );
        rigged_shell.base_color = color;
        rigged_shell.metallic = metallic;
        rigged_shell.roughness = roughness;
        export_rigged_glb(
            &path,
            &item.id,
            recipe.version,
            model.lod,
            &self.mesh(),
            &[rigged_shell],
            &[],
        )?;
        Ok(serde_json::json!({
            "item_id": item.id,
            "placement_id": placement.id,
            "file": file_name,
            "coverage": parametric_coverage,
            "material": equipment.material,
            "triangles": faces.len(),
            "armor_generator_version": adventuresim_armor_model::GENERATOR_VERSION,
            "armor_design_hash": armor.design_hash.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "morph_targets": armor.morphs.len(),
            "surface_uv_domain": armor.surface_domain,
        }))
    }
    fn garment(
        &self,
        output: &std::path::Path,
        item: &ItemDefinition,
        placement: &EquipmentPlacement,
    ) -> Result<serde_json::Value> {
        let model = self.model;
        let recipe = self.recipe;
        let generated = self.generated;
        let morphs = self.morphs;
        let character = &model.mhr.character;
        let equipment = item.equipment.as_ref().expect("filtered equipment");
        let specification = GarmentSpecification::from_catalog(
            format!("{} · {}", item.display_name, placement.id),
            placement,
            equipment
                .material
                .ok_or_else(|| anyhow::anyhow!("item {} has no procedural material", item.id))?,
        );
        let clothed = generate_clothing_shells(
            &[specification],
            &generated.positions,
            &generated.normals,
            &character.mesh.faces,
            &character.skin_weights.index,
            &character.skin_weights.weight,
            &character.skeleton.names,
            &generated.global_joint_states,
        )
        .map_err(anyhow::Error::msg)?;
        let targets = morphs.clothing(&clothed.shells)?;
        let targets = targets[0]
            .iter()
            .map(MorphDelta::rigged)
            .collect::<Vec<_>>();
        let shell = &clothed.shells[0];
        let file_name = format!("{}--{}.glb", item.id, placement.id);
        let path = output.join(&file_name);
        let rigged_shell = rigged_clothing(shell, &targets);
        let rigged_mesh = self.mesh();
        let sockets = self.sockets(item, &rigged_mesh, &rigged_shell)?;
        export_rigged_glb(
            &path,
            &item.id,
            recipe.version,
            model.lod,
            &rigged_mesh,
            &[rigged_shell],
            &sockets,
        )?;
        Ok(serde_json::json!({
            "item_id": item.id,
            "placement_id": placement.id,
            "file": file_name,
            "coverage": placement_coverage(placement),
            "material": equipment.material,
            "triangles": shell.faces.len(),
            "morph_targets": targets.len(),
        }))
    }
    fn sockets<'a>(
        &self,
        item: &'a ItemDefinition,
        rigged_mesh: &RiggedMesh<'_>,
        rigged_shell: &RiggedShell<'_>,
    ) -> Result<Vec<RiggedSocket<'a>>> {
        let character = &self.model.mhr.character;
        let equipment = item.equipment.as_ref().expect("filtered equipment");
        let surface_uv_layout = SurfaceUvLayout {
            domain: MHR_ANATOMICAL_UV_DOMAIN,
            texcoords: &character.mesh.texcoords,
            texcoord_faces: &character.mesh.texcoord_faces,
        };
        equipment
            .attachment_points
            .iter()
            .filter(|point| point.tangent_direction.is_some())
            .map(|point| {
                let tangent = point.tangent_direction.expect("filtered tangent");
                let outward = point
                    .locations
                    .iter()
                    .copied()
                    .find_map(belt_mount_outward)
                    .with_context(|| {
                        format!(
                            "item {} attachment point {} has a tangent but no belt location",
                            item.id, point.id
                        )
                    })?;
                let surface = point.surface_uv.as_ref().with_context(|| {
                    format!(
                        "item {} attachment point {} has a tangent but no anatomical surface UV",
                        item.id, point.id
                    )
                })?;
                if surface.domain != surface_uv_layout.domain {
                    anyhow::bail!(
                        "item {} attachment point {} uses unsupported anatomical UV domain {}",
                        item.id,
                        point.id,
                        surface.domain
                    );
                }
                fitted_equipment_socket_from_uv(
                    rigged_mesh,
                    rigged_shell,
                    &surface_uv_layout,
                    surface.uv,
                    outward,
                    tangent,
                )
                .with_context(|| {
                    format!(
                        "could not fit item {} attachment point {}",
                        item.id, point.id
                    )
                })
                .map(|transform| RiggedSocket {
                    attachment_point_id: &point.id,
                    surface_uv_domain: &surface.domain,
                    surface_uv: surface.uv,
                    transform,
                })
            })
            .collect::<Result<Vec<_>>>()
    }
}
