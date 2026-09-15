//! Hand clearance and working-section proportions of melee components.
use super::*;
pub(super) fn check(shape: &Shape) -> Checked {
    match shape {
        Shape::DiamondBlade(p) => {
            if let Some(taper) = p.taper {
                proportion((0.0..1.0).contains(&taper.get()))?;
            }
        }
        Shape::Grip(p) => {
            let scale = p
                .bottom_scale
                .map_or(1.0, Ratio::get)
                .max(p.top_scale.map_or(1.0, Ratio::get));
            require(
                p.radius.get() * scale <= MAX_ROUND_GRIP_RADIUS + 1e-9,
                RecipeError::Grip,
            )?;
        }
        Shape::OvalGrip(p) => oval_grip(p)?,
        Shape::Pommel(p) => pommel(p)?,
        Shape::Socket(p) => profile(&p.profile, ProfileEnds::Open)?,
        Shape::Guard(p) => {
            if let Some(n) = p.tip_scale {
                proportion((0.45..=1.5).contains(&n.get()))?;
            }
            if let Some(n) = p.terminal_swell {
                proportion((0.0..=1.0).contains(&n.get()))?;
            }
        }
        Shape::Axe(p) => {
            if let Some(root) = p.root_width {
                proportion(root.get() < p.width.get() * 0.55)?;
            }
            if let (Some(upper), Some(lower)) = (p.upper_shoulder, p.lower_shoulder) {
                proportion(upper.get() > lower.get())?;
            }
        }
        Shape::Shaft(p) => super::spear::shaft(p)?,
        Shape::Spear(p) => {
            super::spear::check(p)?;
            if let Some(n) = p.belly_position {
                proportion(n.get() > 0.0 && n.get() < 1.0)?;
            }
            if let Some(root) = p.root_width {
                proportion(root.get() < p.width.get())?;
            }
            if let Some(n) = p.acuteness {
                positive(n.get())?;
            }
        }
        Shape::Hammer(p) => {
            if let Some(n) = p.neck_ratio {
                proportion(n.get() > 0.0 && n.get() <= 0.88)?;
            }
        }
        Shape::Beak(p) => {
            if let Some(n) = p.bend_position {
                proportion(n.get() > 0.0 && n.get() < 1.0)?;
            }
            if let (Some(tip), Some(root)) = (p.tip_section, p.root_section) {
                proportion(tip.get() < root.get())?;
            }
        }
        Shape::FacetedBeak(p) => {
            if let Some(n) = p.bend_position {
                proportion(n.get() > 0.0 && n.get() < 1.0)?;
            }
            if let Some(tip) = p.tip {
                proportion(tip.get() < p.root.get())?;
            }
        }
        Shape::Fork(p) => {
            if let Some(crotch) = p.crotch {
                proportion(
                    crotch.get() > 0.0
                        && crotch.get() + p.crotch_round.map_or(0.0, Ratio::get) < 0.8,
                )?;
            }
            if let Some(tine) = p.tine_width {
                proportion(tine.get() * 2.0 < p.width.get())?;
            }
        }
        Shape::Partisan(p) => {
            if let Some(root) = p.root_width {
                proportion(root.get() < p.width.get())?;
            }
        }
        Shape::Mace(p) => mace(p)?,
        _ => {}
    }
    Ok(())
}

fn pommel(p: &PommelParameters) -> Checked {
    if let Some(points) = &p.profile {
        profile(points, ProfileEnds::Poles)?;
    }
    for value in [p.width_scale, p.length_scale].into_iter().flatten() {
        proportion((0.5..=2.0).contains(&value.get()))?;
    }
    if let Some(n) = p.facets {
        proportion((4..=24).contains(&n.0))?;
    }
    if let Some(n) = p.flute_count {
        proportion((3..=24).contains(&n.0))?;
    }
    for (value, min, max) in [
        (p.flute_depth, 0.0, 0.3),
        (p.face_convexity, 0.0, 0.5),
        (p.rim_bevel, 0.01, 0.45),
        (p.notch_depth, 0.05, 0.5),
        (p.lobe_spread, 0.5, 1.0),
    ] {
        if let Some(value) = value {
            proportion((min..=max).contains(&value.get()))?;
        }
    }
    if let Some(twist) = p.twist {
        proportion((-180.0..=180.0).contains(&twist.get()))?;
    }

    Ok(())
}

fn mace(p: &MaceParameters) -> Checked {
    if let Some(scale) = p.flange_root_scale {
        proportion(scale.get() > 0.0 && scale.get() <= 1.0)?;
    }
    if let Some(points) = &p.core_profile {
        profile(points, ProfileEnds::Poles)?;
        proportion(
            points[0][0].get() == -p.length.get() / 2.0
                && points.last().unwrap()[0].get()
                    == p.length.get() / 2.0 + p.crown_length.map_or(0.0, Metres::get),
        )?;
    }
    positive(p.root_radius.get())?;
    positive(p.shoulder_radius.get())?;
    proportion(p.cusp_radius.get() > p.root_radius.get().max(p.shoulder_radius.get()))?;
    if let Some(n) = p.cusp_height {
        proportion(n.get() > 0.0 && n.get() < 1.0)?;
    }
    proportion(p.flanges.0 >= 3)?;

    Ok(())
}

fn oval_grip(p: &OvalGripParameters) -> Checked {
    let scale = p
        .bottom_scale
        .map_or(1.0, Ratio::get)
        .max(p.top_scale.map_or(1.0, Ratio::get));
    require(
        p.width.get() * scale <= MAX_SWORD_GRIP_WIDTH + 1e-9
            && p.thickness.get() * scale <= MAX_SWORD_GRIP_THICKNESS + 1e-9
            && p.width.get() > p.thickness.get(),
        RecipeError::Grip,
    )?;

    Ok(())
}
