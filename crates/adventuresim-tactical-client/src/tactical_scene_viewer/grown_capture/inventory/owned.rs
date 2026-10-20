//! One installed geometry closure and three independently grounded root bindings.

use super::*;

pub(super) struct RootBinding {
    pub entity: Entity,
    pub placement: layout::RootPlacement,
    pub parts: Vec<BoundPart>,
}

pub(super) struct BoundPart {
    pub entity: Entity,
    pub ordinal: PartOrdinal,
}

struct ExpectedSpan {
    role: ForestPart,
    records: RecordSpan,
}

impl OwnedGrown {
    pub(in super::super) fn install(
        geometry: GrownGeometry,
        encoding: GrownVertexEncoding,
        placements: layout::RootPlacements,
        control: controls::BatchControl,
        world: &mut World,
    ) -> Result<Self> {
        let assets = GrownAssets::install(geometry, encoding, world)?;
        let mut roots = Vec::new();
        for placement in placements.iter().copied() {
            let entity = world
                .spawn((
                    GrownRoot,
                    placement.ordinal,
                    placement.frame,
                    GlobalTransform::default(),
                    Visibility::Visible,
                ))
                .id();
            let mut parts = Vec::new();
            for (ordinal, asset) in assets.parts().enumerate() {
                let ordinal = PartOrdinal(ordinal);
                let mut child = world.spawn((
                    ordinal,
                    Mesh3d(asset.mesh().clone()),
                    asset.bounds(),
                    NoAutoAabb,
                    Transform::IDENTITY,
                    GlobalTransform::default(),
                    Visibility::Inherited,
                    ChildOf(entity),
                ));
                if control == controls::BatchControl::ForcedSingletons {
                    child.insert(NoAutomaticBatching);
                }
                match asset.material() {
                    GrownMaterialHandle::Wood(handle) => {
                        child.insert(MeshMaterial3d(handle.clone()));
                    }
                    GrownMaterialHandle::Leaves(handle) => {
                        child.insert(MeshMaterial3d(handle.clone()));
                    }
                }
                parts.push(BoundPart {
                    entity: child.id(),
                    ordinal,
                });
            }
            roots.push(RootBinding {
                entity,
                placement,
                parts,
            });
        }
        let owned = Self { assets, roots };
        owned.check_source()?;
        Ok(owned)
    }

    pub(in super::super) fn check_source(&self) -> Result<()> {
        let model = self
            .assets
            .bank()
            .library()
            .models
            .get(self.assets.variant().index())
            .context("held grown source")?;
        ensure!(
            model.source == self.assets.source(),
            "held grown source identity drift"
        );
        ensure!(
            self.roots.len() == layout::ROOT_COUNT
                && self
                    .roots
                    .iter()
                    .all(|root| root.parts.len() == self.assets.parts().len()),
            "held grown root/part count drift"
        );
        for expected in [
            ExpectedSpan {
                role: ForestPart::Branches,
                records: model.branches,
            },
            ExpectedSpan {
                role: ForestPart::Leaves,
                records: model.leaves,
            },
        ] {
            let source = expected.records;
            let mut start = source.start;
            for asset in self
                .assets
                .parts()
                .filter(|asset| asset.role() == expected.role)
            {
                let records = asset.records();
                ensure!(
                    records.start == start && records.count > 0,
                    "grown partition gap/overlap"
                );
                start = start
                    .checked_add(records.count)
                    .context("grown partition overflow")?;
            }
            ensure!(
                start
                    == source
                        .start
                        .checked_add(source.count)
                        .context("grown span overflow")?,
                "grown source incomplete"
            );
        }
        Ok(())
    }

    pub(in super::super) fn level(&self) -> Level {
        self.assets.level()
    }

    pub(in super::super) fn last_root(&self) -> Result<Entity> {
        Ok(self.roots.last().context("grown last root")?.entity)
    }

    pub(in super::super) fn last_part(&self) -> Result<Entity> {
        Ok(self
            .roots
            .last()
            .context("grown last root")?
            .parts
            .last()
            .context("grown last part")?
            .entity)
    }
}
