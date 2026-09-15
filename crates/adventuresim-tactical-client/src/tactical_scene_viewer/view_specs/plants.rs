use super::*;
pub(in crate::tactical_scene_viewer) const PLANT_REVIEW_VIEWS: [CaptureViewSpec; 4] = [
    v!(
        "warmup",
        "Production vegetation warmup",
        CapturePose::Ground,
        55.0,
        100
    )
    .warmup()
    .vista(),
    v!(
        "plant-contact",
        "Actual scattered plant and ground contact",
        CapturePose::Plant { distance: 0.4 },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "plant-community",
        "Actual scattered plant at gameplay distance",
        CapturePose::Plant { distance: 4.0 },
        55.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "plant-distance",
        "Plant community at detail fade distance",
        CapturePose::Plant { distance: 20.0 },
        55.0,
        100
    )
    .settled_readback_pair()
    .vista(),
];
