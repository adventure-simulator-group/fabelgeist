use super::*;
pub(in crate::tactical_scene_viewer) const GARDEN_REVIEW_VIEWS: [CaptureViewSpec; 5] = [
    CaptureViewSpec::new(
        "warmup",
        "Garden material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "garden-overview",
        "Owned garden, beds and tending lanes",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "garden-hazel",
        "Accepted common hazel and clear working ground",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "garden-access",
        "Street passage into the garden",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "garden-distant",
        "The accepted specimen in distant presentation",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
];
