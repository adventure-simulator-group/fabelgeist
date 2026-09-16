//! A single replicated leaf remains visible across the Detail/Facade transition.
use super::{BuildingRenderLevel, building_lod_visibility};
use bevy::{camera::visibility::VisibilityRange, prelude::*};

#[derive(Component)]
pub(crate) struct PresentedBuildingClosureMesh {
    building: u64,
    opening: adventuresim_building_generator::OpeningAssemblyId,
    pub(crate) facade: bool,
}
impl PresentedBuildingClosureMesh {
    pub(super) fn new(building: u64, opening: u64) -> Self {
        Self {
            building,
            opening: adventuresim_building_generator::OpeningAssemblyId(opening),
            facade: false,
        }
    }
}
#[derive(Component)]
pub(super) struct FacadeOpenings(
    pub std::collections::BTreeSet<adventuresim_building_generator::OpeningAssemblyId>,
);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BuildingClosureVisibility;

pub(super) fn sync(
    buildings: Query<(&super::SceneBuilding, &FacadeOpenings)>,
    mut leaves: Query<(&mut PresentedBuildingClosureMesh, &mut VisibilityRange)>,
) {
    for (mut leaf, mut range) in &mut leaves {
        let Some((_, openings)) = buildings
            .iter()
            .find(|(building, _)| building.id == leaf.building)
        else {
            continue;
        };
        let facade = openings.0.contains(&leaf.opening);
        if facade != leaf.facade {
            leaf.facade = facade;
            *range = if facade {
                visibility()
            } else {
                building_lod_visibility(BuildingRenderLevel::Lod0)
            };
        }
    }
}

pub(super) fn visibility() -> VisibilityRange {
    VisibilityRange {
        start_margin: 0.0..0.0,
        ..building_lod_visibility(BuildingRenderLevel::Lod1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_selects_all_leaf_batches_without_changing_open_transforms() {
        use adventuresim_building_generator::{
            BuildingArchetype, BuildingProgram, OpeningAssemblyId,
        };
        use adventuresim_tactical_core::prelude::BuildingOrientation;
        let mut app = App::new();
        app.add_systems(Update, sync);
        let building = app
            .world_mut()
            .spawn((
                super::super::SceneBuilding {
                    id: 7,
                    program: BuildingProgram::fixture(BuildingArchetype::TownHouse, 42),
                    orientation: BuildingOrientation::IDENTITY,
                },
                FacadeOpenings(
                    [OpeningAssemblyId(1), OpeningAssemblyId(2)]
                        .into_iter()
                        .collect(),
                ),
            ))
            .id();
        let open = Transform::from_xyz(3.0, 2.0, 1.0).with_rotation(Quat::from_rotation_y(1.1));
        // Door, glass, its two material children, and an unsupported opening.
        let entities = [1, 2, 2, 2, 3].map(|opening| {
            app.world_mut()
                .spawn((
                    PresentedBuildingClosureMesh::new(7, opening),
                    building_lod_visibility(BuildingRenderLevel::Lod0),
                    open,
                ))
                .id()
        });
        app.update();
        for (index, entity) in entities.iter().enumerate() {
            assert_eq!(*app.world().get::<Transform>(*entity).unwrap(), open);
            assert_eq!(
                app.world()
                    .get::<PresentedBuildingClosureMesh>(*entity)
                    .unwrap()
                    .facade,
                index < 4
            );
            let range = app.world().get::<VisibilityRange>(*entity).unwrap();
            assert_eq!(
                range.end_margin,
                building_lod_visibility(if index < 4 {
                    BuildingRenderLevel::Lod1
                } else {
                    BuildingRenderLevel::Lod0
                })
                .end_margin
            );
        }
        app.world_mut()
            .entity_mut(building)
            .insert(FacadeOpenings(Default::default()));
        app.update();
        for entity in entities {
            assert!(
                !app.world()
                    .get::<PresentedBuildingClosureMesh>(entity)
                    .unwrap()
                    .facade
            );
            assert_eq!(*app.world().get::<Transform>(entity).unwrap(), open);
        }
    }

    #[test]
    fn leaf_range_has_no_detail_facade_fade_or_gap() {
        let leaf = visibility();
        let detail = building_lod_visibility(BuildingRenderLevel::Lod0);
        let shell = building_lod_visibility(BuildingRenderLevel::Lod2);
        assert_eq!(leaf.start_margin, 0.0..0.0);
        assert!(leaf.end_margin.start > detail.end_margin.end);
        assert_eq!(leaf.end_margin, shell.start_margin);
    }
}
