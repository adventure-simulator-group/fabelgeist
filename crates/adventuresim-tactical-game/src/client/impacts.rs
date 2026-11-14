//! Impacts: a burst where a strike lands. This player's own strikes burst the moment its
//! prediction lands them (a round trip before the server knows); everyone else's when the
//! server's [`StrikeOutcome`] arrives. A guarded strike bursts small and pale, a hurtful one
//! red, one that puts someone down large.

use bevy::prelude::*;
use bevy_netahoy::{EventId, OwnPlayerId};

use crate::{CombatEvent, StrikeOutcome, combat::StrikeResult};

/// How long a burst lasts (s).
const BURST_SECS: f32 = 0.25;
/// Strikes this client burst itself, so the server's verdict on one isn't a second burst.
const REMEMBERED: usize = 64;

pub(super) struct ImpactsPlugin;

impl Plugin for ImpactsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Burst>()
            .add_systems(Startup, make_burst)
            .add_systems(Update, (predicted_bursts, server_bursts, fade_bursts));
    }
}

/// The burst's mesh, shared.
#[derive(Resource, Default)]
struct Burst {
    mesh: Handle<Mesh>,
    /// The strikes this client burst from its own prediction, newest last.
    own: Vec<EventId>,
}

/// A burst fading out.
#[derive(Component)]
struct Fading {
    left: f32,
    size: f32,
}

/// How a burst looks.
#[derive(Clone, Copy)]
enum Look {
    Guarded,
    Hurt,
    Downed,
}

impl Look {
    fn color(self) -> Color {
        match self {
            Self::Guarded => Color::srgb(0.9, 0.9, 0.8),
            Self::Hurt => Color::srgb(1.0, 0.25, 0.1),
            Self::Downed => Color::srgb(1.0, 0.05, 0.05),
        }
    }

    fn size(self) -> f32 {
        match self {
            Self::Guarded => 0.25,
            Self::Hurt => 0.4,
            Self::Downed => 0.7,
        }
    }
}

fn make_burst(mut burst: ResMut<Burst>, mut meshes: ResMut<Assets<Mesh>>) {
    burst.mesh = meshes.add(Sphere::new(1.0).mesh().ico(2).expect("a small icosphere"));
}

fn spawn_burst(
    commands: &mut Commands,
    burst: &Burst,
    materials: &mut Assets<StandardMaterial>,
    at: Vec3,
    look: Look,
) {
    commands.spawn((
        Name::new("impact"),
        Mesh3d(burst.mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: look.color(),
            emissive: LinearRgba::from(look.color()) * 4.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(at).with_scale(Vec3::splat(look.size() * 0.3)),
        Fading {
            left: BURST_SECS,
            size: look.size(),
        },
    ));
}

/// This player's strikes, as its prediction lands them.
fn predicted_bursts(
    mut events: MessageReader<CombatEvent>,
    mut commands: Commands,
    mut burst: ResMut<Burst>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        let CombatEvent::Struck(contact) = event else {
            continue;
        };
        let look = if contact.guarded {
            Look::Guarded
        } else {
            Look::Hurt
        };
        spawn_burst(&mut commands, &burst, &mut materials, contact.point, look);
        burst.own.push(contact.id);
        if burst.own.len() > REMEMBERED {
            burst.own.remove(0);
        }
    }
}

/// Every strike the server judged that this client didn't burst itself, and a downing blow
/// whoever threw it.
fn server_bursts(
    mut outcomes: MessageReader<StrikeOutcome>,
    own: Res<OwnPlayerId>,
    mut commands: Commands,
    burst: Res<Burst>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for outcome in outcomes.read() {
        let look = match outcome.result {
            StrikeResult::Guarded => Look::Guarded,
            StrikeResult::Hurt(_) => Look::Hurt,
            StrikeResult::Downed => Look::Downed,
        };
        let ours = own.0 == Some(outcome.strike.owner) && burst.own.contains(&outcome.strike);
        if ours && !matches!(look, Look::Downed) {
            continue;
        }
        spawn_burst(&mut commands, &burst, &mut materials, outcome.point, look);
    }
}

fn fade_bursts(
    time: Res<Time>,
    mut commands: Commands,
    mut bursts: Query<(Entity, &mut Fading, &mut Transform)>,
) {
    for (entity, mut fading, mut transform) in &mut bursts {
        fading.left -= time.delta_secs();
        if fading.left <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let grown = 1.0 - fading.left / BURST_SECS;
        transform.scale = Vec3::splat(fading.size * (0.3 + 0.7 * grown));
    }
}
