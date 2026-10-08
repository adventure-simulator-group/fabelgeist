/// Recomputes an existing roof graph under its declared pivot policy.
/// Changing pitch on roofs with child, parent, inset-wall or heating attachments
/// requires a topology change and is rejected. Unchanged-pitch requests are no-ops.
/// Any error leaves the plan unchanged.
pub fn set_roof_pitch(
    plan: &mut BuildingPlan,
    id: RoofAssemblyId,
    pitch_degrees: f32,
) -> Result<(), RoofEditError> {
    if !(15.0..=75.0).contains(&pitch_degrees) {
        return Err(RoofEditError::PitchOutsideProjectRange);
    }
    let assembly_index = plan
        .roof_assemblies
        .iter()
        .position(|roof| roof.id == id)
        .ok_or(RoofEditError::MissingAssembly)?;
    let assembly = &plan.roof_assemblies[assembly_index];
    let old_pitch = assembly
        .faces
        .first()
        .map_or(pitch_degrees, |face| face.pitch_degrees);
    if (old_pitch - pitch_degrees).abs() < 0.0001 {
        return Ok(());
    }
    if !assembly.children.is_empty()
        || assembly.parent.is_some()
        || assembly
            .enclosure_faces
            .iter()
            .any(|face| !face.inset_walls.is_empty())
        || plan
            .domestic_heating
            .as_ref()
            .is_some_and(|h| h.roof.roof == id)
    {
        return Err(RoofEditError::TopologyEvent);
    }
    let old_tan = old_pitch.to_radians().tan();
    if old_tan.abs() <= 0.0001 {
        return Err(RoofEditError::TopologyEvent);
    }
    let factor = pitch_degrees.to_radians().tan() / old_tan;
    let min_y = assembly
        .faces
        .iter()
        .flat_map(|face| face.polygon.iter().map(|point| point.y))
        .fold(f32::INFINITY, f32::min);
    let max_y = assembly
        .faces
        .iter()
        .flat_map(|face| face.polygon.iter().map(|point| point.y))
        .fold(f32::NEG_INFINITY, f32::max);
    let pivot_policy = assembly.pivot_policy;
    let scale_y = |y: f32| match pivot_policy {
        RoofPivotPolicy::KeepEave | RoofPivotPolicy::KeepChildAttachment => {
            min_y + (y - min_y) * factor
        }
        RoofPivotPolicy::KeepRidge => max_y - (max_y - y) * factor,
    };
    let mut reconstructed = assembly.clone();
    let mut geometry = plan.resolved_geometry.clone();
    reconstruct_roof_pitch(
        &mut reconstructed,
        &mut geometry,
        plan,
        pitch_degrees,
        min_y,
        scale_y,
    )
    .map_err(|cause| RoofEditError::Construction {
        roof: id,
        cause: Box::new(cause),
    })?;
    plan.roof_assemblies[assembly_index] = reconstructed;
    plan.resolved_geometry = geometry;
    Ok(())
}

