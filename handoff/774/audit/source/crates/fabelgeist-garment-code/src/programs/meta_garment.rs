//! The meta garment: picks an upper, a belt and a lower garment from the
//! design parameters and stacks them into one piece.
//!
//! Ports `assets.garment_programs.meta_garment`.

use anyhow::{Result, bail};

use super::bands::{WaistbandKind, waistband};
use super::bodice::shirt;
use super::circle_skirt::{SkirtOpts, skirt_circle};
use super::godet::godet_skirt;
use super::pants::pants;
use super::prelude::*;
use super::skirt_levels::skirt_levels;
use super::skirt_paneled::{pencil_skirt, skirt_2, skirt_many_panels};

/// Build the named upper garment.
pub fn build_upper(name: &str, body: &Body, design: &Design) -> CompRef {
    match name {
        "Shirt" => shirt(body, design, false),
        "FittedShirt" => shirt(body, design, true),
        other => panic!("MetaGarment::ERROR::unknown upper garment '{other}'"),
    }
}

/// Build the named lower garment.
pub fn build_lower(name: &str, body: &Body, design: &Design, rise: Option<f64>) -> CompRef {
    let opts = SkirtOpts {
        rise,
        ..Default::default()
    };
    match name {
        "SkirtCircle" => skirt_circle(body, design, &opts, false),
        "AsymmSkirtCircle" => skirt_circle(body, design, &opts, true),
        "GodetSkirt" => godet_skirt(body, design, rise),
        "Pants" => pants(body, design, rise),
        "Skirt2" => skirt_2(body, design, &opts),
        "SkirtManyPanels" => skirt_many_panels(body, design, &opts),
        "PencilSkirt" => pencil_skirt(body, design, &opts),
        "SkirtLevels" => skirt_levels(body, design, rise),
        other => panic!("MetaGarment::ERROR::unknown lower garment '{other}'"),
    }
}

/// Build the named waistband.
pub fn build_belt(name: &str, body: &Body, design: &Design, rise: f64) -> CompRef {
    match name {
        "StraightWB" => waistband(body, design, rise, WaistbandKind::Straight),
        "FittedWB" => waistband(body, design, rise, WaistbandKind::Fitted),
        other => panic!("MetaGarment::ERROR::unknown waistband '{other}'"),
    }
}

/// A whole garment, assembled from the pieces the design selects.
pub struct MetaGarment {
    pub element: Element,
    pub upper_name: Option<String>,
    pub lower_name: Option<String>,
    pub belt_name: Option<String>,
    body_floor: f64,
}

impl MetaGarment {
    pub fn new(name: &str, body: &Body, design: &Design) -> Self {
        let comp = Component::new(name);

        let upper_name = design.s("meta.upper");
        let lower_name = design.s("meta.bottom");
        let belt_name = design.s("meta.wb");

        // Upper garment.
        if let Some(upper) = &upper_name {
            let up = build_upper(upper, body, design);
            add_sub(&comp, ec(&up));
            ec(&up).set_panel_label("body", false);
        }

        // Lower garment. NOTE: fitted tops want a full rise underneath.
        let lower = lower_name.as_ref().map(|lower| {
            let rise = match &upper_name {
                Some(u) if u.contains("Fitted") => Some(1.0),
                _ => None,
            };
            build_lower(lower, body, design, rise)
        });

        // Belt.
        if let Some(belt) = &belt_name {
            // Match the rise to the lower garment, if there is one.
            let rise = lower.as_ref().and_then(|l| l.borrow().rise).unwrap_or(1.0);
            let belt_comp = build_belt(belt, body, design, rise);
            Self::stack(&comp, &ec(&belt_comp));

            // The waistline label goes on the belt, when there is one.
            belt_comp
                .borrow()
                .interfaces
                .get("top")
                .borrow()
                .edges
                .propagate_label("lower_interface");
            ec(&belt_comp).set_panel_label("body", false);
        }

        // Attach the lower garment.
        if let Some(lower) = &lower {
            Self::stack(&comp, &ec(lower));
            if belt_name.is_none() {
                lower
                    .borrow()
                    .interfaces
                    .get("top")
                    .borrow()
                    .edges
                    .propagate_label("lower_interface");
            }
            ec(lower).set_panel_label("leg", false);
        }

        MetaGarment {
            element: ec(&comp),
            upper_name,
            lower_name,
            belt_name,
            body_floor: body.get("height") - body.get("head_l"),
        }
    }

    /// Place `next` below whatever is already there, and stitch the two.
    fn stack(comp: &CompRef, next: &Element) {
        let prev = comp.borrow().subs.last().cloned();

        if let Some(prev) = prev {
            let prev_bottom = prev.interfaces().get("bottom");
            let next_top = next.interfaces().get("top");

            next.place_by_interface(&next_top, &prev_bottom, 5.0, Alignment::Center, None);
            comp.borrow_mut()
                .stitching_rules
                .append(prev_bottom, next_top);
        }
        add_sub(comp, next.clone());
    }

    /// Assemble the sewing pattern.
    pub fn assembly(&self) -> crate::pattern::PatternSpec {
        self.element.assembly()
    }

    pub fn name(&self) -> String {
        self.element.name()
    }

    pub fn is_self_intersecting(&self) -> bool {
        self.element.is_self_intersecting()
    }

    pub fn length(&self) -> f64 {
        self.element.length()
    }

    /// The total length must fit between the shoulders and the floor.
    pub fn assert_total_length(&self, tol: f64) -> Result<()> {
        let length = self.length();
        if length > self.body_floor + tol {
            bail!(
                "MetaGarment::ERROR::Total length {length} exceeds the floor length {}",
                self.body_floor
            );
        }
        Ok(())
    }

    /// The garment must not be empty.
    ///
    /// With `filter_belts`, a garment made only of a waistband counts as empty.
    pub fn assert_non_empty(&self, filter_belts: bool) -> Result<()> {
        if self.upper_name.is_none()
            && self.lower_name.is_none()
            && (filter_belts || self.belt_name.is_none())
        {
            bail!("MetaGarment::ERROR::the garment has no elements");
        }
        Ok(())
    }

    /// A heavy skirt needs something to hang from.
    pub fn assert_skirt_waistband(&self) -> Result<()> {
        let heavy = matches!(
            self.lower_name.as_deref(),
            Some("SkirtCircle") | Some("AsymmSkirtCircle") | Some("SkirtManyPanels")
        );
        if heavy && self.belt_name.is_none() && self.upper_name.is_none() {
            bail!("MetaGarment::ERROR::a heavy skirt needs a waistband or an upper garment");
        }
        Ok(())
    }
}
