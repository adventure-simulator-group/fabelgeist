//! Incapacitation wheels: the player's at the screen centre and each hurt
//! enemy's over its head. Rust decides every arc (order, colour and the 100%
//! cutoff) and the page only draws them (`tactical-hud.js`).
use adventuresim_tactical_core::{player::Player, prelude::*};
use bevy::prelude::*;
use serde::Serialize;

use crate::{
    animation::{BoneRole, HumanoidRig},
    hud::{self, HudView, LastPushed},
    player::ClientPlayer,
    presentation::TacticalGameplayCamera,
};

const PLAYER_WHEEL: WheelSize = WheelSize {
    radius: 26.0,
    width: 8.0,
};
const ENEMY_WHEEL: WheelSize = WheelSize {
    radius: 18.0,
    width: 6.0,
};
const ENEMY_WHEEL_HEAD_CLEARANCE_METRES: f32 = 0.25;
const ENEMY_WHEEL_ROOT_FALLBACK_HEIGHT_METRES: f32 = 1.1;
/// Arcs below this share of the circle are under a pixel and are left out.
const MIN_VISIBLE_INCAPACITATION_SEGMENT: f32 = 0.005;
/// Arc shares cross to the page in thousandths, so sub-pixel drift (fatigue
/// recovering every tick) does not push a new view each frame.
const ARC_QUANTUM: f32 = 1000.0;

/// One impairment source on the wheel, in its fixed strategic order, or the
/// yellow projection of an increase. Projections are a forecast, never
/// current impairment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WheelSegment {
    /// Pain and acute trauma together.
    Pain,
    BloodLoss,
    Fear,
    /// Fatigue, the one general value, drawn black.
    Fatigue,
    Hunger,
    Thirst,
    Thermal,
    Imbalance,
    /// Encumbrance, translucent grey.
    Encumbrance,
    Projected,
}

impl WheelSegment {
    pub(super) fn color(self) -> Color {
        match self {
            Self::Pain => Color::srgb_u8(0xd9, 0x73, 0xa2),
            Self::BloodLoss => Color::srgb_u8(0xc8, 0x47, 0x47),
            Self::Fear => Color::srgb_u8(0x4f, 0x83, 0xcc),
            Self::Fatigue => Color::srgb_u8(0x20, 0x20, 0x20),
            Self::Hunger => Color::srgb_u8(0xb5, 0x7a, 0x35),
            Self::Thirst => Color::srgb_u8(0x3f, 0x9f, 0xa8),
            Self::Thermal => Color::srgb_u8(0x7d, 0x8e, 0xe8),
            Self::Imbalance => Color::WHITE,
            Self::Encumbrance => Color::srgba_u8(160, 160, 160, 120),
            Self::Projected => Color::srgb_u8(0xff, 0xff, 0x00),
        }
    }
}

pub(super) fn incapacitation_wheel_segments(
    sources: TacticalIncapacitationSources,
) -> [(f32, WheelSegment); 9] {
    [
        (sources.pain + sources.acute_trauma, WheelSegment::Pain),
        (sources.blood_loss, WheelSegment::BloodLoss),
        (sources.fear, WheelSegment::Fear),
        (sources.fatigue, WheelSegment::Fatigue),
        (sources.hunger, WheelSegment::Hunger),
        (sources.thirst, WheelSegment::Thirst),
        (sources.thermal, WheelSegment::Thermal),
        (sources.imbalance, WheelSegment::Imbalance),
        (sources.encumbrance, WheelSegment::Encumbrance),
    ]
}

/// Keep all current impairment visible before allocating remaining wheel
/// space to yellow forecasts. Forecasts never displace actual sources.
pub(super) fn forecast_wheel_segments(
    current: TacticalIncapacitationSources,
    forecast: TacticalIncapacitationSources,
) -> Vec<(f32, WheelSegment)> {
    let available = (1.0 - current.total()).clamp(0.0, 1.0);
    let predicted = forecast.total().max(0.0);
    let scale = if predicted > 0.0 {
        (available / predicted).min(1.0)
    } else {
        0.0
    };
    incapacitation_wheel_segments(current)
        .into_iter()
        .zip(incapacitation_wheel_segments(forecast))
        .flat_map(|((actual, segment), (increase, _))| {
            [
                (actual, segment),
                (increase.max(0.0) * scale, WheelSegment::Projected),
            ]
        })
        .collect()
}

fn visible_incapacitation_wheel_amount(raw_amount: f32, remaining: f32) -> Option<f32> {
    let amount = raw_amount.max(0.0).min(remaining);
    (amount >= MIN_VISIBLE_INCAPACITATION_SEGMENT).then_some(amount)
}

fn enemy_incapacitation_wheel_visible(side: TacticalCombatSide, incapacitation: f32) -> bool {
    side == TacticalCombatSide::Enemy && incapacitation > 0.0
}

#[derive(Clone, Copy)]
struct WheelSize {
    radius: f32,
    width: f32,
}

/// One drawn arc: its share of the full circle, clockwise from the top.
#[derive(Debug, PartialEq, Serialize)]
struct WheelArc {
    share: f32,
    color: String,
}

