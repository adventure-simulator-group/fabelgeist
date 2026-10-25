use super::*;
use crate::camera::config::authored_metres;
use adventuresim_plant_generator::fungus::FungusSpecies;

pub(in crate::tactical_scene_viewer) const FUNGUS_REVIEW_VIEWS: [CaptureViewSpec; 7] = [
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
        "fly-agaric-contact",
        "Actual fly agaric and ground contact",
        CapturePose::Fungus {
            distance: authored_metres(0.45),
            species: FungusSpecies::FlyAgaric
        },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "porcini-contact",
        "Actual porcini and ground contact",
        CapturePose::Fungus {
            distance: authored_metres(0.45),
            species: FungusSpecies::Porcini
        },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "chanterelle-contact",
        "Actual chanterelle and ground contact",
        CapturePose::Fungus {
            distance: authored_metres(0.35),
            species: FungusSpecies::Chanterelle
        },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "puffball-contact",
        "Actual puffball and ground contact",
        CapturePose::Fungus {
            distance: authored_metres(0.35),
            species: FungusSpecies::CommonPuffball
        },
        40.0,
        100
    )
    .settled_readback_pair(),
    v!(
        "fungus-community",
        "Actual fungal community at gameplay approach",
        CapturePose::Fungus {
            distance: authored_metres(2.0),
            species: FungusSpecies::FlyAgaric
        },
        55.0,
        100
    )
    .settled_readback_pair()
    .vista(),
    v!(
        "fungus-distance",
        "Fungal community at detail fade distance",
        CapturePose::Fungus {
            distance: authored_metres(20.0),
            species: FungusSpecies::FlyAgaric
        },
        55.0,
        100
    )
    .settled_readback_pair()
    .vista(),
];
