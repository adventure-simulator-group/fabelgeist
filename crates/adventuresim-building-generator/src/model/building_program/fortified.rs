use super::*;
use RoomKind::*;

impl BuildingProgram {
    pub(super) fn castle_gatehouse(seed: u64) -> Self {
        let archetype = BuildingArchetype::CastleGatehouse;
        Self {
            archetype,
            usage: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 10,
                depth: 6,
            },
            storey_height_metres: 3.4,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Passage, 18).exterior(),
                        RoomRequirement::new(Guardroom, 18)
                            .exterior()
                            .beside(Passage),
                        RoomRequirement::new(Armoury, 12).beside(Guardroom),
                        RoomRequirement::new(StairHall, 12),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(GreatHall, 24).exterior(),
                        RoomRequirement::new(Guardroom, 16).exterior(),
                        RoomRequirement::new(Armoury, 10),
                        RoomRequirement::new(StairHall, 10),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::TowerSpiral {
                lowest_storey: 0,
                highest_storey: 1,
            }],
            wall_style: WallStyle::Stone,
            timber_frame_style: None,
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 48.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn courtyard_castle(seed: u64) -> Self {
        let archetype = BuildingArchetype::CourtyardCastle;
        Self {
            archetype,
            usage: None,
            seed,
            footprint: Footprint::Courtyard {
                width: 18,
                depth: 16,
                wing: 4,
                gate_width: 4,
            },
            storey_height_metres: 3.5,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Passage, 24).exterior(),
                        RoomRequirement::new(GreatHall, 55).exterior(),
                        RoomRequirement::new(Kitchen, 30).exterior(),
                        RoomRequirement::new(Guardroom, 35).exterior(),
                        RoomRequirement::new(Armoury, 24),
                        RoomRequirement::new(Storage, 35),
                        RoomRequirement::new(StairHall, 25),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Gallery, 50).exterior(),
                        RoomRequirement::new(GreatHall, 55).exterior(),
                        RoomRequirement::new(Chapel, 28).exterior(),
                        RoomRequirement::new(Bedchamber, 34).exterior(),
                        RoomRequirement::new(Guardroom, 30).exterior(),
                        RoomRequirement::new(StairHall, 25),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::TowerSpiral {
                lowest_storey: 0,
                highest_storey: 1,
            }],
            wall_style: WallStyle::Stone,
            timber_frame_style: None,
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 52.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn walled_keep(seed: u64) -> Self {
        let archetype = BuildingArchetype::WalledKeep;
        Self {
            archetype,
            usage: None,
            seed,
            footprint: Footprint::Rectangle { width: 9, depth: 8 },
            storey_height_metres: 3.4,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(EntranceHall, 14).exterior(),
                        RoomRequirement::new(Guardroom, 18).exterior(),
                        RoomRequirement::new(Armoury, 12),
                        RoomRequirement::new(Storage, 18),
                        RoomRequirement::new(StairHall, 10),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(GreatHall, 28).exterior(),
                        RoomRequirement::new(Kitchen, 12).exterior(),
                        RoomRequirement::new(Guardroom, 12).exterior(),
                        RoomRequirement::new(StairHall, 10),
                        RoomRequirement::new(Storage, 10),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Bedchamber, 20).exterior(),
                        RoomRequirement::new(Guardroom, 16).exterior(),
                        RoomRequirement::new(Armoury, 12),
                        RoomRequirement::new(StairHall, 10),
                        RoomRequirement::new(Storage, 14),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::TowerSpiral {
                lowest_storey: 0,
                highest_storey: 2,
            }],
            wall_style: WallStyle::Stone,
            timber_frame_style: None,
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 0.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn artillery_rondel_castle(seed: u64) -> Self {
        let archetype = BuildingArchetype::ArtilleryRondelCastle;
        Self {
            archetype,
            usage: None,
            seed,
            // The room-grid footprint is the retained older keep. The
            // independent ArtilleryCastleAssembly owns the much larger
            // 36 x 30 m clear court and retrofit enceinte around it.
            footprint: Footprint::Rectangle { width: 9, depth: 8 },
            storey_height_metres: 3.4,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(EntranceHall, 14).exterior(),
                        RoomRequirement::new(Guardroom, 18).exterior(),
                        RoomRequirement::new(Armoury, 12),
                        RoomRequirement::new(Storage, 18),
                        RoomRequirement::new(StairHall, 10),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(GreatHall, 28).exterior(),
                        RoomRequirement::new(Kitchen, 12).exterior(),
                        RoomRequirement::new(Guardroom, 12).exterior(),
                        RoomRequirement::new(StairHall, 10),
                        RoomRequirement::new(Storage, 10),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Bedchamber, 20).exterior(),
                        RoomRequirement::new(Guardroom, 16).exterior(),
                        RoomRequirement::new(Armoury, 12),
                        RoomRequirement::new(StairHall, 10),
                        RoomRequirement::new(Storage, 14),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::TowerSpiral {
                lowest_storey: 0,
                highest_storey: 2,
            }],
            wall_style: WallStyle::Stone,
            timber_frame_style: None,
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 0.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }
}