/// The arcs of one wheel, clipped at 100% of the circle.
fn wheel_arcs(
    state: &TacticalCombatState,
    sources: TacticalIncapacitationSources,
) -> Vec<WheelArc> {
    let mut remaining = 1.0_f32;
    let mut arcs = Vec::new();
    for (raw_amount, segment) in forecast_wheel_segments(sources, state.projected_increase) {
        if remaining <= 0.0 {
            break;
        }
        let Some(amount) = visible_incapacitation_wheel_amount(raw_amount, remaining) else {
            continue;
        };
        remaining -= amount;
        arcs.push(WheelArc {
            share: (amount * ARC_QUANTUM).round() / ARC_QUANTUM,
            color: segment.color().to_srgba().to_hex(),
        });
    }
    arcs
}

#[derive(Debug, PartialEq, Serialize)]
struct WheelView {
    /// Whether the wheel follows a character. Its position then arrives
    /// separately every frame it moves (`hud::wheel_positions`); otherwise it
    /// sits at the screen centre.
    follows: bool,
    radius: f32,
    width: f32,
    incapacitated: bool,
    arcs: Vec<WheelArc>,
}

impl WheelView {
    fn new(
        follows: bool,
        size: WheelSize,
        state: &TacticalCombatState,
        sources: TacticalIncapacitationSources,
    ) -> Self {
        Self {
            follows,
            radius: size.radius,
            width: size.width,
            incapacitated: state.is_incapacitated(),
            arcs: wheel_arcs(state, sources),
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize)]
struct WheelsView {
    wheels: Vec<WheelView>,
    /// Viewport centres of the following wheels, x then y, in whole CSS
    /// pixels. Pushed apart from the shapes: a moving enemy changes them every
    /// frame, and the page moves its wheel with a transform instead of
    /// redrawing it.
    #[serde(skip)]
    positions: Vec<f32>,
}

type EnemyWheelQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        &'static Limbs,
        &'static GlobalTransform,
        Option<&'static HumanoidRig>,
    ),
    (With<Player>, Without<ClientPlayer>),
>;

#[derive(Default)]
pub(super) struct LastPushedWheels {
    view: LastPushed<String>,
    positions: LastPushed<Vec<f32>>,
}

/// Pushes the wheels' shapes when any arc or state changed, and their
/// on-screen positions when one moved. Until the local player is revealed the
/// view is empty.
pub(super) fn push_incapacitation_wheels(
    revealed: Option<Res<crate::loading_screen::LoadingScreen>>,
    player: Query<(Entity, &TacticalCombatState, &Limbs), With<ClientPlayer>>,
    enemies: EnemyWheelQuery,
    bone_transforms: Query<&GlobalTransform, Without<Player>>,
    camera: Query<(&Camera, &GlobalTransform), With<TacticalGameplayCamera>>,
    viewer: TacticalPlayerViewer,
    mut last: Local<LastPushedWheels>,
) {
    let mut view = WheelsView::default();
    if crate::loading_screen::gameplay_unlocked(revealed)
        && let Ok((entity, state, limbs)) = player.single()
        && let Ok(player_view) = viewer.get(entity)
    {
        let will = player_view.skill_check(Skill::Will, LimbWeights::all_equal());
        if state.incapacitation > 0.0 {
            let sources = state.incapacitation_sources(limbs.total_damage(), will);
            view.wheels
                .push(WheelView::new(false, PLAYER_WHEEL, state, sources));
        }
        if let Ok((camera, camera_transform)) = camera.single() {
            push_enemy_wheels(
                &mut view,
                camera,
                camera_transform,
                &enemies,
                &bone_transforms,
                &viewer,
            );
        }
    }
    let Ok(json) = serde_json::to_string(&view) else {
        return;
    };
    if let Some(json) = last.view.update(json) {
        hud::view(HudView::Wheels, json);
        // A redrawn view places its following wheels from scratch.
        last.positions = LastPushed::default();
    }
    if let Some(positions) = last.positions.update(view.positions) {
        hud::wheel_positions(positions);
    }
}

