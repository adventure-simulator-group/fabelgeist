use super::*;

pub(in crate::tactical_scene_viewer) const PARISH_REVIEW_VIEWS: [CaptureViewSpec; 37] = [
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
        "chapel-small-exterior",
        "Chapel small exterior",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-small-chancel-side",
        "Chapel small side and east end",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-small-interior",
        "Chapel small intact interior",
        CapturePose::CityExterior { camera: 2 },
        65.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-small-bell-stage",
        "Chapel small bell structure",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-small-facade",
        "Chapel small facade",
        CapturePose::CityExterior { camera: 4 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-small-distant",
        "Chapel small distant",
        CapturePose::CityExterior { camera: 5 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-medium-exterior",
        "Chapel medium exterior",
        CapturePose::CityExterior { camera: 6 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-medium-chancel-side",
        "Chapel medium side and east end",
        CapturePose::CityExterior { camera: 7 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-medium-interior",
        "Chapel medium intact interior",
        CapturePose::CityExterior { camera: 8 },
        65.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-medium-bell-stage",
        "Chapel medium bell structure",
        CapturePose::CityExterior { camera: 9 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-medium-facade",
        "Chapel medium facade",
        CapturePose::CityExterior { camera: 10 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-medium-distant",
        "Chapel medium distant",
        CapturePose::CityExterior { camera: 11 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-large-exterior",
        "Chapel large exterior",
        CapturePose::CityExterior { camera: 12 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-large-chancel-side",
        "Chapel large side and east end",
        CapturePose::CityExterior { camera: 13 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-large-interior",
        "Chapel large intact interior",
        CapturePose::CityExterior { camera: 14 },
        65.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-large-bell-stage",
        "Chapel large bell structure",
        CapturePose::CityExterior { camera: 15 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-large-facade",
        "Chapel large facade",
        CapturePose::CityExterior { camera: 16 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "chapel-large-distant",
        "Chapel large distant",
        CapturePose::CityExterior { camera: 17 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-small-exterior",
        "Parish church small exterior",
        CapturePose::CityExterior { camera: 18 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-small-chancel-side",
        "Parish church small side and east end",
        CapturePose::CityExterior { camera: 19 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-small-interior",
        "Parish church small intact interior",
        CapturePose::CityExterior { camera: 20 },
        65.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-small-bell-stage",
        "Parish church small bell structure",
        CapturePose::CityExterior { camera: 21 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-small-facade",
        "Parish church small facade",
        CapturePose::CityExterior { camera: 22 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-small-distant",
        "Parish church small distant",
        CapturePose::CityExterior { camera: 23 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-medium-exterior",
        "Parish church medium exterior",
        CapturePose::CityExterior { camera: 24 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-medium-chancel-side",
        "Parish church medium side and east end",
        CapturePose::CityExterior { camera: 25 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-medium-interior",
        "Parish church medium intact interior",
        CapturePose::CityExterior { camera: 26 },
        65.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-medium-bell-stage",
        "Parish church medium bell structure",
        CapturePose::CityExterior { camera: 27 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-medium-facade",
        "Parish church medium facade",
        CapturePose::CityExterior { camera: 28 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-medium-distant",
        "Parish church medium distant",
        CapturePose::CityExterior { camera: 29 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-large-exterior",
        "Parish church large exterior",
        CapturePose::CityExterior { camera: 30 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-large-chancel-side",
        "Parish church large side and east end",
        CapturePose::CityExterior { camera: 31 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-large-interior",
        "Parish church large intact interior",
        CapturePose::CityExterior { camera: 32 },
        65.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-large-bell-stage",
        "Parish church large bell structure",
        CapturePose::CityExterior { camera: 33 },
        58.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-large-facade",
        "Parish church large facade",
        CapturePose::CityExterior { camera: 34 },
        12.0,
        100,
    )
    .vista(),
    CaptureViewSpec::new(
        "parish-large-distant",
        "Parish church large distant",
        CapturePose::CityExterior { camera: 35 },
        12.0,
        100,
    )
    .vista(),
];
