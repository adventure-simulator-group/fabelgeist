//! Fixed light set-outs follow the structural nave, choir, transept and apse axes.
use super::*;
pub(super) struct ChurchWindowTarget {
    pub source: crate::WallSourceId,
    pub profile: ChurchWindowProfile,
}
pub(super) fn schedule(church_program: crate::ChurchProgram) -> Vec<ChurchWindowTarget> {
    // One centered light per structural bay keeps the opening hierarchy tied
    // to the buttress/pier rhythm.  Transept end windows are deliberately
    // larger; the apse uses narrower radial lights.  Rich tracery is outside
    // the MVP, but every opening is already a real two-light stone assembly.
    let mut window_targets = Vec::new();
    for bay_index in 0..church_program.nave_bays {
        for side in [Direction::South, Direction::North] {
            window_targets.push(ChurchWindowTarget {
                source: crate::WallSourceId::ChurchExterior {
                    range: crate::ChurchRange::Nave,
                    side,
                    bay: bay_index,
                },
                profile: ChurchWindowProfile {
                    sill_metres: 1.70,
                    width_metres: 1.45,
                    spring_height_metres: 2.45,
                    apex_height_metres: 4.35,
                },
            });
            window_targets.push(ChurchWindowTarget {
                source: crate::WallSourceId::ChurchArcade {
                    side,
                    bay: bay_index,
                },
                profile: ChurchWindowProfile {
                    // Clear the 8.635 m aisle-roof abutment and its upstand.
                    sill_metres: 9.10,
                    width_metres: 1.35,
                    spring_height_metres: 1.05,
                    apex_height_metres: 2.30,
                },
            });
        }
    }
    for bay_index in 0..church_program.choir_bays {
        for side in [Direction::South, Direction::North] {
            window_targets.push(ChurchWindowTarget {
                source: crate::WallSourceId::ChurchExterior {
                    range: crate::ChurchRange::Choir,
                    side,
                    bay: bay_index,
                },
                profile: ChurchWindowProfile {
                    sill_metres: 2.15,
                    width_metres: 1.60,
                    spring_height_metres: 3.55,
                    apex_height_metres: 6.15,
                },
            });
        }
    }
    for side in [Direction::South, Direction::North] {
        window_targets.push(ChurchWindowTarget {
            source: crate::WallSourceId::ChurchExterior {
                range: crate::ChurchRange::Transept,
                side,
                bay: 0,
            },
            profile: ChurchWindowProfile {
                sill_metres: 1.75,
                width_metres: 2.35,
                spring_height_metres: 4.35,
                apex_height_metres: 7.65,
            },
        });
    }
    for facet in [0_u8, 1, 3, 4] {
        window_targets.push(ChurchWindowTarget {
            source: crate::WallSourceId::ChurchApse { facet },
            profile: ChurchWindowProfile {
                sill_metres: 2.20,
                width_metres: 1.30,
                spring_height_metres: 3.55,
                apex_height_metres: 5.95,
            },
        });
    }
    window_targets
}
