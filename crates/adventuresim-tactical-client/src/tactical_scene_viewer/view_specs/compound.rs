use super::*;

pub(in crate::tactical_scene_viewer) const COMPOUND_REVIEW_VIEWS: [CaptureViewSpec; 8] = [
    CaptureViewSpec::new(
        "warmup",
        "Compound material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "property-overview",
        "Both property access orientations",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "left-street-gate",
        "Left passage street gate",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "left-side-passage",
        "Left side access and court",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "right-street-gate",
        "Right passage street gate",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "right-side-passage",
        "Right side access and court",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "rear-store",
        "Rear storage range and court",
        CapturePose::CityExterior { camera: 5 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "court-exit",
        "House courtyard exit",
        CapturePose::CityExterior { camera: 6 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
];
