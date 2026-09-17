//! The pattern behind each sewn garment preset, in GarmentCode's vocabulary.
//!
//! Every preset starts from the reference T-shirt design and overrides only
//! the parameters that distinguish it, so the values here are the whole
//! difference between one garment and another. The vocabulary is the
//! GarmentCodeData design space: a straight or fitted upper block, sleeves
//! with a cuff width, a collar piece, and trousers or a two-panel skirt.
use super::*;
use std::ops::RangeInclusive;

/// Waistband circumference over the waist measurement, for lower garments
/// worn without an upper.
const WAISTBAND_EASE: f64 = 1.03;
/// An upper block cut at the waist, where a skirt is joined or a bodice ends.
const WAIST_LENGTH: f64 = 1.0;

/// The upper-body block.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Upper {
    /// The straight T-tunic block: `width` scales the bust ease and `flare`
    /// the hem against it.
    Straight { width: f64, flare: f64 },
    /// The darted bodice, always cut at the waist.
    Fitted,
}

/// Sleeves: `length` as a fraction of the arm below the armhole, and
/// `end_width` as a fraction of the armhole span, never narrower than the
/// wrist.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sleeves {
    length: f64,
    end_width: f64,
}

/// A collar piece sewn onto the neckline.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Collar {
    /// A standing band `depth` centimetres tall: GarmentCode's turtle collar.
    Standing { depth: i64 },
}

/// The lower-body garment.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Lower {
    /// Trousers: `length` as a fraction of the leg, `width` easing the
    /// crotch, and `flare` tapering or widening the hem.
    Pants { length: f64, width: f64, flare: f64 },
    /// The two-panel skirt: `length` as a fraction of the leg and `flare` in
    /// centimetres added to each side of every hem.
    Skirt { length: f64, flare: f64 },
}

/// The wearer's choice of length below the shoulder for a straight upper
/// block, in neck-to-waist lengths: 1 reaches the waist, about 2.5 the knee.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Adjustable {
    default: f32,
    min: f32,
    max: f32,
}

/// One preset's pattern choices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Cut {
    upper: Option<Upper>,
    /// Absent when the upper is cut at the waist or there is none.
    length: Option<Adjustable>,
    sleeves: Option<Sleeves>,
    collar: Option<Collar>,
    lower: Option<Lower>,
}

