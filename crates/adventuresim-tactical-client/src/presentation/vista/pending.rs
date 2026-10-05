//! A vista waits for its exact canonical terrain before preparing presentation.
use super::*;
#[derive(Resource, Default)]
pub(in crate::presentation) struct PendingVista(pub Option<SceneVistaBundle>);

impl PendingVista {
    pub(super) fn accept<'a>(
        &mut self,
        bundle: &SceneVistaBundle,
        mut environments: impl Iterator<Item = &'a SceneEnvironment>,
    ) -> bool {
        let ready = environments.any(|environment| environment.scene_digest == bundle.scene_digest);
        self.0 = (!ready).then(|| bundle.clone());
        ready
    }
}

pub(in crate::presentation) fn present_ready_vista(
    mut pending: ResMut<PendingVista>,
    terrains: Query<(&SceneTerrain, &SceneEnvironment)>,
    mut commands: Commands,
) {
    if pending.0.as_ref().is_some_and(|bundle| {
        terrains
            .iter()
            .any(|(_, environment)| environment.scene_digest == bundle.scene_digest)
    }) {
        commands.trigger(pending.0.take().expect("ready pending vista"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Resource, Default)]
    struct Presented(Vec<String>);
    #[test]
    fn late_terrain_releases_only_its_exact_pending_vista_once() {
        let input = TacticalSceneInput::load(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-scenes/flat-dry-grassland.json"
        )))
        .unwrap();
        let mut app = App::new();
        app.init_resource::<PendingVista>()
            .init_resource::<Presented>()
            .add_systems(Update, present_ready_vista)
            .add_observer(
                |vista: On<SceneVistaBundle>, mut presented: ResMut<Presented>| {
                    presented.0.push(vista.scene_digest.clone())
                },
            );
        app.world_mut().resource_mut::<PendingVista>().0 = Some(SceneVistaBundle {
            properties: None,
            scene_digest: "accepted".into(),
            playable_half_extent_metres: Vec2::ONE,
            distant_buildings: vec![],
            establishments: vec![],
            streets: vec![],
            yards: vec![],
            parishes: vec![],
            compounds: vec![],
            gardens: vec![],
            furniture_groups: vec![],
            distant_furniture: vec![],
            lods: vec![],
        });
        app.world_mut().spawn((
            SceneTerrain::new(3, 3, 1.0, |_| 0.0),
            input.environment_snapshot("unrelated".into()),
        ));
        app.update();
        assert!(app.world().resource::<Presented>().0.is_empty());
        app.world_mut().spawn((
            SceneTerrain::new(3, 3, 1.0, |_| 0.0),
            input.environment_snapshot("accepted".into()),
        ));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Presented>().0, ["accepted"]);
        assert!(app.world().resource::<PendingVista>().0.is_none());
    }
}
