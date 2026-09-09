//! Deterministic gameplay and armor inspection camera framing.
use super::*;

pub(super) fn position_capture_camera(
    sequence: Res<CaptureSequence>,
    armor: Res<harness::ArmorCapture>,
    subjects: Query<(&Transform, &PresentedSkeleton), With<CaptureSubject>>,
    mut cameras: Query<&mut Transform, (With<TacticalGameplayCamera>, Without<CaptureSubject>)>,
    mut labels: Query<(&mut Text, &mut Visibility), With<CaptureLabel>>,
) {
    let (Ok((subject, skeleton)), Ok(mut camera)) = (subjects.single(), cameras.single_mut())
    else {
        return;
    };
    let focus = subject.translation + Vec3::Y * 0.95;
    let view = sequence.view();
    match view {
        CaptureView::Gameplay => {
            // Physics simulation is disabled in this fixture, so ahoy does
            // not refresh its controller-follow base transform. Reconstruct
            // that default base and apply the exact gameplay camera offset;
            // otherwise the offset accumulates and the first raw frame is a
            // pelvis-level/empty view.
            camera.translation =
                subject.translation + animation_capture_camera_offset(Quat::IDENTITY);
            camera.rotation = Quat::IDENTITY;
        }
        CaptureView::Side => {
            camera.translation = focus + Vec3::new(5.0, 0.45, 0.0);
            camera.look_at(focus, Vec3::Y);
        }
        CaptureView::Front => {
            camera.translation = focus + Vec3::new(0.0, 0.45, -5.0);
            camera.look_at(focus, Vec3::Y);
        }
    }
    if let Some(review_camera) = armor.review_camera(view, subject) {
        *camera = review_camera;
    }
    if let Some(frame) = sequence.applied_frame() {
        for (mut label, mut visibility) in &mut labels {
            *visibility = if matches!(view, CaptureView::Gameplay) {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            **label = format!(
                "{} | {:>4.2} m/s | phase {:>5.3} | {} view | 64 Hz frame {}",
                frame.scenario,
                frame.speed,
                skeleton.gait_phase,
                view.slug(),
                frame.scenario_frame,
            );
        }
    }
}
