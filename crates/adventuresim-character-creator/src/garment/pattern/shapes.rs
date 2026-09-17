//! Named starting points for a sewn pattern, from the medieval wardrobe.
//!
//! A shape only fills in a pattern and its layer; the garment stays freely
//! editable afterwards.
use super::*;

/// A named pattern and the layer it is usually worn in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    pub name: &'static str,
    pub pattern: Pattern,
    pub layer: ClothLayer,
}

const NOTHING: Pattern = Pattern {
    upper: None,
    sleeves: None,
    collar: None,
    lower: None,
};
/// To the wrist, narrowing towards it.
const LONG_SLEEVES: Sleeves = Sleeves {
    length: 1.0,
    cuff_width: 0.6,
};
/// To the wrist and close on the forearm, as fitted bodices were sleeved.
const LONG_FITTED_SLEEVES: Sleeves = Sleeves {
    length: 1.0,
    cuff_width: 0.55,
};

pub const SHIRT: Shape = Shape {
    name: "Shirt",
    pattern: Pattern {
        upper: Some(Upper::STRAIGHT),
        sleeves: Some(Sleeves::SHORT),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
pub const FITTED_SHIRT: Shape = Shape {
    name: "Fitted shirt",
    pattern: Pattern {
        upper: Some(Upper::Fitted),
        ..SHIRT.pattern
    },
    ..SHIRT
};
/// The cotte: a straight, flared, long-sleeved tunic to the thigh.
pub const TUNIC: Shape = Shape {
    name: "Tunic",
    pattern: Pattern {
        upper: Some(Upper::Straight {
            length: 2.0,
            width: 1.1,
            flare: 1.3,
        }),
        sleeves: Some(LONG_SLEEVES),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
pub const DOUBLET: Shape = Shape {
    name: "Doublet",
    pattern: Pattern {
        upper: Some(Upper::Fitted),
        sleeves: Some(LONG_FITTED_SLEEVES),
        collar: Some(Collar::STANDING),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
/// A hip-length padded coat worn under mail.
pub const GAMBESON: Shape = Shape {
    name: "Gambeson",
    pattern: Pattern {
        upper: Some(Upper::Straight {
            length: 1.7,
            width: 1.08,
            flare: 1.15,
        }),
        sleeves: Some(Sleeves {
            length: 1.0,
            cuff_width: 0.7,
        }),
        collar: Some(Collar::STANDING),
        ..NOTHING
    },
    layer: ClothLayer::Padding,
};
pub const TROUSERS: Shape = Shape {
    name: "Trousers",
    pattern: Pattern {
        lower: Some(Lower::TROUSERS),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
/// Chausses: full-length legs tapering slightly to the ankle.
pub const HOSE: Shape = Shape {
    name: "Hose",
    pattern: Pattern {
        lower: Some(Lower::Trousers {
            length: 0.9,
            width: 1.0,
            flare: 0.9,
        }),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
/// Loose breeches to above the knee.
pub const BRAIES: Shape = Shape {
    name: "Braies",
    pattern: Pattern {
        lower: Some(Lower::Trousers {
            length: 0.35,
            width: 1.3,
            flare: 1.0,
        }),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
pub const SKIRT: Shape = Shape {
    name: "Skirt",
    pattern: Pattern {
        lower: Some(Lower::SKIRT),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
pub const DRESS: Shape = Shape {
    name: "Dress",
    pattern: Pattern {
        lower: Some(Lower::SKIRT),
        ..SHIRT.pattern
    },
    layer: ClothLayer::Clothing,
};
/// A fitted bodice with long sleeves over a flared ankle-length skirt.
pub const KIRTLE: Shape = Shape {
    name: "Kirtle",
    pattern: Pattern {
        upper: Some(Upper::Fitted),
        sleeves: Some(LONG_FITTED_SLEEVES),
        lower: Some(Lower::Skirt {
            length: 0.85,
            flare_cm: 15.0,
        }),
        ..NOTHING
    },
    layer: ClothLayer::Clothing,
};
/// A sleeveless, flared, knee-length overgarment worn over armor.
pub const SURCOAT: Shape = Shape {
    name: "Surcoat",
    pattern: Pattern {
        upper: Some(Upper::Straight {
            length: 2.6,
            width: 1.1,
            flare: 1.5,
        }),
        ..NOTHING
    },
    layer: ClothLayer::Outerwear,
};
/// A voluminous floor-length gown with wide sleeves and a standing collar.
pub const HOUPPELANDE: Shape = Shape {
    name: "Houppelande",
    pattern: Pattern {
        upper: Some(Upper::Straight {
            length: 3.2,
            width: 1.3,
            flare: 1.6,
        }),
        sleeves: Some(Sleeves {
            length: 1.15,
            cuff_width: 2.0,
        }),
        collar: Some(Collar { height_cm: 6 }),
        ..NOTHING
    },
    layer: ClothLayer::Outerwear,
};

pub const SHAPES: [Shape; 13] = [
    SHIRT,
    FITTED_SHIRT,
    TUNIC,
    DOUBLET,
    GAMBESON,
    TROUSERS,
    HOSE,
    BRAIES,
    SKIRT,
    DRESS,
    KIRTLE,
    SURCOAT,
    HOUPPELANDE,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn panel_names(shape: Shape) -> Vec<String> {
        let body = Body::from_yaml_str(assets::BODIES[0].yaml).unwrap();
        let pattern = MetaGarment::new("shape", &body, &shape.pattern.design().unwrap()).assembly();
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
    fn every_shape_is_a_valid_pattern() {
        for shape in SHAPES {
            shape
                .pattern
                .validate()
                .unwrap_or_else(|error| panic!("{}: {error:#}", shape.name));
        }
    }

    #[test]
    fn sleeveless_shapes_leave_the_armholes_open() {
        let surcoat = panel_names(SURCOAT);
        assert!(!has_panel(&surcoat, "sleeve"), "{surcoat:?}");
        assert!(has_panel(&surcoat, "torso"), "{surcoat:?}");
        for shape in [TUNIC, DOUBLET] {
            let names = panel_names(shape);
            assert!(has_panel(&names, "sleeve"), "{}: {names:?}", shape.name);
        }
    }

    #[test]
    fn collar_pieces_are_sewn_on() {
        for shape in [DOUBLET, GAMBESON, HOUPPELANDE] {
            let names = panel_names(shape);
            assert!(has_panel(&names, "collar"), "{}: {names:?}", shape.name);
        }
        assert!(!has_panel(&panel_names(TUNIC), "collar"));
    }

    #[test]
    fn lower_garments_hang_from_a_waistband_only_without_an_upper() {
        for shape in [HOSE, BRAIES] {
            let names = panel_names(shape);
            assert!(has_panel(&names, "pant"), "{}: {names:?}", shape.name);
            assert!(has_panel(&names, "wb"), "{}: {names:?}", shape.name);
        }
        let kirtle = panel_names(KIRTLE);
        assert!(has_panel(&kirtle, "skirt"), "{kirtle:?}");
        assert!(!has_panel(&kirtle, "wb"), "{kirtle:?}");
    }
}
