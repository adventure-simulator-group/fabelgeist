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

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the scene, caches and drape job this system rebuilds from"
)]
pub(super) fn regenerate_mesh(
    mut scene: PreviewScene,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
    mut cache: ResMut<studio_cache::StudioCache>,
    old: Query<(Entity, Has<drape_preview::DrapeMesh>), With<CharacterMesh>>,
    mut drape_job: ResMut<drape_preview::DrapeJob>,
    mut walk: ResMut<WalkPreview>,
    mut contexts: EguiContexts,
) {
    if !studio.dirty || dragging(&mut contexts) {
        return;
    }
    studio.dirty = false;
    let started = std::time::Instant::now();
    let recipe = studio.recipe.clone();
    let prepared = outfit::loadout(&recipe, &catalog).and_then(|loadout| {
        let generated = cache.body(&model, &recipe)?;
        let clothed =
            outfit::clothing(&model, &loadout, &generated).context("Clothing generation failed")?;
        let armor = cache
            .armor(&model, &generated, &loadout)
            .context("Parametric armor generation failed")?;
        let lining = cache.lining(&loadout)?;
        if drape_job.take_again() {
            cache.forget_drape();
        }
        let drape = cache
            .drape_changed(&loadout)?
            .then(|| outfit::drape_inputs(&model, &generated, &loadout, lining.as_ref()));
        Ok((generated, clothed, armor, drape))
    });
    let (generated, clothed, armor, drape) = match prepared {
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
    // Draped cloth stays shown while nothing it lies on changed.
    let redrape = drape.is_some();
    if let Some(inputs) = drape {
        drape_job.request(inputs);
    }
    let clothing_shell_count = clothed.shells.len();
    scene.bounds.0 = studio_scene::CharacterBounds::of(&generated.positions);
    for (entity, draped) in &old {
        if redrape || !draped {
            scene.commands.entity(entity).despawn();
        }
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
    let spawned = scene.spawn_equipment(&catalog, &armor);
    studio.status = match spawned {
        Ok(()) => format!(
            "Generated {} body vertices · {} clothing shells · {} armor pieces in {:.2} s",
            model.mhr.num_vertices(),
            clothing_shell_count,
            armor.len(),
            started.elapsed().as_secs_f32(),
        ),
        Err(error) => format!("{error:#}"),
    };
}

/// Whether a control is being dragged: an edit then rebuilds once, when it is
/// let go, rather than at every value it passes through.
pub(super) fn dragging(contexts: &mut EguiContexts) -> bool {
    contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.egui_is_using_pointer())
}

impl PreviewScene<'_, '_> {
    fn spawn_equipment(
        &mut self,
        catalog: &EquipmentCatalog,
        armor: &[parametric_equipment::SelectedArmor<'_>],
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
            if let Some(lacing) = &piece.lacing {
                preview::spawn_armor(
                    &mut self.commands,
                    &mut self.meshes,
                    &mut self.materials,
                    &lacing.generated,
                    lacing.name.clone(),
                    preview::ArmorShading {
                        plate: preview::lacing_material(&lacing.cord),
                        trim: None,
                    },
                    CharacterMesh,
                )
                .context("Lacing preview failed")?;
            }
        }
        Ok(())
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
