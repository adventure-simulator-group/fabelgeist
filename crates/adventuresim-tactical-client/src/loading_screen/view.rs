//! The loading overlay. Built directly in bevy_ui rather than `ui.css` because
//! the fade drives every colour's alpha, which flair has no property for, and
//! the screen must not depend on a stylesheet that is itself still loading.
use super::{LoadingPhase, LoadingScreen, progress::LoadingStage};
use adventuresim_tactical_netcode::prelude::AdventureSimulatorClient;
use bevy::prelude::*;

const FONT_ASSET: &str = "SNPro-VariableFont_wght.ttf";
/// Above the HUD and every other UI root.
const OVERLAY_Z_INDEX: i32 = 1_000;
const TITLE_SIZE: f32 = 40.0;
const STAGE_TEXT_SIZE: f32 = 16.0;
const BACKDROP: Color = Color::srgb(0.02, 0.024, 0.031);
const TEXT: Color = Color::srgba(1.0, 1.0, 1.0, 0.8);
const TRACK: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);
const ACTIVE_FILL: Color = Color::srgb(0.565, 0.792, 0.976);
const FINISHED_FILL: Color = Color::srgb(0.506, 0.78, 0.518);
const FAILURE_TEXT: Color = Color::srgb(0.898, 0.451, 0.451);
const BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.12);

type OverlayRootOnly = (With<LoadingRoot>, Without<FailurePanel>);

#[derive(Component)]
pub(super) struct LoadingRoot;

/// Base colour of a node or text; the fade scales its alpha.
#[derive(Component, Clone, Copy)]
pub(super) struct FadeColor(Color);

#[derive(Component)]
pub(super) struct StageFill(LoadingStage);

#[derive(Component)]
pub(super) struct FailurePanel;

#[derive(Component)]
pub(super) struct FailureMessage;

#[derive(Component, Clone, Copy)]
pub(super) enum LoadingAction {
    Retry,
    Exit,
}

struct OverlayFont(FontSource);

impl OverlayFont {
    fn sized(&self, size: f32) -> TextFont {
        TextFont {
            font: self.0.clone(),
            font_size: FontSize::Px(size),
            ..default()
        }
    }
}

pub(super) fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = OverlayFont(asset_server.load(FONT_ASSET).into());
    commands
        .spawn((
            LoadingRoot,
            Name::new("loading-screen"),
            GlobalZIndex(OVERLAY_Z_INDEX),
            FadeColor(BACKDROP),
            BackgroundColor(BACKDROP),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Fabelgeist"),
                font.sized(TITLE_SIZE),
                TextColor(TEXT),
                FadeColor(TEXT),
                Node {
                    margin: UiRect::bottom(px(24)),
                    ..default()
                },
            ));
            for stage in LoadingStage::ALL {
                spawn_stage_row(root, &font, stage);
            }
            spawn_failure_panel(root, &font);
        });
}

fn spawn_stage_row(root: &mut ChildSpawnerCommands, font: &OverlayFont, stage: LoadingStage) {
    root.spawn(Node {
        width: px(440),
        max_width: percent(90),
        flex_direction: FlexDirection::Column,
        row_gap: px(3),
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(stage.label()),
            font.sized(STAGE_TEXT_SIZE),
            TextColor(TEXT),
            FadeColor(TEXT),
        ));
        row.spawn((
            BackgroundColor(TRACK),
            FadeColor(TRACK),
            Node {
                width: percent(100),
                height: px(4),
                ..default()
            },
        ))
        .with_child((
            StageFill(stage),
            BackgroundColor(ACTIVE_FILL),
            FadeColor(ACTIVE_FILL),
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
        ));
    });
}

