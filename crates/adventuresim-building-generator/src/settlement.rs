//! Occupied building uses reuse structural families, with purpose-specific room programmes.
use crate::{
    BuildingArchetype, BuildingProgram, Footprint, RoomKind, RoomRequirement, StoreyProgram,
    WallStyle,
};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use fabelgeist_determinism::mix64;

const ROOF_VARIATION_DEGREES: f32 = 4.0;
const STOREY_VARIATION_METRES: f32 = 0.15;
const VALID_RECIPE_ATTEMPTS: u8 = 64;
const RECIPE_VARIATION_DOMAIN: u64 = 0x7661_7269_6174_696f;

pub const fn settlement_archetype(usage: BuildingUse) -> BuildingArchetype {
    use BuildingUse::*;
    if crate::WorkplaceKind::from_use(usage).is_some() {
        return BuildingArchetype::Workplace;
    }
    match usage {
        ParishChurch | Chapel => BuildingArchetype::ParishChurch,
        Cathedral => BuildingArchetype::Cathedral,
        TownHall | WeighHouse | Guildhall | University | Mint => {
            BuildingArchetype::RenaissanceTownHall
        }
        Castle | Arsenal => BuildingArchetype::WalledKeep,
        Inn | Bookshop | PrintingHouse | Apothecary | CustomsHouse | Manor => {
            BuildingArchetype::FachwerkMerchantHouse
        }
        HorseMill | WaterMill | Windmill | FullingMill | PaperMill | Sawmill | Hospital
        | Bathhouse | School | Monastery | Synagogue | WoadStore | SaltWorks | Smelter
        | Brickworks | Glassworks => BuildingArchetype::HallHouse,
        Rectory | ExecutionerHouse => BuildingArchetype::FachwerkCottage,
        _ => BuildingArchetype::TownHouse,
    }
}

