use super::*;

impl BuildingProgram {
    /// A courtyard accessory with storage at ground level and no dwelling capacity.
    pub(super) fn storage_range(seed: u64) -> Self {
        Self {
            archetype: BuildingArchetype::StorageRange,
            usage: None,
            service_size: None,
            seed,
            footprint: Footprint::Rectangle { width: 6, depth: 4 },
            storey_height_metres: 2.8,
            storeys: vec![StoreyProgram {
                rooms: vec![RoomRequirement::new(RoomKind::Storage, 24).exterior()],
            }],
            vertical_connections: Vec::new(),
            wall_style: WallStyle::TimberFrame,
            timber_frame_style: Some(TimberFrameStyle::LateMedieval),
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 48.0,
            roof_demonstrator: None,
            church_program: None,
            domestic_heating: None,
        }
    }
}
