use super::{assembly::Assembly, *};
use crate::CELL_SIZE_METRES;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};

const POST_WIDTH_METRES: f32 = 0.24;
const BEAM_DEPTH_METRES: f32 = 0.28;
const BOARD_WIDTH_METRES: f32 = 0.22;

pub(super) fn build_envelope(a: &mut Assembly<'_>, program: &BuildingProgram) -> Result<()> {
    if a.plan.kind == WorkplaceKind::HorseMill {
        super::horse_mill::build_envelope(a, program)?;
        return Ok(());
    }
    if matches!(a.plan.kind, WorkplaceKind::Dyer | WorkplaceKind::Tannery) {
        super::wet::build_envelope(a, program)?;
        return Ok(());
    }
    if a.plan.kind == WorkplaceKind::Warehouse {
        super::warehouse::build_envelope(a, program)?;
        return Ok(());
    }
    if matches!(
        a.plan.kind,
        WorkplaceKind::TimberYard | WorkplaceKind::Carpenter
    ) {
        super::craft::build_envelope(a, program)?;
        return Ok(());
    }
    let (width, depth) = program.footprint.dimensions();
    let w = f32::from(width) * CELL_SIZE_METRES;
    let d = f32::from(depth) * CELL_SIZE_METRES;
    let h = program.storey_height_metres;
    let kind = a.plan.kind;
    let timber = matches!(
        kind,
        WorkplaceKind::Barn | WorkplaceKind::Stable | WorkplaceKind::MarketHall
    );
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5, 0.08, d * 0.5))?,
        CuboidDimensions::from_metres(Vec3::new(w, 0.16, d))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    if kind == WorkplaceKind::MarketHall {
        market_frame(a, w, d, h)?;
    } else {
        let portal_width = if matches!(kind, WorkplaceKind::Barn | WorkplaceKind::Brewery) {
            3.8
        } else {
            2.4
        };
        let portal_height = if kind == WorkplaceKind::Barn {
            3.2
        } else if kind == WorkplaceKind::Brewery {
            2.8
        } else {
            2.5
        };
        for (z, outward) in [(0.0, Vec2::NEG_Y), (d, Vec2::Y)] {
            if z == d && kind != WorkplaceKind::Barn {
                a.wall(
                    ArchitecturalPlanPoint::try_from(Vec2::new(0.0, z))?,
                    ArchitecturalPlanPoint::try_from(Vec2::new(w, z))?,
                    PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                    Elevation::<crate::Architectural>::from_metres(0.0)?,
                    PositiveLength::from_metres(h)?,
                    if timber {
                        crate::workplace::assembly::WallConstruction::TimberBoards
                    } else {
                        crate::workplace::assembly::WallConstruction::Masonry
                    },
                )?;
            } else {
                let left = (w - portal_width) * 0.5;
                let right = w - left;
                a.wall(
                    ArchitecturalPlanPoint::try_from(Vec2::new(0.0, z))?,
                    ArchitecturalPlanPoint::try_from(Vec2::new(left, z))?,
                    PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                    Elevation::<crate::Architectural>::from_metres(0.0)?,
                    PositiveLength::from_metres(h)?,
                    if timber {
                        crate::workplace::assembly::WallConstruction::TimberBoards
                    } else {
                        crate::workplace::assembly::WallConstruction::Masonry
                    },
                )?;
                a.wall(
                    ArchitecturalPlanPoint::try_from(Vec2::new(right, z))?,
                    ArchitecturalPlanPoint::try_from(Vec2::new(w, z))?,
                    PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                    Elevation::<crate::Architectural>::from_metres(0.0)?,
                    PositiveLength::from_metres(h)?,
                    if timber {
                        crate::workplace::assembly::WallConstruction::TimberBoards
                    } else {
                        crate::workplace::assembly::WallConstruction::Masonry
                    },
                )?;
                if kind == WorkplaceKind::Granary && z == 0.0 {
                    a.wall(
                        ArchitecturalPlanPoint::try_from(Vec2::new(left, z))?,
                        ArchitecturalPlanPoint::try_from(Vec2::new(right, z))?,
                        PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                        Elevation::<crate::Architectural>::from_metres(portal_height)?,
                        PositiveLength::from_metres(3.5 - portal_height)?,
                        crate::workplace::assembly::WallConstruction::Masonry,
                    )?;
                    a.wall(
                        ArchitecturalPlanPoint::try_from(Vec2::new(left, z))?,
                        ArchitecturalPlanPoint::try_from(Vec2::new(right, z))?,
                        PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                        Elevation::<crate::Architectural>::from_metres(5.3)?,
                        PositiveLength::from_metres(h - 5.3)?,
                        crate::workplace::assembly::WallConstruction::Masonry,
                    )?;
                } else {
                    a.wall(
                        ArchitecturalPlanPoint::try_from(Vec2::new(left, z))?,
                        ArchitecturalPlanPoint::try_from(Vec2::new(right, z))?,
                        PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                        Elevation::<crate::Architectural>::from_metres(portal_height)?,
                        PositiveLength::from_metres(h - portal_height)?,
                        if timber {
                            crate::workplace::assembly::WallConstruction::TimberBoards
                        } else {
                            crate::workplace::assembly::WallConstruction::Masonry
                        },
                    )?;
                }
            }
        }
        if timber {
            a.wall(
                ArchitecturalPlanPoint::try_from(Vec2::ZERO)?,
                ArchitecturalPlanPoint::try_from(Vec2::new(0.0, d))?,
                PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_X)?,
                Elevation::<crate::Architectural>::from_metres(0.0)?,
                PositiveLength::from_metres(h)?,
                crate::workplace::assembly::WallConstruction::TimberBoards,
            )?;
        } else {
            ventilated_side(a, 0.0, d, h, Vec2::NEG_X)?;
        }
        if kind == WorkplaceKind::Stable {
            open_side(a, w, d, h)?;
        } else if timber {
            a.wall(
                ArchitecturalPlanPoint::try_from(Vec2::new(w, 0.0))?,
                ArchitecturalPlanPoint::try_from(Vec2::new(w, d))?,
                PlanDirection::<crate::Architectural>::from_normalized(Vec2::X)?,
                Elevation::<crate::Architectural>::from_metres(0.0)?,
                PositiveLength::from_metres(h)?,
                crate::workplace::assembly::WallConstruction::TimberBoards,
            )?;
        } else {
            ventilated_side(a, w, d, h, Vec2::X)?;
        }
        if timber {
            boarding(a, w, d, h, kind)?;
        }
        let half = portal_width * 0.5 - 0.12;
        a.passage(
            WorkplacePassagePurpose::GroundFloorCirculation,
            Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5 - half, 0.18, 0.0))?,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w * 0.5 + half,
                portal_height - 0.1,
                if kind == WorkplaceKind::Barn {
                    d
                } else {
                    d - 0.35
                },
            ))?,
        )?;
    }
    working_yard(a, w, d)?;
    let _: () = if matches!(kind, WorkplaceKind::Barn | WorkplaceKind::Stable) {
        yard_shed(a, w, d)?;
    };
    Ok(())
}

