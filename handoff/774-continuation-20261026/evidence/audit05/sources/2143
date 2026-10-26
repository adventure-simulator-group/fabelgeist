//! Bounded, internally consistent paths and authored guard meshes.
use super::*;
pub(super) fn check(shape: &Shape) -> Checked {
    match shape {
        Shape::Tube(p) => tube(p)?,
        Shape::GuardAssembly(p) => guard(p)?,
        Shape::Pommel(p) => {
            if let Some(ornaments) = &p.ornaments {
                for ornament in ornaments {
                    positive(ornament.scale.get())?;
                    require(
                        p.sockets
                            .as_ref()
                            .is_some_and(|s| s.contains_key(&ornament.socket)),
                        RecipeError::Attachment,
                    )?;
                    if let OrnamentGeometry::Authored {
                        positions, indices, ..
                    } = &ornament.geometry
                    {
                        require(
                            !positions.is_empty()
                                && positions.len() % 3 == 0
                                && indices.len() % 3 == 0
                                && indices.len() <= MAX_AUTHORED_STATIONS * 3
                                && indices.iter().all(|&i| (i as usize) < positions.len() / 3),
                            RecipeError::Profile,
                        )?;
                        for p in positions {
                            bounded(p.get())?;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn tube(p: &TubeParameters) -> Checked {
    require(
        (2..=MAX_AUTHORED_STATIONS).contains(&p.points.len()),
        RecipeError::Budget,
    )?;
    for point in &p.points {
        for value in point {
            bounded(value.get())?;
        }
    }
    proportion(p.points.windows(2).all(|p| p[0] != p[1]))?;
    let points = p
        .points
        .iter()
        .map(|p| [p[0].get(), p[1].get(), 0.0])
        .collect::<Vec<_>>();
    crate::construction::validate_simple_path(&points)
        .map_err(|_| RecipeError::SelfIntersection)?;

    Ok(())
}

fn guard(p: &GuardAssemblyParameters) -> Checked {
    p.binding_order().map_err(|_| RecipeError::Attachment)?;
    require(
        !p.nodes.is_empty()
            && p.nodes.len() <= MAX_AUTHORED_STATIONS
            && !p.members.is_empty()
            && p.members.len() <= MAX_AUTHORED_STATIONS,
        RecipeError::Budget,
    )?;
    for point in p.nodes.values() {
        for value in point {
            bounded(value.get())?;
        }
    }
    require(
        p.anchor_node
            .as_ref()
            .is_some_and(|name| p.nodes.contains_key(name)),
        RecipeError::Attachment,
    )?;
    for member in &p.members {
        positive(member.section_width.get())?;
        positive(member.section_depth.get())?;
        require(
            (2..=MAX_AUTHORED_STATIONS).contains(&member.path.len()),
            RecipeError::Budget,
        )?;
        require(
            member.path.iter().all(|name| p.nodes.contains_key(name)),
            RecipeError::MissingNode,
        )?;
        require(
            member.path.windows(2).all(|pair| pair[0] != pair[1]),
            RecipeError::Attachment,
        )?;
        if let Some(n) = member.radial_segments {
            require(n.0 <= MAX_SAMPLING_REQUEST, RecipeError::Budget)?;
        }
        if let Some(n) = member.tip_scale {
            proportion(n.get() > 0.0 && n.get() <= 2.0)?;
        }
    }
    if let Some(plates) = &p.plates {
        require(plates.len() <= MAX_AUTHORED_STATIONS, RecipeError::Budget)?;
        for plate in plates {
            positive(plate.thickness.get())?;
            require(
                (3..=MAX_AUTHORED_STATIONS).contains(&plate.outline.len())
                    && plate.cutout.len() <= MAX_AUTHORED_STATIONS,
                RecipeError::Budget,
            )?;
            require(
                plate
                    .outline
                    .iter()
                    .chain(&plate.cutout)
                    .all(|name| p.nodes.contains_key(name)),
                RecipeError::Attachment,
            )?;
        }
    }
    if let Some(bindings) = &p.node_bindings {
        for (name, binding) in bindings {
            require(p.nodes.contains_key(name), RecipeError::Attachment)?;
            if let NodeBinding::Between { between, t, .. } = binding {
                require(
                    between.iter().all(|name| p.nodes.contains_key(name)),
                    RecipeError::Attachment,
                )?;
                if let Some(t) = t {
                    proportion((0.0..=1.0).contains(&t.get()))?;
                }
            }
        }
    }

    Ok(())
}
