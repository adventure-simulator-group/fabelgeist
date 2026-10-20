//! Expand original cell visibility into ordinary automatic mesh batches.

use bevy::{
    camera::visibility::{
        RenderLayers, SetViewVisibility, VisibilityClass, VisibilitySystems, VisibleEntities,
        add_visibility_class,
    },
    prelude::*,
};
use std::any::TypeId;

pub(in crate::presentation) struct SharedLitterPlugin;

/// Cell membership is immutable until the scene hierarchy is despawned.
#[derive(Component)]
#[require(VisibilityClass)]
#[component(on_add = add_visibility_class::<LitterCellVisibility>)]
pub(super) struct LitterCellVisibility {
    patches: Vec<Entity>,
}

#[derive(Component)]
pub(super) struct SharedLitterPatch;

impl LitterCellVisibility {
    pub(super) fn new(patches: Vec<Entity>) -> Self {
        Self { patches }
    }
}

impl Plugin for SharedLitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            initialize_patch_visibility.before(VisibilitySystems::CheckVisibility),
        );
        app.add_systems(
            PostUpdate,
            expand_cell_visibility
                .after(VisibilitySystems::CheckVisibility)
                .before(VisibilitySystems::MarkNewlyHiddenEntitiesInvisible),
        );
    }
}

fn initialize_patch_visibility(mut patches: Query<&mut VisibilityClass, Added<SharedLitterPatch>>) {
    for mut class in &mut patches {
        // Mesh3d's add hook runs first. Immutable patches enter each view's
        // Mesh3d list exclusively through their original cell.
        class.clear();
    }
}

fn expand_cell_visibility(
    mut views: Query<(&Camera, Option<&RenderLayers>, &mut VisibleEntities)>,
    cells: Query<&LitterCellVisibility>,
    mut patches: Query<
        (
            &InheritedVisibility,
            Option<&RenderLayers>,
            &mut ViewVisibility,
        ),
        With<SharedLitterPatch>,
    >,
    mut visible_cells: Local<Vec<Entity>>,
) -> Result {
    for (camera, layers, mut visible) in &mut views {
        if !camera.is_active {
            continue;
        }
        visible_cells.clear();
        visible_cells.extend_from_slice(visible.get(TypeId::of::<LitterCellVisibility>()));
        let mesh_entities = visible.get_mut(TypeId::of::<Mesh3d>());
        for cell in visible_cells.iter() {
            let Ok(cell) = cells.get(*cell) else {
                continue;
            };
            for patch in &cell.patches {
                let Ok((inherited, patch_layers, mut view_visibility)) = patches.get_mut(*patch)
                else {
                    continue;
                };
                if !inherited.get()
                    || !layers
                        .unwrap_or_default()
                        .intersects(patch_layers.unwrap_or_default())
                {
                    continue;
                }
                view_visibility.set_visible();
                mesh_entities.push(*patch);
            }
        }
        mesh_entities.sort_unstable();
        if mesh_entities.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(anyhow::anyhow!("duplicate shared litter view membership").into());
        }
    }
    // Empty classes suppress independent draw membership. Bevy may still
    // conservatively extract patch data outside a cell's range. That superset
    // is not a draw and is deliberately left to the normal renderer.
    Ok(())
}
