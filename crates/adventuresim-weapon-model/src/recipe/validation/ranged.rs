//! Mechanical clearances for bows, projectiles, locks, and carriers.
use super::*;
pub(super) fn check(shape: &Shape) -> Checked {
    match shape {
        Shape::ArcheryBow(p) => bow(p)?,
        Shape::Arrow(p) => {
            proportion(
                p.fletching_length.get() + p.nock_length.get() < p.length.get() * 0.45
                    && p.head_width.get() > p.shaft_radius.get() * 2.0,
            )?;
            clearance(
                p.nock_slot_width.get() > 0.0
                    && p.nock_slot_width.get() < p.shaft_radius.get() * 1.5,
            )?;
            clearance(
                p.maximum_string_radius.get() > 0.0
                    && p.nock_clearance.get() > 0.0
                    && p.nock_slot_width.get() + 1e-9
                        >= p.maximum_string_radius.get() * 2.0 + p.nock_clearance.get(),
            )?;
            proportion((2..=6).contains(&p.fletching_count.0))?;
        }
        Shape::ArrowQuiver(p) => {
            proportion(
                p.wall.get() < p.mouth_radius.get() * p.bottom_scale.get() * 0.45
                    && p.rim_radius.get() < p.mouth_radius.get() * 0.3,
            )?;
            clearance(p.strap_drop.get() > p.strap_width.get())?;
        }
        Shape::Crossbow(p) => {
            clearance(
                p.nut_position.get() > p.length.get() * 0.25
                    && p.nut_position.get() < p.prod_position.get() - 0.08,
            )?;
            proportion(
                p.prod_position.get() < p.length.get()
                    && p.prod_span.get() > p.nose_width.get() * 4.0,
            )?;
            clearance(
                p.groove_width.get() > p.string_radius.get() * 2.0
                    && p.groove_width.get() < p.nose_width.get() * 0.7,
            )?;
            if p.prod_construction == CrossbowProdConstruction::Composite {
                proportion(
                    p.horn_thickness.map_or(0.0, Metres::get)
                        + p.sinew_thickness.map_or(0.0, Metres::get)
                        < p.prod_depth.get() * 0.75,
                )?;
            }
            clearance(
                p.nut_width.get() < p.butt_width.get()
                    && p.nut_thickness.get() < p.nut_radius.get() * 1.5
                    && p.rail_height.get() < p.lock_table_height.get() * 0.4,
            )?;
        }
        Shape::CrossbowBolt(p) => {
            proportion(
                p.butt_width.get() >= p.shaft_radius.get() * 1.3
                    && p.butt_height.get() <= p.shaft_radius.get() * 1.5
                    && p.fletching_length.get() + p.butt_length.get() < p.length.get() * 0.5,
            )?;
        }
        Shape::BoltQuiver(p) => {
            proportion(
                p.bottom_width.get() > p.mouth_width.get()
                    && p.wall.get() + p.lining.get() + p.hide_cover.get() < p.depth.get() * 0.12,
            )?;
            clearance(p.strap_drop.get() > p.strap_width.get())?;
        }
        Shape::Firearm(p) => firearm(p)?,
        Shape::LeadBall(p) => proportion(p.segments.is_none_or(|n| n.0 >= 3))?,
        Shape::BallPouch(p) => {
            proportion(
                p.wall.get() < p.width.get().min(p.depth.get()) * 0.15
                    && p.flap_overlap.get() < p.flap_length.get()
                    && p.belt_loop_gap.get() < p.width.get()
                    && (0.0..=120.0).contains(&p.flap_angle.get()),
            )?;
        }
        _ => {}
    }
    Ok(())
}
fn firearm(p: &FirearmParameters) -> Checked {
    proportion(p.band_count.0 >= 1)?;
    let arquebus = p.firearm_family == FirearmFirearmFamily::Arquebus;
    let breech = p.length.get() - p.barrel_length.get();
    clearance(
        breech > if arquebus { 0.12 } else { 0.075 }
            && p.lock_position.get() > 0.07
            && p.lock_position.get() < p.length.get() - 0.08,
    )?;
    proportion(p.bore.get() > 0.008 && p.barrel_wall.get() >= p.bore.get() * 0.15)?;
    proportion((1..=2).contains(&p.barrel_count.0))?;
    if p.barrel_count.0 == 2 {
        proportion(
            p.firearm_family == FirearmFirearmFamily::Pistol
                && p.lock_type == FirearmLockType::Wheellock,
        )?;
        proportion(
            p.secondary_barrel_length
                .is_some_and(|n| n.get() > 0.0 && n.get() < p.barrel_length.get()),
        )?;
    }
    proportion((p.lock_type == FirearmLockType::Matchlock) == arquebus)?;
    proportion(
        p.fore_width.get() < p.waist_width.get() && p.waist_width.get() < p.butt_width.get(),
    )?;
    clearance(
        (p.bore.get() / 2.0 + p.barrel_wall.get()) * (std::f64::consts::PI / 8.0).cos()
            > p.bore.get() / 2.0,
    )
}

fn bow(p: &ArcheryBowParameters) -> Checked {
    proportion(p.samples.is_none_or(|n| n.0 >= 4))?;
    proportion(p.grip_length.get() < p.length.get() * 0.35)?;
    proportion(
        (0.45..=0.57).contains(&p.upper_ratio.get())
            && p.tip_scale.get() > 0.2
            && p.tip_scale.get() <= 0.7,
    )?;
    clearance(
        p.brace_height.get() > p.loop_radius.get() * 3.0
            && p.loop_gap.get() > p.string_radius.get() * 4.0
            && p.loop_radius.get() >= p.string_radius.get() * 2.0,
    )?;
    if p.construction == ArcheryBowConstruction::Composite {
        proportion(
            p.horn_thickness.map_or(0.0, Metres::get)
                + p.backing_thickness.map_or(0.0, Metres::get)
                < p.limb_depth.get() * 0.75,
        )?;
    }

    Ok(())
}