fn spawn_failure_panel(root: &mut ChildSpawnerCommands, font: &OverlayFont) {
    root.spawn((
        FailurePanel,
        Node {
            display: Display::None,
            max_width: percent(90),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(12),
            margin: UiRect::top(px(20)),
            ..default()
        },
    ))
    .with_children(|panel| {
        panel.spawn((
            FailureMessage,
            Text::default(),
            font.sized(STAGE_TEXT_SIZE),
            TextColor(FAILURE_TEXT),
            FadeColor(FAILURE_TEXT),
        ));
        panel
            .spawn(Node {
                column_gap: px(12),
                ..default()
            })
            .with_children(|buttons| {
                for (action, label) in [
                    (LoadingAction::Retry, "Retry"),
                    (LoadingAction::Exit, "Exit"),
                ] {
                    buttons
                        .spawn((
                            action,
                            Button,
                            BackgroundColor(BUTTON),
                            FadeColor(BUTTON),
                            Node {
                                padding: UiRect::axes(px(20), px(8)),
                                ..default()
                            },
                        ))
                        .with_child((
                            Text::new(label),
                            font.sized(STAGE_TEXT_SIZE),
                            TextColor(TEXT),
                            FadeColor(TEXT),
                        ));
                }
            });
    });
}

pub(super) fn handle_actions(
    mut commands: Commands,
    screen: Res<LoadingScreen>,
    actions: Query<(&Interaction, &LoadingAction), Changed<Interaction>>,
    clients: Query<Entity, With<AdventureSimulatorClient>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !matches!(screen.phase, LoadingPhase::Failed(_)) {
        return;
    }
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            LoadingAction::Retry => {
                if let Some(connection) = &screen.connection {
                    // A load can fail with its session still open.
                    for client in &clients {
                        commands.entity(client).despawn();
                    }
                    commands.spawn(connection.clone());
                }
            }
            LoadingAction::Exit => leave_tactical(&mut exit),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn leave_tactical(exit: &mut MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

/// The host page owns navigation, so the browser client only asks it to leave.
#[cfg(target_family = "wasm")]
fn leave_tactical(_exit: &mut MessageWriter<AppExit>) {
    crate::browser_runtime::request_tactical_exit();
}

pub(super) fn sync_stage_bars(
    screen: Res<LoadingScreen>,
    mut fills: Query<(&StageFill, &mut Node, &mut FadeColor)>,
) {
    let LoadingPhase::Loading(run) = &screen.phase else {
        return;
    };
    for (fill, mut node, mut color) in &mut fills {
        let stage = run.progress.get(fill.0);
        node.width = percent(stage.fraction() * 100.0);
        color.0 = if stage.is_finished() {
            FINISHED_FILL
        } else {
            ACTIVE_FILL
        };
    }
}

pub(super) fn sync_overlay(
    screen: Res<LoadingScreen>,
    mut root: Single<&mut Node, OverlayRootOnly>,
    mut failure_panel: Single<&mut Node, With<FailurePanel>>,
    mut failure_message: Single<&mut Text, With<FailureMessage>>,
    mut colors: Query<(
        &FadeColor,
        Option<&mut BackgroundColor>,
        Option<&mut TextColor>,
    )>,
) {
    let opacity = match &screen.phase {
        LoadingPhase::Idle => {
            root.display = Display::None;
            return;
        }
        LoadingPhase::Loading(_) | LoadingPhase::Failed(_) => 1.0,
        LoadingPhase::Revealing(timer) => timer.fraction_remaining(),
    };
    root.display = Display::Flex;
    if let LoadingPhase::Failed(failure) = &screen.phase {
        failure_panel.display = Display::Flex;
        failure_message.0 = failure.to_string();
    } else {
        failure_panel.display = Display::None;
    }
    for (base, background, text) in &mut colors {
        let color = base.0.with_alpha(base.0.alpha() * opacity);
        // Text nodes also carry the `BackgroundColor` that `Node` requires.
        if let Some(mut text) = text {
            text.0 = color;
        } else if let Some(mut background) = background {
            background.0 = color;
        }
    }
}
