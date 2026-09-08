use super::*;
use RoomKind::*;

impl BuildingProgram {
    pub(super) fn town_house(seed: u64) -> Self {
        let archetype = BuildingArchetype::TownHouse;
        Self {
            archetype,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 6,
                depth: 10,
            },
            storey_height_metres: 3.0,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Shop, 18).exterior().beside(Workshop),
                        RoomRequirement::new(EntranceHall, 8)
                            .exterior()
                            .beside(StairHall),
                        RoomRequirement::new(Workshop, 15).beside(Storage),
                        RoomRequirement::new(Storage, 8),
                        RoomRequirement::new(StairHall, 8),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(CommonRoom, 22)
                            .exterior()
                            .beside(Kitchen),
                        RoomRequirement::new(Kitchen, 12).exterior().beside(Pantry),
                        RoomRequirement::new(Pantry, 6),
                        RoomRequirement::new(Bedchamber, 13).exterior(),
                        RoomRequirement::new(StairHall, 7),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::StraightStair {
                lowest_storey: 0,
                highest_storey: 1,
                landing_room: StairHall,
            }],
            wall_style: WallStyle::TimberFrame,
            timber_frame_style: Some(TimberFrameStyle::LateMedieval),
            upper_storey_projection_metres: 0.22,
            roof_pitch_degrees: 55.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn hall_house(seed: u64) -> Self {
        let archetype = BuildingArchetype::HallHouse;
        Self {
            archetype,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 9,
                depth: 13,
            },
            storey_height_metres: 3.3,
            storeys: vec![StoreyProgram {
                rooms: vec![
                    RoomRequirement::new(GreatHall, 52)
                        .exterior()
                        .beside(Kitchen),
                    RoomRequirement::new(EntranceHall, 14).exterior(),
                    RoomRequirement::new(Kitchen, 20).exterior().beside(Pantry),
                    RoomRequirement::new(Pantry, 10),
                    RoomRequirement::new(Storage, 15).exterior(),
                ],
            }],
            vertical_connections: Vec::new(),
            wall_style: WallStyle::TimberFrame,
            timber_frame_style: Some(TimberFrameStyle::NorthernCloseStudded),
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 50.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn fachwerk_cottage(seed: u64) -> Self {
        let archetype = BuildingArchetype::FachwerkCottage;
        Self {
            archetype,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle { width: 7, depth: 8 },
            storey_height_metres: 2.8,
            storeys: vec![StoreyProgram {
                rooms: vec![
                    RoomRequirement::new(CommonRoom, 18)
                        .exterior()
                        .beside(Kitchen),
                    RoomRequirement::new(Kitchen, 10).exterior().beside(Pantry),
                    RoomRequirement::new(Pantry, 5),
                    RoomRequirement::new(Bedchamber, 12).exterior(),
                    RoomRequirement::new(EntranceHall, 7).exterior(),
                ],
            }],
            vertical_connections: Vec::new(),
            wall_style: WallStyle::TimberFrame,
            timber_frame_style: Some(TimberFrameStyle::NorthernCloseStudded),
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 53.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn fachwerk_merchant_house(seed: u64) -> Self {
        let archetype = BuildingArchetype::FachwerkMerchantHouse;
        Self {
            archetype,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 8,
                depth: 11,
            },
            storey_height_metres: 3.0,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Shop, 24).exterior().beside(Workshop),
                        RoomRequirement::new(EntranceHall, 10)
                            .exterior()
                            .beside(StairHall),
                        RoomRequirement::new(Workshop, 22).beside(Storage),
                        RoomRequirement::new(Storage, 16),
                        RoomRequirement::new(StairHall, 16),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(CommonRoom, 30)
                            .exterior()
                            .beside(Kitchen),
                        RoomRequirement::new(Kitchen, 18).exterior().beside(Pantry),
                        RoomRequirement::new(Pantry, 8),
                        RoomRequirement::new(Bedchamber, 20).exterior(),
                        RoomRequirement::new(StairHall, 12),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(Gallery, 26).exterior(),
                        RoomRequirement::new(Bedchamber, 24).exterior(),
                        RoomRequirement::new(Bedchamber, 20).exterior(),
                        RoomRequirement::new(Storage, 10),
                        RoomRequirement::new(StairHall, 8),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::StraightStair {
                lowest_storey: 0,
                highest_storey: 2,
                landing_room: StairHall,
            }],
            wall_style: WallStyle::TimberFrame,
            timber_frame_style: Some(TimberFrameStyle::EarlyModernOrnate),
            upper_storey_projection_metres: 0.28,
            roof_pitch_degrees: 57.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }
}
