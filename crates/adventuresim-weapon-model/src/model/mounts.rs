//! Receiving-shaft constraints for the explicit shaft mount constructions.
use super::*;

pub(super) fn socket_fit(
    component: &Component,
    shaft: Option<&Shaft>,
    parents: &[ResolvedComponent],
    offset: Point,
    rotation: Point,
) -> Result<(), String> {
    let Shape::Spear(p) = &component.shape else {
        return Ok(());
    };
    let Some(socket) = &p.socket else {
        return Ok(());
    };
    let receiver = if let Some(attachment) = &component.attach {
        let owner = attachment.to.rsplit_once('.').map(|p| p.0).unwrap_or("");
        if let Some(parent) = parents.iter().find(|p| p.id == owner) {
            if let Shape::Shaft(shaft) = &parent.component.shape {
                Some((shaft, parent.offset, parent.rotation))
            } else {
                None
            }
        } else if matches!(owner, "shaft" | "weapon") {
            shaft.map(|p| (p, [0.0; 3], [0.0; 3]))
        } else {
            None
        }
    } else if component.mount.is_some() {
        shaft.map(|p| (p, [0.0; 3], [0.0; 3]))
    } else {
        None
    };
    let Some((shaft, parent_offset, parent_rotation)) = receiver else {
        return Ok(());
    };
    let offset = inverse_rotate(sub(offset, parent_offset), parent_rotation);
    let axis = inverse_rotate(rotate([0.0, 1.0, 0.0], rotation), parent_rotation);
    if offset[0].abs() > 1e-8
        || offset[2].abs() > 1e-8
        || magnitude(sub(axis, [0.0, 1.0, 0.0])) > 1e-8
    {
        return Err("receiving socket must align with its shaft".into());
    }
    let rim = offset[1] - socket.length.get();
    if shaft
        .wrappings
        .iter()
        .flatten()
        .any(|w| w.start.get() + w.length.get() > rim)
    {
        return Err("receiving socket must seat over an unwrapped shaft tenon".into());
    }
    let penetration = shaft.length.get() - rim;
    if (penetration - socket.insertion_depth.get()).abs() > 1e-8 || rim < 0.0 {
        return Err("socket placement does not match its declared shaft insertion".into());
    }
    // Both profiles are piecewise linear: endpoints and every shaft breakpoint
    // prove clearance throughout the full inserted length, not only at the tip.
    let mut heights = vec![rim, shaft.length.get()];
    heights.extend(
        shaft
            .profile()
            .iter()
            .map(|p| p[0])
            .filter(|y| *y > rim && *y < shaft.length.get()),
    );
    // The coarsest permitted circular section supplies the conservative bore
    // apothem. Finer display levels therefore retain the same accepted fit.
    let radial = Detail::Low.radial(socket.base_radius.get(), 24).div_ceil(4) * 4;
    let apothem = (std::f64::consts::PI / radial as f64).cos();
    for height in heights {
        if shaft.radius_at(height) > socket.bore_radius(height - rim) * apothem + 1e-9 {
            return Err("socket bore cannot clear the complete receiving shaft".into());
        }
    }
    Ok(())
}

pub(super) fn check(component: &Component, shaft: &Shaft) -> Result<(), String> {
    let offset = component.offset.map_or([0.0; 3], |p| p.map(Metres::get));
    let concentric = is_axial(&component.shape);
    if concentric && (offset[0].abs() > 1e-8 || offset[2].abs() > 1e-8) {
        return Err("shaft-mounted axial construction must seat concentrically".into());
    }
    let radius = shaft.radius.get() * shaft.top_scale.map_or(0.92, Ratio::get);
    match &component.shape {
        Shape::Socket(p) if p.fit_shaft == Some(false) => {
            let wall = p.wall.map_or(0.003, Metres::get);
            if p.profile
                .iter()
                .any(|station| station[1].get() - wall < radius - 1e-8)
            {
                return Err("socket bore cannot fit shaft".into());
            }
        }
        Shape::Sleeve(p) if p.fit_shaft == Some(false) => {
            let inner = p
                .radius
                .get()
                .min(p.top_radius.map_or(p.radius.get(), Metres::get))
                - p.wall.map_or(0.003, Metres::get);
            if inner < radius - 1e-8 {
                return Err("sleeve bore cannot fit shaft".into());
            }
        }
        _ => {}
    }
    Ok(())
}

/// Reject a declared parent whose material envelope cannot meet the child.
/// Bounds are evaluated in the parent's rotated local frame, so rotating a
/// narrow guard cannot make its former width available for a lateral fitting.
fn is_axial(shape: &Shape) -> bool {
    matches!(
        shape,
        Shape::Spear(_)
            | Shape::Socket(_)
            | Shape::Sleeve(_)
            | Shape::Mace(_)
            | Shape::Partisan(_)
            | Shape::Fork(_)
            | Shape::Glaive(_)
            | Shape::Bill(_)
    )
}

