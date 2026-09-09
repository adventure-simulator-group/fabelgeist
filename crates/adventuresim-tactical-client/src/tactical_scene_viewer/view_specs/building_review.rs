use super::*;

pub(in crate::tactical_scene_viewer) const SHOP_REVIEW_VIEWS: [CaptureViewSpec; 13] = [
    CaptureViewSpec::new(
        "warmup",
        "Production building material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "grenze-wall",
        "Grenze wall",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "grenze-projecting",
        "Grenze projecting",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "fraktur-wall",
        "Fraktur wall",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "fraktur-projecting",
        "Fraktur projecting",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "reverse",
        "Reverse",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "long-name",
        "Long name",
        CapturePose::CityExterior { camera: 5 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "street",
        "Street",
        CapturePose::CityExterior { camera: 6 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "mounting-detail",
        "Mounting detail",
        CapturePose::CityExterior { camera: 7 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "window-glass",
        "Window glass",
        CapturePose::CityExterior { camera: 8 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "plaster-facade",
        "Plaster facade",
        CapturePose::CityExterior { camera: 9 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "facade-lod",
        "Facade lod",
        CapturePose::CityExterior { camera: 10 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "shell-lod",
        "Shell lod",
        CapturePose::CityExterior { camera: 11 },
        12.0,
        100,
    )
    .vista(),
];

pub(in crate::tactical_scene_viewer) const WORKPLACE_REVIEW_VIEWS: [CaptureViewSpec; 19] = [
    CaptureViewSpec::new(
        "warmup",
        "Production building material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "barn-exterior",
        "Barn exterior",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "barn-interior",
        "Barn interior",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "barn-distant",
        "Barn distant",
        CapturePose::CityExterior { camera: 2 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "stable-exterior",
        "Stable exterior",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "stable-interior",
        "Stable interior",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "stable-distant",
        "Stable distant",
        CapturePose::CityExterior { camera: 5 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "granary-exterior",
        "Granary exterior",
        CapturePose::CityExterior { camera: 6 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "granary-interior",
        "Granary interior",
        CapturePose::CityExterior { camera: 7 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "granary-distant",
        "Granary distant",
        CapturePose::CityExterior { camera: 8 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "smithy-exterior",
        "Smithy exterior",
        CapturePose::CityExterior { camera: 9 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "smithy-interior",
        "Smithy interior",
        CapturePose::CityExterior { camera: 10 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "smithy-distant",
        "Smithy distant",
        CapturePose::CityExterior { camera: 11 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "bakehouse-exterior",
        "Bakehouse exterior",
        CapturePose::CityExterior { camera: 12 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "bakehouse-interior",
        "Bakehouse interior",
        CapturePose::CityExterior { camera: 13 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "bakehouse-distant",
        "Bakehouse distant",
        CapturePose::CityExterior { camera: 14 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "market-hall-exterior",
        "Market hall exterior",
        CapturePose::CityExterior { camera: 15 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "market-hall-interior",
        "Market hall interior",
        CapturePose::CityExterior { camera: 16 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "market-hall-distant",
        "Market hall distant",
        CapturePose::CityExterior { camera: 17 },
        12.0,
        100,
    )
    .vista(),
];
