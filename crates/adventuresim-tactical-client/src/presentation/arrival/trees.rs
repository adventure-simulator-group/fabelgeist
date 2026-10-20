//! CPU installation joins; actual recorded GPU membership is checked separately.
use super::super::obstacles::tree::{
    GrownCatalogueAssets, PendingTreePresentation, StreamedTreePresentation,
};
use super::super::vista::{GrownVistaPart, VistaTreeCensus, VistaTreePresentation};
use super::super::{GrownPartBinding, GrownRootAttachment, PresentedTree};
use adventuresim_environment::render::{
    Level,
    grown::{GrownMaterialHandle, GrownPartAsset},
};
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub(super) struct InstalledTree {
    pub level: Level,
    pub parts: usize,
}

pub(super) fn local(
    world: &World,
    scene: Entity,
    root: Entity,
    catalogue: &GrownCatalogueAssets,
) -> Result<Option<InstalledTree>> {
    if world.get::<PresentedTree>(root).is_none()
        || world.get::<PendingTreePresentation>(root).is_some()
    {
        return Ok(None);
    }
    let Some(StreamedTreePresentation::Grown(stream)) = world.get::<StreamedTreePresentation>(root)
    else {
        return Ok(None);
    };
    let Some(level) = stream.selected_level() else {
        return Ok(None);
    };
    let attachment: GrownRootAttachment = stream.attachment(root)?;
    let frame = world
        .get::<Transform>(root)
        .context("arrival physical tree transform missing")?;
    ensure!(
        attachment.scene == scene
            && attachment.bank == catalogue.bank_assets().bank().id()
            && attachment.source == catalogue.select(frame.translation),
        "arrival local tree owner/source changed"
    );
    let group = attachment
        .groups()
        .find(|group| group.level == level)
        .context("arrival selected tree group missing")?;
    let Some(children) = world.get::<Children>(group.entity) else {
        return Ok(None);
    };
    let closure = catalogue.assets(attachment.source, level)?;
    if children.len() != closure.parts().len() {
        return Ok(None);
    }
    for asset in closure.parts() {
        let expected = GrownPartBinding {
            scene,
            root,
            bank: attachment.bank,
            source: attachment.source,
            level,
            role: asset.role(),
            records: asset.records(),
        };
        let matching = children
            .iter()
            .filter(|entity| world.get::<GrownPartBinding>(*entity) == Some(&expected))
            .collect::<Vec<_>>();
        ensure!(
            matching.len() == 1,
            "arrival local source partition missing/duplicated"
        );
        ensure!(
            part_matches(world, matching[0], group.entity, asset),
            "arrival selected tree part is incomplete"
        );
    }
    Ok(Some(InstalledTree {
        level,
        parts: children.len(),
    }))
}

pub(super) fn vista(
    world: &World,
    scene: Entity,
    catalogue: &GrownCatalogueAssets,
) -> Result<Option<usize>> {
    let Some(census) = world.get::<VistaTreeCensus>(scene) else {
        return Ok(None);
    };
    let roots = census.roots.iter().copied().collect::<HashSet<_>>();
    let parts = census.parts.iter().copied().collect::<HashSet<_>>();
    ensure!(
        roots.len() == census.roots.len() && parts.len() == census.parts.len(),
        "arrival vista census has duplicate identities"
    );
    let mut counts = HashMap::<Entity, usize>::new();
    for entity in &census.parts {
        let actual = world
            .get::<GrownVistaPart>(*entity)
            .context("arrival vista part binding missing")?;
        let binding = &actual.0;
        ensure!(
            roots.contains(&binding.root)
                && binding.scene == scene
                && binding.bank == catalogue.bank_assets().bank().id()
                && binding.level == Level::Coarse,
            "arrival vista part owner/bank/level differs"
        );
        let closure = catalogue.assets(binding.source, Level::Coarse)?;
        let mut matching = closure
            .parts()
            .filter(|asset| asset.role() == binding.role && asset.records() == binding.records);
        let asset = matching
            .next()
            .context("arrival vista source partition absent")?;
        ensure!(
            matching.next().is_none() && part_matches(world, *entity, binding.root, asset),
            "arrival vista part mesh/material/hierarchy differs"
        );
        *counts.entry(binding.root).or_default() += 1;
    }
    for root in &census.roots {
        ensure!(
            world.get::<VistaTreePresentation>(*root).is_some(),
            "arrival vista root marker missing"
        );
        let frame = world
            .get::<Transform>(*root)
            .context("arrival vista ground frame missing")?;
        let source = catalogue.select(frame.translation);
        let expected = catalogue.assets(source, Level::Coarse)?.parts().len();
        ensure!(
            counts.get(root).copied() == Some(expected),
            "arrival vista root part census differs"
        );
        for entity in world
            .get::<Children>(*root)
            .context("arrival vista children missing")?
        {
            ensure!(
                parts.contains(entity),
                "arrival vista has an undeclared child"
            );
            ensure!(
                world
                    .get::<GrownVistaPart>(*entity)
                    .is_some_and(|part| part.0.source == source),
                "arrival vista source differs from placement"
            );
        }
    }
    Ok(Some(parts.len()))
}

fn part_matches(world: &World, entity: Entity, parent: Entity, asset: &GrownPartAsset) -> bool {
    world
        .get::<Mesh3d>(entity)
        .is_some_and(|mesh| mesh.0.id() == asset.mesh().id())
        && world
            .get::<ChildOf>(entity)
            .is_some_and(|child| child.parent() == parent)
        && match asset.material() {
            GrownMaterialHandle::Wood(material) => world
                .get::<MeshMaterial3d<adventuresim_environment::render::grown::GrownWoodMaterial>>(
                    entity,
                )
                .is_some_and(|actual| actual.0.id() == material.id()),
            GrownMaterialHandle::Leaves(material) => world
                .get::<MeshMaterial3d<adventuresim_environment::render::grown::GrownLeafMaterial>>(
                    entity,
                )
                .is_some_and(|actual| actual.0.id() == material.id()),
        }
}
