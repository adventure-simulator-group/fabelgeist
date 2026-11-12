fn refit_roof_edge_treatments(
    assemblies: &mut [RoofAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), crate::GenerationError> {
    // Tower/child clipping can shorten a verge after its treatment was first
    // resolved. Refit the authoritative treatment to the final typed edge;
    // retaining the pre-cut bar would create a detached rod across the cut.
    let mut orphan_treatments = HashSet::new();
    for assembly in assemblies {
        for treatment in geometry.solids.iter_mut().filter(|solid| {
            solid.owner == assembly.owner && solid.role == SolidRole::RoofEdgeTreatment
        }) {
            let pitch_cosine = treatment.longfall_radians.radians().cos();
            let axis = Vec3::new(
                treatment.yaw_radians.radians().cos() * pitch_cosine,
                treatment.longfall_radians.radians().sin(),
                treatment.yaw_radians.radians().sin() * pitch_cosine,
            );
            let endpoints = [
                treatment.centre.metres() - axis * treatment.size.metres().x * 0.5,
                treatment.centre.metres() + axis * treatment.size.metres().x * 0.5,
            ];
            let aligned = assembly.edges.iter().any(|edge| {
                if !matches!(
                    edge.kind,
                    RoofEdgeKind::Ridge | RoofEdgeKind::Hip | RoofEdgeKind::GableVerge
                ) {
                    return false;
                }
                let delta = edge.end - edge.start;
                let length_squared = delta.length_squared().max(0.000_001);
                treatment.size.metres().x <= delta.length() + 0.03
                    && endpoints.iter().all(|point| {
                        let raw_t = (*point - edge.start).dot(delta) / length_squared;
                        let t = raw_t.clamp(0.0, 1.0);
                        point.distance(edge.start + delta * t) <= 0.075
                            && (-0.02..=1.02).contains(&raw_t)
                    })
            });
            if !aligned {
                orphan_treatments.insert(treatment.id);
            }
        }
        for edge in &mut assembly.edges {
            if edge
                .flashing
                .is_some_and(|id| orphan_treatments.contains(&id))
            {
                edge.flashing = None;
            }
        }
    }
    geometry
        .solids
        .retain(|solid| !orphan_treatments.contains(&solid.id));
    geometry.support_interfaces.retain(|interface| {
        !orphan_treatments.iter().any(|id| {
            interface.id == ResolvedItemId((0x9_u64 << 60) | (id.0 & 0x0FFF_FFFF_FFFF_FFFF))
        })
    });
    let _: () = for treatment in geometry
        .solids
        .iter()
        .filter(|solid| solid.role == SolidRole::RoofEdgeTreatment)
    {
        let interface_id =
            ResolvedItemId((0x9_u64 << 60) | (treatment.id.0 & 0x0FFF_FFFF_FFFF_FFFF));
        if let Some(interface) = geometry
            .support_interfaces
            .iter_mut()
            .find(|interface| interface.id == interface_id)
        {
            interface.bounds = SpatialBounds::<Architectural>::from_metres(
                treatment.centre.metres() - Vec3::new(0.08, 0.025, 0.08),
                treatment.centre.metres() + Vec3::new(0.08, 0.025, 0.08),
            )?;
        }
    };
    Ok(())
}

fn bind_roof_junctions(
    assemblies: &[RoofAssembly],
    geometry: &mut ResolvedGeometry,
) -> Result<(), crate::GenerationError> {
    let roof_owners = assemblies
        .iter()
        .map(|roof| roof.owner)
        .collect::<HashSet<_>>();
    let mut roof_bonds = Vec::new();
    for left in 0..geometry.solids.len() {
        for right in left + 1..geometry.solids.len() {
            let a = &geometry.solids[left];
            let b = &geometry.solids[right];
            if a.owner == b.owner
                || (!roof_owners.contains(&a.owner) && !roof_owners.contains(&b.owner))
            {
                continue;
            }
            let a_bounds = yaw_bounds(a)?;
            let b_bounds = yaw_bounds(b)?;
            let min = a_bounds.min().metres().max(b_bounds.min().metres());
            let max = a_bounds.max().metres().min(b_bounds.max().metres());
            let overlap = max - min;
            if overlap.min_element() > 0.001 {
                roof_bonds.push(JunctionBond {
                    id: ResolvedItemId((0x6_u64 << 60) | roof_bonds.len() as u64),
                    owners: [a.owner, b.owner],
                    bounds: SpatialBounds::<Architectural>::from_metres(
                        min - Vec3::splat(0.01),
                        max + Vec3::splat(0.01),
                    )?,
                    minimum_interface_area_square_metres: 0.005,
                    maximum_penetration_metres: overlap.x.min(overlap.z).min(0.18),
                });
            }
        }
    }
    geometry.junction_bonds.extend(roof_bonds);

    Ok(())
}

fn yaw_bounds(
    solid: &ResolvedSolid,
) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
    let cosine = solid.yaw_radians.radians().cos().abs();
    let sine = solid.yaw_radians.radians().sin().abs();
    let half = Vec3::new(
        (solid.size.metres().x * cosine + solid.size.metres().z * sine) * 0.5,
        solid.size.metres().y * 0.5,
        (solid.size.metres().x * sine + solid.size.metres().z * cosine) * 0.5,
    );
    Ok(SpatialBounds::<Architectural>::from_metres(
        solid.centre.metres() - half,
        solid.centre.metres() + half,
    )?)
}
