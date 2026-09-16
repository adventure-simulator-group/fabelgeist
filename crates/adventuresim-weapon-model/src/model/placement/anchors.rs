//! Rotated attachment anchors and registration of component frames.
use super::*;
pub(super) fn attached(
    component: &mut Component,
    frames: &BTreeMap<String, Point>,
    rotations: &BTreeMap<String, Point>,
    rotation: Point,
    local: Point,
    range: [f64; 2],
    mut offset: Point,
) -> Result<Point, String> {
    let id = component.id.as_deref().unwrap();
    if let Some(stretch) = &component.stretch_between {
        if magnitude(local) > 1e-8 {
            return Err("stretched start and end must meet their declared frames".into());
        }
        let from = *frames
            .get(&stretch[0])
            .ok_or("missing stretch source frame")?;
        let to = *frames
            .get(&stretch[1])
            .ok_or("missing stretch target frame")?;
        if to[1] <= from[1] {
            return Err("stretch target must lie above source".into());
        }
        let Shape::KnuckleBow(p) = &mut component.shape else {
            return Err("stretchBetween requires knuckle bow".into());
        };
        p.length = Metres::new(to[1] - from[1])?;
        offset = add(from, local);
    } else if let Some(attachment) = &component.attach {
        let target = *frames
            .get(&attachment.to)
            .ok_or_else(|| format!("{id}: missing frame {}", attachment.to))?;
        let anchor = component
            .shape
            .attachment_anchor(attachment.at.unwrap_or_default(), range)?;
        let (owner, name) = attachment
            .to
            .rsplit_once('.')
            .ok_or("frame needs owner and name")?;
        let inward = rotate(
            [
                0.0,
                if matches!(name, "root" | "base" | "bottom") {
                    1.0
                } else {
                    -1.0
                },
                0.0,
            ],
            *rotations.get(owner).unwrap_or(&[0.0; 3]),
        );
        let overlap = attachment.overlap.map_or(0.0, Metres::get);
        if overlap < 0.0 {
            return Err("attachment overlap cannot be negative".into());
        }
        let expected = add(
            add(
                target,
                attachment.offset.map_or([0.0; 3], |p| p.map(Metres::get)),
            ),
            mul(inward, overlap),
        );
        offset = sub(expected, rotate(anchor, rotation));
    }

    Ok(offset)
}
pub(super) fn register(
    component: &Component,
    id: &str,
    range: [f64; 2],
    rotation: Point,
    offset: Point,
    frames: &mut BTreeMap<String, Point>,
    rotations: &mut BTreeMap<String, Point>,
) -> Result<(), String> {
    for (name, y) in [
        ("base", range[0]),
        ("bottom", range[0]),
        ("top", range[1]),
        ("center", (range[0] + range[1]) / 2.0),
    ] {
        frames.insert(
            format!("{id}.{name}"),
            add(rotate([0.0, y, 0.0], rotation), offset),
        );
    }
    if let Some(shield) = shields::Shield::from_shape(&component.shape) {
        let grip = add(rotate(shield.grip()?, rotation), offset);
        frames.insert(format!("{id}.grip"), grip);
        frames.insert(SHIELD_GRIP_FRAME.into(), grip);
    }
    if let Shape::LoftedBlade(p) = &component.shape {
        frames.insert(
            format!("{id}.ricasso"),
            add(rotate([0.0, p.ricasso.get(), 0.0], rotation), offset),
        );
    }
    if let Shape::Spear(p) = &component.shape
        && p.socket.is_some()
    {
        for (name, y) in [
            ("bladeBase", 0.0),
            ("tip", p.length.get()),
            (
                "socketRim",
                -p.socket.as_ref().map_or(0.0, |s| s.length.get()),
            ),
        ] {
            frames.insert(
                format!("{id}.{name}"),
                add(rotate([0.0, y, 0.0], rotation), offset),
            );
        }
    }
    frames.insert(format!("{id}.origin"), offset);
    if component.role == Some(crate::ComponentRole::Grip) {
        for (name, y) in [
            (GRIP_BASE_FRAME, range[0]),
            (GRIP_TOP_FRAME, range[1]),
            (GRIP_CENTER_FRAME, (range[0] + range[1]) / 2.0),
        ] {
            frames.insert(name.into(), add(rotate([0.0, y, 0.0], rotation), offset));
        }
    }
    rotations.insert(id.to_owned(), rotation);

    Ok(())
}
