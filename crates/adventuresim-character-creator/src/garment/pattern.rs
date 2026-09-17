//! A sewn garment's pattern, in GarmentCode's vocabulary.
//!
//! A pattern chooses an upper-body block with its sleeves and collar, and a
//! lower garment. It is written over the reference T-shirt design, so these
//! choices are the whole difference between one garment and another. Value
//! ranges follow the GarmentCodeData design space.
use super::*;
use std::fmt::Debug;
use std::ops::RangeInclusive;

pub mod shapes;

/// Waistband circumference over the waist measurement, for lower garments
/// worn without an upper.
const WAISTBAND_EASE: f64 = 1.03;
/// A fitted bodice always ends at the waist, one neck-to-waist length down.
const WAIST_LENGTH: f64 = 1.0;
/// Hem depth, in neck-to-waist lengths, from which a straight body hangs across
/// both legs instead of following the torso.
const CROTCH_LENGTH: f64 = 1.8;

/// The upper-body block.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Upper {
    /// The straight T-tunic block. `length` runs down from the shoulder in
    /// neck-to-waist lengths, `width` eases the bust and `flare` widens the
    /// hem against it.
    Straight { length: f64, width: f64, flare: f64 },
    /// The darted bodice, always cut at the waist.
    Fitted,
}

impl Upper {
    pub const LENGTH: RangeInclusive<f64> = 0.5..=3.5;
    pub const WIDTH: RangeInclusive<f64> = 1.0..=1.3;
    pub const FLARE: RangeInclusive<f64> = 0.7..=1.6;
    /// A plain shirt to just below the waist.
    pub const STRAIGHT: Self = Self::Straight {
        length: 1.2,
        width: 1.05,
        flare: 1.0,
    };
}

/// Sleeves: `length` as a fraction of the arm below the armhole, and
/// `cuff_width` as a fraction of the armhole span, never narrower than the
/// wrist.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sleeves {
    pub length: f64,
    pub cuff_width: f64,
}

impl Sleeves {
    pub const LENGTH: RangeInclusive<f64> = 0.1..=1.15;
    pub const CUFF_WIDTH: RangeInclusive<f64> = 0.2..=2.0;
    pub const SHORT: Self = Self {
        length: 0.3,
        cuff_width: 1.0,
    };
}

/// A standing band collar sewn onto the neckline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Collar {
    pub height_cm: i64,
}

impl Collar {
    pub const HEIGHT_CM: RangeInclusive<i64> = 2..=8;
    pub const STANDING: Self = Self { height_cm: 4 };
}

/// The lower-body garment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Lower {
    /// `length` as a fraction of the leg, `width` easing the crotch, and
    /// `flare` tapering or widening the hem.
    Trousers { length: f64, width: f64, flare: f64 },
    /// The two-panel skirt: `length` as a fraction of the leg and `flare_cm`
    /// added to each side of both hems.
    Skirt { length: f64, flare_cm: f64 },
}

impl Lower {
    pub const TROUSERS_LENGTH: RangeInclusive<f64> = 0.2..=0.9;
    pub const TROUSERS_WIDTH: RangeInclusive<f64> = 1.0..=1.5;
    /// A narrower leg slides down the wearer while settling.
    pub const TROUSERS_FLARE: RangeInclusive<f64> = 0.9..=1.2;
    pub const SKIRT_LENGTH: RangeInclusive<f64> = 0.1..=0.95;
    pub const SKIRT_FLARE_CM: RangeInclusive<f64> = 0.0..=20.0;
    pub const TROUSERS: Self = Self::Trousers {
        length: 0.9,
        width: 1.0,
        flare: 1.0,
    };
    pub const SKIRT: Self = Self::Skirt {
        length: 0.45,
        flare_cm: 1.0,
    };
}

/// A sewn garment's pattern choices.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub upper: Option<Upper>,
    /// Only with an upper block.
    pub sleeves: Option<Sleeves>,
    /// Only with an upper block.
    pub collar: Option<Collar>,
    pub lower: Option<Lower>,
}

impl Default for Pattern {
    /// A short-sleeved shirt.
    fn default() -> Self {
        Self {
            upper: Some(Upper::STRAIGHT),
            sleeves: Some(Sleeves::SHORT),
            collar: None,
            lower: None,
        }
    }
}

fn within<T: PartialOrd + Debug>(what: &str, value: T, range: RangeInclusive<T>) -> Result<()> {
    if !range.contains(&value) {
        bail!("{what} {value:?} must be within {range:?}");
    }
    Ok(())
}