const SHORT_SLEEVES: Sleeves = Sleeves {
    length: 0.3,
    end_width: 1.0,
};
/// To the wrist, narrowing towards it.
const LONG_SLEEVES: Sleeves = Sleeves {
    length: 1.0,
    end_width: 0.6,
};
/// To the wrist and close on the forearm, as fitted bodices were sleeved.
const LONG_FITTED_SLEEVES: Sleeves = Sleeves {
    length: 1.0,
    end_width: 0.55,
};
const T_SHIRT: Cut = Cut {
    upper: Some(Upper::Straight {
        width: 1.05,
        flare: 1.0,
    }),
    length: Some(Adjustable {
        default: 1.2,
        min: 0.5,
        max: 3.5,
    }),
    sleeves: Some(SHORT_SLEEVES),
    collar: None,
    lower: None,
};
const NOTHING: Cut = Cut {
    upper: None,
    length: None,
    sleeves: None,
    collar: None,
    lower: None,
};
const FITTED_SHIRT: Cut = Cut {
    upper: Some(Upper::Fitted),
    length: None,
    ..T_SHIRT
};
const TUNIC: Cut = Cut {
    upper: Some(Upper::Straight {
        width: 1.1,
        flare: 1.3,
    }),
    length: Some(Adjustable {
        default: 2.0,
        min: 1.2,
        max: 3.0,
    }),
    sleeves: Some(LONG_SLEEVES),
    ..NOTHING
};
const DOUBLET: Cut = Cut {
    upper: Some(Upper::Fitted),
    sleeves: Some(LONG_FITTED_SLEEVES),
    collar: Some(Collar::Standing { depth: 4 }),
    ..NOTHING
};
const GAMBESON: Cut = Cut {
    upper: Some(Upper::Straight {
        width: 1.08,
        flare: 1.15,
    }),
    length: Some(Adjustable {
        default: 1.7,
        min: 1.2,
        max: 2.4,
    }),
    sleeves: Some(Sleeves {
        length: 1.0,
        end_width: 0.7,
    }),
    collar: Some(Collar::Standing { depth: 4 }),
    ..NOTHING
};
const TROUSERS: Cut = Cut {
    lower: Some(Lower::Pants {
        length: 0.9,
        width: 1.0,
        flare: 1.0,
    }),
    ..NOTHING
};
/// A narrower leg slides down the wearer while settling, so the taper is slight.
const HOSE: Cut = Cut {
    lower: Some(Lower::Pants {
        length: 0.9,
        width: 1.0,
        flare: 0.9,
    }),
    ..NOTHING
};
const BRAIES: Cut = Cut {
    lower: Some(Lower::Pants {
        length: 0.35,
        width: 1.3,
        flare: 1.0,
    }),
    ..NOTHING
};
const SKIRT: Cut = Cut {
    lower: Some(Lower::Skirt {
        length: 0.45,
        flare: 1.0,
    }),
    ..NOTHING
};
const DRESS: Cut = Cut {
    length: None,
    lower: Some(Lower::Skirt {
        length: 0.45,
        flare: 1.0,
    }),
    ..T_SHIRT
};
const KIRTLE: Cut = Cut {
    upper: Some(Upper::Fitted),
    sleeves: Some(LONG_FITTED_SLEEVES),
    lower: Some(Lower::Skirt {
        length: 0.85,
        flare: 15.0,
    }),
    ..NOTHING
};
const SURCOAT: Cut = Cut {
    upper: Some(Upper::Straight {
        width: 1.1,
        flare: 1.5,
    }),
    length: Some(Adjustable {
        default: 2.6,
        min: 1.4,
        max: 3.4,
    }),
    ..NOTHING
};
const HOUPPELANDE: Cut = Cut {
    upper: Some(Upper::Straight {
        width: 1.3,
        flare: 1.6,
    }),
    length: Some(Adjustable {
        default: 3.2,
        min: 2.4,
        max: 3.5,
    }),
    sleeves: Some(Sleeves {
        length: 1.15,
        end_width: 2.0,
    }),
    collar: Some(Collar::Standing { depth: 6 }),
    ..NOTHING
};

impl GarmentPreset {
    /// The pattern the preset is cut from, or none for a fitted surface.
    pub(super) fn cut(self) -> Option<Cut> {
        Some(match self {
            Self::Shirt => T_SHIRT,
            Self::FittedShirt => FITTED_SHIRT,
            Self::Tunic => TUNIC,
            Self::Doublet => DOUBLET,
            Self::Gambeson => GAMBESON,
            Self::Trousers => TROUSERS,
            Self::Hose => HOSE,
            Self::Braies => BRAIES,
            Self::Skirt => SKIRT,
            Self::Dress => DRESS,
            Self::Kirtle => KIRTLE,
            Self::Surcoat => SURCOAT,
            Self::Houppelande => HOUPPELANDE,
            Self::Coif => return None,
        })
    }
}

impl Cut {
    /// The wearer's length choice, when the cut offers one.
    pub(super) fn length_range(&self) -> Option<RangeInclusive<f32>> {
        self.length.map(|length| length.min..=length.max)
    }

    pub(super) fn default_length(&self) -> f32 {
        self.length
            .map_or(WAIST_LENGTH as f32, |length| length.default)
    }

