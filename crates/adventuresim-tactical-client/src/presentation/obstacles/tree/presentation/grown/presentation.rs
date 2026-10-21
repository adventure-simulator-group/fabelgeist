//! Attach shared ordinary meshes without replacing or moving physics roots.

use super::streaming::GrownLevelGroup;
use super::*;
use crate::presentation::obstacles::tree::presentation::{
    PendingTreePresentation, PresentedTree, RecipeTreeAssetResidencyDiagnostics,
    RetainedTreePresentations, StreamedTreePresentation, TreePresentationCache,
};
use adventuresim_environment::render::grown::GrownMaterialHandle;
use bevy::camera::visibility::NoAutoAabb;

pub(in super::super) fn present(
    world: &mut World,
    scene: Entity,
    request: &GrownSceneRequest,
) -> Result<()> {
    let environment = world
        .get::<SceneEnvironment>(scene)
        .context("grown scene environment")?
        .clone();
    adventuresim_tactical_core::prelude::SceneTreeContent::Grown
        .validate_environment(&environment)?;
    let roots = super::roots::admit(world, scene)?;
    let catalogue = world.resource_scope(|world, mut cache: Mut<TreePresentationCache>| {
        world.resource_scope(|world, mut retained: Mut<RetainedTreePresentations>| {
            world.resource_scope(
                |world, mut diagnostics: Mut<RecipeTreeAssetResidencyDiagnostics>| {
                    retained.activate_grown(
                        scene,
                        &environment,
                        request,
                        &mut cache,
                        &mut diagnostics,
                        world,
                    )
                },
            )
        })
    })?;
    for root in roots {
        attach(world, scene, root, &catalogue, request.selection())?;
    }
    Ok(())
}

fn attach(
    world: &mut World,
    scene: Entity,
    root: super::roots::PhysicalTreeRoot,
    catalogue: &GrownCatalogueAssets,
    selection: LevelSelection,
) -> Result<()> {
    let source = catalogue.select(root.frame.translation);
    let visual = world
        .spawn((
            Name::new("Grown tree visual frame"),
            GrownVisualFrame,
            Transform::from_translation(-super::super::super::source::source_normalization()),
            GlobalTransform::default(),
            Visibility::Inherited,
            ChildOf(root.entity),
        ))
        .id();
    let groups = match selection {
        LevelSelection::Fixed(level) => {
            attach_parts(
                world,
                scene,
                root.entity,
                visual,
                source,
                catalogue.assets(source, level)?,
            )?;
            None
        }
        LevelSelection::Adaptive => Some(GrownLevels {
            full: attach_level(
                world,
                scene,
                root.entity,
                visual,
                source,
                catalogue.assets(source, Level::Full)?,
            )?,
            middle: attach_level(
                world,
                scene,
                root.entity,
                visual,
                source,
                catalogue.assets(source, Level::Middle)?,
            )?,
            coarse: attach_level(
                world,
                scene,
                root.entity,
                visual,
                source,
                catalogue.assets(source, Level::Coarse)?,
            )?,
        }),
    };
    world
        .entity_mut(root.entity)
        .insert((
            Name::new("Presented grown tree"),
            PresentedTree,
            Visibility::Inherited,
            StreamedTreePresentation::Grown(GrownTreeStream::new(
                scene, visual, source, catalogue, selection, root.frame, groups,
            )),
        ))
        .remove::<PendingTreePresentation>();
    Ok(())
}

fn attach_level(
    world: &mut World,
    scene: Entity,
    root: Entity,
    visual: Entity,
    source: GrownSource,
    assets: &GrownAssets,
) -> Result<Entity> {
    let group = world
        .spawn((
            Name::new("Grown detail level"),
            GrownLevelGroup {
                root,
                level: assets.level(),
            },
            Transform::IDENTITY,
            GlobalTransform::default(),
            Visibility::Hidden,
            ChildOf(visual),
        ))
        .id();
    attach_parts(world, scene, root, group, source, assets)?;
    Ok(group)
}

fn attach_parts(
    world: &mut World,
    scene: Entity,
    root: Entity,
    visual: Entity,
    source: GrownSource,
    assets: &GrownAssets,
) -> Result<()> {
    for part in assets.parts() {
        let mut child = world.spawn((
            GrownPartBinding {
                scene,
                root,
                bank: assets.bank().id(),
                source,
                level: assets.level(),
                role: part.role(),
                records: part.records(),
            },
            Mesh3d(part.mesh().clone()),
            part.bounds(),
            NoAutoAabb,
            Transform::IDENTITY,
            GlobalTransform::default(),
            Visibility::Inherited,
            ChildOf(visual),
        ));
        match part.material() {
            GrownMaterialHandle::Wood(material) => {
                child.insert(MeshMaterial3d(material.clone()));
            }
            GrownMaterialHandle::Leaves(material) => {
                child.insert(MeshMaterial3d(material.clone()));
            }
        }
    }
    Ok(())
}
