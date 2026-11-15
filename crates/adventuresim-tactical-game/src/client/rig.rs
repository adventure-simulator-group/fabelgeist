//! The fabelgeist rig as the client poses it: where its files are, and the animation targets
//! its bones need. `base.glb` carries no animation, so Bevy's loader gives it no player and no
//! targets; the motion files name each bone by its path of node names from [`SKELETON`].

use bevy::{
    animation::{AnimatedBy, AnimationTargetId},
    prelude::*,
};

/// The pack every fighter is drawn from, under the assets directory.
pub const PACK: &str = "animations/biped/unarmed";
/// The rig's root bone: the motion files' paths start at it.
pub const SKELETON: &str = "Skeleton";
/// The motion files the client plays.
pub const CLIP_FILES: [&str; 4] = ["combat_stance", "run", "swing", "thrust"];

/// Gives `node` and everything under it the target its name path makes, animated by
/// `skeleton`.
pub fn target_bones(
    commands: &mut Commands,
    skeleton: Entity,
    node: Entity,
    path: &mut Vec<Name>,
    children: &Query<&Children>,
    names: &Query<&Name>,
) {
    let Ok(name) = names.get(node) else {
        return;
    };
    path.push(name.clone());
    commands.entity(node).insert((
        AnimationTargetId::from_names(path.iter()),
        AnimatedBy(skeleton),
    ));
    for child in children.get(node).into_iter().flatten() {
        target_bones(commands, skeleton, *child, path, children, names);
    }
    path.pop();
}