fn reconstruct_roof_pitch(
    assembly: &mut RoofAssembly,
    geometry: &mut ResolvedGeometry,
    plan: &BuildingPlan,
    pitch_degrees: f32,
    min_y: f32,
    scale_y: impl Fn(f32) -> f32,
) -> Result<(), GenerationError> {
    for face in &mut assembly.faces {
        for point in &mut face.polygon {
            point.y = scale_y(point.y);
        }
        let bounds = roof_polygon_bounds(face.id, &face.polygon)?;
        face.plane = roof_plane(&face.polygon);
        face.pitch_degrees = pitch_degrees;
        if let Some(surface) = geometry
            .surfaces
            .iter_mut()
            .find(|surface| surface.id == face.drainage_catchment)
        {
            surface.bounds = bounds;
        }
        if let Some(catchment) = geometry
            .drainage_catchments
            .iter_mut()
            .find(|catchment| catchment.id == face.drainage_catchment)
        {
            let centre = face.polygon.iter().copied().sum::<Vec3>() / face.polygon.len() as f32;
            let low = face
                .polygon
                .iter()
                .min_by(|a, b| a.y.total_cmp(&b.y))
                .copied()
                .ok_or(GenerationError::EmptyRoofFace { face: face.id })?;
            catchment.centre = centre;
            catchment.inner_elevation_metres = face
                .polygon
                .iter()
                .map(|point| point.y)
                .fold(f32::NEG_INFINITY, f32::max);
            catchment.outer_elevation_metres = low.y;
            if let Some(route) = geometry
                .drainage_routes
                .iter_mut()
                .find(|route| route.id == catchment.outlet_route)
            {
                route.inlet = centre;
                route.outlet = low;
            }
        }
    }
    gable_enclosure::update_pitch(
        &mut assembly.enclosure_faces,
        &assembly.faces,
        assembly
            .source_piece_index
            .filter(|_| assembly.kind == RoofKind::Gable)
            .map(|index| plan.roofs[index]),
        &plan.wall_assemblies,
        min_y,
        &scale_y,
    );
    for edge in &mut assembly.edges {
        edge.start.y = scale_y(edge.start.y);
        edge.end.y = scale_y(edge.end.y);
    }
    refresh_edge_weathering(assembly, geometry)?;
    Ok(())
}