    /// The reference T-shirt design with this cut's choices written over it.
    pub(super) fn design(&self, length: f32) -> Result<Design> {
        let design = Design::from_yaml_str(assets::DESIGNS[0].yaml)?;
        let named = |name: Option<&str>| name.map_or(Value::Null, |name| Value::Str(name.into()));
        design.set_v(
            "meta.upper",
            named(match self.upper {
                Some(Upper::Straight { .. }) => Some("Shirt"),
                Some(Upper::Fitted) => Some("FittedShirt"),
                None => None,
            }),
        );
        design.set_v(
            "meta.bottom",
            named(match self.lower {
                Some(Lower::Pants { .. }) => Some("Pants"),
                Some(Lower::Skirt { .. }) => Some("Skirt2"),
                None => None,
            }),
        );
        // A lower garment without an upper hangs from a waistband.
        design.set_v("meta.wb", named(self.upper.is_none().then_some("FittedWB")));
        design.set_f("waistband.waist", WAISTBAND_EASE);
        if let Some(Upper::Straight { width, flare }) = self.upper {
            design.set_f("shirt.width", width);
            design.set_f("shirt.flare", flare);
        }
        design.set_f(
            "shirt.length",
            if self.length.is_some() {
                length as f64
            } else {
                WAIST_LENGTH
            },
        );
        design.set_v("sleeve.sleeveless", Value::Bool(self.sleeves.is_none()));
        if let Some(Sleeves { length, end_width }) = self.sleeves {
            design.set_f("sleeve.length", length);
            design.set_f("sleeve.end_width", end_width);
        }
        design.set_v(
            "collar.component.style",
            named(match self.collar {
                Some(Collar::Standing { .. }) => Some("Turtle"),
                None => None,
            }),
        );
        if let Some(Collar::Standing { depth }) = self.collar {
            design.set_v("collar.component.depth", Value::Int(depth));
        }
        match self.lower {
            Some(Lower::Pants {
                length,
                width,
                flare,
            }) => {
                design.set_f("pants.length", length);
                design.set_f("pants.width", width);
                design.set_f("pants.flare", flare);
            }
            Some(Lower::Skirt { length, flare }) => {
                design.set_f("skirt.length", length);
                design.set_f("skirt.flare", flare);
                // Flat at the waist: the skirt is not gathered.
                design.set_f("skirt.ruffle", 1.0);
            }
            None => {}
        }
        Ok(design)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel_names(preset: GarmentPreset) -> Vec<String> {
        let body = Body::from_yaml_str(assets::BODIES[0].yaml).unwrap();
        let pattern = MetaGarment::new("cut", &body, &preset.design().unwrap()).assembly();
        pattern
            .panels
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn has_panel(names: &[String], part: &str) -> bool {
        names.iter().any(|name| name.contains(part))
    }

    #[test]
    fn every_sewn_preset_defaults_inside_its_length_range() {
        for preset in GarmentPreset::ALL {
            if let Some(range) = preset.length_range() {
                assert!(
                    range.contains(&preset.default_length()),
                    "{}: {} outside {range:?}",
                    preset.label(),
                    preset.default_length()
                );
            }
        }
    }

    #[test]
    fn the_fitted_coif_has_no_pattern() {
        assert!(GarmentPreset::Coif.cut().is_none());
        assert!(GarmentPreset::Coif.design().is_err());
    }

    #[test]
    fn sleeveless_cuts_leave_the_armholes_open() {
        let surcoat = panel_names(GarmentPreset::Surcoat);
        assert!(!has_panel(&surcoat, "sleeve"), "{surcoat:?}");
        assert!(has_panel(&surcoat, "torso"), "{surcoat:?}");
        for preset in [GarmentPreset::Tunic, GarmentPreset::Doublet] {
            let names = panel_names(preset);
            assert!(has_panel(&names, "sleeve"), "{}: {names:?}", preset.label());
        }
    }

    #[test]
    fn collar_pieces_are_sewn_on() {
        for preset in [
            GarmentPreset::Doublet,
            GarmentPreset::Gambeson,
            GarmentPreset::Houppelande,
        ] {
            let names = panel_names(preset);
            assert!(has_panel(&names, "collar"), "{}: {names:?}", preset.label());
        }
        assert!(!has_panel(&panel_names(GarmentPreset::Tunic), "collar"));
    }

    #[test]
    fn lower_garments_hang_from_a_waistband_only_without_an_upper() {
        for preset in [GarmentPreset::Hose, GarmentPreset::Braies] {
            let names = panel_names(preset);
            assert!(has_panel(&names, "pant"), "{}: {names:?}", preset.label());
            assert!(has_panel(&names, "wb"), "{}: {names:?}", preset.label());
        }
        let kirtle = panel_names(GarmentPreset::Kirtle);
        assert!(has_panel(&kirtle, "skirt"), "{kirtle:?}");
        assert!(!has_panel(&kirtle, "wb"), "{kirtle:?}");
    }
}
