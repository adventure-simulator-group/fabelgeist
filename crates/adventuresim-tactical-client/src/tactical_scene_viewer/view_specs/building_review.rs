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

pub(in crate::tactical_scene_viewer) const WORKPLACE_REVIEW_VIEWS: [CaptureViewSpec; 45] = [
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
    CaptureViewSpec::new(
        "brewery-exterior",
        "Brewery exterior",
        CapturePose::CityExterior { camera: 18 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "brewery-interior",
        "Brewery interior",
        CapturePose::CityExterior { camera: 19 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "brewery-work-bay",
        "Brewery work-bay",
        CapturePose::CityExterior { camera: 20 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "brewery-facade",
        "Brewery facade",
        CapturePose::CityExterior { camera: 21 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "brewery-distant",
        "Brewery distant",
        CapturePose::CityExterior { camera: 22 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "malthouse-exterior",
        "Malthouse exterior",
        CapturePose::CityExterior { camera: 23 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "malthouse-interior",
        "Malthouse interior",
        CapturePose::CityExterior { camera: 24 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "malthouse-work-bay",
        "Malthouse work-bay",
        CapturePose::CityExterior { camera: 25 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "malthouse-facade",
        "Malthouse facade",
        CapturePose::CityExterior { camera: 26 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "malthouse-distant",
        "Malthouse distant",
        CapturePose::CityExterior { camera: 27 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "timber-yard-exterior",
        "Timber yard exterior",
        CapturePose::CityExterior { camera: 28 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "timber-yard-interior",
        "Timber yard interior",
        CapturePose::CityExterior { camera: 29 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "timber-yard-work-bay",
        "Timber yard work-bay",
        CapturePose::CityExterior { camera: 30 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "timber-yard-facade",
        "Timber yard facade",
        CapturePose::CityExterior { camera: 31 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "timber-yard-distant",
        "Timber yard distant",
        CapturePose::CityExterior { camera: 32 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "carpenter-exterior",
        "Carpenter exterior",
        CapturePose::CityExterior { camera: 33 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "carpenter-interior",
        "Carpenter interior",
        CapturePose::CityExterior { camera: 34 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "carpenter-work-bay",
        "Carpenter work-bay",
        CapturePose::CityExterior { camera: 35 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "carpenter-facade",
        "Carpenter facade",
        CapturePose::CityExterior { camera: 36 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "carpenter-distant",
        "Carpenter distant",
        CapturePose::CityExterior { camera: 37 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "craft-street",
        "Working-building comparison",
        CapturePose::CityExterior { camera: 38 },
        88.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "warehouse-exterior",
        "Warehouse exterior",
        CapturePose::CityExterior { camera: 39 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "warehouse-interior",
        "Warehouse interior",
        CapturePose::CityExterior { camera: 40 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "warehouse-loading-bay",
        "Warehouse loading-bay",
        CapturePose::CityExterior { camera: 41 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "warehouse-facade",
        "Warehouse facade",
        CapturePose::CityExterior { camera: 42 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "warehouse-distant",
        "Warehouse distant",
        CapturePose::CityExterior { camera: 43 },
        12.0,
        100,
    )
    .vista(),
];
