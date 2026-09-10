//! Authored furniture demand. Counts are targets; clear circulation wins over density.
use crate::furniture::FurnitureKind;
use crate::{BuildingProgram, CELL_SIZE_METRES, Room, RoomKind};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FurniturePosition {
    Wall,
    Rows,
    Centre,
    CounterRun,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FurnitureBudget {
    pub kind: FurnitureKind,
    pub count: usize,
    pub position: FurniturePosition,
}

/// All settlement uses select a specific trade kit; room roles decide where it belongs.
pub fn furniture_budgets(program: &BuildingProgram, room: &Room) -> Vec<FurnitureBudget> {
    use FurnitureKind::*;
    use FurniturePosition::*;
    let area = room.cells.len() as f32 * CELL_SIZE_METRES * CELL_SIZE_METRES;
    let usage = program.usage.unwrap_or(BuildingUse::Dwelling);
    let trade = trade_kit(usage);
    let mut result = Vec::new();
    let mut add = |kind, metres_per_item: f32, cap: usize, position| {
        result.push(FurnitureBudget {
            kind,
            count: (area / metres_per_item).floor().max(1.0) as usize,
            position,
        });
        result.last_mut().unwrap().count = result.last().unwrap().count.min(cap);
    };
    match room.kind {
        RoomKind::EntranceHall | RoomKind::Passage | RoomKind::StairHall | RoomKind::Gallery => {}
        RoomKind::Bedchamber => {
            add(
                if matches!(
                    usage,
                    BuildingUse::Prison | BuildingUse::Guardhouse | BuildingUse::Arsenal
                ) {
                    BunkBed
                } else {
                    Bed
                },
                10.0,
                6,
                Wall,
            );
            add(StorageChest, 18.0, 3, Wall);
        }
        RoomKind::Kitchen => {
            add(Workbench, 15.0, 2, Wall);
            add(Cupboard, 20.0, 2, Wall);
            add(KneadingTrough, 30.0, 1, Wall);
        }
        RoomKind::Pantry => {
            add(Shelving, 8.0, 6, Wall);
            add(StorageChest, 20.0, 2, Wall);
        }
        RoomKind::Storage => {
            add(trade.storage, 8.0, 12, Rows);
            add(Shelving, 20.0, 3, Wall);
        }
        RoomKind::Shop => {
            add(Counter, 50.0, 2, CounterRun);
            add(Shelving, 10.0, 8, Wall);
            add(DisplayCounter, 30.0, 2, Centre);
        }
        RoomKind::Workshop | RoomKind::KilnRoom | RoomKind::VatRoom | RoomKind::MillingFloor => {
            add(trade.work, 12.0, 8, Wall);
            add(trade.secondary, 20.0, 4, Wall);
        }
        RoomKind::Stalls => {
            add(HayRack, 14.0, 8, Wall);
            add(FeedTrough, 18.0, 6, Wall);
        }
        RoomKind::Ward => {
            add(WardBed, 12.0, 16, Rows);
            add(WashStand, 35.0, 3, Wall);
            add(Cupboard, 40.0, 2, Wall);
        }
        RoomKind::Schoolroom => {
            add(WritingDesk, 40.0, 1, Wall);
            add(DiningTable, 12.0, 10, Rows);
        }
        RoomKind::CountingRoom => {
            add(WritingDesk, 18.0, 5, Wall);
            add(Cupboard, 22.0, 4, Wall);
        }
        RoomKind::Guardroom | RoomKind::Armoury | RoomKind::TowerChamber => {
            add(trade.work, 18.0, 6, Wall);
            add(WeaponRack, 14.0, 8, Wall);
            add(StorageChest, 24.0, 3, Wall);
        }
        RoomKind::Nave | RoomKind::Chapel => {
            add(ChurchBench, 7.0, 48, Rows);
            add(Lectern, 120.0, 1, Wall);
        }
        RoomKind::Chancel => {
            add(
                if usage == BuildingUse::Synagogue {
                    Lectern
                } else {
                    Altar
                },
                100.0,
                1,
                Wall,
            );
            add(Cupboard, 80.0, 1, Wall);
        }
        RoomKind::Sacristy => {
            add(Cupboard, 10.0, 4, Wall);
            add(WritingDesk, 30.0, 1, Wall);
        }
        RoomKind::GreatHall | RoomKind::CommonRoom => {
            if usage == BuildingUse::Bathhouse {
                add(BathTub, 9.0, 12, Rows);
                add(Bench, 30.0, 3, Wall);
            } else {
                if usage == BuildingUse::Inn {
                    add(Counter, 100.0, 1, CounterRun);
                }
                add(DiningTable, 18.0, 8, Centre);
                add(Cupboard, 45.0, 2, Wall);
            }
        }
    }
    result
}

struct TradeKit {
    work: FurnitureKind,
    secondary: FurnitureKind,
    storage: FurnitureKind,
}
fn trade_kit(usage: BuildingUse) -> TradeKit {
    use BuildingUse::*;
    use FurnitureKind as F;
    let (work, secondary, storage) = match usage {
        Dwelling | Inn | Rectory | Manor | ExecutionerHouse => {
            (F::DiningTable, F::Cupboard, F::StorageChest)
        }
        ParishChurch | Cathedral | Chapel | Monastery | Synagogue => {
            (F::ChurchBench, F::Lectern, F::Cupboard)
        }
        GeneralShop | MarketHall | Herbalist | Apothecary | Bookshop | Fishmonger => {
            (F::Workbench, F::Shelving, F::StorageCrate)
        }
        Smithy | Smelter | AssayHouse => (F::Workbench, F::ToolRack, F::StorageCrate),
        Weaponsmith => (F::Workbench, F::WeaponRack, F::StorageCrate),
        Armorer => (F::Workbench, F::ArmourStand, F::StorageCrate),
        Tailor | Weaver | PrintingHouse | PaperMill => {
            (F::CuttingTable, F::Shelving, F::StorageCrate)
        }
        Bakehouse => (F::KneadingTrough, F::Workbench, F::GrainBin),
        Brewery | Malthouse => (F::CaskRack, F::Workbench, F::GrainBin),
        Butcher => (F::ButchersBlock, F::Workbench, F::StorageCrate),
        Stable | Barn => (F::FeedTrough, F::HayRack, F::GrainBin),
        Granary | HorseMill | WaterMill | Windmill => (F::GrainBin, F::Workbench, F::GrainBin),
        Cooper | Carpenter | Wheelwright | Cobbler | Stonecutter | Sawmill => {
            (F::Workbench, F::ToolRack, F::StorageCrate)
        }
        Tannery | Dyer | FullingMill | Ropemaker => (F::DryingRack, F::Workbench, F::StorageCrate),
        Chandler | Potter | SaltWorks | Brickworks | Glassworks => {
            (F::Workbench, F::DryingRack, F::StorageCrate)
        }
        TimberYard | Warehouse | WoadStore => (F::Workbench, F::Shelving, F::StorageCrate),
        TownHall | WeighHouse | Guildhall | Mint | CustomsHouse => {
            (F::WritingDesk, F::Cupboard, F::StorageChest)
        }
        Hospital => (F::WardBed, F::WashStand, F::Cupboard),
        Bathhouse => (F::BathTub, F::Bench, F::StorageChest),
        School | University => (F::WritingDesk, F::Bench, F::Shelving),
        Guardhouse | Prison | Castle | Arsenal => (F::BunkBed, F::ArmourStand, F::WeaponRack),
    };
    TradeKit {
        work,
        secondary,
        storage,
    }
}
