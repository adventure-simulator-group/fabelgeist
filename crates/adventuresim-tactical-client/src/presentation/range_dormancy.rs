//! Disables entities that sit far outside every view's [`VisibilityRange`].
//!
//! `VisibilityRange` hides an entity but keeps it in every per-frame query:
//! transform propagation, range checks, CPU culling and the view-visibility
//! passes all still visit it. Bevy's [`Disabled`] removes it from those queries
//! instead. A [`RangeDormancy`] entity is disabled once every view is well
//! beyond its band and re-enabled before any view can reach the band again;
//! the range keeps doing the fade inside the band.
//!
//! Distances are only re-measured after a view has travelled
//! [`REEVALUATION_TRAVEL_METRES`], so a still or slowly moving camera costs one
//! comparison per view per frame rather than one per entity.

use bevy::{
    camera::{ShadowLodOrigin, primitives::Aabb, visibility::VisibilityRange},
    ecs::entity_disabling::Disabled,
    prelude::*,
};

/// View travel after which dormancy is re-measured, in metres.
const REEVALUATION_TRAVEL_METRES: f32 = 64.0;

/// How far beyond its band an entity must be to go dormant, in metres. Twice
/// the re-evaluation travel: a view can move up to one travel between
/// measurements, and the second keeps a full travel of slack before the band.
const DORMANCY_MARGIN_METRES: f32 = 2.0 * REEVALUATION_TRAVEL_METRES;

/// Every view Bevy measures visibility ranges from.
type RangeView = Or<(With<Camera>, With<ShadowLodOrigin>)>;

type DormancyCandidate<'a> = (
    Entity,
    &'a GlobalTransform,
    Option<&'a Aabb>,
    &'a VisibilityRange,
    &'a ViewVisibility,
    Has<Disabled>,
);

/// Opts a root entity with a static transform and a fixed [`VisibilityRange`]
/// into being disabled while it is far outside that range.
#[derive(Component, Clone, Copy, Debug, Default)]
pub(in crate::presentation) struct RangeDormancy;

/// Registers range dormancy for [`RangeDormancy`] entities.
pub(in crate::presentation) struct RangeDormancyPlugin;

/// View positions dormancy was last measured from.
#[derive(Resource, Default)]
struct DormancyViews {
    measured_from: Vec<Vec3>,
    stale: bool,
}

impl Plugin for RangeDormancyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DormancyViews>()
            .add_observer(
                |_: On<Add, RangeDormancy>, mut views: ResMut<DormancyViews>| {
                    views.stale = true;
                },
            )
            .add_systems(
                PostUpdate,
                update_range_dormancy
                    .after(bevy::transform::TransformSystems::Propagate)
                    .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
            );
    }
}

impl DormancyViews {
    /// Whether any view may have come within reach of a dormant band.
    fn need_measurement(&self, views: &[Vec3]) -> bool {
        self.stale
            || views.len() != self.measured_from.len()
            || views
                .iter()
                .zip(&self.measured_from)
                .any(|(view, measured)| view.distance(*measured) >= REEVALUATION_TRAVEL_METRES)
    }
}

/// Re-enables an entity and marks its transform and visibility changed.
///
/// Propagation skipped the entity while it was disabled, and both passes only
/// revisit changed subtrees, so without the marks a child could keep the
/// global transform or inherited visibility it had when it went dormant.
pub(in crate::presentation) fn wake(mut entity: EntityWorldMut) {
    entity.remove::<Disabled>();
    if let Some(mut transform) = entity.get_mut::<Transform>() {
        transform.set_changed();
    }
    if let Some(mut visibility) = entity.get_mut::<Visibility>() {
        visibility.set_changed();
    }
}

/// Whether an entity at `position` is beyond `range` from every view, with
/// the dormancy margin on both sides of the band.
fn is_dormant(range: &VisibilityRange, position: Vec3, views: &[Vec3]) -> bool {
    let near = range.start_margin.start - DORMANCY_MARGIN_METRES;
    let far = range.end_margin.end + DORMANCY_MARGIN_METRES;
    views.iter().all(|view| {
        let distance = view.distance(position);
        distance < near || distance > far
    })
}