fn yard_shed(a: &mut Assembly<'_>, w: f32, d: f32) -> Result<()> {
    let west = w + 1.8;
    let east = w + 4.2;
    let front = d - 4.2;
    let back = d - 0.6;
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(west, front))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(west, back))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_X)?,
        Elevation::<crate::Architectural>::from_metres(0.0)?,
        PositiveLength::from_metres(2.2)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(east, front))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(east, back))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::X)?,
        Elevation::<crate::Architectural>::from_metres(0.0)?,
        PositiveLength::from_metres(2.2)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(west, back))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(east, back))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::Y)?,
        Elevation::<crate::Architectural>::from_metres(0.0)?,
        PositiveLength::from_metres(2.2)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(west, front))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(east, front))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
        Elevation::<crate::Architectural>::from_metres(1.95)?,
        PositiveLength::from_metres(0.25)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;

    Ok(())
}

fn market_frame(a: &mut Assembly<'_>, w: f32, d: f32, h: f32) -> Result<()> {
    // Each side is a continuous wall-plate on independent grounded posts.
    for x in [0.0, w] {
        for bay in 0..=(d / 3.0) as u32 {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    (h - BEAM_DEPTH_METRES) * 0.5,
                    bay as f32 * 3.0,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(
                    POST_WIDTH_METRES,
                    h - BEAM_DEPTH_METRES,
                    POST_WIDTH_METRES,
                ))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, 0.0))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, d))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(h - BEAM_DEPTH_METRES)?,
            PositiveLength::from_metres(BEAM_DEPTH_METRES)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    for z in [0.0, d] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(0.0, z))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(w, z))?,
            PlanDirection::<crate::Architectural>::from_normalized(if z == 0.0 {
                Vec2::NEG_Y
            } else {
                Vec2::Y
            })?,
            Elevation::<crate::Architectural>::from_metres(h - BEAM_DEPTH_METRES)?,
            PositiveLength::from_metres(BEAM_DEPTH_METRES)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    a.passage(
        WorkplacePassagePurpose::GroundFloorCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5 - 1.4, 0.18, 0.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5 + 1.4, 2.8, d))?,
    )?;

    Ok(())
}

