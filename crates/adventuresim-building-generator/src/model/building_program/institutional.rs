use super::*;
use RoomKind::*;

impl BuildingProgram {
    pub(super) fn renaissance_town_hall(seed: u64) -> Self {
        let archetype = BuildingArchetype::RenaissanceTownHall;
        Self {
            archetype,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 14,
                depth: 10,
            },
            storey_height_metres: 3.4,
            storeys: vec![
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(EntranceHall, 30).exterior(),
                        RoomRequirement::new(GreatHall, 48).exterior(),
                        RoomRequirement::new(Shop, 24).exterior(),
                        RoomRequirement::new(Storage, 18),
                        RoomRequirement::new(StairHall, 20),
                    ],
                },
                StoreyProgram {
                    rooms: vec![
                        RoomRequirement::new(GreatHall, 54).exterior(),
                        RoomRequirement::new(Gallery, 34).exterior(),
                        RoomRequirement::new(Chapel, 20).exterior(),
                        RoomRequirement::new(Storage, 14),
                        RoomRequirement::new(StairHall, 18),
                    ],
                },
            ],
            vertical_connections: vec![VerticalConnectionRequirement::StraightStair {
                lowest_storey: 0,
                highest_storey: 1,
                landing_room: StairHall,
            }],
            wall_style: WallStyle::TimberFrame,
            timber_frame_style: Some(TimberFrameStyle::EarlyModernOrnate),
            upper_storey_projection_metres: 0.24,
            roof_pitch_degrees: 54.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    pub(super) fn cathedral(seed: u64) -> Self {
        let archetype = BuildingArchetype::Cathedral;
        Self {
            archetype,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 28,
                depth: 14,
            },
            storey_height_metres: 5.8,
            storeys: vec![StoreyProgram {
                rooms: vec![
                    RoomRequirement::new(Nave, 190).exterior().beside(Chancel),
                    RoomRequirement::new(Chancel, 70).exterior().beside(Nave),
                    RoomRequirement::new(Chapel, 32).exterior(),
                    RoomRequirement::new(Sacristy, 24)
                        .exterior()
                        .beside(Chancel),
                    RoomRequirement::new(EntranceHall, 20).exterior(),
                ],
            }],
            vertical_connections: Vec::new(),
            wall_style: WallStyle::Stone,
            timber_frame_style: None,
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 58.0,
            roof_demonstrator: None,
            church_program: Some(ChurchProgram::URBAN_BRICK_BASILICA),
        }
    }
}
