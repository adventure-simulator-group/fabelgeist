//! A property translation carries every owned physical member and its street hook.
use super::*;

type YardKey = [(u32, u32); 4];

pub(super) fn apply(
    layout: &mut CompiledCityLayout,
    context: &CityPackingContext,
    translations: &BTreeMap<CityPropertyId, PlanDisplacement>,
) -> CityCompileResult<BTreeMap<crate::scene_input::SceneBuildingId, PlanDisplacement>> {
    let mut members: BTreeMap<_, _> = context
        .frontages
        .keys()
        .map(|&owner| (crate::scene_input::SceneBuildingId(owner.0), owner))
        .collect();
    let mut yards: BTreeMap<_, _> = context
        .frontages
        .iter()
        .map(|(&owner, frontage)| {
            Ok((
                yard_key(plots::corners(plots::reservation(frontage.lot)?)),
                owner,
            ))
        })
        .collect::<CityCompileResult<_>>()?;
    for compound in &layout.compounds {
        members.insert(compound.rear_building_id, compound.id);
    }
    for garden in &layout.gardens {
        yards.extend(
            garden
                .beds
                .iter()
                .map(|bed| (yard_key(bed.corners()), garden.owner)),
        );
    }
    let building_translations: BTreeMap<_, _> = members
        .into_iter()
        .map(|(member, owner)| (member, translations[&owner]))
        .collect();
    for building in &mut layout.buildings {
        building.centre_metres = building
            .centre_metres
            .translated(building_translations[&building.id])?;
    }
    for property in &mut layout.single_properties {
        property.plot = property.plot.translated(translations[&property.id])?;
    }
    for compound in &mut layout.compounds {
        let delta = translations[&compound.id];
        compound.plot = compound.plot.translated(delta)?;
        compound.court = compound.court.translated(delta)?;
        compound.boundary.gate.centre_metres =
            compound.boundary.gate.centre_metres.translated(delta)?;
        for wall in &mut compound.boundary.walls {
            wall.start_metres = wall.start_metres.translated(delta)?;
            wall.end_metres = wall.end_metres.translated(delta)?;
        }
        compound::translate_property_access(
            &mut compound.access,
            delta,
            context.frontages[&compound.id].tangent(),
        )?;
    }
    for garden in &mut layout.gardens {
        garden.translate(
            translations[&garden.owner],
            context.frontages[&garden.owner].tangent(),
        )?;
    }
    for yard in &mut layout.yards {
        if let Some(owner) = yards.get(&yard_key(
            yard.corners_metres
                .map(crate::scene_coordinates::ScenePlanPoint::metres),
        )) {
            for point in &mut yard.corners_metres {
                *point = point.translated(translations[owner])?;
            }
        }
    }
    Ok(building_translations)
}

pub(super) fn validate_gardens(
    layout: &CompiledCityLayout,
    envelopes: &[MeasuredBuildingEnvelope],
    building_translations: &BTreeMap<crate::scene_input::SceneBuildingId, PlanDisplacement>,
) -> CityCompileResult<()> {
    for garden in &layout.gardens {
        garden
            .validate_geometry(&layout.streets)
            .map_err(|issue| CityCompileError::Packing {
                property: garden.owner,
                issue: CityPackingIssue::Garden { issue },
            })?;
        for MeasuredBuildingEnvelope {
            building,
            body: envelope,
            ..
        } in envelopes
        {
            let (building, envelope) = (*building, *envelope);
            let moved = envelope.translated(building_translations[&building])?;
            if !garden.clears_building(moved) {
                return Err(CityCompileError::Packing {
                    property: garden.owner,
                    issue: CityPackingIssue::GardenBlocked { building },
                });
            }
        }
    }
    for (index, first) in envelopes.iter().enumerate() {
        let moved = |envelope: &MeasuredBuildingEnvelope| {
            envelope
                .body
                .translated(building_translations[&envelope.building])
        };
        for second in &envelopes[index + 1..] {
            if moved(first)?.intersects(moved(second)?) {
                // Members of one compound were validated during its assembly.
                // This check also covers roof projections across block boundaries.
                let same_property = layout.compounds.iter().any(|p| {
                    [p.front_building_id, p.rear_building_id].contains(&first.building)
                        && [p.front_building_id, p.rear_building_id].contains(&second.building)
                });
                if !same_property {
                    return Err(CityCompileError::Packing {
                        property: layout
                            .compounds
                            .iter()
                            .find(|p| {
                                [p.front_building_id, p.rear_building_id].contains(&first.building)
                            })
                            .map_or(CityPropertyId(first.building.0), |p| p.id),
                        issue: CityPackingIssue::BuildingOverlap {
                            first: first.building,
                            second: second.building,
                        },
                    });
                }
            }
        }
    }
    Ok(())
}
fn yard_key(corners: [Vec2; 4]) -> YardKey {
    corners.map(|point| (point.x.to_bits(), point.y.to_bits()))
}
