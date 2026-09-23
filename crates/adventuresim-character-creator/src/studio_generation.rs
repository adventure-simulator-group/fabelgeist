use super::*;
use bevy::ecs::system::SystemParam;

/// The scene and asset stores the character preview spawns into.
#[derive(SystemParam)]
pub(super) struct PreviewScene<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub equipment_maps: ResMut<'w, underlayer_preview::EquipmentMaps>,
    /// Where the character stands, for the camera's shots.
    pub bounds: ResMut<'w, studio_scene::CharacterBounds>,
    inverse_bindposes: ResMut<'w, Assets<SkinnedMeshInverseBindposes>>,
}

pub(super) fn regenerate_mesh(
    mut scene: PreviewScene,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
    old: Query<Entity, With<CharacterMesh>>,
    mut drape_job: ResMut<drape_preview::DrapeJob>,
    mut walk: ResMut<WalkPreview>,
) {
    if !studio.dirty {
        return;
    }
    studio.dirty = false;
    drape_job.request(Vec::new());
    let recipe = studio.recipe.clone();
    let prepared = outfit::loadout(&recipe, &catalog).and_then(|loadout| {
        let generated = generate_character(&model, &recipe).context("Generation failed")?;
        let clothed =
            outfit::clothing(&model, &loadout, &generated).context("Clothing generation failed")?;
        let armor = parametric_equipment::selected(&model, &generated, &loadout, &[])
            .context("Parametric armor generation failed")?;
        Ok((loadout, generated, clothed, armor))
    });
    let (loadout, generated, clothed, armor) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            studio.status = format!("{error:#}");
            return;
        }
    };
    animation_preview::rebuild(
        &mut walk,
        &mut scene.commands,
        &mut scene.inverse_bindposes,
        &model,
        &generated,
    );
    drape_job.request(outfit::drape_inputs(&model, &generated, &loadout));
    let clothing_shell_count = clothed.shells.len();
    scene.bounds.0 = studio_scene::CharacterBounds::of(&generated.positions);
    for entity in &old {
        scene.commands.entity(entity).despawn();
    }
    let PreviewScene {
        commands,
        meshes,
        materials,
        ..
    } = &mut scene;
    let mesh = visible_body_mesh(&generated, &clothed.visible_body_faces);
    preview::spawn_body(commands, meshes, materials, &walk, &model, &generated, mesh);
    preview::spawn_clothing(commands, meshes, materials, &walk, &model, clothed.shells);
    let spawned = scene.spawn_equipment(&catalog, &model, &generated, &walk, &armor, loadout.plate);
    studio.status = match spawned {
        Ok(()) => format!(
            "Generated {} body vertices · {} clothing shells · {} armor pieces",
            model.mhr.num_vertices(),
            clothing_shell_count,
            armor.len(),
        ),
        Err(error) => format!("{error:#}"),
    };
}

impl PreviewScene<'_, '_> {
    fn spawn_equipment(
        &mut self,
        catalog: &EquipmentCatalog,
        model: &BodyModel,
        generated: &GeneratedCharacter,
        walk: &WalkPreview,
        armor: &[parametric_equipment::SelectedArmor<'_>],
        plate: Option<&fabelgeist_armor::Armor>,
    ) -> Result<()> {
        self.equipment_maps.begin_generation();
        for piece in armor {
            let material = catalog.material(&piece.piece.piece.item.id)?;
            let material = self
                .equipment_maps
                .material(
                    &mut self.images,
                    material,
                    piece.piece.design.recipe(),
                    piece.piece.decoration.engraving.as_ref(),
                )
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("{} material failed", piece.name))?;
            let trim = piece
                .trim
                .as_ref()
                .map(|trim| {
                    Ok::<_, anyhow::Error>(preview::TrimPreview {
                        texcoords: &trim.texcoords,
                        material: self
                            .equipment_maps
                            .metal(&mut self.images, &trim.metal, 1.0)
                            .map_err(anyhow::Error::msg)
                            .with_context(|| format!("{} trim material failed", piece.name))?,
                    })
                })
                .transpose()?;
            preview::spawn_armor(
                &mut self.commands,
                &mut self.meshes,
                &mut self.materials,
                &piece.generated,
                piece.name.clone(),
                preview::ArmorShading {
                    plate: material,
                    trim,
                },
                CharacterMesh,
            )
            .context("Armor preview failed")?;
        }
        let Some(plate) = plate else {
            return Ok(());
        };
        armor_preview::spawn(
            plate,
            model,
            &generated.global_joint_states,
            walk,
            &mut self.commands,
            &mut self.meshes,
            &mut self.materials,
            &mut self.images,
        )
        .map_err(anyhow::Error::msg)
        .context("Plate armor generation failed")
    }
}

pub(super) fn visible_body_mesh(generated: &GeneratedCharacter, faces: &[[u32; 3]]) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, generated.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, generated.normals.clone())
    .with_inserted_indices(Indices::U32(faces.iter().flatten().copied().collect()))
}
