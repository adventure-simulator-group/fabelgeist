//! Reusable recipes; historical names describe their reference studies.
use crate::document::*;
#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Recipe {
    GermanLion,
    DurerLion,
    WoensamLions,
    WernigerodeEagle,
    WernigerodeDoubleEagle,
    Quartered,
    Counterchanged,
}

pub const PRESETS: &[&str] = &[
    "german-lion",
    "durer-lion",
    "woensam-lions",
    "wernigerode-eagle",
    "wernigerode-double-eagle",
    "quartered",
    "counterchanged",
];
impl Default for Document {
    fn default() -> Self {
        Self {
            name: "german-lion".into(),
            arms: ArmsDesign::lion(),
            drawing: DrawingStyle::default(),
            surface: PaintedSurface::default(),
            view: Viewing {
                yaw: Degrees(-12.0),
                pitch: Degrees(4.0),
                light: Degrees(-40.0),
                exposure: 0.0,
                zoom: Ratio(1.0),
            },
        }
    }
}
impl Default for DrawingStyle {
    fn default() -> Self {
        Self {
            painted_modeling: PaintedModeling::MODELED,
            stroke_width: Ratio(0.004),
            asymmetry: Ratio(0.0),
            lion: LionDrawing {
                body_width: Ratio(1.0),
                spine_arch: Ratio(1.0),
                head_size: Ratio(1.0),
                foreleg_reach: Ratio(1.0),
                hindleg_spread: Ratio(1.0),
                mane_fullness: Ratio(1.0),
                tail_curl: Ratio(1.0),
                paw_size: Ratio(1.0),
            },
        }
    }
}
impl Default for PaintedSurface {
    fn default() -> Self {
        Self {
            shape: DisplayShape::Shield,
            width: Millimeters(450.0),
            height: Millimeters(450.0),
            thickness: Millimeters(12.0),
            curvature: Millimeters(35.0),
            shoulder: Ratio(0.035),
            point: Ratio(0.55),
            covering: Covering::Canvas,
            ground: Millimeters(0.65),
            pigment: Millimeters(0.04),
            substrate_relief: Millimeters(0.18),
            brush_relief: Millimeters(0.012),
            brush_width: Millimeters(5.0),
            brush_angle: Degrees(12.0),
            gold: MetalFinish::Pigment,
            silver: MetalFinish::Pigment,
            glaze: Ratio(0.0),
            glaze_roughness: Ratio(0.32),
            palette: crate::paint::PaintPalette::default(),
            seed: Seed(1544),
        }
    }
}
impl ArmsDesign {
    pub fn plain(tincture: Tincture) -> Self {
        Self {
            field: Field::Solid { tincture },
            ordinaries: vec![],
            charges: vec![],
            inescutcheon: None,
        }
    }
    pub fn lion() -> Self {
        let mut a = Self::plain(Tincture::Azure);
        a.charges.push(Charge::new(
            ChargeKind::Lion {
                tails: LionTails::One,
                facing: Facing::Dexter,
            },
            Tincture::Or,
        ));
        a.charges[0].size = [Ratio(0.72), Ratio(0.78)];
        a.charges[0].center = [0.47, 0.45];
        a
    }
}
impl Charge {
    pub fn new(shape: ChargeKind, tincture: Tincture) -> Self {
        Self {
            shape,
            color: Coloring::Solid { tincture },
            center: [0.5, 0.48],
            size: [Ratio(0.84), Ratio(0.86)],
            rotation: Degrees(0.0),
            armed: Tincture::Gules,
            langued: Tincture::Gules,
        }
    }
}
pub fn preset(name: &str) -> Result<Document, crate::Error> {
    let mut d = Document::default();
    let recipe: Recipe = serde_json::from_value(serde_json::Value::String(name.into()))?;
    match recipe {
        Recipe::GermanLion => (),
        Recipe::WernigerodeEagle | Recipe::WernigerodeDoubleEagle => {
            d.arms = ArmsDesign::plain(Tincture::Or);
            let heads = if matches!(recipe, Recipe::WernigerodeEagle) {
                EagleHeads::One
            } else {
                EagleHeads::Two
            };
            let mut eagle = Charge::new(
                ChargeKind::Eagle {
                    heads,
                    facing: Facing::Dexter,
                },
                Tincture::Sable,
            );
            eagle.center = [0.5, 0.46];
            eagle.size = [Ratio(0.88), Ratio(0.86)];
            d.arms.charges.push(eagle);
        }
        Recipe::DurerLion => {
            d.surface.gold = MetalFinish::RAISED_MORDANT;
            d.drawing.lion.spine_arch = Ratio(1.3);
            d.drawing.lion.mane_fullness = Ratio(1.2);
            d.drawing.lion.tail_curl = Ratio(1.3);
        }
        Recipe::WoensamLions => {
            d.arms = ArmsDesign::plain(Tincture::Gules);
            for (i, facing) in [Facing::Sinister, Facing::Dexter].into_iter().enumerate() {
                let mut c = Charge::new(
                    ChargeKind::Lion {
                        tails: LionTails::One,
                        facing,
                    },
                    if i == 0 {
                        Tincture::Argent
                    } else {
                        Tincture::Sable
                    },
                );
                c.center = [0.30 + i as f32 * 0.40, 0.43];
                c.size = [Ratio(0.38), Ratio(0.64)];
                d.arms.charges.push(c);
            }
            d.drawing.lion.body_width = Ratio(0.85);
            d.drawing.lion.foreleg_reach = Ratio(1.3);
            d.drawing.lion.mane_fullness = Ratio(0.75);
        }
        Recipe::Quartered => {
            let mut lozenge = ArmsDesign::plain(Tincture::Or);
            lozenge
                .charges
                .push(Charge::new(ChargeKind::Lozenge, Tincture::Sable));
            d.arms.field = Field::Quarterly {
                quarters: Box::new([
                    lozenge.clone(),
                    ArmsDesign::lion(),
                    ArmsDesign::lion(),
                    lozenge,
                ]),
            };
            if let Field::Quarterly { quarters } = &mut d.arms.field {
                for (index, x) in [(2, 0.68), (3, 0.32)] {
                    quarters[index].charges[0].center = [x, 0.33];
                    quarters[index].charges[0].size = [Ratio(0.52), Ratio(0.58)];
                }
            }
            d.arms.charges.clear();
            let mut a = ArmsDesign::plain(Tincture::Gules);
            a.ordinaries.push(Ordinary {
                kind: OrdinaryKind::Fess,
                width: Ratio(0.3),
                boundary: Boundary::Straight,
                tincture: Tincture::Argent,
            });
            d.arms.inescutcheon = Some(Box::new(a));
        }
        Recipe::Counterchanged => {
            d.arms.field = Field::Divided {
                division: Division::Pale,
                boundary: Boundary::Wavy,
                tinctures: [Tincture::Azure, Tincture::Argent],
            };
            d.arms.charges[0].color = Coloring::Counterchanged {
                tinctures: [Tincture::Azure, Tincture::Argent],
            };
        }
    }
    d.name = name.into();
    d.validate()?;
    Ok(d)
}