impl BuildingProgram {
    /// Search a bounded deterministic sequence and expose the structural error if it fails.
    pub fn validated_settlement(
        archetype: BuildingArchetype,
        usage: BuildingUse,
        initial_seed: u64,
        size: Option<crate::WorkplaceSize>,
    ) -> Result<Self, crate::GenerationError> {
        let mut first_error = None;
        for attempt in 0..VALID_RECIPE_ATTEMPTS {
            let seed = if attempt == 0 {
                initial_seed
            } else {
                mix64(initial_seed ^ u64::from(attempt))
            };
            let mut program = Self::settlement(archetype, Some(usage), seed);
            if let Some(size) = size {
                program = program.with_workplace_size(size);
            }
            match crate::generate(&program) {
                Ok(_) => return Ok(program),
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        Err(first_error.expect("the bounded recipe search always attempts the initial seed"))
    }

    /// The same compact recipe is used for playable buildings and distant shells.
    pub fn settlement(archetype: BuildingArchetype, usage: Option<BuildingUse>, seed: u64) -> Self {
        let mut program = Self::fixture(archetype, seed);
        program.usage = usage;
        if let Some(usage) = usage {
            program.assign_use(usage);
        }
        if matches!(
            archetype,
            BuildingArchetype::TownHouse
                | BuildingArchetype::HallHouse
                | BuildingArchetype::FachwerkCottage
                | BuildingArchetype::FachwerkMerchantHouse
                | BuildingArchetype::RenaissanceTownHall
                | BuildingArchetype::ParishChurch
        ) {
            let sample = mix64(seed ^ RECIPE_VARIATION_DOMAIN);
            let roof = (sample as u16 as f32 / u16::MAX as f32) * 2.0 - 1.0;
            let height = ((sample >> 16) as u16 as f32 / u16::MAX as f32) * 2.0 - 1.0;
            // Half-hip gable framing needs the curated minimum pitch to clear
            // the opening heads below it. Vary those roofs upward from that seat.
            let roof = if matches!(
                archetype,
                BuildingArchetype::HallHouse | BuildingArchetype::RenaissanceTownHall
            ) {
                roof.abs()
            } else {
                roof
            };
            program.roof_pitch_degrees += roof * ROOF_VARIATION_DEGREES;
            program.storey_height_metres += height * STOREY_VARIATION_METRES;
        }
        if program.workplace_kind().is_some() {
            program = program.with_workplace_size(crate::WorkplaceSize::Medium);
        }
        program
    }

    pub(crate) fn parish_church(seed: u64) -> Self {
        use RoomKind::*;
        Self {
            archetype: BuildingArchetype::ParishChurch,
            usage: None,
            workplace_size: None,
            seed,
            footprint: Footprint::Rectangle {
                width: 9,
                depth: 13,
            },
            storey_height_metres: 6.0,
            storeys: vec![StoreyProgram {
                rooms: vec![
                    RoomRequirement::new(Nave, 80).exterior().beside(Chancel),
                    RoomRequirement::new(Chancel, 25)
                        .exterior()
                        .beside(Sacristy),
                    RoomRequirement::new(Sacristy, 12).exterior(),
                ],
            }],
            vertical_connections: Vec::new(),
            wall_style: WallStyle::Stone,
            timber_frame_style: None,
            upper_storey_projection_metres: 0.0,
            roof_pitch_degrees: 55.0,
            roof_demonstrator: None,
            church_program: None,
        }
    }

    fn assign_use(&mut self, usage: BuildingUse) {
        use BuildingUse::*;
        let (main, secondary) = match usage {
            MarketHall => (RoomKind::GreatHall, RoomKind::Storage),
            Stable => (RoomKind::Stalls, RoomKind::Storage),
            HorseMill | WaterMill | Windmill | FullingMill | PaperMill | Sawmill => {
                (RoomKind::MillingFloor, RoomKind::Storage)
            }
            Barn | Granary | Warehouse | WoadStore | Malthouse | TimberYard => {
                (RoomKind::Storage, RoomKind::CountingRoom)
            }
            Hospital => (RoomKind::Ward, RoomKind::Kitchen),
            School | University => (RoomKind::Schoolroom, RoomKind::Storage),
            Brewery | Dyer | Tannery | Bathhouse => (RoomKind::VatRoom, RoomKind::Storage),
            Bakehouse | Potter | Brickworks | SaltWorks | Smelter | Glassworks => {
                (RoomKind::KilnRoom, RoomKind::Storage)
            }
            Inn => (RoomKind::CommonRoom, RoomKind::Kitchen),
            Guardhouse | Prison | Arsenal => (RoomKind::Guardroom, RoomKind::Storage),
            TownHall | WeighHouse | CustomsHouse | Mint | AssayHouse => {
                (RoomKind::CountingRoom, RoomKind::Storage)
            }
            Dwelling | ParishChurch | Chapel | Cathedral | Castle | Monastery | Synagogue
            | Manor | Rectory | ExecutionerHouse => return,
            _ => (RoomKind::Shop, RoomKind::Workshop),
        };
        // Retain circulation, upper-floor dwellings and load-bearing family; replace the working rooms.
        for room in &mut self.storeys[0].rooms {
            if matches!(room.kind, RoomKind::GreatHall | RoomKind::Shop) {
                room.kind = main;
            } else if matches!(room.kind, RoomKind::Kitchen | RoomKind::Workshop) {
                room.kind = secondary;
            }
            for neighbour in &mut room.preferred_neighbours {
                if matches!(*neighbour, RoomKind::GreatHall | RoomKind::Shop) {
                    *neighbour = main;
                } else if matches!(*neighbour, RoomKind::Kitchen | RoomKind::Workshop) {
                    *neighbour = secondary;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile_building_collision, generate};

    #[test]
    fn market_hall_keeps_a_connected_public_trading_floor() {
        let program = BuildingProgram::settlement(
            BuildingArchetype::HallHouse,
            Some(BuildingUse::MarketHall),
            232_833_052_103_632_759,
        );
        generate(&program).unwrap();
    }

    #[test]
    fn occupied_uses_generate_audited_rooms_and_collision() {
        for (usage, room) in [
            (BuildingUse::Stable, RoomKind::Stalls),
            (BuildingUse::HorseMill, RoomKind::MillingFloor),
            (BuildingUse::Bakehouse, RoomKind::KilnRoom),
            (BuildingUse::Brewery, RoomKind::VatRoom),
            (BuildingUse::Hospital, RoomKind::Ward),
            (BuildingUse::School, RoomKind::Schoolroom),
            (BuildingUse::ParishChurch, RoomKind::Nave),
            (BuildingUse::Inn, RoomKind::CommonRoom),
        ] {
            let archetype = settlement_archetype(usage);
            let mut failures = Vec::new();
            let found = (0..64).find_map(|attempt| {
                let seed = if attempt == 0 {
                    42
                } else {
                    mix64(42 ^ attempt)
                };
                let program = BuildingProgram::settlement(archetype, Some(usage), seed);
                match generate(&program) {
                    Ok(plan) => Some((program, plan)),
                    Err(error) => {
                        failures.push(error);
                        None
                    }
                }
            });
            let (program, plan) =
                found.unwrap_or_else(|| panic!("{usage:?}: {:?}", failures.first()));
            assert!(
                plan.storeys[0].rooms.iter().any(|r| r.kind == room),
                "{usage:?}"
            );
            assert_eq!(
                program,
                BuildingProgram::settlement(archetype, Some(usage), program.seed)
            );
            let collision = compile_building_collision(&plan);
            assert!(collision.bounds.max.x > collision.bounds.min.x);
        }
    }

    #[test]
    fn settlement_recipes_vary_roofline_without_changing_the_reserved_footprint() {
        let first = BuildingProgram::settlement(
            BuildingArchetype::TownHouse,
            Some(BuildingUse::Dwelling),
            42,
        );
        let second = BuildingProgram::settlement(
            BuildingArchetype::TownHouse,
            Some(BuildingUse::Dwelling),
            43,
        );
        assert_ne!(first.roof_pitch_degrees, second.roof_pitch_degrees);
        assert_ne!(first.storey_height_metres, second.storey_height_metres);
        assert_eq!(first.footprint, second.footprint);
    }
}