impl Pattern {
    /// How the sewn garment hangs, from what it covers.
    pub fn form(&self) -> GarmentForm {
        match (self.upper, self.lower) {
            (_, Some(Lower::Skirt { .. })) => GarmentForm::Skirted,
            (_, Some(Lower::Trousers { .. })) => GarmentForm::Legged,
            (Some(Upper::Straight { length, .. }), None) if length >= CROTCH_LENGTH => {
                GarmentForm::Skirted
            }
            _ => GarmentForm::Upper,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.upper.is_none() && self.lower.is_none() {
            bail!("a pattern needs an upper or a lower garment");
        }
        if self.upper.is_none() && (self.sleeves.is_some() || self.collar.is_some()) {
            bail!("sleeves and collars need an upper garment");
        }
        if let Some(Upper::Straight {
            length,
            width,
            flare,
        }) = self.upper
        {
            within("body length", length, Upper::LENGTH)?;
            within("body width", width, Upper::WIDTH)?;
            within("body flare", flare, Upper::FLARE)?;
        }
        if let Some(sleeves) = self.sleeves {
            within("sleeve length", sleeves.length, Sleeves::LENGTH)?;
            within("cuff width", sleeves.cuff_width, Sleeves::CUFF_WIDTH)?;
        }
        if let Some(collar) = self.collar {
            within("collar height", collar.height_cm, Collar::HEIGHT_CM)?;
        }
        match self.lower {
            Some(Lower::Trousers {
                length,
                width,
                flare,
            }) => {
                within("trouser length", length, Lower::TROUSERS_LENGTH)?;
                within("trouser width", width, Lower::TROUSERS_WIDTH)?;
                within("trouser flare", flare, Lower::TROUSERS_FLARE)?;
            }
            Some(Lower::Skirt { length, flare_cm }) => {
                within("skirt length", length, Lower::SKIRT_LENGTH)?;
                within("skirt flare", flare_cm, Lower::SKIRT_FLARE_CM)?;
            }
            None => {}
        }
        Ok(())
    }

    /// The reference T-shirt design with this pattern's choices written over it.
    pub fn design(&self) -> Result<Design> {
        let design = Design::from_yaml_str(assets::DESIGNS[0].yaml)?;
        let named = |name: Option<&str>| name.map_or(Value::Null, |name| Value::Str(name.into()));
        design.set_v(
            "meta.upper",
            named(self.upper.map(|upper| match upper {
                Upper::Straight { .. } => "Shirt",
                Upper::Fitted => "FittedShirt",
            })),
        );
        design.set_v(
            "meta.bottom",
            named(self.lower.map(|lower| match lower {
                Lower::Trousers { .. } => "Pants",
                Lower::Skirt { .. } => "Skirt2",
            })),
        );
        // A lower garment without an upper hangs from a waistband.
        design.set_v("meta.wb", named(self.upper.is_none().then_some("FittedWB")));
        design.set_f("waistband.waist", WAISTBAND_EASE);
        design.set_f("shirt.length", WAIST_LENGTH);
        if let Some(Upper::Straight {
            length,
            width,
            flare,
        }) = self.upper
        {
            design.set_f("shirt.length", length);
            design.set_f("shirt.width", width);
            design.set_f("shirt.flare", flare);
        }
        design.set_v("sleeve.sleeveless", Value::Bool(self.sleeves.is_none()));
        if let Some(sleeves) = self.sleeves {
            design.set_f("sleeve.length", sleeves.length);
            design.set_f("sleeve.end_width", sleeves.cuff_width);
        }
        design.set_v(
            "collar.component.style",
            named(self.collar.map(|_| "Turtle")),
        );
        if let Some(collar) = self.collar {
            design.set_v("collar.component.depth", Value::Int(collar.height_cm));
        }
        match self.lower {
            Some(Lower::Trousers {
                length,
                width,
                flare,
            }) => {
                design.set_f("pants.length", length);
                design.set_f("pants.width", width);
                design.set_f("pants.flare", flare);
            }
            Some(Lower::Skirt { length, flare_cm }) => {
                design.set_f("skirt.length", length);
                design.set_f("skirt.flare", flare_cm);
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

    #[test]
    fn form_follows_what_the_pattern_covers() {
        let shirt = Pattern::default();
        assert_eq!(shirt.form(), GarmentForm::Upper);
        let long = Pattern {
            upper: Some(Upper::Straight {
                length: CROTCH_LENGTH,
                width: 1.1,
                flare: 1.3,
            }),
            ..shirt
        };
        assert_eq!(long.form(), GarmentForm::Skirted);
        let trousers = Pattern {
            lower: Some(Lower::TROUSERS),
            ..long
        };
        assert_eq!(trousers.form(), GarmentForm::Legged);
        let dress = Pattern {
            lower: Some(Lower::SKIRT),
            ..shirt
        };
        assert_eq!(dress.form(), GarmentForm::Skirted);
    }

    #[test]
    fn invalid_patterns_are_rejected() {
        Pattern::default().validate().unwrap();
        let empty = Pattern {
            upper: None,
            sleeves: None,
            collar: None,
            lower: None,
        };
        assert!(empty.validate().is_err());
        let loose_sleeves = Pattern {
            upper: None,
            lower: Some(Lower::TROUSERS),
            ..Pattern::default()
        };
        assert!(loose_sleeves.validate().is_err());
        let tall_collar = Pattern {
            collar: Some(Collar { height_cm: 20 }),
            ..Pattern::default()
        };
        assert!(tall_collar.validate().is_err());
        let unset_length = Pattern {
            upper: Some(Upper::Straight {
                length: f64::NAN,
                width: 1.0,
                flare: 1.0,
            }),
            ..Pattern::default()
        };
        assert!(unset_length.validate().is_err());
    }
}
