//! Receiving shafts and shaped grip seats.
use super::*;
pub(super) fn mounted(
    component: &mut Component,
    shaft: Option<&Shaft>,
    frames: &BTreeMap<String, Point>,
    rotation: Point,
    local: Point,
    range: [f64; 2],
) -> Result<(Point, Option<f64>), String> {
    let mut offset = local;
    let mut shaft_contact = None;
    if let Some(mount) = component.mount {
        match mount {
            Mount::ComponentEnd => {
                let anchor = component
                    .anchor
                    .as_ref()
                    .ok_or("component-end needs anchor")?;
                component.attach = Some(Attachment {
                    to: format!("{anchor}.top"),
                    at: Some(AttachmentAnchor::Center),
                    offset: None,
                    overlap: None,
                });
            }
            _ => {
                mounts::check(component, shaft.ok_or("shaft-top mount requires shaft")?)?;
                let top = *frames
                    .get(SHAFT_TOP_FRAME)
                    .ok_or("shaft-top mount requires shaft")?;
                offset = add(top, local);
                if let Shape::Spear(p) = &component.shape
                    && let Some(socket) = &p.socket
                {
                    if mount != Mount::ShaftTop {
                        return Err("socketed spear uses shaft-top receiving mount".into());
                    }
                    offset[1] += socket.length.get() - socket.insertion_depth.get();
                }
                match mount {
                    Mount::ShaftTopCentered | Mount::ShaftTopSleeve => {
                        let contact = if mount == Mount::ShaftTopCentered {
                            range[0]
                        } else {
                            range[1]
                        };
                        offset = sub(offset, rotate([0.0, contact, 0.0], rotation));
                        offset[1] -= component.insertion.map_or(0.012, Metres::get);
                    }
                    _ => {}
                }
                let shaft = shaft.ok_or("shaft-top mount requires shaft")?;
                let radius = shaft.radius.get() * shaft.top_scale.map_or(0.92, Ratio::get);
                match &mut component.shape {
                    Shape::Socket(p) if p.fit_shaft != Some(false) => {
                        let wall = p.wall.map_or(0.003, Metres::get);
                        for point in &mut p.profile {
                            point[1] = Metres::new(radius + wall)?;
                        }
                        shaft_contact = Some(radius);
                    }
                    Shape::Sleeve(p) if p.fit_shaft != Some(false) => {
                        let wall = p.wall.map_or(0.003, Metres::get);
                        p.radius = Metres::new(radius + wall)?;
                        p.top_radius = Some(p.radius);
                        shaft_contact = Some(radius);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok((offset, shaft_contact))
}
pub(super) fn seating(
    component: &Component,
    components: &[ResolvedComponent],
    offset: Point,
    rotation: Point,
) -> Option<f64> {
    if let Shape::OvalGrip(p) = &component.shape {
        let base = p.width.get() * p.bottom_scale.map_or(1.0, Ratio::get) / 2.0;
        let parent = components.iter().find(|parent| {
            component
                .attach
                .as_ref()
                .is_some_and(|a| a.to == format!("{}.top", parent.id))
        });
        parent.and_then(|parent| {
            if let Shape::Pommel(pommel) = &parent.component.shape {
                if parent.rotation != [0.0; 3] || rotation != [0.0; 3] {
                    return None;
                }
                if pommel.construction == PommelConstruction::Lathed {
                    let profile = pommel.profile.as_ref()?;
                    let y = offset[1] - parent.offset[1];
                    let upper = profile
                        .iter()
                        .position(|point| point[0].get() >= y)
                        .unwrap_or(0);
                    let a = profile[upper.saturating_sub(1)];
                    let b = profile[upper];
                    let t = if a[0] == b[0] {
                        0.0
                    } else {
                        ((y - a[0].get()) / (b[0].get() - a[0].get())).clamp(0.0, 1.0)
                    };
                    Some(base.min(a[1].get() + (b[1].get() - a[1].get()) * t))
                } else {
                    let plate = pommel.construction == PommelConstruction::Plate
                        || pommel.base_construction == Some(PommelBaseConstruction::Plate);
                    let neck = if pommel.outline_style == Some(PommelOutlineStyle::Fan) {
                        0.36
                    } else {
                        pommel.shoulder_width.map_or(0.42, Ratio::get)
                    };
                    Some(
                        base.min(if plate {
                            0.009
                        } else {
                            pommel.diameter.map_or(0.055, Metres::get)
                                * neck
                                * pommel.width_scale.map_or(1.0, Ratio::get)
                                / 2.0
                        })
                        .min(
                            pommel.thickness.map_or(0.018, Metres::get) * 0.48 * p.width.get()
                                / p.thickness.get(),
                        ),
                    )
                }
            } else {
                None
            }
        })
    } else {
        None
    }
}
