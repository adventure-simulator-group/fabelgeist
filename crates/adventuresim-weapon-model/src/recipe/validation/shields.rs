//! Shield curvature, wall thickness and interior fitting envelopes.
use super::*;
macro_rules! common {
    ($p:ident,$strapped:expr,$material:expr) => {{
        let steel = matches!($material, Some(Material::Steel | Material::DarkSteel));
        proportion($p.thickness.get() >= if steel { 0.001 } else { 0.006 })?;
        for n in [
            $p.grip_length,
            $p.grip_radius,
            $p.fitting_clearance,
            $p.strap_width,
            $p.strap_thickness,
        ] {
            positive(n.ok_or(RecipeError::Dimension)?.get())?;
        }
        for n in [$p.rim_radius, $p.boss_radius, $p.boss_height]
            .into_iter()
            .flatten()
        {
            nonnegative(n.get())?;
        }
        proportion(
            $p.fitting_angle
                .is_some_and(|n| (0.0..=90.0).contains(&n.get())),
        )?;
        if $strapped {
            positive($p.fitting_spacing.ok_or(RecipeError::Dimension)?.get())?;
        }
        proportion(
            $p.rim_radius.map_or(0.0, Metres::get) <= 0.006_f64.max($p.thickness.get() * 1.5),
        )?;
    }};
}
pub(super) fn check(component: &Component) -> Checked {
    match &component.shape {
        Shape::RoundShield(p) => {
            let detail = crate::Detail::High;
            let faces = 4
                * detail.radial(p.radius.get(), p.radial_segments.0 as usize)
                * (detail.samples(p.rings.0 as usize, 3) + 1);
            require(
                crate::construction::construction_budget(faces as f64).is_ok(),
                RecipeError::Budget,
            )?;
            let strapped = p.fitting_mode == RoundShieldFittingMode::GripAndStrap;
            common!(p, strapped, component.material);
            nonnegative(p.outer_curve.get())?;
            nonnegative(p.center_curve.get())?;
            positive(p.center_radius.get())?;
            proportion(
                p.center_radius.get() < p.radius.get()
                    && p.rings.0 >= 3
                    && p.radial_segments.0 >= 12
                    && p.outer_curve.get() + p.center_curve.get() <= p.radius.get() * 0.8,
            )?;
            let interior = p.radius.get() - p.rim_radius.map_or(0.0, Metres::get);
            proportion(p.boss_radius.map_or(0.0, Metres::get) < interior)?;
            let center = if strapped {
                p.fitting_spacing.map_or(0.0, Metres::get) / 2.0
            } else {
                0.0
            };
            clearance(
                center.hypot(p.grip_length.unwrap().get() / 2.0)
                    + p.grip_radius
                        .unwrap()
                        .get()
                        .max(p.strap_width.unwrap().get() / 2.0)
                    < interior,
            )?;
        }
        Shape::ShapedShield(p) => {
            require(
                p.outline_resolution(crate::Detail::High) <= MAX_OUTLINE_STATIONS,
                RecipeError::Budget,
            )?;
            let strapped = p.fitting_mode == ShapedShieldFittingMode::GripAndStrap;
            common!(p, strapped, component.material);
            for n in [
                p.top_depth,
                p.bottom_depth,
                p.corner_radius,
                p.cylindrical_curve,
                p.center_curve,
            ] {
                nonnegative(n.get())?;
            }
            positive(p.center_width.get())?;
            positive(p.center_height.get())?;
            proportion(
                (0.0..=1.0).contains(&p.top_roundness.get())
                    && (0.0..=1.0).contains(&p.bottom_roundness.get())
                    && (0.0..0.8).contains(&p.side_taper.get()),
            )?;
            proportion(
                p.corner_radius.get() <= p.width.get().min(p.height.get()) * 0.25
                    && p.top_depth.get() <= p.height.get() * 0.45
                    && p.bottom_depth.get() <= p.height.get() * 0.65
                    && p.cylindrical_curve.get() <= p.width.get() * 0.45
                    && p.center_width.get() <= p.width.get()
                    && p.edge_segments.0 >= 6,
            )?;
            require(
                p.center_width.get() >= p.width.get() / 128.0,
                RecipeError::Budget,
            )?;
            let rim = p.rim_radius.map_or(0.0, Metres::get);
            proportion(
                p.boss_radius.map_or(0.0, Metres::get)
                    < p.width.get().min(p.height.get()) / 2.0 - rim,
            )?;
            let lateral = if strapped {
                p.fitting_spacing.map_or(0.0, Metres::get) / 2.0
            } else {
                0.0
            };
            clearance(
                lateral + p.strap_width.unwrap().get() / 2.0 < p.width.get() / 2.0 - rim
                    && p.grip_length.unwrap().get() / 2.0 + p.grip_radius.unwrap().get()
                        < p.height.get() / 2.0,
            )?;
        }
        _ => {}
    }
    Ok(())
}
