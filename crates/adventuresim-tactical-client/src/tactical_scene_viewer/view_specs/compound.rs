use super::*;

pub(in crate::tactical_scene_viewer) const COMPOUND_REVIEW_VIEWS: [CaptureViewSpec; 6] = [
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
        "Complete merchant property",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "street-gate",
        "Street gate and merchant frontage",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "side-passage",
        "Side access and courtyard",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "rear-store",
        "Rear storage range and court",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "court-exit",
        "House courtyard exit",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista()
    .settled_readback_pair(),
];