fn push_enemy_wheels(
    view: &mut WheelsView,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    enemies: &EnemyWheelQuery,
    bone_transforms: &Query<&GlobalTransform, Without<Player>>,
    viewer: &TacticalPlayerViewer,
) {
    let Some(viewport) = camera.logical_viewport_size() else {
        return;
    };
    let bounds = Rect::from_corners(Vec2::ZERO, viewport).inflate(ENEMY_WHEEL.radius);
    for (entity, side, state, limbs, root_transform, rig) in enemies {
        if !enemy_incapacitation_wheel_visible(*side, state.incapacitation) {
            continue;
        }
        let Ok(enemy) = viewer.get(entity) else {
            continue;
        };
        let will = enemy.skill_check(Skill::Will, LimbWeights::all_equal());
        let sources = state.incapacitation_sources(limbs.total_damage(), will);
        let head_position = rig
            .and_then(|rig| rig.get(&BoneRole::Head))
            .and_then(|head| bone_transforms.get(*head).ok())
            .map_or_else(
                || root_transform.translation() + Vec3::Y * ENEMY_WHEEL_ROOT_FALLBACK_HEIGHT_METRES,
                |head| head.translation() + Vec3::Y * ENEMY_WHEEL_HEAD_CLEARANCE_METRES,
            );
        let Ok(center) = camera.world_to_viewport(camera_transform, head_position) else {
            continue;
        };
        if bounds.contains(center) {
            view.wheels
                .push(WheelView::new(true, ENEMY_WHEEL, state, sources));
            view.positions.extend([center.x.round(), center.y.round()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_uses_strategic_order_then_white_imbalance() {
        let segments = incapacitation_wheel_segments(TacticalIncapacitationSources {
            pain: 0.1,
            acute_trauma: 0.05,
            blood_loss: 0.2,
            fear: 0.3,
            fatigue: 0.4,
            hunger: 0.5,
            thirst: 0.6,
            thermal: 0.7,
            imbalance: 0.9,
            encumbrance: 0.1,
        });

        assert_eq!(
            segments.map(|(amount, _)| amount),
            [0.15, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.9, 0.1]
        );
        assert_eq!(segments[0].1.color().to_srgba().to_hex(), "#D973A2");
        assert_eq!(segments[1].1.color().to_srgba().to_hex(), "#C84747");
        assert_eq!(segments[3].1.color().to_srgba().to_hex(), "#202020");
        assert_eq!(segments[7].1.color(), Color::WHITE);
    }

    #[test]
    fn forecasts_are_yellow_and_cannot_hide_actual_impairment() {
        let current = TacticalIncapacitationSources {
            fatigue: 0.5,
            encumbrance: 0.3,
            ..default()
        };
        let forecast = TacticalIncapacitationSources {
            pain: 0.4,
            blood_loss: 0.2,
            ..default()
        };
        let segments = forecast_wheel_segments(current, forecast);
        let yellow: f32 = segments
            .iter()
            .filter(|(_, segment)| *segment == WheelSegment::Projected)
            .map(|(v, _)| v)
            .sum();
        assert!((yellow - 0.2).abs() < 0.0001);
        let actual: f32 = segments.iter().step_by(2).map(|(v, _)| v).sum();
        assert!((actual - current.total()).abs() < 0.0001);
        let current_segments = incapacitation_wheel_segments(current);
        assert!(
            current_segments
                .iter()
                .all(|(_, segment)| *segment != WheelSegment::Projected)
        );
        assert!(current_segments[8].1.color().alpha() < 1.0);
        assert_ne!(current_segments[8].1.color(), current_segments[3].1.color());
    }

    #[test]
    fn every_source_can_show_a_forecast_and_overflow_still_clips() {
        let forecast = TacticalIncapacitationSources {
            pain: 0.01,
            acute_trauma: 0.01,
            blood_loss: 0.01,
            fear: 0.01,
            fatigue: 0.01,
            hunger: 0.01,
            thirst: 0.01,
            thermal: 0.01,
            imbalance: 0.01,
            encumbrance: 0.01,
        };
        assert!(
            forecast_wheel_segments(default(), forecast)
                .iter()
                .skip(1)
                .step_by(2)
                .all(|(v, s)| *v > 0.0 && *s == WheelSegment::Projected)
        );
        let unconscious = TacticalIncapacitationSources {
            blood_loss: 1.2,
            ..default()
        };
        assert!(
            forecast_wheel_segments(unconscious, forecast)
                .iter()
                .skip(1)
                .step_by(2)
                .all(|(v, _)| *v == 0.0)
        );
    }

    #[test]
    fn wheel_arcs_stop_at_the_full_circle() {
        let state = TacticalCombatState {
            projected_increase: TacticalIncapacitationSources {
                fear: 0.5,
                ..default()
            },
            ..default()
        };
        let arcs = wheel_arcs(
            &state,
            TacticalIncapacitationSources {
                blood_loss: 0.7,
                fatigue: 0.6,
                ..default()
            },
        );
        let total: f32 = arcs.iter().map(|arc| arc.share).sum();
        assert!((total - 1.0).abs() < 0.0001);
        assert_eq!(arcs.len(), 2);
        assert_eq!(arcs[1].color, "#202020");
    }

    #[test]
    fn wheel_hides_subpixel_segments_without_changing_state() {
        assert_eq!(visible_incapacitation_wheel_amount(0.0049, 1.0), None);
        assert_eq!(visible_incapacitation_wheel_amount(0.005, 1.0), Some(0.005));
    }

    #[test]
    fn enemy_wheel_is_absent_at_zero() {
        assert!(!enemy_incapacitation_wheel_visible(
            TacticalCombatSide::Enemy,
            0.0
        ));
        assert!(enemy_incapacitation_wheel_visible(
            TacticalCombatSide::Enemy,
            0.001
        ));
        assert!(!enemy_incapacitation_wheel_visible(
            TacticalCombatSide::Party,
            0.5
        ));
    }
}
