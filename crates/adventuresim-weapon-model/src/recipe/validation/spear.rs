//! Blade/socket dimensions checked before ring allocation or shaft fitting.
use super::*;

pub(super) fn check(p: &SpearParameters) -> Checked {
    if let Some(roundness) = p.shoulder_roundness {
        proportion((0.0..=1.0).contains(&roundness.get()))?;
    }
    let Some(socket) = &p.socket else {
        return Ok(());
    };
    proportion(p.section != Some(SpearSection::Flat))?;
    for dimension in [
        socket.length,
        socket.neck_length,
        socket.base_radius,
        socket.wall,
        socket.cavity_depth,
        socket.insertion_depth,
        socket.bore_tip_radius,
    ] {
        positive(dimension.get())?;
    }
    let root = p.root_width.map_or(p.width.get() * 0.4, Metres::get) / 2.0;
    let barrel = socket.length.get() - socket.neck_length.get();
    let depth = p.thickness.get() / 2.0;
    proportion(socket.minimum_neck_axis(root, root, 0.0) > 0.0)?;
    proportion(socket.minimum_neck_axis(root, depth, -depth * 0.9 / p.length.get()) > 0.0)?;
    proportion(barrel > socket.cavity_depth.get() + socket.wall.get())?;
    clearance(socket.insertion_depth.get() + socket.wall.get() <= socket.cavity_depth.get())?;
    clearance(socket.base_radius.get() > socket.wall.get())?;
    let exterior_cap = socket.base_radius.get()
        + (root - socket.base_radius.get()) * socket.cavity_depth.get() / barrel;
    clearance(socket.bore_tip_radius.get() + socket.wall.get() <= exterior_cap)?;
    if let Some(stops) = &socket.stops {
        if let Some(blend) = stops.root_blend {
            nonnegative(blend.get())?;
            proportion(blend.get() <= stops.thickness.get().min(stops.end_height.get()))?;
            let lower = stops.center_height.get() - stops.root_height.get();
            let upper = stops.center_height.get() + stops.end_height.get() / 2.0;
            let socket_radius = socket
                .outer_barrel_radius(root, lower)
                .max(socket.outer_barrel_radius(root, upper));
            proportion(blend.get() <= stops.span.get() / 2.0 - socket_radius)?;
        }
        for dimension in [
            stops.span,
            stops.center_height,
            stops.end_height,
            stops.root_height,
            stops.thickness,
        ] {
            positive(dimension.get())?;
        }
        proportion(stops.orientation.get().abs() <= 360.0)?;
        proportion(stops.root_height.get() > stops.end_height.get() / 2.0)?;
        proportion(stops.center_height.get() > stops.root_height.get())?;
        proportion(stops.center_height.get() + stops.end_height.get() / 2.0 < barrel)?;
        proportion(stops.span.get() / 2.0 > socket.base_radius.get().max(root))?;
        proportion(stops.thickness.get() / 2.0 < socket.base_radius.get().min(root))?;
    }
    Ok(())
}

pub(super) fn shaft(p: &Shaft) -> Checked {
    if let Some(wrappings) = &p.wrappings {
        require(
            !wrappings.is_empty() && wrappings.len() <= 16,
            RecipeError::Budget,
        )?;
        for (index, wrapping) in wrappings.iter().enumerate() {
            if let Some(WrappingSection::Rounded { crest_fraction }) = &wrapping.section {
                proportion(crest_fraction.get() > 0.0 && crest_fraction.get() < 1.0)?;
            }
            if let Some(support) = wrapping.on_wrapping {
                require((support.0 as usize) < index, RecipeError::Attachment)?;
                let parent = &wrappings[support.0 as usize];
                proportion(
                    parent.pattern != WrappingPattern::Crossed
                        && wrapping.pattern != WrappingPattern::Crossed,
                )?;
                proportion(wrapping.underlay.is_none())?;
                proportion(
                    wrapping.start.get() >= parent.start.get()
                        && wrapping.start.get() + wrapping.length.get()
                            <= parent.start.get() + parent.length.get(),
                )?;
                let Some(WrappingSection::Rounded { crest_fraction }) = parent.section else {
                    return Err(RecipeError::Attachment);
                };
                proportion(wrapping_contact(parent, wrapping, crest_fraction.get()))?;
            }
            if let Some(underlay) = &wrapping.underlay {
                positive(underlay.thickness.get())?;
                proportion(underlay.thickness.get() < p.radius.get() / 2.0)?;
            }
            nonnegative(wrapping.start.get())?;
            for dimension in [
                wrapping.length,
                wrapping.pitch,
                wrapping.width,
                wrapping.thickness,
            ] {
                positive(dimension.get())?;
            }
            proportion(wrapping.width.get() < wrapping.length.get())?;
            let spacing = if wrapping.pattern == WrappingPattern::Crossed {
                4.0
            } else {
                1.0
            };
            proportion(wrapping.width.get() * spacing < wrapping.pitch.get())?;
            proportion(wrapping.phase.get().abs() <= 360.0)?;
            proportion(wrapping.thickness.get() < p.radius.get() / 2.0)?;
            proportion(
                wrapping.start.get() + wrapping.length.get()
                    <= p.length.get() - p.tenon.as_ref().map_or(0.0, |t| t.length.get()),
            )?;
            require(
                wrapping.length.get() / wrapping.pitch.get() <= 128.0,
                RecipeError::Budget,
            )?;
        }
        for (index, a) in wrappings.iter().enumerate() {
            for b in &wrappings[index + 1..] {
                let mut support = b.on_wrapping;
                let mut rests_on_a = false;
                while let Some(parent) = support {
                    if parent.0 as usize == index {
                        rests_on_a = true;
                        break;
                    }
                    support = wrappings[parent.0 as usize].on_wrapping;
                }
                proportion(
                    rests_on_a
                        || a.start.get() + a.length.get() <= b.start.get()
                        || b.start.get() + b.length.get() <= a.start.get(),
                )?;
            }
        }
    }
    if let Some(tenon) = &p.tenon {
        positive(tenon.length.get())?;
        positive(tenon.tip_radius.get())?;
        proportion(tenon.length.get() < p.length.get())?;
        proportion(
            tenon.tip_radius.get() <= p.radius.get() * p.top_scale.map_or(0.92, Ratio::get),
        )?;
    }
    Ok(())
}

/// The two helical footprints must overlap a finite portion of the lower crest.
fn wrapping_contact(parent: &ShaftWrapping, child: &ShaftWrapping, crest: f64) -> bool {
    let direction = |p: WrappingPattern| {
        if p == WrappingPattern::LeftHanded {
            -1.0
        } else {
            1.0
        }
    };
    let start = child.start.get() + child.width.get() / 2.0;
    let end = child.start.get() + child.length.get() - child.width.get() / 2.0;
    let parent_start = parent.start.get() + parent.width.get() / 2.0;
    let phase = |y: f64| {
        child.phase.get() / 360.0 + direction(child.pattern) * (y - start) / child.pitch.get()
            - parent.phase.get() / 360.0
            - direction(parent.pattern) * (y - parent_start) / parent.pitch.get()
    };
    let a = phase(start);
    let b = phase(end);
    let contact = (parent.width.get() * crest + child.width.get()) / (2.0 * parent.pitch.get());
    (a.min(b) - contact).floor() + 1.0 < a.max(b) + contact
}
