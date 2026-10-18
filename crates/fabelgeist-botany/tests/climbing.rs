//! Plants among buildings: crowns growing round a tower, and climbers on
//! walls and trunks.

use fabelgeist_botany::vine::{Hold, Support};
use fabelgeist_botany::{Climate, Ecosystem, PlantType, Terrain, VineType};
use fabelgeist_buildings::{Material, SolidIndex, Structure};

/// Level ground with `structure` on it, its footprint unfit to root in.
fn among(structure: Structure, species: Vec<PlantType>) -> Ecosystem {
    let index = SolidIndex::new(structure.solids.clone(), 2.0);
    let terrain = Terrain::from_fn([-20.0, -20.0], [40.0, 40.0], [81, 81], |_, _| 0.0)
        .with_soil(|x, z, h| index.distance([x, h + 0.3, z]) < 0.5);
    let climate = Climate::new(
        fabelgeist_botany::units::Celsius::from_degrees(12.0),
        fabelgeist_botany::units::Millimetres::from_millimetres(800.0),
    );
    Ecosystem::new(terrain, climate, species)
        .with_obstacles(structure)
        .with_vines(vec![VineType::ivy()])
}

fn grow(ecosystem: &mut Ecosystem, until: f32) {
    while ecosystem.time.years() < until {
        pollster::block_on(ecosystem.step()).unwrap();
    }
}

fn lone(species: PlantType) -> PlantType {
    PlantType {
        seeding_frequency: 0.0,
        ..species
    }
}

#[test]
fn a_crown_grows_round_a_tower_and_not_into_it() {
    let tower = || {
        let mut s = Structure::new();
        s.drum([0.0, -0.5, 0.0], 3.0, 16.5, Material::Stone, 1.0);
        s
    };
    let grown = |obstacles: bool| {
        let species = vec![lone(PlantType::deciduous())];
        let mut ecosystem = if obstacles {
            among(tower(), species)
        } else {
            Ecosystem::new(
                Terrain::flat(40.0, 40.0),
                Climate::new(
                    fabelgeist_botany::units::Celsius::from_degrees(12.0),
                    fabelgeist_botany::units::Millimetres::from_millimetres(800.0),
                ),
                species,
            )
        };
        ecosystem
            .plant(
                fabelgeist_botany::identity::SpeciesId::from_word(0),
                fabelgeist_botany::units::GroundPoint::new(
                    fabelgeist_math::units::Metres::from_metres(4.4),
                    fabelgeist_math::units::Metres::from_metres(0.0),
                ),
            )
            .expect("open soil beside the tower");
        grow(&mut ecosystem, 40.0);
        ecosystem.plants.remove(0)
    };
    let inside = |plant: &fabelgeist_botany::Plant| {
        let drum = tower();
        let inside = plant
            .nodes
            .iter()
            .filter(|n| drum.distance(n.position.to_array()) < 0.0)
            .count();
        inside as f32 / plant.nodes.len() as f32
    };
    let free = grown(false);
    let walled = grown(true);
    // Alone, the crown would fill the tower's space; beside it, it keeps out.
    assert!(
        inside(&free) > 0.05,
        "{} of the free crown in the tower",
        inside(&free)
    );
    assert!(
        inside(&walled) < 0.01,
        "{} of the crown in the tower",
        inside(&walled)
    );
    // And it leans away from it, into the light.
    let middle = walled.nodes.iter().map(|n| n.position.x).sum::<f32>() / walled.nodes.len() as f32;
    assert!(
        middle > walled.position.x + 0.3,
        "crown centred at x {middle}, trunk at {}",
        walled.position.x
    );
    assert!(walled.height() > 6.0, "height {}", walled.height());
}

#[test]
fn ivy_climbs_a_wall_holding_on_to_its_face() {
    let mut wall = Structure::new();
    wall.block([0.0, -0.5, 0.0], [5.0, 0.6], 8.5, 0.0, Material::Stone, 1.0);
    let mut ecosystem = among(wall, vec![]);
    ecosystem
        .plant_vine(
            fabelgeist_botany::identity::VineSpeciesId::from_word(0),
            fabelgeist_botany::units::GroundPoint::new(
                fabelgeist_math::units::Metres::from_metres(0.0),
                fabelgeist_math::units::Metres::from_metres(1.4),
            ),
        )
        .expect("open soil by the wall");
    grow(&mut ecosystem, 12.0);
    let vine = &ecosystem.vines[0];
    let wall = ecosystem.obstacles.clone().unwrap();
    assert!(vine.height() > 4.0, "ivy {} m up", vine.height());
    let mut held = 0;
    for node in &vine.nodes {
        assert!(
            wall.distance(node.position) > -1e-3,
            "a node inside the wall at {:?}",
            node.position
        );
        if node.hold == Hold::Fixed && node.position.y > 1.0 {
            held += 1;
            assert!(
                wall.distance(node.position) < 0.02,
                "held off the wall at {:?}",
                node.position
            );
        }
    }
    assert!(held > 20, "{held} nodes held up the wall");
}

#[test]
fn ivy_climbs_a_trunk_and_rides_on_it_as_it_grows() {
    let mut ecosystem = Ecosystem::new(
        Terrain::flat(30.0, 30.0),
        Climate::new(
            fabelgeist_botany::units::Celsius::from_degrees(12.0),
            fabelgeist_botany::units::Millimetres::from_millimetres(800.0),
        ),
        vec![lone(PlantType::deciduous())],
    )
    .with_vines(vec![VineType::ivy()]);
    ecosystem
        .plant(
            fabelgeist_botany::identity::SpeciesId::from_word(0),
            fabelgeist_botany::units::GroundPoint::new(
                fabelgeist_math::units::Metres::from_metres(0.0),
                fabelgeist_math::units::Metres::from_metres(0.0),
            ),
        )
        .unwrap();
    grow(&mut ecosystem, 20.0);
    ecosystem
        .plant_vine(
            fabelgeist_botany::identity::VineSpeciesId::from_word(0),
            fabelgeist_botany::units::GroundPoint::new(
                fabelgeist_math::units::Metres::from_metres(0.5),
                fabelgeist_math::units::Metres::from_metres(0.0),
            ),
        )
        .unwrap();
    grow(&mut ecosystem, 34.0);
    let supports = ecosystem.supports();
    let vine = &ecosystem.vines[0];
    let on_bark: Vec<_> = vine
        .nodes
        .iter()
        .filter(|n| matches!(n.hold, Hold::Bark(_)))
        .collect();
    assert!(on_bark.len() > 15, "{} nodes on the bark", on_bark.len());
    assert!(vine.height() > 2.0, "ivy {} m up the tree", vine.height());
    // Each sits on the bark as the trunk is now -- thicker than when it took
    // hold -- a stem's radius out.
    let stem = VineType::ivy().shoot_diameter * 0.5;
    for node in on_bark {
        let bark = supports
            .nearest(node.position, 0.3, |c| {
                matches!(c.kind, Support::Bark { .. })
            })
            .expect("bark beside a node held on it");
        assert!(
            (bark.distance - stem).abs() < 0.01,
            "{} m off the bark",
            bark.distance
        );
    }
}