fn update_range_dormancy(
    mut commands: Commands,
    mut state: ResMut<DormancyViews>,
    views: Query<&GlobalTransform, RangeView>,
    candidates: Query<DormancyCandidate, (With<RangeDormancy>, Allow<Disabled>)>,
) {
    let positions = views
        .iter()
        .map(GlobalTransform::translation)
        .collect::<Vec<_>>();
    if positions.is_empty() || !state.need_measurement(&positions) {
        return;
    }
    let mut deferred = false;
    for (entity, transform, aabb, range, view_visibility, disabled) in &candidates {
        // Mirror Bevy's range check, which measures from the bounds centre
        // only when the range asks for it.
        let position = match (range.use_aabb, aabb) {
            (true, Some(aabb)) => transform.transform_point(Vec3::from(aabb.center)),
            _ => transform.translation(),
        };
        match (is_dormant(range, position, &positions), disabled) {
            // Disabling freezes `ViewVisibility`; an entity drawn last frame
            // would keep stale render state, so wait for it to be culled.
            (true, false) if view_visibility.get() => deferred = true,
            (true, false) => {
                commands.entity(entity).insert(Disabled);
            }
            (false, true) => {
                commands.entity(entity).queue(wake);
            }
            _ => {}
        }
    }
    state.measured_from = positions;
    state.stale = deferred;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        camera::visibility::{VisibilityPlugin, VisibilityRangePlugin},
        ecs::system::RunSystemOnce,
    };

    const BAND_END_METRES: f32 = 1_900.0;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            TransformPlugin,
            VisibilityPlugin,
            VisibilityRangePlugin,
            RangeDormancyPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>();
        app
    }

    fn spawn_camera(app: &mut App, position: Vec3) -> Entity {
        app.world_mut()
            .spawn((Camera::default(), Transform::from_translation(position)))
            .id()
    }

    fn spawn_chunk(app: &mut App, position: Vec3) -> Entity {
        app.world_mut()
            .spawn((
                RangeDormancy,
                Transform::from_translation(position),
                // Already propagated, so dormancy can run before the first update.
                GlobalTransform::from_translation(position),
                Visibility::Inherited,
                VisibilityRange::abrupt(0.0, BAND_END_METRES),
            ))
            .id()
    }

    fn is_disabled(app: &App, entity: Entity) -> bool {
        app.world().entity(entity).contains::<Disabled>()
    }

    fn move_camera(app: &mut App, camera: Entity, position: Vec3) {
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation = position;
    }

    #[test]
    fn far_entities_sleep_and_wake_before_any_view_reaches_their_band() {
        let mut app = app();
        let camera = spawn_camera(&mut app, Vec3::ZERO);
        let chunk = spawn_chunk(&mut app, Vec3::new(5_000.0, 0.0, 0.0));
        app.update();
        assert!(is_disabled(&app, chunk));

        // Still beyond the band, but within the margin: awake before entry.
        let outside_band = 5_000.0 - BAND_END_METRES - REEVALUATION_TRAVEL_METRES;
        move_camera(&mut app, camera, Vec3::new(outside_band, 0.0, 0.0));
        app.update();
        assert!(!is_disabled(&app, chunk));
    }

    #[test]
    fn small_view_movements_never_reach_a_dormant_band() {
        let mut app = app();
        let camera = spawn_camera(&mut app, Vec3::ZERO);
        let edge = BAND_END_METRES + DORMANCY_MARGIN_METRES + 1.0;
        let chunk = spawn_chunk(&mut app, Vec3::new(edge, 0.0, 0.0));
        app.update();
        assert!(is_disabled(&app, chunk));

        // Step towards the chunk below the re-measurement threshold each time.
        let step = REEVALUATION_TRAVEL_METRES * 0.75;
        let mut x = 0.0;
        while x < edge {
            x += step;
            move_camera(&mut app, camera, Vec3::new(x, 0.0, 0.0));
            app.update();
            let distance = edge - x;
            if distance <= BAND_END_METRES {
                assert!(!is_disabled(&app, chunk), "dormant inside its band");
            }
        }
    }

    #[test]
    fn every_view_must_be_beyond_the_band() {
        let mut app = app();
        spawn_camera(&mut app, Vec3::ZERO);
        spawn_camera(&mut app, Vec3::new(4_000.0, 0.0, 0.0));
        let chunk = spawn_chunk(&mut app, Vec3::new(5_000.0, 0.0, 0.0));
        app.update();
        assert!(!is_disabled(&app, chunk));
    }

    #[test]
    fn an_entity_drawn_last_frame_waits_for_culling_before_sleeping() {
        let mut app = app();
        spawn_camera(&mut app, Vec3::ZERO);
        let chunk = spawn_chunk(&mut app, Vec3::new(5_000.0, 0.0, 0.0));
        app.world_mut()
            .entity_mut(chunk)
            .insert(ViewVisibility::VISIBLE);
        // Run only dormancy: the visibility passes would clear the flag.
        app.world_mut()
            .run_system_once(update_range_dormancy)
            .unwrap();
        assert!(!is_disabled(&app, chunk));

        app.update();
        app.update();
        assert!(is_disabled(&app, chunk));
    }

    #[test]
    fn waking_restores_transform_and_visibility_propagated_while_disabled() {
        let mut app = app();
        let parent = app
            .world_mut()
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let child = app
            .world_mut()
            .spawn((
                Transform::from_xyz(1.0, 0.0, 0.0),
                Visibility::Inherited,
                ChildOf(parent),
            ))
            .id();
        app.update();
        app.world_mut().entity_mut(child).insert(Disabled);
        app.update();

        app.world_mut()
            .get_mut::<Transform>(parent)
            .unwrap()
            .translation = Vec3::new(0.0, 10.0, 0.0);
        *app.world_mut().get_mut::<Visibility>(parent).unwrap() = Visibility::Hidden;
        app.update();
        app.update();
        wake(app.world_mut().entity_mut(child));
        app.update();

        let child = app.world().entity(child);
        assert_eq!(
            child.get::<GlobalTransform>().unwrap().translation(),
            Vec3::new(1.0, 10.0, 0.0)
        );
        assert!(!child.get::<InheritedVisibility>().unwrap().get());
    }
}