fn refresh_edge_weathering(
    assembly: &RoofAssembly,
    geometry: &mut ResolvedGeometry,
) -> Result<(), GenerationError> {
    for (edge_index, edge) in assembly.edges.iter().enumerate() {
        let delta = edge.end - edge.start;
        let plan_length = Vec2::new(delta.x, delta.z).length().max(0.05);
        let edge_pitch = delta.y.atan2(plan_length);
        let weather_id =
            ResolvedItemId((0x8_u64 << 60) | (assembly.id.0 << 16) | 0x5000 | edge_index as u64);
        if let Some(solid) = geometry
            .solids
            .iter_mut()
            .find(|solid| solid.id == weather_id)
        {
            let treated_plan_length = if edge.kind == RoofEdgeKind::Eave {
                (plan_length - 0.36_f32.min(plan_length * 0.5)).max(0.05)
            } else {
                plan_length
            };
            solid.centre = Position::<Architectural>::from_metres(
                (edge.start + edge.end) * 0.5
                    + if edge.kind == RoofEdgeKind::Eave {
                        Vec3::NEG_Y * 0.06
                    } else {
                        Vec3::Y * 0.035
                    },
            )?;
            {
                let mut native_geometry = solid.size.metres();
                native_geometry.x = if edge.kind == RoofEdgeKind::Eave {
                    treated_plan_length
                } else {
                    treated_plan_length / edge_pitch.cos().abs().max(0.01)
                };
                solid.size = CuboidDimensions::from_metres(native_geometry).map_err(|cause| {
                    crate::CollisionError {
                        source_id: solid.id,
                        cause,
                    }
                })?;
            };
            solid.yaw_radians = Radians::new(delta.z.atan2(delta.x))?;
            solid.longfall_radians = Radians::new(if edge.kind == RoofEdgeKind::Eave {
                0.012
            } else {
                edge_pitch
            })?;
        }
        if let Some(flashing_id) = edge.flashing
            && let Some(flashing) = geometry
                .solids
                .iter_mut()
                .find(|solid| solid.id == flashing_id)
        {
            flashing.centre = Position::<Architectural>::from_metres(
                (edge.start + edge.end) * 0.5 + Vec3::Y * (flashing.size.metres().y * 0.5),
            )?;
            {
                let mut native_geometry = flashing.size.metres();
                native_geometry.x = delta.length().max(0.05);
                flashing.size =
                    CuboidDimensions::from_metres(native_geometry).map_err(|cause| {
                        crate::CollisionError {
                            source_id: flashing.id,
                            cause,
                        }
                    })?;
            };
            flashing.yaw_radians = Radians::new(delta.z.atan2(delta.x))?;
            flashing.longfall_radians = Radians::new(if edge.kind == RoofEdgeKind::Valley {
                edge_pitch
            } else {
                0.0
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod roof_edit_transaction_tests {
    use super::*;
    use crate::spatial_geometry::{CoordinateAxis, GeometryError, GeometryRole};

    fn editable_plan() -> BuildingPlan {
        generate(&BuildingProgram::fixture(
            BuildingArchetype::CastleGatehouse,
            fabelgeist_determinism::Seed::from_u64(42),
        ))
        .unwrap()
    }

    fn editable_roof(plan: &BuildingPlan) -> usize {
        plan.roof_assemblies
            .iter()
            .position(|roof| {
                roof.children.is_empty() && roof.parent.is_none() && roof.faces.len() >= 2
            })
            .unwrap()
    }

    #[test]
    fn roof_edit_rejects_a_later_invalid_face_without_changing_the_plan() {
        let mut plan = editable_plan();
        let index = editable_roof(&plan);
        let roof = &mut plan.roof_assemblies[index];
        let id = roof.id;
        let face = &mut roof.faces[1];
        let face_id = face.id;
        face.polygon.truncate(2);
        let before = postcard::to_allocvec(&plan).unwrap();

        let error = set_roof_pitch(&mut plan, id, 75.0).unwrap_err();

        assert_eq!(
            error,
            RoofEditError::Construction {
                roof: id,
                cause: Box::new(GenerationError::InvalidRoofFace {
                    face: face_id,
                    vertices: 2,
                }),
            }
        );
        assert_eq!(postcard::to_allocvec(&plan).unwrap(), before);
    }

    #[test]
    fn roof_edit_rejects_a_later_face_overflow_without_changing_the_plan() {
        let mut plan = editable_plan();
        let index = editable_roof(&plan);
        let roof = &mut plan.roof_assemblies[index];
        let id = roof.id;
        roof.pivot_policy = RoofPivotPolicy::KeepEave;
        let face = &mut roof.faces[1];
        let face_id = face.id;
        face.polygon[1].y = f32::MAX;
        let before = postcard::to_allocvec(&plan).unwrap();

        let error = set_roof_pitch(&mut plan, id, 75.0).unwrap_err();

        assert_eq!(
            error,
            RoofEditError::Construction {
                roof: id,
                cause: Box::new(GenerationError::RoofFaceConstruction {
                    face: face_id,
                    cause: GeometryError::NonFinite {
                        role: GeometryRole::Position,
                        axis: CoordinateAxis::Y,
                    },
                }),
            }
        );
        assert_eq!(postcard::to_allocvec(&plan).unwrap(), before);
    }

    #[test]
    fn roof_edit_rejects_weathering_overflow_without_changing_the_plan() {
        let mut plan = editable_plan();
        let index = editable_roof(&plan);
        let weathering = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| {
                matches!(
                    solid.role,
                    SolidRole::RoofEdgeTreatment | SolidRole::RoofGutter
                )
            })
            .unwrap()
            .id;
        let roof = &mut plan.roof_assemblies[index];
        let id = roof.id;
        let edge = roof
            .edges
            .iter_mut()
            .find(|edge| edge.kind == RoofEdgeKind::Eave)
            .unwrap();
        // Bind a weathering solid to exercise rejection after face and edge updates.
        edge.flashing = Some(weathering);
        edge.start.x = f32::MAX;
        edge.end.x = f32::MAX;
        let before = postcard::to_allocvec(&plan).unwrap();

        let error = set_roof_pitch(&mut plan, id, 75.0).unwrap_err();

        assert_eq!(
            error,
            RoofEditError::Construction {
                roof: id,
                cause: Box::new(GenerationError::Geometry(GeometryError::NonFinite {
                    role: GeometryRole::Position,
                    axis: CoordinateAxis::X,
                })),
            }
        );
        assert_eq!(postcard::to_allocvec(&plan).unwrap(), before);
    }
}