pub(super) fn attachment_contact(
    child: &ResolvedComponent,
    parents: &[ResolvedComponent],
    shaft: Option<&Shaft>,
) -> Result<(), String> {
    let Some(attachment) = &child.component.attach else {
        return Ok(());
    };
    let Some((owner, _)) = attachment.to.rsplit_once('.') else {
        return Ok(());
    };
    let implicit = shaft
        .filter(|_| matches!(owner, "shaft" | "weapon"))
        .map(|shaft| {
            let mut component = child.component.clone();
            component.shape = Shape::Shaft(shaft.clone());
            ResolvedComponent {
                component,
                id: "shaft".into(),
                label: "shaft".into(),
                offset: [0.0; 3],
                rotation: [0.0; 3],
                shaft_contact: None,
                grip_width: None,
                grip_seat_radius: None,
            }
        });
    let Some(parent) = parents.iter().find(|p| p.id == owner).or(implicit.as_ref()) else {
        return Ok(());
    };
    mortised_guard::check_mating(child, parent)?;
    if implicit.is_some() && is_axial(&child.component.shape) {
        let mut receiving = child.component.clone();
        receiving.offset = Some([
            Metres::new(child.offset[0])?,
            Metres::new(0.0)?,
            Metres::new(child.offset[2])?,
        ]);
        check(&receiving, shaft.unwrap())?;
    }
    let parent_bounds = bounds(shapes::construct(parent, Detail::High)?, &|p| p);
    let child_bounds = bounds(shapes::construct(child, Detail::High)?, &|p| {
        inverse_rotate(
            sub(add(rotate(p, child.rotation), child.offset), parent.offset),
            parent.rotation,
        )
    });
    let at = attachment.at.unwrap_or_default();
    if at != AttachmentAnchor::Origin {
        let range = child.component.shape.range()?;
        let local_anchor = child.component.shape.attachment_anchor(at, range)?;
        let anchor = inverse_rotate(
            sub(
                add(rotate(local_anchor, child.rotation), child.offset),
                parent.offset,
            ),
            parent.rotation,
        );
        let seating_radius = ((child_bounds.1[0] - child_bounds.0[0])
            .min(child_bounds.1[2] - child_bounds.0[2]))
            / 2.0;
        if anchor[1] < parent_bounds.0[1] - seating_radius - 1e-8
            || anchor[1] > parent_bounds.1[1] + seating_radius + 1e-8
        {
            return Err(format!(
                "declared contact lies outside parent axial geometry {}",
                attachment.to
            ));
        }
    }
    let separated = |axis| {
        child_bounds.0[axis] > parent_bounds.1[axis] + 1e-8
            || child_bounds.1[axis] < parent_bounds.0[axis] - 1e-8
    };
    if (separated(0) || separated(1) || separated(2))
        && connected_through_assembly(child, parent, parents, implicit.as_ref())?
    {
        return Ok(());
    }
    if separated(1) {
        return Err(format!(
            "{} attachment lies outside parent axial geometry {}",
            child.id, attachment.to
        ));
    }
    if separated(0) || separated(2) {
        return Err(format!(
            "{} attachment lies outside parent footprint {}",
            child.id, attachment.to
        ));
    }
    Ok(())
}

/// A coordinate parent may be joined through another explicitly constructed
/// fitting, such as a rear beak carried by a socketed axe head. Require a
/// continuous material-envelope path instead of treating empty space as a seat.
fn connected_through_assembly(
    child: &ResolvedComponent,
    parent: &ResolvedComponent,
    components: &[ResolvedComponent],
    shaft: Option<&ResolvedComponent>,
) -> Result<bool, String> {
    let all = components.iter().chain(shaft).collect::<Vec<_>>();
    let mut envelopes = Vec::new();
    for component in &all {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for p in shapes::construct(component, Detail::High)?
            .iter()
            .flat_map(|s| s.solid.positions.iter())
        {
            let p = add(rotate(*p, component.rotation), component.offset);
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        envelopes.push((min, max));
    }
    let Some(start) = all.iter().position(|c| c.id == child.id) else {
        return Ok(false);
    };
    let Some(end) = all.iter().position(|c| c.id == parent.id) else {
        return Ok(false);
    };
    let mut visited = vec![false; all.len()];
    let mut pending = vec![start];
    visited[start] = true;
    while let Some(current) = pending.pop() {
        for next in 0..all.len() {
            if visited[next] {
                continue;
            }
            if (0..3).all(|axis| {
                envelopes[current].0[axis] <= envelopes[next].1[axis] + 1e-8
                    && envelopes[current].1[axis] >= envelopes[next].0[axis] - 1e-8
            }) {
                if next == end {
                    return Ok(true);
                }
                visited[next] = true;
                pending.push(next);
            }
        }
    }
    Ok(false)
}

fn bounds(parts: Vec<PartSource>, transform: &dyn Fn(Point) -> Point) -> (Point, Point) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in parts.iter().flat_map(|p| p.solid.positions.iter().copied()) {
        let point = transform(point);
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}
