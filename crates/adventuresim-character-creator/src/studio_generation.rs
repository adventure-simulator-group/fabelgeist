use super::*;
pub(super) fn regenerate_mesh(
    mut commands: Commands,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
    old: Query<Entity, With<CharacterMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    let indices = clothed
        .visible_body_faces
        .iter()
        .flat_map(|face| face.iter().copied())
        .collect::<Vec<_>>();
    let clothing_shell_count = clothed.shells.len();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, generated.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, generated.normals.clone())
    .with_inserted_indices(Indices::U32(indices));
    for entity in &old {
        commands.entity(entity).despawn();
    }
    preview::spawn_body(&mut commands, &mut meshes, &mut materials, mesh);
    preview::spawn_clothing(&mut commands, &mut meshes, &mut materials, clothed.shells);
    for piece in &armor {
        let material = catalog
            .material(&piece.item_id)
            .expect("selected catalog equipment has a material");
        preview::spawn_armor(
            &mut commands,
            &mut meshes,
            &mut materials,
            &piece.generated,
            piece.name.clone(),
            material,
        );
    }
    studio.status = format!(
        "Generated {} body vertices · {} clothing shells · {} armor pieces",
        model.mhr.num_vertices(),
        clothing_shell_count,
        armor.len(),
    );
}
