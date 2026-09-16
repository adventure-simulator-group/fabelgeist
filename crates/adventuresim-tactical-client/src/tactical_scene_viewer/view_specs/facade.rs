use super::super::building_review::ReviewLod;
use super::*;
pub(in crate::tactical_scene_viewer) const FACADE_REVIEW_VIEWS: [CaptureViewSpec; 32] = [
    CaptureViewSpec::new(
        "warmup",
        "Facade material warmup",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .warmup()
    .vista(),
    CaptureViewSpec::new(
        "cottage-detail",
        "cottage detail",
        CapturePose::CityExterior { camera: 0 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-facade",
        "cottage facade",
        CapturePose::CityExterior { camera: 1 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-shell",
        "cottage shell",
        CapturePose::CityExterior { camera: 2 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-detail",
        "hall detail",
        CapturePose::CityExterior { camera: 3 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-facade",
        "hall facade",
        CapturePose::CityExterior { camera: 4 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-shell",
        "hall shell",
        CapturePose::CityExterior { camera: 5 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-detail",
        "town detail",
        CapturePose::CityExterior { camera: 6 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-facade",
        "town facade",
        CapturePose::CityExterior { camera: 7 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "town-shell",
        "town shell",
        CapturePose::CityExterior { camera: 8 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-detail",
        "merchant detail",
        CapturePose::CityExterior { camera: 9 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-facade",
        "merchant facade",
        CapturePose::CityExterior { camera: 10 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "merchant-shell",
        "merchant shell",
        CapturePose::CityExterior { camera: 11 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Shell)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shutter-closed-detail",
        "shutter closed detail",
        CapturePose::CityExterior { camera: 12 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shutter-closed-facade",
        "shutter closed facade",
        CapturePose::CityExterior { camera: 13 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shutter-open-detail",
        "shutter open detail",
        CapturePose::CityExterior { camera: 14 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shutter-open-facade",
        "shutter open facade",
        CapturePose::CityExterior { camera: 15 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "glazed-closed-detail",
        "glazed closed detail",
        CapturePose::CityExterior { camera: 16 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "glazed-closed-facade",
        "glazed closed facade",
        CapturePose::CityExterior { camera: 17 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "glazed-open-detail",
        "glazed open detail",
        CapturePose::CityExterior { camera: 18 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "glazed-open-facade",
        "glazed open facade",
        CapturePose::CityExterior { camera: 19 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "barred-open-detail",
        "barred open detail",
        CapturePose::CityExterior { camera: 20 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "barred-open-facade",
        "barred open facade",
        CapturePose::CityExterior { camera: 21 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "door-open-detail",
        "door open detail",
        CapturePose::CityExterior { camera: 22 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "door-open-facade",
        "door open facade",
        CapturePose::CityExterior { camera: 23 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "fixed-glass-open-detail",
        "fixed glass open detail",
        CapturePose::CityExterior { camera: 24 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "fixed-glass-open-facade",
        "fixed glass open facade",
        CapturePose::CityExterior { camera: 25 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "cottage-tile-courses",
        "cottage tile courses",
        CapturePose::CityExterior { camera: 26 },
        68.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "hall-tile-courses",
        "hall tile courses",
        CapturePose::CityExterior { camera: 27 },
        68.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shed-flue-exterior",
        "shed flue exterior",
        CapturePose::CityExterior { camera: 28 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shed-flue-roof-detail",
        "shed flue roof detail",
        CapturePose::CityExterior { camera: 29 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Detail)
    .settled_readback_pair(),
    CaptureViewSpec::new(
        "shed-flue-roof-facade",
        "shed flue roof facade",
        CapturePose::CityExterior { camera: 30 },
        58.0,
        100,
    )
    .vista()
    .building_lod(ReviewLod::Facade)
    .settled_readback_pair(),
];
