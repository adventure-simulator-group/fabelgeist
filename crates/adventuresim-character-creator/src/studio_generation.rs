use super::*;
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the independent scene, asset and generation resources into this system"
)]
pub(super) fn regenerate_mesh(
    mut commands: Commands,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
    old: Query<Entity, With<CharacterMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut mail_maps: ResMut<underlayer_preview::MailMaps>,
) {
    if !studio.dirty {
        return;
    }
    studio.dirty = false;
    let generated = match generate_character(&model, &studio.recipe) {
        Ok(generated) => generated,
        Err(error) => {
            studio.status = format!("Generation failed: {error:#}");
            return;
        }
    };
    let faces = &model.mhr.character.mesh.faces;
    let specifications = match selected_garments(&studio.recipe, &catalog) {
        Ok(specifications) => specifications,
        Err(error) => {
            studio.status = format!("Clothing selection failed: {error}");
            return;
        }
    };
    let clothed = match generate_clothing_shells(
        &specifications,
        &generated.positions,
        &generated.normals,
        faces,
        &model.mhr.character.skin_weights.index,
        &model.mhr.character.skin_weights.weight,
        &model.mhr.character.skeleton.names,
        &generated.global_joint_states,
    ) {
        Ok(clothed) => clothed,
        Err(error) => {
            studio.status = format!("Clothing generation failed: {error}");
            return;
        }
    };
    let armor = match parametric_equipment::selected(
        &model,
        &generated,
        &studio.recipe,
        &catalog,
        &studio.bracer_design,
        &studio.breastplate_design,
        &[],
    ) {
        Ok(armor) => armor,
        Err(error) => {
            studio.status = format!("Parametric armor generation failed: {error:#}");
            return;
        }
    };
    let clothing_shell_count = clothed.shells.len();
    let mesh = visible_body_mesh(&generated, &clothed.visible_body_faces);
    for entity in &old {
        commands.entity(entity).despawn();
    }
    preview::spawn_body(&mut commands, &mut meshes, &mut materials, mesh);
    preview::spawn_clothing(&mut commands, &mut meshes, &mut materials, clothed.shells);
    for piece in &armor {
        let material = catalog
            .material(&piece.item_id)
            .expect("selected catalog equipment has a material");
        if let Err(error) = preview::spawn_armor(
            &mut commands,
            &mut meshes,
            &mut materials,
            &piece.generated,
            piece.name.clone(),
            mail_maps.material(
                &mut images,
                material,
                catalog.design(&piece.item_id, &piece.placement_id).as_ref(),
            ),
        ) {
            studio.status = format!("Armor preview failed: {error:#}");
            return;
        }
    }
    studio.status = format!(
        "Generated {} body vertices · {} clothing shells · {} armor pieces",
        model.mhr.num_vertices(),
        clothing_shell_count,
        armor.len(),
    );
}

fn visible_body_mesh(generated: &GeneratedCharacter, faces: &[[u32; 3]]) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, generated.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, generated.normals.clone())
    .with_inserted_indices(Indices::U32(faces.iter().flatten().copied().collect()))
}