fn open_side(a: &mut Assembly<'_>, w: f32, d: f32, h: f32) -> Result<()> {
    for bay in 0..=(d / 3.0) as u32 {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(w, 1.25, bay as f32 * 3.0))?,
            CuboidDimensions::from_metres(Vec3::new(POST_WIDTH_METRES, 2.5, POST_WIDTH_METRES))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(w, 0.0))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(w, d))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::X)?,
        Elevation::<crate::Architectural>::from_metres(2.5)?,
        PositiveLength::from_metres(h - 2.5)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;

    Ok(())
}

fn boarding(a: &mut Assembly<'_>, w: f32, d: f32, h: f32, kind: WorkplaceKind) -> Result<()> {
    // Raised board edges catch light at close range; the continuous panel remains at distance.
    for x in [0.0, w] {
        if x == w && kind == WorkplaceKind::Stable {
            continue;
        }
        for board in 0..(d / BOARD_WIDTH_METRES) as u32 {
            a.part(
                WorkplaceFeature::Boarding,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x + if x == 0.0 { -0.10 } else { 0.10 },
                    h * 0.5,
                    (board as f32 + 0.5) * BOARD_WIDTH_METRES,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.035, h, BOARD_WIDTH_METRES - 0.025))?,
                crate::workplace::WorkplacePartVisibility::DetailOnly,
            )?;
        }
    }
    let _: () = for x in [0.0, w] {
        for z in [0.0, d * 0.5, d] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, h * 0.5, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.3, h, 0.3))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    };
    Ok(())
}

fn working_yard(a: &mut Assembly<'_>, w: f32, d: f32) -> Result<()> {
    let yard = a.plan.kind.yard_width_metres();
    if yard == 0.0 {
        return Ok(());
    }
    let edge = w + yard - 0.3;
    for bay in 0..=(d / 3.0) as u32 {
        a.part(
            WorkplaceFeature::Fence,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(edge, 0.7, bay as f32 * 3.0))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, 1.4, 0.18))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for y in [0.55, 1.05] {
        // Rails bear laterally against the posts; the support ledger records that connection.
        a.part(
            WorkplaceFeature::Fence,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(edge, y, d * 0.5))?,
            CuboidDimensions::from_metres(Vec3::new(0.14, 0.12, d))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.passage(
        WorkplacePassagePurpose::OutdoorRoute,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 0.4, 0.05, 0.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 1.7, 2.3, d))?,
    )?;

    Ok(())
}

fn ventilated_side(a: &mut Assembly<'_>, x: f32, d: f32, h: f32, outward: Vec2) -> Result<()> {
    if a.plan.kind == WorkplaceKind::Malthouse {
        super::brewing::drying_wall(
            a,
            ArchitecturalPlanPoint::from_metres(Vec2::new(x, 0.0))?,
            PositiveLength::from_metres(d)?,
            PositiveLength::from_metres(h)?,
            PlanDirection::from_normalized(outward)?,
        )?;
        return Ok(());
    }
    let bays = (d / 3.0) as u32;
    let bay_length = d / bays as f32;
    let _: () = for bay in 0..bays {
        let start = bay as f32 * bay_length;
        let left = start + bay_length * 0.5 - 0.4;
        let right = left + 0.8;
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, start))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, left))?,
            PlanDirection::<crate::Architectural>::from_normalized(outward)?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            crate::workplace::assembly::WallConstruction::Masonry,
        )?;
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, right))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, start + bay_length))?,
            PlanDirection::<crate::Architectural>::from_normalized(outward)?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            crate::workplace::assembly::WallConstruction::Masonry,
        )?;
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, left))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, right))?,
            PlanDirection::<crate::Architectural>::from_normalized(outward)?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(2.0)?,
            crate::workplace::assembly::WallConstruction::Masonry,
        )?;
        if a.plan.kind == WorkplaceKind::Granary {
            a.wall(
                ArchitecturalPlanPoint::try_from(Vec2::new(x, left))?,
                ArchitecturalPlanPoint::try_from(Vec2::new(x, right))?,
                PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                Elevation::<crate::Architectural>::from_metres(2.5)?,
                PositiveLength::from_metres(2.2)?,
                crate::workplace::assembly::WallConstruction::Masonry,
            )?;
            a.wall(
                ArchitecturalPlanPoint::try_from(Vec2::new(x, left))?,
                ArchitecturalPlanPoint::try_from(Vec2::new(x, right))?,
                PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                Elevation::<crate::Architectural>::from_metres(5.2)?,
                PositiveLength::from_metres(h - 5.2)?,
                crate::workplace::assembly::WallConstruction::Masonry,
            )?;
        } else {
            a.wall(
                ArchitecturalPlanPoint::try_from(Vec2::new(x, left))?,
                ArchitecturalPlanPoint::try_from(Vec2::new(x, right))?,
                PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                Elevation::<crate::Architectural>::from_metres(2.5)?,
                PositiveLength::from_metres(h - 2.5)?,
                crate::workplace::assembly::WallConstruction::Masonry,
            )?;
        }
    };
    Ok(())
}
